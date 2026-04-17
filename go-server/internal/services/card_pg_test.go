//go:build integration

package services

import (
	"fmt"
	"testing"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// ============================================================================
// CreateCard
// ============================================================================

func TestPG_CreateCard_HappyPath(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	card, err := svc.CreateCard(
		uniqueTitle(t, "CreateCard Happy"),
		"# Hello World\nThis is content.",
		"This is content.",
		models.JSONB(`{"type":"doc"}`),
		models.JSONB(`{"type":"toc"}`),
		nil,
	)
	require.NoError(t, err)
	assert.NotEmpty(t, card.ID)
	assert.Equal(t, "# Hello World\nThis is content.", card.RawMd)
	assert.Equal(t, "This is content.", card.Excerpt)
}

func TestPG_CreateCard_EmptyContent(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	card, err := svc.CreateCard(
		uniqueTitle(t, "Empty"),
		"", // empty content
		"",
		models.JSONB("{}"),
		models.JSONB("{}"),
		nil,
	)
	assert.Nil(t, card)
	assert.EqualError(t, err, "card content cannot be empty")
}

func TestPG_CreateCard_GhostCardUpsert(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	// Pre-create a ghost card (empty raw_md)
	ghost := seedCard(t, tx, uniqueTitle(t, "Ghost"), "")
	ghostID := ghost.ID

	// CreateCard with same title should upsert the ghost instead of creating new
	card, err := svc.CreateCard(
		ghost.Title,
		"Real content now",
		"Real content now",
		models.JSONB(`{"type":"doc"}`),
		models.JSONB(`{"type":"toc"}`),
		nil,
	)
	require.NoError(t, err)
	assert.Equal(t, ghostID, card.ID, "should reuse the ghost card ID")
	assert.Equal(t, "Real content now", card.RawMd)
}

func TestPG_CreateCard_GhostCardNotReusedIfTitleDiffers(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	// Pre-create a ghost card
	seedCard(t, tx, uniqueTitle(t, "Ghost A"), "")

	// CreateCard with a different title — should create a new card
	card, err := svc.CreateCard(
		uniqueTitle(t, "Ghost B"),
		"Content B",
		"Content B",
		models.JSONB("{}"),
		models.JSONB("{}"),
		nil,
	)
	require.NoError(t, err)
	assert.NotEmpty(t, card.ID)

	// Verify 2 cards exist
	var count int64
	tx.Model(&models.Card{}).Where("title LIKE ?", "%[test:"+t.Name()+"]%").Count(&count)
	assert.Equal(t, int64(2), count)
}

func TestPG_CreateCard_WithCategory(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	cat := seedCategory(t, tx, uniqueTitle(t, "CreateCardCategory"), nil)

	card, err := svc.CreateCard(
		uniqueTitle(t, "CreateCard With Category"),
		"# Categorized\ncontent",
		"content",
		models.JSONB(`{"type":"doc"}`),
		models.JSONB(`{"type":"toc"}`),
		&cat.ID,
	)
	require.NoError(t, err)
	require.NotNil(t, card.CategoryID)
	assert.Equal(t, cat.ID, *card.CategoryID)
}

// ============================================================================
// GetCardByID
// ============================================================================

func TestPG_GetCardByID_HappyPath(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	created := seedCard(t, tx, uniqueTitle(t, "GetByID"), "# Content")

	// We also need metrics for the Preload test
	tx.Exec("INSERT INTO card_metrics (card_id, view_count, hot_score) VALUES (?, 42, 3.14)", created.ID)

	card, err := svc.GetCardByID(created.ID)
	require.NoError(t, err)
	assert.Equal(t, created.ID, card.ID)
	assert.Equal(t, created.Title, card.Title)
	require.NotNil(t, card.Metrics)
	assert.Equal(t, int64(42), card.Metrics.ViewCount)
}

func TestPG_GetCardByID_NotFound(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	card, err := svc.GetCardByID("00000000-0000-0000-0000-000000000000")
	assert.Nil(t, card)
	assert.Error(t, err)
}

func TestPG_GetCardByID_WithCategory(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	cat := seedCategory(t, tx, uniqueTitle(t, "CatForGetByID"), nil)
	card := seedCard(t, tx, uniqueTitle(t, "WithCat"), "content")
	tx.Model(card).Update("category_id", cat.ID)

	found, err := svc.GetCardByID(card.ID)
	require.NoError(t, err)
	require.NotNil(t, found.Category)
	assert.Equal(t, cat.Name, found.Category.Name)
}

// ============================================================================
// UpdateCard
// ============================================================================

func TestPG_UpdateCard_HappyPath(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	card := seedCard(t, tx, uniqueTitle(t, "ToUpdate"), "old content")

	err := svc.UpdateCard(
		card.ID,
		"Updated Title",
		"new content",
		"new excerpt",
		models.JSONB(`{"updated":true}`),
		models.JSONB(`{"updated":true}`),
		nil,
	)
	require.NoError(t, err)

	var updated models.Card
	tx.First(&updated, "id = ?", card.ID)
	assert.Equal(t, "Updated Title", updated.Title)
	assert.Equal(t, "new content", updated.RawMd)
	assert.Equal(t, "new excerpt", updated.Excerpt)
}

func TestPG_UpdateCard_WithCategory(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	card := seedCard(t, tx, uniqueTitle(t, "CatUpdate"), "content")
	cat := seedCategory(t, tx, uniqueTitle(t, "CatForUpdate"), nil)

	err := svc.UpdateCard(card.ID, "Title", "content", "excerpt",
		models.JSONB("{}"), models.JSONB("{}"), &cat.ID)
	require.NoError(t, err)

	var updated models.Card
	tx.First(&updated, "id = ?", card.ID)
	require.NotNil(t, updated.CategoryID)
	assert.Equal(t, cat.ID, *updated.CategoryID)
}

func TestPG_UpdateCard_NotFound(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	err := svc.UpdateCard(
		"00000000-0000-0000-0000-000000000000",
		"Ghost", "x", "x",
		models.JSONB("{}"), models.JSONB("{}"), nil,
	)
	// GORM Updates on non-existent row returns nil error but 0 rows affected
	assert.NoError(t, err)
}

func TestPG_UpdateCard_NullCategory(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	cat := seedCategory(t, tx, uniqueTitle(t, "ToUnlink"), nil)
	card := seedCard(t, tx, uniqueTitle(t, "UnlinkMe"), "content")
	tx.Model(card).Update("category_id", cat.ID)

	err := svc.UpdateCard(card.ID, "Unlinked", "content", "excerpt",
		models.JSONB("{}"), models.JSONB("{}"), nil)
	require.NoError(t, err)

	var updated models.Card
	tx.First(&updated, "id = ?", card.ID)
	assert.Nil(t, updated.CategoryID)
}

// ============================================================================
// DeleteCard
// ============================================================================

func TestPG_DeleteCard_HappyPath(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	card := seedCard(t, tx, uniqueTitle(t, "ToDelete"), "bye")

	// Create edges, layout, metrics — all should be cascade-deleted
	otherCard := seedCard(t, tx, uniqueTitle(t, "Other"), "other")
	seedEdge(t, tx, card.ID, otherCard.ID, "reference")
	seedEdge(t, tx, otherCard.ID, card.ID, "sequence")
	tx.Exec("INSERT INTO card_layouts (card_id, x, y) VALUES (?, 10, 20)", card.ID)
	tx.Exec("INSERT INTO card_metrics (card_id, view_count, hot_score) VALUES (?, 5, 1.0)", card.ID)

	err := svc.DeleteCard(card.ID)
	require.NoError(t, err)

	assertCardExists(t, tx, card.ID, false)

	// Verify edges are gone
	assertEdgeExists(t, tx, card.ID, otherCard.ID, "reference", false)
	assertEdgeExists(t, tx, otherCard.ID, card.ID, "sequence", false)

	// Verify layout is gone
	var layoutCount int64
	tx.Model(&models.CardLayout{}).Where("card_id = ?", card.ID).Count(&layoutCount)
	assert.Equal(t, int64(0), layoutCount)

	// Verify metrics are gone
	var metricsCount int64
	tx.Model(&models.CardMetrics{}).Where("card_id = ?", card.ID).Count(&metricsCount)
	assert.Equal(t, int64(0), metricsCount)

	// Verify other card is untouched
	assertCardExists(t, tx, otherCard.ID, true)
}

func TestPG_DeleteCard_NotFound(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	// Deleting non-existent card should not error (GORM DELETE WHERE is idempotent)
	err := svc.DeleteCard("00000000-0000-0000-0000-000000000000")
	assert.NoError(t, err)
}

func TestPG_DeleteCard_WithBidirectionalEdges(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	A := seedCard(t, tx, uniqueTitle(t, "A"), "a")
	B := seedCard(t, tx, uniqueTitle(t, "B"), "b")
	C := seedCard(t, tx, uniqueTitle(t, "C"), "c")

	// A → B, B → C, C → A (cycle)
	seedEdge(t, tx, A.ID, B.ID, "reference")
	seedEdge(t, tx, B.ID, C.ID, "reference")
	seedEdge(t, tx, C.ID, A.ID, "reference")

	err := svc.DeleteCard(B.ID)
	require.NoError(t, err)

	// B gone
	assertCardExists(t, tx, B.ID, false)
	// A and C still exist
	assertCardExists(t, tx, A.ID, true)
	assertCardExists(t, tx, C.ID, true)
	// All edges involving B are gone
	assertEdgeExists(t, tx, A.ID, B.ID, "reference", false)
	assertEdgeExists(t, tx, B.ID, C.ID, "reference", false)
	// C → A still exists (not involving B)
	assertEdgeExists(t, tx, C.ID, A.ID, "reference", true)
}

// ============================================================================
// IncrementView
// ============================================================================

func TestPG_IncrementView_HappyPath(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	card := seedCard(t, tx, uniqueTitle(t, "ViewCount"), "content")

	// First increment
	err := svc.IncrementView(card.ID)
	require.NoError(t, err)

	var m models.CardMetrics
	tx.First(&m, "card_id = ?", card.ID)
	assert.Equal(t, int64(1), m.ViewCount)

	// Second increment
	err = svc.IncrementView(card.ID)
	require.NoError(t, err)

	tx.First(&m, "card_id = ?", card.ID)
	assert.Equal(t, int64(2), m.ViewCount)
	assert.Greater(t, m.HotScore, 0.0)
}

func TestPG_IncrementView_RootResolution(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	// Create cards without incoming edges (orphan)
	orphan := seedCard(t, tx, uniqueTitle(t, "Orphan"), "content")

	// "root" should resolve to the orphan card (no incoming edges, earliest)
	err := svc.IncrementView("root")
	require.NoError(t, err)

	var m models.CardMetrics
	tx.First(&m, "card_id = ?", orphan.ID)
	assert.Equal(t, int64(1), m.ViewCount)
}

func TestPG_IncrementView_NonExistentCard(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	// UPSERT on non-existent card_id should succeed (PG allows INSERT with any UUID)
	err := svc.IncrementView("00000000-0000-0000-0000-00000000dead")
	// card_metrics has FK constraint, so this should fail
	assert.Error(t, err)
}

// ============================================================================
// GetBacklinks
// ============================================================================

func TestPG_GetBacklinks_HappyPath(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	target := seedCard(t, tx, uniqueTitle(t, "Target"), "I am the target")
	source := seedCard(t, tx, uniqueTitle(t, "Source"), "Check out [["+target.Title+"]] for more info")

	seedEdge(t, tx, source.ID, target.ID, "reference")

	backlinks, err := svc.GetBacklinks(target.ID)
	require.NoError(t, err)
	require.Len(t, backlinks, 1)
	assert.Equal(t, source.ID, backlinks[0].SourceID)
	assert.Equal(t, source.Title, backlinks[0].SourceTitle)
	assert.Equal(t, "reference", backlinks[0].RelationType)
	assert.Contains(t, backlinks[0].ContextSnippet, "[["+target.Title+"]]")
}

func TestPG_GetBacklinks_NoBacklinks(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	card := seedCard(t, tx, uniqueTitle(t, "Lonely"), "no one links to me")

	backlinks, err := svc.GetBacklinks(card.ID)
	require.NoError(t, err)
	assert.Empty(t, backlinks)
}

func TestPG_GetBacklinks_TargetNotFound(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	backlinks, err := svc.GetBacklinks("00000000-0000-0000-0000-000000000000")
	assert.Error(t, err)
	assert.Nil(t, backlinks)
}

func TestPG_GetBacklinks_MultipleSources(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	target := seedCard(t, tx, uniqueTitle(t, "MultiTarget"), "target content")
	src1 := seedCard(t, tx, uniqueTitle(t, "Src1"), "See [["+target.Title+"]]")
	src2 := seedCard(t, tx, uniqueTitle(t, "Src2"), "Ref: [["+target.Title+"]]")
	src3 := seedCard(t, tx, uniqueTitle(t, "Src3"), "Also [["+target.Title+"]]")

	seedEdge(t, tx, src1.ID, target.ID, "reference")
	seedEdge(t, tx, src2.ID, target.ID, "reference")
	seedEdge(t, tx, src3.ID, target.ID, "sequence")

	backlinks, err := svc.GetBacklinks(target.ID)
	require.NoError(t, err)
	assert.Len(t, backlinks, 3)
}

// ============================================================================
// ListCards
// ============================================================================

func TestPG_ListCards_HappyPath(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	// Seed 5 cards with unique titles
	for i := 0; i < 5; i++ {
		seedCard(t, tx, uniqueTitle(t, fmt.Sprintf("ListItem-%d", i)), "content")
	}

	result, err := svc.ListCards(CursorPage{Limit: 3})
	require.NoError(t, err)
	assert.GreaterOrEqual(t, result.TotalCount, int64(5))
	assert.True(t, result.HasMore)
	assert.Len(t, result.Data, 3)
	assert.NotEmpty(t, result.NextCursor)
}

func TestPG_ListCards_CursorPagination(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	for i := 0; i < 5; i++ {
		seedCard(t, tx, uniqueTitle(t, fmt.Sprintf("PageItem-%d", i)), "content")
	}

	// Page 1
	page1, err := svc.ListCards(CursorPage{Limit: 2})
	require.NoError(t, err)
	assert.True(t, page1.HasMore)
	assert.NotEmpty(t, page1.NextCursor)

	// Page 2 using cursor — should return different cards than page 1
	page2, err := svc.ListCards(CursorPage{Limit: 2, Cursor: page1.NextCursor})
	require.NoError(t, err)
	assert.NotEmpty(t, page2.Data)

	// Verify pages don't overlap
	p1IDs := make(map[string]bool)
	for _, c := range page1.Data.([]models.Card) {
		p1IDs[c.ID] = true
	}
	for _, c := range page2.Data.([]models.Card) {
		assert.False(t, p1IDs[c.ID], "page2 should not contain cards from page1")
	}
}

func TestPG_ListCards_PaginationWorks(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	// Seed cards and verify pagination mechanics
	for i := 0; i < 3; i++ {
		seedCard(t, tx, uniqueTitle(t, fmt.Sprintf("Pag-%d", i)), "content")
	}

	result, err := svc.ListCards(CursorPage{Limit: 10})
	require.NoError(t, err)
	assert.GreaterOrEqual(t, result.TotalCount, int64(3))
	assert.False(t, result.HasMore) // 3 cards, limit 10
}

func TestPG_ListCards_DefaultLimit(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	for i := 0; i < 25; i++ {
		seedCard(t, tx, uniqueTitle(t, fmt.Sprintf("DefaultLimit-%d", i)), "c")
	}

	// Limit=0 should default to 20
	result, err := svc.ListCards(CursorPage{Limit: 0})
	require.NoError(t, err)
	assert.Len(t, result.Data, 20)
}

// ============================================================================
// GetDiscover — orphan cards (no edges)
// ============================================================================

func TestPG_GetDiscover_OrphanCards(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	// Orphan cards (no edges)
	orphan1 := seedCard(t, tx, uniqueTitle(t, "Orphan1"), "alone")
	orphan2 := seedCard(t, tx, uniqueTitle(t, "Orphan2"), "lonely")

	// Connected cards (should NOT appear)
	connected1 := seedCard(t, tx, uniqueTitle(t, "Conn1"), "has edge")
	connected2 := seedCard(t, tx, uniqueTitle(t, "Conn2"), "has edge")
	seedEdge(t, tx, connected1.ID, connected2.ID, "reference")

	result, err := svc.GetDiscover("updated", OffsetPage{Page: 1, PageSize: 20})
	require.NoError(t, err)
	assert.Equal(t, int64(2), result.TotalCount)

	// Verify only orphans appear
	cards := result.Data.([]models.Card)
	titles := make(map[string]bool)
	for _, c := range cards {
		titles[c.ID] = true
	}
	assert.True(t, titles[orphan1.ID])
	assert.True(t, titles[orphan2.ID])
	assert.False(t, titles[connected1.ID])
	assert.False(t, titles[connected2.ID])
}

func TestPG_GetDiscover_Empty(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	result, err := svc.GetDiscover("updated", OffsetPage{Page: 1, PageSize: 10})
	require.NoError(t, err)
	assert.Equal(t, int64(0), result.TotalCount)
}

func TestPG_GetDiscover_HotSort(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	// Two orphans with different hot scores
	c1 := seedCard(t, tx, uniqueTitle(t, "LowHot"), "lo")
	c2 := seedCard(t, tx, uniqueTitle(t, "HighHot"), "hi")

	tx.Exec("INSERT INTO card_metrics (card_id, view_count, hot_score) VALUES (?, 1, 1.0)", c1.ID)
	tx.Exec("INSERT INTO card_metrics (card_id, view_count, hot_score) VALUES (?, 100, 99.0)", c2.ID)

	result, err := svc.GetDiscover("hot", OffsetPage{Page: 1, PageSize: 10})
	require.NoError(t, err)
	cards := result.Data.([]models.Card)
	require.Len(t, cards, 2)
	// Higher hot score should come first
	assert.Equal(t, c2.ID, cards[0].ID)
}

// ============================================================================
// FindOrCreateByTitle — real PG tests
// ============================================================================

func TestPG_FindOrCreateByTitle_HappyPath(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	title := uniqueTitle(t, "FoC")

	// First call creates ghost
	card, err := svc.FindOrCreateByTitle(title)
	require.NoError(t, err)
	assert.NotEmpty(t, card.ID)
	assert.Equal(t, title, card.Title)
	assert.Equal(t, "", card.RawMd) // ghost card

	// Second call returns same card
	card2, err := svc.FindOrCreateByTitle(title)
	require.NoError(t, err)
	assert.Equal(t, card.ID, card2.ID)
}

func TestPG_FindOrCreateByTitle_ExistingRealCard(t *testing.T) {
	tx := testTx(t)
	svc := &CardService{db: tx}

	existing := seedCard(t, tx, uniqueTitle(t, "Existing"), "real content")

	found, err := svc.FindOrCreateByTitle(existing.Title)
	require.NoError(t, err)
	assert.Equal(t, existing.ID, found.ID)
	assert.Equal(t, "real content", found.RawMd)
}

// ============================================================================
// Concurrency — concurrent IncrementView on same card
// ============================================================================

func TestPG_IncrementView_Concurrent(t *testing.T) {
	db := ensureTestDB(t)
	svc := &CardService{db: db} // Use raw db, not tx (concurrent goroutines need separate connections)

	card := &models.Card{
		Title:   uniqueTitle(t, "Concurrent"),
		RawMd:   "concurrent",
		Excerpt: "concurrent",
		AstData: models.JSONB("{}"),
	}
	require.NoError(t, db.Create(card).Error)
	t.Cleanup(func() {
		db.Where("title = ?", card.Title).Delete(&models.Card{})
		db.Where("card_id = ?", card.ID).Delete(&models.CardMetrics{})
	})

	// Run 10 concurrent increments
	done := make(chan error, 10)
	for i := 0; i < 10; i++ {
		go func() {
			done <- svc.IncrementView(card.ID)
		}()
	}

	for i := 0; i < 10; i++ {
		assert.NoError(t, <-done)
	}

	// Verify final count is exactly 10 (UPSERT should handle concurrency)
	var m models.CardMetrics
	require.NoError(t, db.First(&m, "card_id = ?", card.ID).Error)
	assert.Equal(t, int64(10), m.ViewCount)
}
