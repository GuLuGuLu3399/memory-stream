//go:build integration

package services

import (
	"testing"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

// helper to prepare in-memory db and EdgeService
func newTestEdgeService(t *testing.T) *EdgeService {
	db, err := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{
		DisableForeignKeyConstraintWhenMigrating: true,
	})
	if err != nil {
		t.Fatalf("failed to connect sqlite: %v", err)
	}
	if err := db.AutoMigrate(&models.CardEdge{}); err != nil {
		t.Fatalf("auto migrate failed: %v", err)
	}
	return NewEdgeService(db, nil)
}

// helper to fetch all reference edges for a source card
func getReferenceEdges(t *testing.T, svc *EdgeService, sourceID string) []string {
	var edges []models.CardEdge
	if err := svc.db.Where("source_id = ? AND relation_type = ?", sourceID, "reference").Find(&edges).Error; err != nil {
		t.Fatalf("failed to fetch reference edges: %v", err)
	}
	targetIDs := make([]string, len(edges))
	for i, e := range edges {
		targetIDs[i] = e.TargetID
	}
	return targetIDs
}

func TestSyncReferenceEdges_AddNew(t *testing.T) {
	svc := newTestEdgeService(t)
	sourceID := "source-card-123"
	targetIDs := []string{"id-A", "id-B"}

	err := svc.SyncReferenceEdges(sourceID, targetIDs)
	if err != nil {
		t.Fatalf("SyncReferenceEdges failed: %v", err)
	}

	// Verify 2 reference edges created
	edges := getReferenceEdges(t, svc, sourceID)
	if len(edges) != 2 {
		t.Fatalf("expected 2 edges, got %d", len(edges))
	}

	// Verify both target IDs are present
	edgeMap := make(map[string]bool)
	for _, id := range edges {
		edgeMap[id] = true
	}
	if !edgeMap["id-A"] || !edgeMap["id-B"] {
		t.Fatalf("expected edges to id-A and id-B, got %v", edges)
	}
}

func TestSyncReferenceEdges_RemoveOld(t *testing.T) {
	svc := newTestEdgeService(t)
	sourceID := "source-card-456"

	// Pre-create edges to id-A and id-B
	svc.db.Create(&models.CardEdge{
		SourceID:     sourceID,
		TargetID:     "id-A",
		RelationType: "reference",
	})
	svc.db.Create(&models.CardEdge{
		SourceID:     sourceID,
		TargetID:     "id-B",
		RelationType: "reference",
	})

	// Sync with only id-A (should remove id-B)
	err := svc.SyncReferenceEdges(sourceID, []string{"id-A"})
	if err != nil {
		t.Fatalf("SyncReferenceEdges failed: %v", err)
	}

	// Verify only id-A remains
	edges := getReferenceEdges(t, svc, sourceID)
	if len(edges) != 1 {
		t.Fatalf("expected 1 edge, got %d", len(edges))
	}
	if edges[0] != "id-A" {
		t.Fatalf("expected edge to id-A, got %s", edges[0])
	}
}

func TestSyncReferenceEdges_RemoveAll(t *testing.T) {
	svc := newTestEdgeService(t)
	sourceID := "source-card-789"

	// Pre-create reference edge to id-A
	svc.db.Create(&models.CardEdge{
		SourceID:     sourceID,
		TargetID:     "id-A",
		RelationType: "reference",
	})

	// Sync with empty target list (should delete all)
	err := svc.SyncReferenceEdges(sourceID, []string{})
	if err != nil {
		t.Fatalf("SyncReferenceEdges failed: %v", err)
	}

	// Verify no reference edges remain
	edges := getReferenceEdges(t, svc, sourceID)
	if len(edges) != 0 {
		t.Fatalf("expected 0 edges, got %d", len(edges))
	}
}

func TestSyncReferenceEdges_DoesNotTouchSequence(t *testing.T) {
	svc := newTestEdgeService(t)
	sourceID := "source-card-seq"

	// Pre-create both sequence and reference edges
	svc.db.Create(&models.CardEdge{
		SourceID:     sourceID,
		TargetID:     "id-seq",
		RelationType: "sequence",
	})
	svc.db.Create(&models.CardEdge{
		SourceID:     sourceID,
		TargetID:     "id-ref",
		RelationType: "reference",
	})

	// Sync with empty target list (should only remove reference, not sequence)
	err := svc.SyncReferenceEdges(sourceID, []string{})
	if err != nil {
		t.Fatalf("SyncReferenceEdges failed: %v", err)
	}

	// Verify no reference edges remain
	refEdges := getReferenceEdges(t, svc, sourceID)
	if len(refEdges) != 0 {
		t.Fatalf("expected 0 reference edges, got %d", len(refEdges))
	}

	// Verify sequence edge still exists
	var seqEdge models.CardEdge
	if err := svc.db.Where("source_id = ? AND relation_type = ?", sourceID, "sequence").First(&seqEdge).Error; err != nil {
		t.Fatalf("sequence edge should still exist, got error: %v", err)
	}
	if seqEdge.TargetID != "id-seq" {
		t.Fatalf("expected sequence edge to id-seq, got %s", seqEdge.TargetID)
	}
}

func TestSyncReferenceEdges_Deduplicates(t *testing.T) {
	svc := newTestEdgeService(t)
	sourceID := "source-card-dedup"

	// Sync with duplicate target IDs
	err := svc.SyncReferenceEdges(sourceID, []string{"id-A", "id-A"})
	if err != nil {
		t.Fatalf("SyncReferenceEdges failed: %v", err)
	}

	// Verify only 1 edge created
	edges := getReferenceEdges(t, svc, sourceID)
	if len(edges) != 1 {
		t.Fatalf("expected 1 edge (deduplicated), got %d", len(edges))
	}
	if edges[0] != "id-A" {
		t.Fatalf("expected edge to id-A, got %s", edges[0])
	}
}

func TestSyncReferenceEdges_NoChange(t *testing.T) {
	svc := newTestEdgeService(t)
	sourceID := "source-card-nochange"

	// Pre-create edges to id-A and id-B
	svc.db.Create(&models.CardEdge{
		SourceID:     sourceID,
		TargetID:     "id-A",
		RelationType: "reference",
	})
	svc.db.Create(&models.CardEdge{
		SourceID:     sourceID,
		TargetID:     "id-B",
		RelationType: "reference",
	})

	// Count total edges before
	var countBefore int64
	svc.db.Model(&models.CardEdge{}).Where("source_id = ?", sourceID).Count(&countBefore)

	// Sync with same target IDs (should not INSERT or DELETE)
	err := svc.SyncReferenceEdges(sourceID, []string{"id-A", "id-B"})
	if err != nil {
		t.Fatalf("SyncReferenceEdges failed: %v", err)
	}

	// Count total edges after (should be same)
	var countAfter int64
	svc.db.Model(&models.CardEdge{}).Where("source_id = ?", sourceID).Count(&countAfter)

	if countBefore != countAfter {
		t.Fatalf("expected no change in edge count, before=%d, after=%d", countBefore, countAfter)
	}

	// Verify both edges still exist
	edges := getReferenceEdges(t, svc, sourceID)
	if len(edges) != 2 {
		t.Fatalf("expected 2 edges, got %d", len(edges))
	}
}
