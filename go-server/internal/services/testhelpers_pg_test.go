//go:build integration

package services

import (
	"fmt"
	"os"
	"testing"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

// testDB is a singleton connection shared across all integration tests in this package.
// It is initialized once by ensureTestDB and reused.
var testDB *gorm.DB

// ensureTestDB lazily connects to the real PostgreSQL instance and migrates the schema.
// It reads DATABASE_URL from the environment (or falls back to the development default).
func ensureTestDB(t *testing.T) *gorm.DB {
	t.Helper()

	if testDB != nil {
		return testDB
	}

	dsn := os.Getenv("DATABASE_URL")
	if dsn == "" {
		dsn = "host=localhost user=root password=rootpassword dbname=devdb port=5432 sslmode=disable TimeZone=Asia/Shanghai"
	}

	var err error
	testDB, err = gorm.Open(postgres.Open(dsn), &gorm.Config{
		Logger:                 logger.Default.LogMode(logger.Silent),
		SkipDefaultTransaction: true,
	})
	if err != nil {
		t.Fatalf("failed to connect to PostgreSQL: %v", err)
	}

	// Schema is managed by migration files (001_schema.sql).
	// AutoMigrate conflicts with manually created constraints, so we skip it.
	// The test database is expected to have tables already created.

	return testDB
}

// testTx begins a transaction and registers a rollback on test cleanup.
// This ensures every test starts from a clean state without polluting the database.
// The returned *gorm.DB is the transaction handle — all writes within it are rolled back.
func testTx(t *testing.T) *gorm.DB {
	t.Helper()
	db := ensureTestDB(t)
	tx := db.Begin()
	if tx.Error != nil {
		t.Fatalf("failed to begin transaction: %v", tx.Error)
	}
	t.Cleanup(func() {
		tx.Rollback()
	})
	return tx
}

// seedCard creates a card with the given title and content inside the given tx.
// Returns the created card (with ID populated).
func seedCard(t *testing.T, tx *gorm.DB, title, rawMd string) *models.Card {
	t.Helper()
	card := &models.Card{
		Title:   title,
		RawMd:   rawMd,
		Excerpt: rawMd,
		AstData: models.JSONB("{}"),
		TocData: models.JSONB("{}"),
	}
	if err := tx.Create(card).Error; err != nil {
		t.Fatalf("seedCard failed: %v", err)
	}
	return card
}

// seedEdge creates a card edge inside the given tx.
func seedEdge(t *testing.T, tx *gorm.DB, sourceID, targetID, relationType string) {
	t.Helper()
	edge := &models.CardEdge{
		SourceID:     sourceID,
		TargetID:     targetID,
		RelationType: relationType,
	}
	if err := tx.Create(edge).Error; err != nil {
		t.Fatalf("seedEdge failed: %v", err)
	}
}

// seedCategory creates a category inside the given tx.
func seedCategory(t *testing.T, tx *gorm.DB, name string, parentID *uint) *models.Category {
	t.Helper()
	cat := &models.Category{
		Name:     name,
		ParentID: parentID,
	}
	if err := tx.Create(cat).Error; err != nil {
		t.Fatalf("seedCategory failed: %v", err)
	}
	return cat
}

// assertCardExists checks that a card with the given ID exists (or not) in the tx.
func assertCardExists(t *testing.T, tx *gorm.DB, cardID string, shouldExist bool) {
	t.Helper()
	var count int64
	tx.Model(&models.Card{}).Where("id = ?", cardID).Count(&count)
	if shouldExist && count == 0 {
		t.Fatalf("expected card %s to exist, but it doesn't", cardID)
	}
	if !shouldExist && count > 0 {
		t.Fatalf("expected card %s to NOT exist, but it does", cardID)
	}
}

// assertEdgeExists checks that an edge with given source/target/relation exists.
func assertEdgeExists(t *testing.T, tx *gorm.DB, sourceID, targetID, relationType string, shouldExist bool) {
	t.Helper()
	var count int64
	tx.Model(&models.CardEdge{}).
		Where("source_id = ? AND target_id = ? AND relation_type = ?", sourceID, targetID, relationType).
		Count(&count)
	if shouldExist && count == 0 {
		t.Fatalf("expected edge %s→%s (%s) to exist, but it doesn't", sourceID, targetID, relationType)
	}
	if !shouldExist && count > 0 {
		t.Fatalf("expected edge %s→%s (%s) to NOT exist, but it does", sourceID, targetID, relationType)
	}
}

// countEdges counts edges matching the given condition.
func countEdges(t *testing.T, tx *gorm.DB, query string, args ...interface{}) int64 {
	t.Helper()
	var count int64
	if err := tx.Model(&models.CardEdge{}).Where(query, args...).Count(&count).Error; err != nil {
		t.Fatalf("countEdges failed: %v", err)
	}
	return count
}

// uniqueTitle generates a unique card title using the test name + suffix.
func uniqueTitle(t *testing.T, suffix string) string {
	return fmt.Sprintf("[test:%s] %s", t.Name(), suffix)
}
