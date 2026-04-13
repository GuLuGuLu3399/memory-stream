package services

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"regexp"
	"strings"
	"time"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"
	"github.com/redis/go-redis/v9"
	"gorm.io/gorm"
)

type CursorPage struct {
	Cursor string
	Limit  int
}

type OffsetPage struct {
	Page     int
	PageSize int
}

type PaginatedResult struct {
	Data       interface{} `json:"data"`
	HasMore    bool        `json:"has_more"`
	NextCursor string      `json:"next_cursor,omitempty"`
	TotalCount int64       `json:"total_count"`
}

type CardService struct {
	db  *gorm.DB
	rdb *redis.Client
}

func NewCardService(db *gorm.DB, rdb *redis.Client) *CardService {
	return &CardService{db: db, rdb: rdb}
}

func (s *CardService) FindOrCreateByTitle(ctx context.Context, title string) (*models.Card, error) {
	var result models.Card

	if err := s.db.Where("title = ?", title).First(&result).Error; err == nil {
		return &result, nil
	}

	// Use raw SQL to match the partial unique index:
	//   idx_cards_title_unique ON cards (title) WHERE title IS NOT NULL AND title != ''
	// GORM's clause.OnConflict cannot express the WHERE clause needed for partial indexes.
	s.db.Exec(`INSERT INTO cards (title, raw_md, excerpt, ast_data, toc_data)
		VALUES (?, '', '', '{}', '{}')
		ON CONFLICT (title) WHERE title IS NOT NULL AND title != '' DO NOTHING`, title)

	if err := s.db.Where("title = ?", title).First(&result).Error; err != nil {
		return nil, err
	}
	return &result, nil
}

func (s *CardService) CreateCard(ctx context.Context, title string, rawMd string, excerpt string, astData models.JSONB, tocData models.JSONB) (*models.Card, error) {
	if rawMd == "" {
		return nil, errors.New("card content cannot be empty")
	}

	var card models.Card
	err := s.db.Transaction(func(tx *gorm.DB) error {
		if title != "" {
			var existing models.Card
			if err := tx.Where("title = ? AND raw_md = ''", title).First(&existing).Error; err == nil {
				existing.RawMd = rawMd
				existing.Excerpt = excerpt
				existing.AstData = astData
				existing.TocData = tocData
				if err := tx.Save(&existing).Error; err != nil {
					return err
				}
				card = existing
				return nil
			}
		}

		card = models.Card{
			Title:   title,
			RawMd:   rawMd,
			Excerpt: excerpt,
			AstData: astData,
			TocData: tocData,
		}
		return tx.Create(&card).Error
	})

	if err != nil {
		return nil, err
	}
	return &card, nil
}

func (s *CardService) GetCardByID(ctx context.Context, id string) (*models.Card, error) {
	cacheKey := fmt.Sprintf("card:detail:%s", id)

	if s.rdb != nil {
		cached, err := s.rdb.Get(ctx, cacheKey).Bytes()
		if err == nil {
			var card models.Card
			if json.Unmarshal(cached, &card) == nil {
				return &card, nil
			}
		}
	}

	var card models.Card
	if err := s.db.Preload("Metrics").Preload("Category").First(&card, "id = ?", id).Error; err != nil {
		return nil, err
	}

	if s.rdb != nil {
		data, err := json.Marshal(&card)
		if err == nil {
			s.rdb.Set(ctx, cacheKey, data, 24*time.Hour)
		}
	}

	return &card, nil
}

func (s *CardService) ListCards(ctx context.Context, page CursorPage) (*PaginatedResult, error) {
	if page.Limit < 1 || page.Limit > 100 {
		page.Limit = 20
	}

	var totalCount int64
	s.db.Model(&models.Card{}).Count(&totalCount)

	query := s.db.Select("id, title, excerpt, category_id, created_at, updated_at").
		Preload("Category", func(db *gorm.DB) *gorm.DB {
			return db.Select("id, name")
		}).
		Preload("Metrics", func(db *gorm.DB) *gorm.DB {
			return db.Select("card_id, view_count, hot_score")
		}).
		Order("updated_at DESC")

	if page.Cursor != "" {
		query = query.Where("updated_at < ?", page.Cursor)
	}

	var cards []models.Card
	if err := query.Limit(page.Limit + 1).Find(&cards).Error; err != nil {
		return nil, err
	}

	hasMore := len(cards) > page.Limit
	if hasMore {
		cards = cards[:page.Limit]
	}

	nextCursor := ""
	if hasMore && len(cards) > 0 {
		nextCursor = cards[len(cards)-1].UpdatedAt.Format("2006-01-02T15:04:05Z07:00")
	}

	return &PaginatedResult{
		Data:       cards,
		HasMore:    hasMore,
		NextCursor: nextCursor,
		TotalCount: totalCount,
	}, nil
}

