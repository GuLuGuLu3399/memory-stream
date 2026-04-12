//go:build integration

package services

import (
	"context"
	"fmt"
	"math/rand"
	"sync"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// ──────────────────────────────────────────────
// CreateEdge
// ──────────────────────────────────────────────

func TestPG_CreateEdge_HappyPath_Sequence(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	a := seedCard(t, tx, uniqueTitle(t, "A"), "# A")
	b := seedCard(t, tx, uniqueTitle(t, "B"), "# B")

	err := svc.CreateEdge(context.Background(),a.ID, b.ID, "sequence")
	require.NoError(t, err)
	assertEdgeExists(t, tx, a.ID, b.ID, "sequence", true)
}

func TestPG_CreateEdge_HappyPath_Reference(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	a := seedCard(t, tx, uniqueTitle(t, "A"), "# A")
	b := seedCard(t, tx, uniqueTitle(t, "B"), "# B")

	err := svc.CreateEdge(context.Background(),a.ID, b.ID, "reference")
	require.NoError(t, err)
	assertEdgeExists(t, tx, a.ID, b.ID, "reference", true)
}

func TestPG_CreateEdge_EmptySourceID(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	err := svc.CreateEdge(context.Background(),"", "some-target", "sequence")
	assert.EqualError(t, err, "source_id and target_id are required")
}

func TestPG_CreateEdge_EmptyTargetID(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	err := svc.CreateEdge(context.Background(),"some-source", "", "sequence")
	assert.EqualError(t, err, "source_id and target_id are required")
}

func TestPG_CreateEdge_InvalidRelationType(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	err := svc.CreateEdge(context.Background(),"a", "b", "invalid")
	assert.EqualError(t, err, "relation_type must be 'sequence' or 'reference'")
}

func TestPG_CreateEdge_DuplicateEdge(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	a := seedCard(t, tx, uniqueTitle(t, "A"), "# A")
	b := seedCard(t, tx, uniqueTitle(t, "B"), "# B")

	require.NoError(t, svc.CreateEdge(context.Background(),a.ID, b.ID, "reference"))

	// Second create with same (source, target, relation) should fail on PK constraint
	err := svc.CreateEdge(context.Background(),a.ID, b.ID, "reference")
	assert.Error(t, err, "expected error on duplicate edge")
}

func TestPG_CreateEdge_SameSourceAndTarget(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	a := seedCard(t, tx, uniqueTitle(t, "A"), "# A")

	// DB does not enforce a no-self-loop constraint, so CreateEdge should succeed.
	err := svc.CreateEdge(context.Background(),a.ID, a.ID, "reference")
	require.NoError(t, err)
	assertEdgeExists(t, tx, a.ID, a.ID, "reference", true)
}

// ──────────────────────────────────────────────
// DeleteEdge
// ──────────────────────────────────────────────

func TestPG_DeleteEdge_HappyPath(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	a := seedCard(t, tx, uniqueTitle(t, "A"), "# A")
	b := seedCard(t, tx, uniqueTitle(t, "B"), "# B")
	seedEdge(t, tx, a.ID, b.ID, "reference")

	err := svc.DeleteEdge(context.Background(),a.ID, b.ID)
	require.NoError(t, err)
	assertEdgeExists(t, tx, a.ID, b.ID, "reference", false)
}

func TestPG_DeleteEdge_NonExistentEdge(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	// Deleting a non-existent edge should NOT error (GORM soft-pass)
	err := svc.DeleteEdge(context.Background(),"00000000-0000-0000-0000-000000000000", "11111111-1111-1111-1111-111111111111")
	assert.NoError(t, err)
}

func TestPG_DeleteEdge_OnlyDeletesMatchingEdge(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	a := seedCard(t, tx, uniqueTitle(t, "A"), "# A")
	b := seedCard(t, tx, uniqueTitle(t, "B"), "# B")
	c := seedCard(t, tx, uniqueTitle(t, "C"), "# C")
	seedEdge(t, tx, a.ID, b.ID, "reference")
	seedEdge(t, tx, a.ID, c.ID, "reference")

	err := svc.DeleteEdge(context.Background(),a.ID, b.ID)
	require.NoError(t, err)

	assertEdgeExists(t, tx, a.ID, b.ID, "reference", false)
	assertEdgeExists(t, tx, a.ID, c.ID, "reference", true)
}

// ──────────────────────────────────────────────
// UpdateEdgeType
// ──────────────────────────────────────────────

func TestPG_UpdateEdgeType_HappyPath(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	a := seedCard(t, tx, uniqueTitle(t, "A"), "# A")
	b := seedCard(t, tx, uniqueTitle(t, "B"), "# B")
	seedEdge(t, tx, a.ID, b.ID, "reference")

	err := svc.UpdateEdgeType(context.Background(),a.ID, b.ID, "sequence")
	require.NoError(t, err)
	assertEdgeExists(t, tx, a.ID, b.ID, "sequence", true)
	assertEdgeExists(t, tx, a.ID, b.ID, "reference", false)
}

func TestPG_UpdateEdgeType_EdgeNotFound(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	err := svc.UpdateEdgeType(context.Background(),"00000000-0000-0000-0000-000000000000", "11111111-1111-1111-1111-111111111111", "sequence")
	assert.EqualError(t, err, "edge not found")
}

func TestPG_UpdateEdgeType_InvalidRelationType(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	err := svc.UpdateEdgeType(context.Background(),"a", "b", "invalid")
	assert.EqualError(t, err, "relation_type must be 'sequence' or 'reference'")
}

// ──────────────────────────────────────────────
// FindRoot
// ──────────────────────────────────────────────

func TestPG_FindRoot_IsolatedCard_ReturnsSelf(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	a := seedCard(t, tx, uniqueTitle(t, "A"), "# A")

	root := svc.FindRoot(a.ID)
	assert.Equal(t, a.ID, root)
}

func TestPG_FindRoot_SingleChain(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	a := seedCard(t, tx, uniqueTitle(t, "A"), "# A")
	b := seedCard(t, tx, uniqueTitle(t, "B"), "# B")
	c := seedCard(t, tx, uniqueTitle(t, "C"), "# C")
	// sequence chain: A -> B -> C (A is root)
	seedEdge(t, tx, a.ID, b.ID, "sequence")
	seedEdge(t, tx, b.ID, c.ID, "sequence")

	assert.Equal(t, a.ID, svc.FindRoot(c.ID))
	assert.Equal(t, a.ID, svc.FindRoot(b.ID))
	assert.Equal(t, a.ID, svc.FindRoot(a.ID))
}

func TestPG_FindRoot_ReferenceEdgesIgnored(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	a := seedCard(t, tx, uniqueTitle(t, "A"), "# A")
	b := seedCard(t, tx, uniqueTitle(t, "B"), "# B")
	seedEdge(t, tx, a.ID, b.ID, "reference")

	// reference edges should NOT be traversed — b should return itself
	assert.Equal(t, b.ID, svc.FindRoot(b.ID))
}

func TestPG_FindRoot_NonExistentCard_ReturnsSelf(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	nonExistent := "00000000-0000-0000-0000-000000009999"
	root := svc.FindRoot(nonExistent)
	assert.Equal(t, nonExistent, root)
}

// ──────────────────────────────────────────────
// GetAllEdges
// ──────────────────────────────────────────────

func TestPG_GetAllEdges_HappyPath(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	a := seedCard(t, tx, uniqueTitle(t, "A"), "# A")
	b := seedCard(t, tx, uniqueTitle(t, "B"), "# B")
	c := seedCard(t, tx, uniqueTitle(t, "C"), "# C")
	seedEdge(t, tx, a.ID, b.ID, "reference")
	seedEdge(t, tx, b.ID, c.ID, "sequence")

	// GetAllEdges queries the full table, so we check relative count.
	before := countEdges(t, tx, "1=1")
	edges, err := svc.GetAllEdges()
	require.NoError(t, err)
	assert.Equal(t, before, int64(len(edges)))
}

func TestPG_GetAllEdges_EmptyResult(t *testing.T) {
	tx := testTx(t)
	svc := NewEdgeService(tx, nil)

	// Count before seeding — verifies GetAllEdges returns without error.
	edges, err := svc.GetAllEdges()
	require.NoError(t, err)
	// Not asserting length == 0 because dev DB may have existing data.
	_ = edges
}

// ──────────────────────────────────────────────
// Concurrency
// ──────────────────────────────────────────────

func TestPG_CreateEdge_Concurrent(t *testing.T) {
	// For concurrent test we need a separate transaction per goroutine,
	// so we use the shared testDB directly.
	db := ensureTestDB(t)

	svc := NewEdgeService(db, nil)

	rnd := rand.Int63()
	a := seedCard(t, db, fmt.Sprintf("[concurrent:%d] source", rnd), "# A")

	var wg sync.WaitGroup
	errCh := make(chan error, 10)

	for i := 0; i < 10; i++ {
		c := seedCard(t, db, fmt.Sprintf("[concurrent:%d] target-%d", rnd, i), "# T")
		wg.Add(1)
		go func(targetID string) {
			defer wg.Done()
			if err := svc.CreateEdge(context.Background(),a.ID, targetID, "reference"); err != nil {
				errCh <- err
			}
		}(c.ID)
	}
	wg.Wait()
	close(errCh)

	for err := range errCh {
		t.Errorf("concurrent CreateEdge failed: %v", err)
	}

	// Verify all 10 edges created
	total := countEdges(t, db, "source_id = ?", a.ID)
	assert.Equal(t, int64(10), total)
}
