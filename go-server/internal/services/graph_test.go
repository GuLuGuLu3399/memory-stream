package services

import (
	"regexp"
	"testing"
	"time"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/stretchr/testify/assert"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

func setupGraphTestDB(t *testing.T) (*gorm.DB, sqlmock.Sqlmock) {
	t.Helper()
	sqlDB, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create sqlmock: %v", err)
	}

	dialector := postgres.New(postgres.Config{
		Conn: sqlDB,
	})
	db, err := gorm.Open(dialector, &gorm.Config{
		SkipDefaultTransaction: true,
	})
	if err != nil {
		t.Fatalf("failed to open gorm db: %v", err)
	}

	return db, mock
}

// --- resolveIdentifier tests ---

func TestResolveIdentifier_NonRoot(t *testing.T) {
	db, _ := setupGraphTestDB(t)
	svc := NewGraphService(db)

	// Non-root IDs pass through without any SQL
	id, err := svc.resolveIdentifier("some-card-id")
	assert.NoError(t, err)
	assert.Equal(t, "some-card-id", id)
}

func TestResolveIdentifier_Root_HasOrphanCard(t *testing.T) {
	db, mock := setupGraphTestDB(t)
	svc := NewGraphService(db)

	// First query: find card with no incoming edges
	rows := sqlmock.NewRows([]string{"id"}).AddRow("orphan-card-id")
	mock.ExpectQuery(`SELECT c.id FROM cards c`).
		WillReturnRows(rows)

	id, err := svc.resolveIdentifier("root")
	assert.NoError(t, err)
	assert.Equal(t, "orphan-card-id", id)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestResolveIdentifier_Root_NoOrphan_FallsBackToEarliest(t *testing.T) {
	db, mock := setupGraphTestDB(t)
	svc := NewGraphService(db)

	// First query returns empty (no orphan cards)
	mock.ExpectQuery(`SELECT c.id FROM cards c`).
		WillReturnRows(sqlmock.NewRows([]string{"id"}))

	// Fallback query: earliest card
	rows := sqlmock.NewRows([]string{"id"}).AddRow("earliest-card-id")
	mock.ExpectQuery(`SELECT id FROM cards ORDER BY created_at ASC LIMIT 1`).
		WillReturnRows(rows)

	id, err := svc.resolveIdentifier("root")
	assert.NoError(t, err)
	assert.Equal(t, "earliest-card-id", id)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestResolveIdentifier_Root_EmptyDatabase(t *testing.T) {
	db, mock := setupGraphTestDB(t)
	svc := NewGraphService(db)

	// First query returns empty
	mock.ExpectQuery(`SELECT c.id FROM cards c`).
		WillReturnRows(sqlmock.NewRows([]string{"id"}))

	// Fallback query also returns empty
	mock.ExpectQuery(`SELECT id FROM cards ORDER BY created_at ASC LIMIT 1`).
		WillReturnRows(sqlmock.NewRows([]string{"id"}))

	id, err := svc.resolveIdentifier("root")
	assert.Error(t, err)
	assert.Equal(t, "knowledge base is empty", err.Error())
	assert.Empty(t, id)
	assert.NoError(t, mock.ExpectationsWereMet())
}

// --- GetGraph tests ---

func TestGetGraph_NonRoot_EmptyResult(t *testing.T) {
	db, mock := setupGraphTestDB(t)
	svc := NewGraphService(db)

	// CTE query returns no edges
	mock.ExpectQuery(`WITH RECURSIVE reachable AS`).
		WithArgs("card-1", 2).
		WillReturnRows(sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}))

	mock.ExpectQuery(`SELECT id, title FROM "cards" WHERE id IN`).
		WithArgs("card-1").
		WillReturnRows(sqlmock.NewRows([]string{"id", "title"}).AddRow("card-1", "Test Card"))

	result, err := svc.GetGraph("card-1", 2)
	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Len(t, result.Nodes, 1)
	assert.Equal(t, "card-1", result.Nodes[0].ID)
	assert.Equal(t, "Test Card", result.Nodes[0].Title)
	assert.Empty(t, result.Edges)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGetGraph_DepthClamping(t *testing.T) {
	db, mock := setupGraphTestDB(t)
	svc := NewGraphService(db)

	// Requesting depth 10 should be clamped to 5
	mock.ExpectQuery(`WITH RECURSIVE reachable AS`).
		WithArgs("card-1", 5). // clamped from 10 to 5
		WillReturnRows(sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}))

	mock.ExpectQuery(`SELECT id, title FROM "cards" WHERE id IN`).
		WithArgs("card-1").
		WillReturnRows(sqlmock.NewRows([]string{"id", "title"}).AddRow("card-1", "Root"))

	result, err := svc.GetGraph("card-1", 10)
	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGetGraph_WithEdges(t *testing.T) {
	db, mock := setupGraphTestDB(t)
	svc := NewGraphService(db)

	now := time.Now()

	// CTE returns edges
	edgeRows := sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}).
		AddRow("card-1", "card-2", "reference", now).
		AddRow("card-2", "card-3", "sequence", now)
	mock.ExpectQuery(`WITH RECURSIVE reachable AS`).
		WithArgs("card-1", 2).
		WillReturnRows(edgeRows)

	// Card titles for 3 unique nodes
	titleRows := sqlmock.NewRows([]string{"id", "title"}).
		AddRow("card-1", "Card One").
		AddRow("card-2", "Card Two").
		AddRow("card-3", "Card Three")
	mock.ExpectQuery(`SELECT id, title FROM "cards" WHERE id IN`).
		WithArgs(sqlmock.AnyArg(), sqlmock.AnyArg(), sqlmock.AnyArg()).
		WillReturnRows(titleRows)

	result, err := svc.GetGraph("card-1", 2)
	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Len(t, result.Nodes, 3)
	assert.Len(t, result.Edges, 2)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGetGraph_CTEError(t *testing.T) {
	db, mock := setupGraphTestDB(t)
	svc := NewGraphService(db)

	mock.ExpectQuery(`WITH RECURSIVE reachable AS`).
		WithArgs("card-1", 2).
		WillReturnError(gorm.ErrInvalidDB)

	result, err := svc.GetGraph("card-1", 2)
	assert.Error(t, err)
	assert.Nil(t, result)
	assert.NoError(t, mock.ExpectationsWereMet())
}

