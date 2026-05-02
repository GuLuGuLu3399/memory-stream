package services

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"gorm.io/gorm"
)

type GlossaryService struct {
	db *gorm.DB
}

func NewGlossaryService(db *gorm.DB) *GlossaryService {
	return &GlossaryService{db: db}
}

func (s *GlossaryService) GetAll(ctx context.Context) ([]models.GlossaryItem, error) {
	var items []models.GlossaryItem
	err := s.db.WithContext(ctx).
		Where("deleted_at IS NULL").
		Order("term").
		Find(&items).Error
	if err != nil {
		return nil, fmt.Errorf("failed to get glossary: %w", err)
	}
	return items, nil
}

func (s *GlossaryService) GetAllSlim(ctx context.Context) (map[string]string, error) {
	type row struct {
		Term       string
		Definition string
	}
	var rows []row
	err := s.db.WithContext(ctx).
		Model(&models.GlossaryItem{}).
		Select("term, definition").
		Where("deleted_at IS NULL").
		Order("term").
		Find(&rows).Error
	if err != nil {
		return nil, fmt.Errorf("failed to get glossary slim: %w", err)
	}
	m := make(map[string]string, len(rows))
	for _, r := range rows {
		m[r.Term] = r.Definition
	}
	return m, nil
}

func (s *GlossaryService) GetManifest(ctx context.Context, since time.Time) (models.GlossarySyncManifest, error) {
	items := make([]models.GlossaryManifestItem, 0)
	err := s.db.WithContext(ctx).
		Model(&models.GlossaryItem{}).
		Select("term, version, hash, updated_at, deleted_at IS NOT NULL AS is_deleted").
		Where("updated_at >= ?", since.UTC()).
		Order("updated_at ASC").
		Scan(&items).Error
	if err != nil {
		return models.GlossarySyncManifest{}, fmt.Errorf("failed to get glossary manifest: %w", err)
	}

	cursor := since.UTC()
	if len(items) > 0 {
		cursor = items[len(items)-1].UpdatedAt.UTC().Add(time.Microsecond)
	}

	return models.GlossarySyncManifest{
		Cursor:  cursor,
		Changes: items,
	}, nil
}

func (s *GlossaryService) BatchUpsert(ctx context.Context, req models.GlossaryBatchRequest) (models.GlossaryBatchResponse, error) {
	resp := models.GlossaryBatchResponse{
		Accepted:  make([]string, 0),
		Conflicts: make([]models.GlossaryConflict, 0),
		Rejected:  make([]models.GlossaryReject, 0),
	}

	err := s.db.WithContext(ctx).Transaction(func(tx *gorm.DB) error {
		for _, item := range req.Items {
			if item.Term == "" {
				resp.Rejected = append(resp.Rejected, models.GlossaryReject{
					Term:   item.Term,
					Reason: "term is required",
				})
				continue
			}

			var server models.GlossaryItem
			result := tx.Where("term = ?", item.Term).First(&server)

			if result.Error != nil && !errors.Is(result.Error, gorm.ErrRecordNotFound) {
				return fmt.Errorf("failed to query glossary item %s: %w", item.Term, result.Error)
			}

			isNew := errors.Is(result.Error, gorm.ErrRecordNotFound)

			if !isNew && item.Version < server.Version {
				resp.Conflicts = append(resp.Conflicts, models.GlossaryConflict{
					Term:          item.Term,
					ServerVersion: server.Version,
					ServerHash:    server.Hash,
				})
				continue
			}

			now := time.Now().UTC()
			upsert := models.GlossaryItem{
				Term:       item.Term,
				Definition: item.Definition,
				Version:    item.Version,
				Hash:       item.Hash,
			}

			if err := tx.Where("term = ?", item.Term).
				Assign(map[string]interface{}{
					"definition": upsert.Definition,
					"version":    upsert.Version,
					"hash":       upsert.Hash,
					"updated_at": now,
					"deleted_at": nil,
				}).
				FirstOrCreate(&upsert).Error; err != nil {
				return fmt.Errorf("failed to upsert glossary item %s: %w", item.Term, err)
			}

			resp.Accepted = append(resp.Accepted, item.Term)
		}
		return nil
	})

	if err != nil {
		return models.GlossaryBatchResponse{}, err
	}

	return resp, nil
}