func (s *CardService) GetDiscover(ctx context.Context, sort string, page OffsetPage) (*PaginatedResult, error) {
	if page.Page < 1 {
		page.Page = 1
	}
	if page.PageSize < 1 || page.PageSize > 100 {
		page.PageSize = 20
	}

	// LEFT JOIN 反连接：查找没有任何边关联的孤岛卡片（无入边也无出边）
	// 比 NOT EXISTS 子查询更高效，PostgreSQL 优化器可利用 hash/merge join
	islandJoins := func(db *gorm.DB) *gorm.DB {
		return db.
			Joins("LEFT JOIN card_edges ce1 ON ce1.source_id = cards.id").
			Joins("LEFT JOIN card_edges ce2 ON ce2.target_id = cards.id").
			Where("ce1.source_id IS NULL AND ce2.target_id IS NULL")
	}

	var totalCount int64
	islandJoins(s.db.Model(&models.Card{})).Count(&totalCount)

	query := s.db.Select("DISTINCT cards.id, cards.title, cards.excerpt, cards.category_id, cards.created_at, cards.updated_at")
	query = islandJoins(query)

	switch sort {
	case "hot":
		query = query.Select("DISTINCT cards.id, cards.title, cards.excerpt, cards.category_id, cards.created_at, cards.updated_at, COALESCE(card_metrics.hot_score, 0) as hot_score").
			Joins("LEFT JOIN card_metrics ON card_metrics.card_id = cards.id").
			Order("COALESCE(card_metrics.hot_score, 0) DESC NULLS LAST")
	default:
		query = query.Order("cards.updated_at DESC")
	}

	offset := (page.Page - 1) * page.PageSize
	var cards []models.Card
	err := query.
		Preload("Category", func(db *gorm.DB) *gorm.DB {
			return db.Select("id, name")
		}).
		Preload("Metrics", func(db *gorm.DB) *gorm.DB {
			return db.Select("card_id, view_count, hot_score")
		}).
		Offset(offset).Limit(page.PageSize).
		Find(&cards).Error
	if err != nil {
		return nil, err
	}

	return &PaginatedResult{
		Data:       cards,
		HasMore:    int64(offset+len(cards)) < totalCount,
		TotalCount: totalCount,
	}, nil
}

func (s *CardService) UpdateCard(ctx context.Context, id string, title string, rawMd string, excerpt string, astData models.JSONB, tocData models.JSONB, categoryID *uint) error {
	updates := map[string]interface{}{
		"title":    title,
		"raw_md":   rawMd,
		"excerpt":  excerpt,
		"ast_data": astData,
		"toc_data": tocData,
	}
	if categoryID != nil {
		updates["category_id"] = *categoryID
	} else {
		updates["category_id"] = nil
	}

	err := s.db.Model(&models.Card{}).Where("id = ?", id).Updates(updates).Error
	if err != nil {
		return fmt.Errorf("failed to update card %s: %w", id, err)
	}

	if s.rdb != nil {
		s.rdb.Del(ctx, fmt.Sprintf("card:detail:%s", id))
	}

	return nil
}

func (s *CardService) DeleteCard(ctx context.Context, id string) error {
	err := s.db.Transaction(func(tx *gorm.DB) error {
		if err := tx.Where("source_id = ? OR target_id = ?", id, id).Delete(&models.CardEdge{}).Error; err != nil {
			return err
		}
		if err := tx.Where("card_id = ?", id).Delete(&models.CardLayout{}).Error; err != nil {
			return err
		}
		if err := tx.Where("card_id = ?", id).Delete(&models.CardMetrics{}).Error; err != nil {
			return err
		}
		return tx.Where("id = ?", id).Delete(&models.Card{}).Error
	})
	if err != nil {
		return fmt.Errorf("failed to delete card %s: %w", id, err)
	}

	if s.rdb != nil {
		s.rdb.Del(ctx, fmt.Sprintf("card:detail:%s", id))
	}

	return nil
}