// --- GetAllGraph tests ---

func TestGetAllGraph_Success(t *testing.T) {
	db, mock := setupGraphTestDB(t)
	svc := NewGraphService(db)

	// Cards query
	cardRows := sqlmock.NewRows([]string{"id", "title"}).
		AddRow("card-1", "Card One").
		AddRow("card-2", "Card Two")
	mock.ExpectQuery(regexp.QuoteMeta(`SELECT id, title FROM "cards"`)).
		WillReturnRows(cardRows)

	// Edges query — note: GetAllGraph does db.Find(&allEdges) which scans full table
	now := time.Now()
	edgeRows := sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}).
		AddRow("card-1", "card-2", "reference", now)
	mock.ExpectQuery(regexp.QuoteMeta(`SELECT * FROM "card_edges"`)).
		WillReturnRows(edgeRows)

	result, err := svc.GetAllGraph()
	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Len(t, result.Nodes, 2)
	assert.Len(t, result.Edges, 1)
	assert.Equal(t, "card-1", result.Edges[0].Source)
	assert.Equal(t, "card-2", result.Edges[0].Target)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGetAllGraph_CardsDBError(t *testing.T) {
	db, mock := setupGraphTestDB(t)
	svc := NewGraphService(db)

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT id, title FROM "cards"`)).
		WillReturnError(gorm.ErrInvalidDB)

	result, err := svc.GetAllGraph()
	assert.Error(t, err)
	assert.Nil(t, result)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGetAllGraph_EdgesDBError(t *testing.T) {
	db, mock := setupGraphTestDB(t)
	svc := NewGraphService(db)

	cardRows := sqlmock.NewRows([]string{"id", "title"}).
		AddRow("card-1", "Card One")
	mock.ExpectQuery(regexp.QuoteMeta(`SELECT id, title FROM "cards"`)).
		WillReturnRows(cardRows)

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT * FROM "card_edges"`)).
		WillReturnError(gorm.ErrInvalidDB)

	result, err := svc.GetAllGraph()
	assert.Error(t, err)
	assert.Nil(t, result)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGetAllGraph_EmptyDatabase(t *testing.T) {
	db, mock := setupGraphTestDB(t)
	svc := NewGraphService(db)

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT id, title FROM "cards"`)).
		WillReturnRows(sqlmock.NewRows([]string{"id", "title"}))

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT * FROM "card_edges"`)).
		WillReturnRows(sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}))

	result, err := svc.GetAllGraph()
	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Empty(t, result.Nodes)
	assert.Empty(t, result.Edges)
	assert.NoError(t, mock.ExpectationsWereMet())
}

// --- GetOutline tests ---

