//go:build integration

package services

import (
	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
	"testing"
)

func newTestCardService(t *testing.T) *CardService {
	db, err := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{
		DisableForeignKeyConstraintWhenMigrating: true,
	})
	if err != nil {
		t.Fatalf("failed to connect sqlite: %v", err)
	}

	if err := db.AutoMigrate(&models.Card{}); err != nil {
		t.Fatalf("auto migrate failed: %v", err)
	}
	return &CardService{db: db}
}

func TestFindOrCreateByTitle_ExistingCard(t *testing.T) {
	svc := newTestCardService(t)

	existing := &models.Card{Title: "Existing Card", RawMd: "# hello"}
	if err := svc.db.Create(existing).Error; err != nil {
		t.Fatalf("failed to pre-create card: %v", err)
	}

	found, err := svc.FindOrCreateByTitle("Existing Card")
	if err != nil {
		t.Fatalf("FindOrCreateByTitle failed: %v", err)
	}
	if found == nil {
		t.Fatalf("expected a card, got nil")
	}
	if found.ID != existing.ID {
		t.Fatalf("expected same id, got %s vs %s", found.ID, existing.ID)
	}
}

func TestFindOrCreateByTitle_NewGhostCard(t *testing.T) {
	svc := newTestCardService(t)
	c, err := svc.FindOrCreateByTitle("Ghost Card")
	if err != nil {
		t.Fatalf("FindOrCreateByTitle failed: %v", err)
	}
	if c == nil {
		t.Fatalf("expected a card, got nil")
	}
	if c.Title != "Ghost Card" {
		t.Fatalf("unexpected title: %s", c.Title)
	}
	if c.RawMd != "" {
		t.Fatalf("expected ghost card to have empty content, got: %s", c.RawMd)
	}
}

func TestFindOrCreateByTitle_Idempotent(t *testing.T) {
	svc := newTestCardService(t)
	// first call creates ghost
	first, err := svc.FindOrCreateByTitle("Idempotent Card")
	if err != nil {
		t.Fatalf("first call failed: %v", err)
	}
	// second call should return the same card
	second, err := svc.FindOrCreateByTitle("Idempotent Card")
	if err != nil {
		t.Fatalf("second call failed: %v", err)
	}
	if first == nil || second == nil {
		t.Fatalf("nil card returned on one of the calls")
	}
	if first.ID != second.ID {
		t.Fatalf("id should be the same for idempotent calls: %s vs %s", first.ID, second.ID)
	}
}

func TestFindOrCreateByTitle_CaseSensitive(t *testing.T) {
	svc := newTestCardService(t)
	// pre-create one title with exact case
	exact := &models.Card{Title: "Test Card", RawMd: "content"}
	if err := svc.db.Create(exact).Error; err != nil {
		t.Fatalf("failed to pre-create exact card: %v", err)
	}

	// different case should create a ghost independently
	ghost, err := svc.FindOrCreateByTitle("test Card")
	if err != nil {
		t.Fatalf("ghost creation failed: %v", err)
	}
	if ghost == nil {
		t.Fatalf("expected a card, got nil")
	}
	if ghost.Title != "test Card" {
		t.Fatalf("case-sensitive mismatch, got: %s", ghost.Title)
	}
	if ghost.ID == exact.ID {
		t.Fatalf("ghost should have different id from existing card with same title but different case")
	}
}
