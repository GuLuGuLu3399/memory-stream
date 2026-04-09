//go:build integration

package services

import (
	"testing"
	"time"

	"gorm.io/driver/sqlite"
	"gorm.io/gorm"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
)

// helper to prepare in-memory db and service
func newTestCategoryService(t *testing.T) *CategoryService {
	db, err := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{
		DisableForeignKeyConstraintWhenMigrating: true,
	})
	if err != nil {
		t.Fatalf("failed to connect sqlite: %v", err)
	}
	// migrate models
	if err := db.AutoMigrate(&models.Category{}); err != nil {
		t.Fatalf("auto migrate failed: %v", err)
	}
	// ensure CreatedAt is set for tests
	// seed a zero time for deterministic tests
	_ = time.Now()
	return &CategoryService{db: db}
}

func TestCircularReference(t *testing.T) {
	svc := newTestCategoryService(t)
	// Create A
	a, err := svc.Create("A", "root", nil, nil)
	if err != nil {
		t.Fatalf("create A failed: %v", err)
	}
	// Create B under A
	b, err := svc.Create("B", "child", nil, &a.ID)
	if err != nil {
		t.Fatalf("create B failed: %v", err)
	}
	// Create C under B
	c, err := svc.Create("C", "grandchild", nil, &b.ID)
	if err != nil {
		t.Fatalf("create C failed: %v", err)
	}
	// Attempt to move A under C (would create cycle)
	if err := svc.Update(a.ID, "A", "root updated", nil, &c.ID); err == nil {
		t.Fatalf("expected circular reference error, got nil")
	}
}

func TestDepthLimit(t *testing.T) {
	svc := newTestCategoryService(t)
	// create a chain of 5 ancestors
	root, err := svc.Create("R", "root", nil, nil)
	if err != nil {
		t.Fatalf("create root failed: %v", err)
	}
	p1, err := svc.Create("P1", "p1", nil, &root.ID)
	if err != nil {
		t.Fatalf("create p1 failed: %v", err)
	}
	p2, err := svc.Create("P2", "p2", nil, &p1.ID)
	if err != nil {
		t.Fatalf("create p2 failed: %v", err)
	}
	p3, err := svc.Create("P3", "p3", nil, &p2.ID)
	if err != nil {
		t.Fatalf("create p3 failed: %v", err)
	}
	p4, err := svc.Create("P4", "p4", nil, &p3.ID)
	if err != nil {
		t.Fatalf("create p4 failed: %v", err)
	}
	// Attempt to create one more level under P4, which would exceed depth 5
	if _, err := svc.Create("P5", "p5", nil, &p4.ID); err == nil {
		t.Fatalf("expected depth limit exceeded error, got nil")
	}
	// Ensure no panic on valid depth within limit: create under P4 with acceptable parent (root path shorter)
	// Create a sibling under P4 with a nil parent (not part of chain)
	if _, err := svc.Create("S", "sibling", nil, &root.ID); err != nil {
		t.Fatalf("unexpected error for valid depth: %v", err)
	}
}