func TestGetOutline_AllCategories(t *testing.T) {
	db, mock := setupGraphTestDB(t)
	svc := NewGraphService(db)

	now := time.Now()

	// Categories query
	catRows := sqlmock.NewRows([]string{"id", "name", "description", "created_at", "parent_id", "sort_order", "theme_color"}).
		AddRow(1, "Go", "Go programming", now, nil, 0, nil).
		AddRow(2, "Rust", "Rust programming", now, nil, 0, nil)
	mock.ExpectQuery(regexp.QuoteMeta(`SELECT * FROM "categories"`)).
		WillReturnRows(catRows)

	// Card count query
	countRows := sqlmock.NewRows([]string{"category_id", "count"}).
		AddRow(1, 5).
		AddRow(2, 3)
	mock.ExpectQuery(`SELECT category_id, count\(id\) as count FROM "cards" GROUP BY`).
		WillReturnRows(countRows)

	// Recent cards query
	cardRows := sqlmock.NewRows([]string{"id", "title", "category_id", "created_at"}).
		AddRow("card-1", "Go Basics", uintPtr(1), now).
		AddRow("card-2", "Rust Intro", uintPtr(2), now)
	mock.ExpectQuery(regexp.QuoteMeta(
		`SELECT id, title, category_id, created_at FROM "cards" ORDER BY created_at DESC LIMIT $1`,
	)).WithArgs(50).WillReturnRows(cardRows)

	result, err := svc.GetOutline("")
	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Len(t, result.Topics, 2)
	assert.Equal(t, "1", result.Topics[0].ID)
	assert.Equal(t, "Go", result.Topics[0].Label)
	assert.Equal(t, 5, result.Topics[0].CardCount)
	assert.Len(t, result.Clusters, 2)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGetOutline_FilterByCategory(t *testing.T) {
	db, mock := setupGraphTestDB(t)
	svc := NewGraphService(db)

	now := time.Now()

	// Categories query with filter
	catRows := sqlmock.NewRows([]string{"id", "name", "description", "created_at", "parent_id", "sort_order", "theme_color"}).
		AddRow(1, "Go", "Go programming", now, nil, 0, nil)
	mock.ExpectQuery(regexp.QuoteMeta(`SELECT * FROM "categories" WHERE id = $1`)).
		WithArgs("cat-1").
		WillReturnRows(catRows)

	// Card count with filter
	countRows := sqlmock.NewRows([]string{"category_id", "count"}).
		AddRow(1, 5)
	mock.ExpectQuery(`SELECT category_id, count\(id\) as count FROM "cards" WHERE category_id = .+ GROUP BY`).
		WillReturnRows(countRows)

	// Recent cards with filter
	cardRows := sqlmock.NewRows([]string{"id", "title", "category_id", "created_at"}).
		AddRow("card-1", "Go Basics", uintPtr(1), now)
	mock.ExpectQuery(regexp.QuoteMeta(
		`SELECT id, title, category_id, created_at FROM "cards" WHERE category_id = $1 ORDER BY created_at DESC LIMIT $2`,
	)).WithArgs("cat-1", 50).WillReturnRows(cardRows)

	result, err := svc.GetOutline("cat-1")
	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Len(t, result.Topics, 1)
	assert.Len(t, result.Clusters, 1)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGetOutline_CategoriesDBError(t *testing.T) {
	db, mock := setupGraphTestDB(t)
	svc := NewGraphService(db)

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT * FROM "categories"`)).
		WillReturnError(gorm.ErrInvalidDB)

	result, err := svc.GetOutline("")
	assert.Error(t, err)
	assert.Nil(t, result)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGetOutline_CardsDBError(t *testing.T) {
	db, mock := setupGraphTestDB(t)
	svc := NewGraphService(db)

	now := time.Now()

	catRows := sqlmock.NewRows([]string{"id", "name", "description", "created_at", "parent_id", "sort_order", "theme_color"}).
		AddRow(1, "Go", "Go programming", now, nil, 0, nil)
	mock.ExpectQuery(regexp.QuoteMeta(`SELECT * FROM "categories"`)).
		WillReturnRows(catRows)

	mock.ExpectQuery(`SELECT category_id, count\(id\) as count FROM "cards" GROUP BY`).
		WillReturnRows(sqlmock.NewRows([]string{"category_id", "count"}))

	mock.ExpectQuery(regexp.QuoteMeta(
		`SELECT id, title, category_id, created_at FROM "cards" ORDER BY created_at DESC LIMIT $1`,
	)).WithArgs(50).WillReturnError(gorm.ErrInvalidDB)

	result, err := svc.GetOutline("")
	assert.Error(t, err)
	assert.Nil(t, result)
	assert.NoError(t, mock.ExpectationsWereMet())
}

// helper to create uint pointer
func uintPtr(v uint) *uint {
	return &v
}
