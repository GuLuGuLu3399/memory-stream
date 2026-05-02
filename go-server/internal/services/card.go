package services

import (
	"context"
	"fmt"
	"strings"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type CardService struct {
	db *gorm.DB
}

func NewCardService(db *gorm.DB) *CardService {
	return &CardService{db: db}
}

func (s *CardService) ListCards(ctx context.Context, category string, limit, offset int) ([]models.CardListItem, error) {
	if limit <= 0 || limit > 100 {
		limit = 20
	}

	q := s.db.WithContext(ctx).
		Model(&models.Card{}).
		Select("uuid, title, category, excerpt, version, updated_at")

	if category != "" {
		q = q.Where("category = ?", category)
	}

	var items []models.CardListItem
	err := q.Order("updated_at DESC").
		Limit(limit).
		Offset(offset).
		Find(&items).Error
	if err != nil {
		return nil, fmt.Errorf("failed to list cards: %w", err)
	}
	return items, nil
}

func (s *CardService) ListCategories(ctx context.Context) ([]models.CategoryItem, error) {
	var items []models.CategoryItem
	err := s.db.WithContext(ctx).
		Model(&models.Card{}).
		Select("category as name, COUNT(*) as count").
		Where("deleted_at IS NULL AND category != ''").
		Group("category").
		Order("category").
		Find(&items).Error
	if err != nil {
		return nil, fmt.Errorf("failed to list categories: %w", err)
	}
	return items, nil
}

func (s *CardService) GetRandomCards(ctx context.Context, count int) ([]models.CardListItem, error) {
	if count <= 0 || count > 20 {
		count = 5
	}

	var items []models.CardListItem
	err := s.db.WithContext(ctx).
		Model(&models.Card{}).
		Select("uuid, title, category, excerpt, version, updated_at").
		Order("RANDOM()").
		Limit(count).
		Find(&items).Error
	if err != nil {
		return nil, fmt.Errorf("failed to get random cards: %w", err)
	}
	return items, nil
}

func (s *CardService) ResolveByTitle(ctx context.Context, title string) (*uuid.UUID, error) {
	title = strings.TrimSpace(title)
	if title == "" {
		return nil, fmt.Errorf("title must not be empty")
	}
	var card models.Card
	if err := s.db.WithContext(ctx).
		Select("uuid").
		Where("title = ? AND deleted_at IS NULL", title).
		First(&card).Error; err != nil {
		return nil, fmt.Errorf("card not found by title %q: %w", title, err)
	}
	return &card.UUID, nil
}

func (s *CardService) GetBacklinks(ctx context.Context, rawUUID string) ([]models.BacklinkItem, error) {
	cardUUID, err := uuid.Parse(rawUUID)
	if err != nil {
		return nil, fmt.Errorf("invalid uuid: %w", err)
	}

	var items []models.BacklinkItem
	err = s.db.WithContext(ctx).
		Table("relations r").
		Select("c.uuid, c.title, r.relation_type").
		Joins("JOIN cards c ON c.uuid = r.source_uuid").
		Where("r.target_uuid = ? AND c.deleted_at IS NULL", cardUUID).
		Find(&items).Error
	if err != nil {
		return nil, fmt.Errorf("failed to get backlinks: %w", err)
	}
	return items, nil
}

func (s *CardService) GetCardRead(ctx context.Context, rawUUID string) (*models.Card, error) {
	cardUUID, err := uuid.Parse(rawUUID)
	if err != nil {
		return nil, fmt.Errorf("invalid uuid: %w", err)
	}

	var card models.Card
	if err := s.db.WithContext(ctx).Where("uuid = ?", cardUUID).First(&card).Error; err != nil {
		return nil, fmt.Errorf("card not found: %w", err)
	}
	return &card, nil
}
