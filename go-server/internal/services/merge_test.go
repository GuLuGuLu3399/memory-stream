package services

import (
	"context"
	"testing"
	"time"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/stretchr/testify/assert"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

func setupMergeTestDB(t *testing.T) (*gorm.DB, sqlmock.Sqlmock) {
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

func TestMergeCards_ValidationError_SurvivorInVictims(t *testing.T) {
	db, _ := setupMergeTestDB(t)

	req := MergeRequest{
		SurvivorID: "card-1",
		VictimIDs:  []string{"card-1", "card-2"},
	}

	result, err := NewMergeService(db).Merge(context.Background(), req)
	assert.Error(t, err)
	assert.Equal(t, "survivor_id cannot be in victim_ids", err.Error())
	assert.Nil(t, result)
}

func TestMergeCards_CardNotFound(t *testing.T) {
	db, mock := setupMergeTestDB(t)

	mock.ExpectBegin()
	// Row-level lock: SELECT ... FOR UPDATE returns only 1 row (missing card-2, card-3)
	rows := sqlmock.NewRows([]string{"id", "title", "raw_md", "excerpt", "ast_data", "toc_data", "category_id", "created_at", "updated_at"}).
		AddRow("card-1", "Card 1", "content", "excerpt", "{}", "{}", nil, time.Now(), time.Now())
	mock.ExpectQuery(`SELECT \* FROM "cards" WHERE id IN`).
		WillReturnRows(rows)
	mock.ExpectRollback()

	req := MergeRequest{
		SurvivorID: "card-1",
		VictimIDs:  []string{"card-2", "card-3"},
	}

	result, err := NewMergeService(db).Merge(context.Background(), req)
	assert.Error(t, err)
	assert.Equal(t, "one or more card IDs not found", err.Error())
	assert.Nil(t, result)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestMergeCards_DBErrorOnRowLock(t *testing.T) {
	db, mock := setupMergeTestDB(t)

	mock.ExpectBegin()
	mock.ExpectQuery(`SELECT \* FROM "cards" WHERE id IN`).
		WillReturnError(gorm.ErrInvalidDB)
	mock.ExpectRollback()

	req := MergeRequest{
		SurvivorID: "card-1",
		VictimIDs:  []string{"card-2"},
	}

	result, err := NewMergeService(db).Merge(context.Background(), req)
	assert.Error(t, err)
	assert.Nil(t, result)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestMergeCards_Success_NoExistingEdges(t *testing.T) {
	db, mock := setupMergeTestDB(t)

	now := time.Now()

	mock.ExpectBegin()

	// Row lock: return all 3 cards
	lockRows := sqlmock.NewRows([]string{"id", "title", "raw_md", "excerpt", "ast_data", "toc_data", "category_id", "created_at", "updated_at"}).
		AddRow("survivor-1", "Survivor", "content", "excerpt", "{}", "{}", nil, now, now).
		AddRow("victim-1", "Victim 1", "content", "excerpt", "{}", "{}", nil, now, now).
		AddRow("victim-2", "Victim 2", "content", "excerpt", "{}", "{}", nil, now, now)
	mock.ExpectQuery(`SELECT \* FROM "cards" WHERE id IN`).
		WillReturnRows(lockRows)

	// Incoming edges to victims
	mock.ExpectQuery(`SELECT source_id, target_id, relation_type, created_at FROM "card_edges" WHERE target_id IN`).
		WillReturnRows(sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}))

	// Outgoing edges from victims
	mock.ExpectQuery(`SELECT source_id, target_id, relation_type, created_at FROM "card_edges" WHERE source_id IN`).
		WillReturnRows(sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}))

	// Delete all edges involving victims
	mock.ExpectExec(`DELETE FROM "card_edges" WHERE source_id IN`).
		WillReturnResult(sqlmock.NewResult(0, 0))

	// Existing survivor edges (none)
	mock.ExpectQuery(`SELECT \* FROM "card_edges" WHERE`).
		WillReturnRows(sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}))

	// Dedup edges (no edges to dedup)
	mock.ExpectExec(`WITH ranked AS`).
		WithArgs("survivor-1", "survivor-1").
		WillReturnResult(sqlmock.NewResult(0, 0))

	// Remove sequence self-loops
	mock.ExpectExec(`DELETE FROM "card_edges" WHERE source_id = .+ AND target_id = .+ AND relation_type = .+`).
		WithArgs("survivor-1", "survivor-1", "sequence").
		WillReturnResult(sqlmock.NewResult(0, 0))

	// Delete victim cards
	mock.ExpectExec(`DELETE FROM "cards" WHERE id IN`).
		WillReturnResult(sqlmock.NewResult(0, 2))

	mock.ExpectCommit()

	req := MergeRequest{
		SurvivorID: "survivor-1",
		VictimIDs:  []string{"victim-1", "victim-2"},
	}

	result, err := NewMergeService(db).Merge(context.Background(), req)
	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Equal(t, 2, result.NodesDeleted)
	assert.Equal(t, 0, result.EdgesMigrated)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestMergeCards_Success_WithIncomingEdges(t *testing.T) {
	db, mock := setupMergeTestDB(t)

	now := time.Now()

	mock.ExpectBegin()

	// Row lock
	lockRows := sqlmock.NewRows([]string{"id", "title", "raw_md", "excerpt", "ast_data", "toc_data", "category_id", "created_at", "updated_at"}).
		AddRow("survivor-1", "Survivor", "content", "excerpt", "{}", "{}", nil, now, now).
		AddRow("victim-1", "Victim", "content", "excerpt", "{}", "{}", nil, now, now)
	mock.ExpectQuery(`SELECT \* FROM "cards" WHERE id IN`).
		WillReturnRows(lockRows)

	// Incoming edges
	incomingRows := sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}).
		AddRow("external-1", "victim-1", "reference", now)
	mock.ExpectQuery(`SELECT source_id, target_id, relation_type, created_at FROM "card_edges" WHERE target_id IN`).
		WillReturnRows(incomingRows)

	// Outgoing edges
	mock.ExpectQuery(`SELECT source_id, target_id, relation_type, created_at FROM "card_edges" WHERE source_id IN`).
		WillReturnRows(sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}))

	// Delete all edges involving victims
	mock.ExpectExec(`DELETE FROM "card_edges" WHERE source_id IN`).
		WillReturnResult(sqlmock.NewResult(0, 1))

	// Existing survivor edges
	mock.ExpectQuery(`SELECT \* FROM "card_edges" WHERE`).
		WillReturnRows(sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}))

	// Insert migrated edge
	mock.ExpectExec(`INSERT INTO "card_edges"`).
		WithArgs("external-1", "survivor-1", "reference", sqlmock.AnyArg()).
		WillReturnResult(sqlmock.NewResult(0, 1))

	// Dedup
	mock.ExpectExec(`WITH ranked AS`).
		WithArgs("survivor-1", "survivor-1").
		WillReturnResult(sqlmock.NewResult(0, 0))

	// Remove self-loops
	mock.ExpectExec(`DELETE FROM "card_edges" WHERE source_id = .+ AND target_id = .+ AND relation_type = .+`).
		WithArgs("survivor-1", "survivor-1", "sequence").
		WillReturnResult(sqlmock.NewResult(0, 0))

	// Delete victims
	mock.ExpectExec(`DELETE FROM "cards" WHERE id IN`).
		WillReturnResult(sqlmock.NewResult(0, 1))

	mock.ExpectCommit()

	req := MergeRequest{
		SurvivorID: "survivor-1",
		VictimIDs:  []string{"victim-1"},
	}

	result, err := NewMergeService(db).Merge(context.Background(), req)
	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Equal(t, 1, result.NodesDeleted)
	assert.Equal(t, 1, result.EdgesMigrated)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestMergeCards_SelfLoopWarning_IncomingEdge(t *testing.T) {
	db, mock := setupMergeTestDB(t)

	now := time.Now()

	mock.ExpectBegin()

	lockRows := sqlmock.NewRows([]string{"id", "title", "raw_md", "excerpt", "ast_data", "toc_data", "category_id", "created_at", "updated_at"}).
		AddRow("survivor-1", "Survivor", "content", "excerpt", "{}", "{}", nil, now, now).
		AddRow("victim-1", "Victim", "content", "excerpt", "{}", "{}", nil, now, now)
	mock.ExpectQuery(`SELECT \* FROM "cards" WHERE id IN`).
		WillReturnRows(lockRows)

	// Incoming: survivor → victim (would become self-loop)
	incomingRows := sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}).
		AddRow("survivor-1", "victim-1", "reference", now)
	mock.ExpectQuery(`SELECT source_id, target_id, relation_type, created_at FROM "card_edges" WHERE target_id IN`).
		WillReturnRows(incomingRows)

	mock.ExpectQuery(`SELECT source_id, target_id, relation_type, created_at FROM "card_edges" WHERE source_id IN`).
		WillReturnRows(sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}))

	mock.ExpectExec(`DELETE FROM "card_edges" WHERE source_id IN`).
		WillReturnResult(sqlmock.NewResult(0, 1))

	// Existing survivor edges
	mock.ExpectQuery(`SELECT \* FROM "card_edges" WHERE`).
		WillReturnRows(sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}))

	// Dedup (no edges to dedup)
	mock.ExpectExec(`WITH ranked AS`).
		WithArgs("survivor-1", "survivor-1").
		WillReturnResult(sqlmock.NewResult(0, 0))

	mock.ExpectExec(`DELETE FROM "card_edges" WHERE source_id = .+ AND target_id = .+ AND relation_type = .+`).
		WithArgs("survivor-1", "survivor-1", "sequence").
		WillReturnResult(sqlmock.NewResult(0, 0))

	mock.ExpectExec(`DELETE FROM "cards" WHERE id IN`).
		WillReturnResult(sqlmock.NewResult(0, 1))

	mock.ExpectCommit()

	req := MergeRequest{
		SurvivorID: "survivor-1",
		VictimIDs:  []string{"victim-1"},
	}

	result, err := NewMergeService(db).Merge(context.Background(), req)
	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Equal(t, 1, result.NodesDeleted)
	assert.Equal(t, 0, result.EdgesMigrated)
	assert.Contains(t, result.Warnings, "skipped incoming self-loop edge")
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestMergeCards_TransactionError_OnIncomingQuery(t *testing.T) {
	db, mock := setupMergeTestDB(t)

	now := time.Now()

	mock.ExpectBegin()

	lockRows := sqlmock.NewRows([]string{"id", "title", "raw_md", "excerpt", "ast_data", "toc_data", "category_id", "created_at", "updated_at"}).
		AddRow("survivor-1", "Survivor", "content", "excerpt", "{}", "{}", nil, now, now).
		AddRow("victim-1", "Victim", "content", "excerpt", "{}", "{}", nil, now, now)
	mock.ExpectQuery(`SELECT \* FROM "cards" WHERE id IN`).
		WillReturnRows(lockRows)

	mock.ExpectQuery(`SELECT source_id, target_id, relation_type, created_at FROM "card_edges" WHERE target_id IN`).
		WillReturnError(gorm.ErrInvalidDB)

	mock.ExpectRollback()

	req := MergeRequest{
		SurvivorID: "survivor-1",
		VictimIDs:  []string{"victim-1"},
	}

	result, err := NewMergeService(db).Merge(context.Background(), req)
	assert.Error(t, err)
	assert.Nil(t, result)
	assert.NoError(t, mock.ExpectationsWereMet())
}