func (s *CardService) IncrementView(ctx context.Context, cardID string) error {
	if cardID == "root" {
		var realID string
		err := s.db.Raw(`
					SELECT c.id FROM cards c
					LEFT JOIN card_edges e ON c.id = e.target_id
					WHERE e.target_id IS NULL
					ORDER BY c.created_at LIMIT 1
				`).Scan(&realID).Error
		if err != nil || realID == "" {
			s.db.Raw(`SELECT id FROM cards ORDER BY created_at LIMIT 1`).Scan(&realID)
		}
		if realID == "" {
			return errors.New("knowledge base is empty")
		}
		cardID = realID
	}

	return s.db.Exec(`
		INSERT INTO card_metrics (card_id, view_count, hot_score, updated_at)
		VALUES (?, 1, 0.4, NOW())
		ON CONFLICT (card_id) DO UPDATE SET
			view_count = card_metrics.view_count + 1,
			hot_score = card_metrics.hot_score + 0.4,
			updated_at = NOW()
	`, cardID).Error
}

func (s *CardService) GetGraphWithCache(ctx context.Context, cardID string, depth int) (*GraphResult, error) {
	cacheKey := fmt.Sprintf("graph:detail:%s:%d", cardID, depth)

	if s.rdb != nil {
		cached, err := s.rdb.Get(ctx, cacheKey).Bytes()
		if err == nil {
			var result GraphResult
			if json.Unmarshal(cached, &result) == nil {
				return &result, nil
			}
		}
	}

	graphSvc := NewGraphService(s.db)
	result, err := graphSvc.GetGraph(ctx, cardID, depth)
	if err != nil {
		return nil, err
	}

	if s.rdb != nil {
		data, err := json.Marshal(result)
		if err == nil {
			s.rdb.Set(ctx, cacheKey, data, 1*time.Hour)
		}
	}

	return result, nil
}

// BacklinkItem 表示一条反向引用（其他卡片指向当前卡片）
type BacklinkItem struct {
	SourceID       string `json:"source_id"`
	SourceTitle    string `json:"source_title"`
	RelationType   string `json:"relation_type"`
	ContextSnippet string `json:"context_snippet"`
}

// extractContextSnippet finds the markdown block containing the wikilink to
// targetTitle, sanitizes it to plain text, and returns a clean context snippet.
//
// Block-level capture: instead of crude radius slicing, this walks the line
// structure to find the containing paragraph/heading/list-item block (bounded
// by blank lines), then strips all markdown syntax before truncating.
func extractContextSnippet(rawMd string, targetTitle string) string {
	wikilink := "[[" + targetTitle + "]]"
	pos := strings.Index(rawMd, wikilink)
	if pos == -1 {
		return ""
	}

	// Split into lines and locate which line contains the wikilink
	lines := strings.Split(rawMd, "\n")
	byteOffset := 0
	targetIdx := -1
	for i, line := range lines {
		if byteOffset <= pos && pos < byteOffset+len(line)+1 {
			targetIdx = i
			break
		}
		byteOffset += len(line) + 1
	}
	if targetIdx == -1 {
		return ""
	}

	// Walk up: expand to block start (stop at blank line)
	start := targetIdx
	for i := targetIdx - 1; i >= 0; i-- {
		if strings.TrimSpace(lines[i]) == "" {
			break
		}
		start = i
	}

	// Walk down: expand to block end (stop at blank line)
	end := targetIdx
	for i := targetIdx + 1; i < len(lines); i++ {
		if strings.TrimSpace(lines[i]) == "" {
			break
		}
		end = i
	}

	// Extract block content and sanitize
	block := strings.Join(lines[start:end+1], "\n")
	clean := sanitizeSnippet(block)

	// Truncate to max length at a word boundary
	const maxSnippetLen = 80
	runes := []rune(clean)
	if len(runes) > maxSnippetLen {
		cut := maxSnippetLen
		for cut > maxSnippetLen*2/3 {
			r := runes[cut-1]
			if r == ' ' || r == ',' || r == '，' || r == '。' || r == '、' {
				break
			}
			cut--
		}
		return strings.TrimSpace(string(runes[:cut])) + "..."
	}

	return clean
}

