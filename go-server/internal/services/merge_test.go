package services

import (
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

	result, err := MergeCards(db, req)
	assert.Error(t, err)
	assert.Equal(t, "survivor_id cannot be in victim_ids", err.Error())
	assert.Nil(t, result)
}

func TestMergeCards_CardNotFound(t *testing.T) {
	db, mock := setupMergeTestDB(t)

	mock.ExpectQuery(`SELECT count\(\*\) FROM "cards" WHERE id IN`).
		WillReturnRows(sqlmock.NewRows([]string{"count"}).AddRow(1))

	req := MergeRequest{
		SurvivorID: "card-1",
		VictimIDs:  []string{"card-2", "card-3"},
	}

	result, err := MergeCards(db, req)
	assert.Error(t, err)
	assert.Equal(t, "one or more card IDs not found", err.Error())
	assert.Nil(t, result)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestMergeCards_CountDBError(t *testing.T) {
	db, mock := setupMergeTestDB(t)

	mock.ExpectQuery(`SELECT count\(\*\) FROM "cards" WHERE id IN`).
		WillReturnError(gorm.ErrInvalidDB)

	req := MergeRequest{
		SurvivorID: "card-1",
		VictimIDs:  []string{"card-2"},
	}

	result, err := MergeCards(db, req)
	assert.Error(t, err)
	assert.Nil(t, result)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestMergeCards_Success_NoExistingEdges(t *testing.T) {
	db, mock := setupMergeTestDB(t)

	mock.ExpectQuery(`SELECT count\(\*\) FROM "cards" WHERE id IN`).
		WillReturnRows(sqlmock.NewRows([]string{"count"}).AddRow(3))

	mock.ExpectBegin()

	mock.ExpectQuery(`SELECT source_id, target_id, relation_type, created_at FROM "card_edges" WHERE target_id IN`).
		WillReturnRows(sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}))

	mock.ExpectQuery(`SELECT source_id, target_id, relation_type, created_at FROM "card_edges" WHERE source_id IN`).
		WillReturnRows(sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}))

	mock.ExpectExec(`DELETE FROM "card_edges" WHERE source_id IN`).
		WillReturnResult(sqlmock.NewResult(0, 0))

	mock.ExpectExec(`WITH ranked AS`).
		WithArgs("survivor-1", "survivor-1").
		WillReturnResult(sqlmock.NewResult(0, 0))

	mock.ExpectExec(`DELETE FROM "card_edges" WHERE source_id = .+ AND target_id = .+ AND relation_type = .+`).
		WithArgs("survivor-1", "survivor-1", "sequence").
		WillReturnResult(sqlmock.NewResult(0, 0))

	mock.ExpectExec(`DELETE FROM "cards" WHERE id IN`).
		WillReturnResult(sqlmock.NewResult(0, 2))

	mock.ExpectCommit()

	req := MergeRequest{
		SurvivorID: "survivor-1",
		VictimIDs:  []string{"victim-1", "victim-2"},
	}

	result, err := MergeCards(db, req)
	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Equal(t, 2, result.NodesDeleted)
	assert.Equal(t, 0, result.EdgesMigrated)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestMergeCards_Success_WithIncomingEdges(t *testing.T) {
	db, mock := setupMergeTestDB(t)

	now := time.Now()

	mock.ExpectQuery(`SELECT count\(\*\) FROM "cards" WHERE id IN`).
		WillReturnRows(sqlmock.NewRows([]string{"count"}).AddRow(2))

	mock.ExpectBegin()

	incomingRows := sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}).
		AddRow("external-1", "victim-1", "reference", now)
	mock.ExpectQuery(`SELECT source_id, target_id, relation_type, created_at FROM "card_edges" WHERE target_id IN`).
		WillReturnRows(incomingRows)

	mock.ExpectQuery(`SELECT source_id, target_id, relation_type, created_at FROM "card_edges" WHERE source_id IN`).
		WillReturnRows(sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}))

	mock.ExpectExec(`DELETE FROM "card_edges" WHERE source_id IN`).
		WillReturnResult(sqlmock.NewResult(0, 1))

	mock.ExpectExec(`INSERT INTO "card_edges"`).
		WithArgs("external-1", "survivor-1", "reference", sqlmock.AnyArg()).
		WillReturnResult(sqlmock.NewResult(0, 1))

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

	result, err := MergeCards(db, req)
	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Equal(t, 1, result.NodesDeleted)
	assert.Equal(t, 1, result.EdgesMigrated)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestMergeCards_SelfLoopWarning_IncomingEdge(t *testing.T) {
	db, mock := setupMergeTestDB(t)

	now := time.Now()

	mock.ExpectQuery(`SELECT count\(\*\) FROM "cards" WHERE id IN`).
		WillReturnRows(sqlmock.NewRows([]string{"count"}).AddRow(2))

	mock.ExpectBegin()

	incomingRows := sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}).
		AddRow("survivor-1", "victim-1", "reference", now)
	mock.ExpectQuery(`SELECT source_id, target_id, relation_type, created_at FROM "card_edges" WHERE target_id IN`).
		WillReturnRows(incomingRows)

	mock.ExpectQuery(`SELECT source_id, target_id, relation_type, created_at FROM "card_edges" WHERE source_id IN`).
		WillReturnRows(sqlmock.NewRows([]string{"source_id", "target_id", "relation_type", "created_at"}))

	mock.ExpectExec(`DELETE FROM "card_edges" WHERE source_id IN`).
		WillReturnResult(sqlmock.NewResult(0, 1))

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

	result, err := MergeCards(db, req)
	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Equal(t, 1, result.NodesDeleted)
	assert.Equal(t, 0, result.EdgesMigrated)
	assert.Contains(t, result.Warnings, "skipped incoming self-loop edge")
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestMergeCards_TransactionError_OnIncomingQuery(t *testing.T) {
	db, mock := setupMergeTestDB(t)

	mock.ExpectQuery(`SELECT count\(\*\) FROM "cards" WHERE id IN`).
		WillReturnRows(sqlmock.NewRows([]string{"count"}).AddRow(2))

	mock.ExpectBegin()

	mock.ExpectQuery(`SELECT source_id, target_id, relation_type, created_at FROM "card_edges" WHERE target_id IN`).
		WillReturnError(gorm.ErrInvalidDB)

	mock.ExpectRollback()

	req := MergeRequest{
		SurvivorID: "survivor-1",
		VictimIDs:  []string{"victim-1"},
	}

	result, err := MergeCards(db, req)
	assert.Error(t, err)
	assert.Nil(t, result)
	assert.NoError(t, mock.ExpectationsWereMet())
}
