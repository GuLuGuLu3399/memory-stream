package services

import (
	"context"
	"fmt"
	"strings"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"gorm.io/gorm"
)

type SearchService struct {
	db *gorm.DB
}

func NewSearchService(db *gorm.DB) *SearchService {
	return &SearchService{db: db}
}

type SearchHit struct {
	UUID    string `json:"uuid"`
	Title   string `json:"title"`
	Excerpt string `json:"excerpt"`
}

func (s *SearchService) SearchCards(ctx context.Context, query string, limit, offset int) ([]SearchHit, error) {
	if query == "" {
		return nil, nil
	}
	if limit <= 0 || limit > 100 {
		limit = 20
	}

	pattern := "%" + escapeLike(query) + "%"

	var hits []SearchHit
	err := s.db.WithContext(ctx).
		Model(&models.Card{}).
		Select("uuid, title, excerpt").
		Where("deleted_at IS NULL AND (title ILIKE ? OR excerpt ILIKE ?)", pattern, pattern).
		Order("updated_at DESC").
		Limit(limit).
		Offset(offset).
		Find(&hits).Error
	if err != nil {
		return nil, fmt.Errorf("failed to search: %w", err)
	}
	return hits, nil
}

func escapeLike(s string) string {
	s = strings.ReplaceAll(s, `\`, `\\`)
	s = strings.ReplaceAll(s, `%`, `\%`)
	s = strings.ReplaceAll(s, `_`, `\_`)
	return s
}