// sanitizeSnippet strips markdown syntax from a text block,
// returning clean plain text suitable for display as a context snippet.
func sanitizeSnippet(text string) string {
	// Remove fenced code blocks (```...```)
	text = regexp.MustCompile("(?s)```.*?```").ReplaceAllString(text, "")

	// Remove remaining code fence markers and inline backticks
	text = strings.ReplaceAll(text, "```", "")
	text = strings.ReplaceAll(text, "`", "")

	// Extract image alt text: ![alt](url) → alt
	text = regexp.MustCompile(`!\[([^\]]*)\]\([^)]*\)`).ReplaceAllString(text, "$1")

	// Extract link text: [text](url) → text
	text = regexp.MustCompile(`\[([^\]]*)\]\([^)]*\)`).ReplaceAllString(text, "$1")

	// Extract wikilink text: [[target]] → target
	text = regexp.MustCompile(`\[\[([^\]]+)\]\]`).ReplaceAllString(text, "$1")

	// Remove bold/italic markers: ***text***, **text**, *text*
	text = regexp.MustCompile(`\*{1,3}([^*]+)\*{1,3}`).ReplaceAllString(text, "$1")
	text = regexp.MustCompile(`_{1,3}([^_]+)_{1,3}`).ReplaceAllString(text, "$1")

	// Remove heading markers: # ## ### etc.
	text = regexp.MustCompile(`(?m)^#{1,6}\s+`).ReplaceAllString(text, "")

	// Remove blockquote markers: > text
	text = regexp.MustCompile(`(?m)^>\s?`).ReplaceAllString(text, "")

	// Remove horizontal rules: --- or ***
	text = regexp.MustCompile(`(?m)^[-*]{3,}\s*$`).ReplaceAllString(text, "")

	// Remove unordered list markers: - or * followed by space
	text = regexp.MustCompile(`(?m)^[-*]\s+`).ReplaceAllString(text, "")

	// Remove ordered list markers: 1. etc.
	text = regexp.MustCompile(`(?m)^\d+\.\s+`).ReplaceAllString(text, "")

	// Flatten newlines into spaces and collapse whitespace
	text = strings.ReplaceAll(text, "\n", " ")
	text = strings.Join(strings.Fields(text), " ")

	return strings.TrimSpace(text)
}

// GetBacklinks 获取所有指向当前卡片的边（反向引用）。
// 利用 idx_card_edges_target 索引高效查询。
func (s *CardService) GetBacklinks(ctx context.Context, cardID string) ([]BacklinkItem, error) {
	var targetCard models.Card
	if err := s.db.Select("title").First(&targetCard, "id = ?", cardID).Error; err != nil {
		return nil, err
	}

	type backlinkQuery struct {
		SourceID     string
		SourceTitle  string
		RelationType string
		RawMd        string
	}

	var queryResults []backlinkQuery
	err := s.db.Table("card_edges").
		Select("card_edges.source_id, cards.title as source_title, card_edges.relation_type, cards.raw_md").
		Joins("JOIN cards ON cards.id = card_edges.source_id").
		Where("card_edges.target_id = ?", cardID).
		Scan(&queryResults).Error
	if err != nil {
		return nil, err
	}

	results := make([]BacklinkItem, len(queryResults))
	for i, qr := range queryResults {
		results[i] = BacklinkItem{
			SourceID:       qr.SourceID,
			SourceTitle:    qr.SourceTitle,
			RelationType:   qr.RelationType,
			ContextSnippet: extractContextSnippet(qr.RawMd, targetCard.Title),
		}
	}

	return results, nil
}

func (s *CardService) InvalidateGraphCache(ctx context.Context, cardID string) {
	if s.rdb == nil {
		return
	}
	var cursor uint64
	for {
		keys, nextCursor, err := s.rdb.Scan(ctx, cursor, fmt.Sprintf("graph:detail:%s:*", cardID), 100).Result()
		if err != nil {
			logger.Log.Errorf("Redis SCAN error: %v", err)
			return
		}
		if len(keys) > 0 {
			s.rdb.Del(ctx, keys...)
		}
		cursor = nextCursor
		if cursor == 0 {
			break
		}
	}
}
