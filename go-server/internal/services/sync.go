package services

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type SyncService struct {
	db *gorm.DB
}

func NewSyncService(db *gorm.DB) *SyncService {
	return &SyncService{db: db}
}

// ── Pull: Incremental Manifest ─────────────────────────────────

func (s *SyncService) GetManifest(ctx context.Context, since time.Time) (models.SyncManifestResponse, error) {
	items := make([]models.SyncManifestItem, 0)
	err := s.db.WithContext(ctx).
		Model(&models.Card{}).
		Select("uuid, version, hash, updated_at, deleted_at IS NOT NULL AS is_deleted").
		Where("updated_at >= ?", since.UTC()).
		Order("updated_at ASC").
		Scan(&items).Error
	if err != nil {
		return models.SyncManifestResponse{}, fmt.Errorf("failed to get manifest: %w", err)
	}

	cursor := since.UTC()
	if len(items) > 0 {
		cursor = items[len(items)-1].UpdatedAt.UTC().Add(time.Microsecond)
	}

	return models.SyncManifestResponse{
		Cursor:  cursor,
		Changes: items,
	}, nil
}

// ── Push: Batch Upsert ─────────────────────────────────────────

func (s *SyncService) BatchUpsert(ctx context.Context, req models.SyncBatchRequest) (models.SyncBatchResponse, error) {
	resp := models.SyncBatchResponse{
		Accepted:  make([]models.SyncAcceptedItem, 0),
		Conflicts: make([]models.SyncConflict, 0),
		Rejected:  make([]models.SyncReject, 0),
	}

	err := s.db.WithContext(ctx).Transaction(func(tx *gorm.DB) error {
		for _, card := range req.Cards {
			cardUUID, err := uuid.Parse(card.UUID)
			if err != nil {
				resp.Rejected = append(resp.Rejected, models.SyncReject{
					UUID:   card.UUID,
					Reason: "invalid uuid",
				})
				continue
			}

			// LWW check: fetch server version
			var serverCard models.Card
			result := tx.Where("uuid = ?", cardUUID).First(&serverCard)

			if result.Error != nil && !errors.Is(result.Error, gorm.ErrRecordNotFound) {
				return fmt.Errorf("failed to query card %s: %w", card.UUID, result.Error)
			}

			isNew := errors.Is(result.Error, gorm.ErrRecordNotFound)

			if !isNew && card.Version < serverCard.Version {
				resp.Conflicts = append(resp.Conflicts, models.SyncConflict{
					UUID:          card.UUID,
					ServerVersion: serverCard.Version,
					ServerHash:    serverCard.Hash,
				})
				continue
			}

			// Upsert card
			tocData := card.TocData
			if tocData == nil {
				tocData = []byte("[]")
			}
			astData := card.AstData
			if astData == nil {
				astData = []byte("{}")
			}

			now := time.Now().UTC()
			upsert := models.Card{
				UUID:     cardUUID,
				Title:    card.Title,
				Category: card.Category,
				Content:  card.Content,
				AstData:  astData,
				TocData:  tocData,
				Excerpt:  card.Excerpt,
				Version:  card.Version,
				Hash:     card.Hash,
			}

			if err := tx.Where("uuid = ?", cardUUID).
				Assign(map[string]interface{}{
					"title":      upsert.Title,
					"category":   upsert.Category,
					"content":    upsert.Content,
					"ast_data":   upsert.AstData,
					"toc_data":   upsert.TocData,
					"excerpt":    upsert.Excerpt,
					"version":    upsert.Version,
					"hash":       upsert.Hash,
					"updated_at": now,
					"deleted_at": nil,
				}).
				FirstOrCreate(&upsert).Error; err != nil {
				return fmt.Errorf("failed to upsert card %s: %w", card.UUID, err)
			}

				// Rewrite link edges (trunks managed via /sync/relations)
				if err := tx.Where("source_uuid = ? AND relation_type = ?", cardUUID, models.RelationTypeLink).
					Delete(&models.Relation{}).Error; err != nil {
					return fmt.Errorf("failed to clear link relations for %s: %w", card.UUID, err)
				}

				for _, edge := range card.Edges {
					if edge.RelationType != "link" {
						continue
					}
					targetUUID, err := uuid.Parse(edge.TargetUUID)
					if err != nil {
						continue
					}
					rel := models.Relation{
						SourceUUID:   cardUUID,
						TargetUUID:   targetUUID,
						RelationType: models.RelationTypeLink,
					}
					if err := tx.Create(&rel).Error; err != nil {
						return fmt.Errorf("failed to create relation for %s: %w", card.UUID, err)
					}
				}

			// Audit log
			if err := tx.Exec(
				"INSERT INTO sync_change_log (card_uuid, op, client_version, client_hash) VALUES (?, 'upsert', ?, ?)",
				cardUUID, card.Version, card.Hash,
			).Error; err != nil {
				return fmt.Errorf("failed to log change for %s: %w", card.UUID, err)
			}

			// Fetch the actual updated_at (trigger may have overwritten it)
			var finalCard models.Card
			if err := tx.Select("updated_at").Where("uuid = ?", cardUUID).First(&finalCard).Error; err == nil {
				now = finalCard.UpdatedAt
			}

			resp.Accepted = append(resp.Accepted, models.SyncAcceptedItem{
				UUID:      card.UUID,
				Version:   card.Version,
				UpdatedAt: now,
			})
		}
		return nil
	})

	if err != nil {
		return models.SyncBatchResponse{}, err
	}

	return resp, nil
}

