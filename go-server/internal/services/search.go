package services

import (
	"context"
	"errors"

	"gorm.io/gorm"
)

type SearchResult struct {
	ID      string  `json:"id"`
	Title   string  `json:"title"`
	Excerpt string  `json:"excerpt"`
	Rank    float64 `json:"rank"`
}

type SearchService struct {
	db *gorm.DB
}

func NewSearchService(db *gorm.DB) *SearchService {
	return &SearchService{db: db}
}

func (s *SearchService) SearchCards(ctx context.Context, query string, limit, offset int) ([]SearchResult, int, error) {
	if query == "" {
		return nil, 0, errors.New("search query cannot be empty")
	}

	if limit < 1 {
		limit = 20
	}
	if limit > 100 {
		limit = 100
	}
	if offset < 0 {
		offset = 0
	}

	var total int64
	err := s.db.WithContext(ctx).Raw(`SELECT count(*) FROM cards WHERE search_vector @@ plainto_tsquery('simple', ?)`, query).Scan(&total).Error
	if err != nil {
		return nil, 0, err
	}

	var results []SearchResult
	err = s.db.WithContext(ctx).Raw(`
		SELECT 
			id, 
			title, 
			excerpt, 
			ts_rank(search_vector, plainto_tsquery('simple', ?)) AS rank
		FROM cards
		WHERE search_vector @@ plainto_tsquery('simple', ?)
		ORDER BY rank DESC
		LIMIT ? OFFSET ?
	`, query, query, limit, offset).Scan(&results).Error
	if err != nil {
		return nil, 0, err
	}

	return results, int(total), nil
}
