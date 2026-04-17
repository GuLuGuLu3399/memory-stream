package services

import (
	"context"
	"errors"
	"fmt"
	"sort"
	"time"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"gorm.io/gorm"
	"gorm.io/gorm/clause"
)

// Sentinel errors for merge operations — use errors.Is() to check.
var (
	ErrSurvivorInVictims = errors.New("survivor_id cannot be in victim_ids")
	ErrCardNotFound      = errors.New("one or more card IDs not found")
)

// MergeRequest represents the input for merging cards
type MergeRequest struct {
	SurvivorID string   `json:"survivor_id" binding:"required"`
	VictimIDs  []string `json:"victim_ids" binding:"required,min=1"`
}

// MergeResult represents the result of a merge operation
type MergeResult struct {
	EdgesMigrated int      `json:"edges_migrated"`
	NodesDeleted  int      `json:"nodes_deleted"`
	Warnings      []string `json:"warnings,omitempty"`
}

// MergeService handles card merge operations via atomic transactions.
type MergeService struct {
	db *gorm.DB
}

// NewMergeService creates a MergeService instance.
func NewMergeService(db *gorm.DB) *MergeService {
	return &MergeService{db: db}
}

// Merge merges multiple victim cards into a survivor card.
// All edges pointing to/from victims are redirected to the survivor.
// Duplicate edges are removed, and self-loops for sequence edges are cleaned up.
// The entire operation is atomic within a single PostgreSQL transaction.
func (s *MergeService) Merge(ctx context.Context, req MergeRequest) (*MergeResult, error) {
	// Validation: survivor not in victims
	for _, vid := range req.VictimIDs {
		if vid == req.SurvivorID {
			return nil, ErrSurvivorInVictims
		}
	}

	// Collect all IDs and sort to prevent deadlocks (consistent lock ordering)
	allIDs := append([]string{req.SurvivorID}, req.VictimIDs...)
	sort.Strings(allIDs)

	result := &MergeResult{
		Warnings: []string{},
	}

	err := s.db.WithContext(ctx).Transaction(func(tx *gorm.DB) error {
		// Row-level lock: lock all involved cards to prevent concurrent merge.
		// Use SELECT ... FOR UPDATE (not COUNT(*)) since PG forbids FOR UPDATE with aggregates.
		var lockedCards []models.Card
		if err := tx.Clauses(clause.Locking{Strength: "UPDATE"}).
			Where("id IN ?", allIDs).
			Find(&lockedCards).Error; err != nil {
			return fmt.Errorf("failed to lock cards for merge: %w", err)
		}
		if len(lockedCards) != len(allIDs) {
			return ErrCardNotFound
		}
		// Step 1: Collect all edges to migrate
		type EdgeInfo struct {
			SourceID     string
			TargetID     string
			RelationType string
			CreatedAt    time.Time
		}

		// Get all incoming edges to victims
		var incomingEdges []EdgeInfo
		if err := tx.Model(&models.CardEdge{}).
			Where("target_id IN ?", req.VictimIDs).
			Select("source_id, target_id, relation_type, created_at").
			Scan(&incomingEdges).Error; err != nil {
			return fmt.Errorf("failed to query incoming edges: %w", err)
		}

		// Get all outgoing edges from victims
		var outgoingEdges []EdgeInfo
		if err := tx.Model(&models.CardEdge{}).
			Where("source_id IN ?", req.VictimIDs).
			Select("source_id, target_id, relation_type, created_at").
			Scan(&outgoingEdges).Error; err != nil {
			return fmt.Errorf("failed to query outgoing edges: %w", err)
		}

		// Step 2: Delete all edges involving victims
		if err := tx.Where("source_id IN ? OR target_id IN ?", req.VictimIDs, req.VictimIDs).
			Delete(&models.CardEdge{}).Error; err != nil {
		return fmt.Errorf("failed to delete victim edges: %w", err)
		}

		// Step 3: Collect existing survivor edges to avoid PK conflicts on INSERT
		var existingSurvivorEdges []models.CardEdge
		if err := tx.Where("source_id = ? OR target_id = ?", req.SurvivorID, req.SurvivorID).
			Find(&existingSurvivorEdges).Error; err != nil {
		return fmt.Errorf("failed to query survivor edges: %w", err)
		}
		existingEdgeKeys := make(map[string]bool, len(existingSurvivorEdges))
		for _, e := range existingSurvivorEdges {
			existingEdgeKeys[e.SourceID+":"+e.TargetID] = true
		}

		// Step 4: Build deduplicated edge set
		edgeMap := make(map[string]EdgeInfo) // key: "source:target:type"

		// Process incoming edges (redirect target to survivor)
		for _, e := range incomingEdges {
			// Skip self-loops that would be created
			if e.SourceID == req.SurvivorID {
				result.Warnings = append(result.Warnings, "skipped incoming self-loop edge")
				continue
			}
			// Skip if survivor already has this edge (avoid PK conflict)
			pkKey := e.SourceID + ":" + req.SurvivorID
			if existingEdgeKeys[pkKey] {
				result.Warnings = append(result.Warnings, "skipped duplicate edge (survivor already connected)")
				continue
			}
			key := e.SourceID + ":" + req.SurvivorID + ":" + e.RelationType
			if existing, ok := edgeMap[key]; !ok || e.CreatedAt.Before(existing.CreatedAt) {
				edgeMap[key] = EdgeInfo{
					SourceID:     e.SourceID,
					TargetID:     req.SurvivorID,
					RelationType: e.RelationType,
					CreatedAt:    e.CreatedAt,
				}
			}
		}

		// Process outgoing edges (redirect source to survivor)
		for _, e := range outgoingEdges {
			// Skip self-loops that would be created
			if e.TargetID == req.SurvivorID {
				if e.RelationType == "sequence" {
					// Sequence self-loops are removed, reference ones generate warning
					result.Warnings = append(result.Warnings, "removed sequence self-loop edge")
				} else {
					result.Warnings = append(result.Warnings, "skipped outgoing self-loop edge")
				}
				continue
			}
			// Skip if survivor already has this edge (avoid PK conflict)
			pkKey := req.SurvivorID + ":" + e.TargetID
			if existingEdgeKeys[pkKey] {
				result.Warnings = append(result.Warnings, "skipped duplicate edge (survivor already connected)")
				continue
			}
			key := req.SurvivorID + ":" + e.TargetID + ":" + e.RelationType
			if existing, ok := edgeMap[key]; !ok || e.CreatedAt.Before(existing.CreatedAt) {
				edgeMap[key] = EdgeInfo{
					SourceID:     req.SurvivorID,
					TargetID:     e.TargetID,
					RelationType: e.RelationType,
					CreatedAt:    e.CreatedAt,
				}
			}
		}

		// Step 5: Insert deduplicated edges
		if len(edgeMap) > 0 {
			newEdges := make([]models.CardEdge, 0, len(edgeMap))
			for _, e := range edgeMap {
				newEdges = append(newEdges, models.CardEdge{
					SourceID:     e.SourceID,
					TargetID:     e.TargetID,
					RelationType: e.RelationType,
					CreatedAt:    e.CreatedAt,
				})
			}
			if err := tx.Create(&newEdges).Error; err != nil {
				return fmt.Errorf("failed to create merged edges: %w", err)
			}
			result.EdgesMigrated = len(newEdges)
		}

		// Step 6: Optional deduplication of edges (keep earliest CreatedAt)
		if err := deduplicateEdges(tx, req.SurvivorID); err != nil {
		return fmt.Errorf("failed to deduplicate edges: %w", err)
		}

		// Step 7: Remove any sequence self-loops on survivor (cleanup)
		if err := removeSequenceSelfLoops(tx, req.SurvivorID); err != nil {
		return fmt.Errorf("failed to remove self-loops: %w", err)
		}

		// Step 8: Delete victim cards
		if err := tx.Where("id IN ?", req.VictimIDs).Delete(&models.Card{}).Error; err != nil {
		return fmt.Errorf("failed to delete victim cards: %w", err)
		}
		result.NodesDeleted = len(req.VictimIDs)

		return nil
	})

	if err != nil {
		return nil, err
	}

	return result, nil
}

// deduplicateEdges removes duplicate edges involving the given survivor,
// keeping the earliest CreatedAt per (source_id, target_id, relation_type).
// This operates within the provided transaction and is scoped to edges
// connected to the survivor.
func deduplicateEdges(tx *gorm.DB, survivorID string) error {
	sql := `
WITH ranked AS (
  SELECT ctid, ROW_NUMBER() OVER (PARTITION BY source_id, target_id, relation_type ORDER BY created_at) AS rn
  FROM card_edges
  WHERE source_id = ? OR target_id = ?
)
DELETE FROM card_edges
WHERE ctid IN (SELECT ctid FROM ranked WHERE rn > 1);
`
	return tx.Exec(sql, survivorID, survivorID).Error
}

// removeSequenceSelfLoops deletes sequence self-loops for the given survivor
// while preserving reference self-loops (valid for [[Survivor]] references).
func removeSequenceSelfLoops(tx *gorm.DB, survivorID string) error {
	return tx.Where("source_id = ? AND target_id = ? AND relation_type = ?",
		survivorID, survivorID, "sequence").Delete(&models.CardEdge{}).Error
}