func (s *SyncService) DeleteCard(ctx context.Context, rawUUID string) error {
	cardUUID, err := uuid.Parse(rawUUID)
	if err != nil {
		return fmt.Errorf("invalid uuid: %w", err)
	}

	return s.db.WithContext(ctx).Transaction(func(tx *gorm.DB) error {
		result := tx.Model(&models.Card{}).Where("uuid = ?", cardUUID).Update("deleted_at", gorm.Expr("NOW()"))
		if result.Error != nil {
			return fmt.Errorf("failed to delete card: %w", result.Error)
		}
		if result.RowsAffected == 0 {
			return fmt.Errorf("card not found: %s", rawUUID)
		}

		if err := tx.Exec("INSERT INTO sync_change_log (card_uuid, op) VALUES (?, 'delete')", cardUUID).Error; err != nil {
			return fmt.Errorf("failed to log change: %w", err)
		}

		return nil
	})
}

func (s *SyncService) GetCard(ctx context.Context, rawUUID string) (*models.CardWithEdges, error) {
	cardUUID, err := uuid.Parse(rawUUID)
	if err != nil {
		return nil, fmt.Errorf("invalid uuid: %w", err)
	}

	var card models.Card
	if err := s.db.WithContext(ctx).Unscoped().Where("uuid = ?", cardUUID).First(&card).Error; err != nil {
		return nil, fmt.Errorf("card not found: %w", err)
	}

	var relations []models.Relation
	if err := s.db.WithContext(ctx).Where("source_uuid = ?", cardUUID).Find(&relations).Error; err != nil {
		return nil, fmt.Errorf("failed to query relations for %s: %w", rawUUID, err)
	}

	edges := make([]models.SyncEdgePayload, 0, len(relations))
	for _, r := range relations {
		edges = append(edges, models.SyncEdgePayload{
			TargetUUID:   r.TargetUUID.String(),
			RelationType: string(r.RelationType),
		})
	}

	return &models.CardWithEdges{
		UUID:       card.UUID.String(),
		Title:      card.Title,
		Category:   card.Category,
		Content:    card.Content,
		AstData:    card.AstData,
		TocData:    card.TocData,
		Excerpt:    card.Excerpt,
		Version:    card.Version,
		Hash:       card.Hash,
		CreatedAt:  card.CreatedAt,
		UpdatedAt:  card.UpdatedAt,
		Edges:      edges,
	}, nil
}

// SyncRelations replaces all trunk relations with the client-provided list.
// Idempotent: clears existing trunks then inserts the new set.
func (s *SyncService) SyncRelations(ctx context.Context, req models.RelationsSyncRequest) (models.RelationsSyncResponse, error) {
	accepted := 0
	err := s.db.WithContext(ctx).Transaction(func(tx *gorm.DB) error {
		// Collect all unique source UUIDs from the payload
		sourceSet := make(map[uuid.UUID]bool)
		for _, r := range req.Relations {
			src, err := uuid.Parse(r.SourceUUID)
			if err != nil {
				continue
			}
			sourceSet[src] = true
		}

		// Delete existing trunk relations for affected sources
		for src := range sourceSet {
			if err := tx.Where("source_uuid = ? AND relation_type = ?", src, models.RelationTypeTrunk).
				Delete(&models.Relation{}).Error; err != nil {
				return fmt.Errorf("failed to clear trunks for %s: %w", src, err)
			}
		}

		// Insert new relations
		for _, r := range req.Relations {
			src, err := uuid.Parse(r.SourceUUID)
			if err != nil {
				continue
			}
			tgt, err := uuid.Parse(r.TargetUUID)
			if err != nil {
				continue
			}
			relType := models.RelationTypeLink
			if r.RelationType == "trunk" {
				relType = models.RelationTypeTrunk
			} else if r.RelationType != "link" {
				continue
			}

			rel := models.Relation{
				SourceUUID:   src,
				TargetUUID:   tgt,
				RelationType: relType,
			}
			if err := tx.Create(&rel).Error; err != nil {
				return fmt.Errorf("failed to create relation %s→%s: %w", src, tgt, err)
			}
			accepted++
		}
		return nil
	})
	if err != nil {
		return models.RelationsSyncResponse{}, err
	}
	return models.RelationsSyncResponse{Accepted: accepted}, nil
}

// GetAllTrunks returns all trunk relations (for full trunk pull).
func (s *SyncService) GetAllTrunks(ctx context.Context) ([]models.Relation, error) {
	var trunks []models.Relation
	if err := s.db.WithContext(ctx).
		Where("relation_type = ?", models.RelationTypeTrunk).
		Find(&trunks).Error; err != nil {
		return nil, fmt.Errorf("failed to query trunks: %w", err)
	}
	return trunks, nil
}
