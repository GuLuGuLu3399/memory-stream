//go:build integration

package services

import (
	"context"
	"fmt"
	"sync"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// ============================================================================
// MergeCards — Happy Path
// ============================================================================

func TestPG_MergeCards_HappyPath_NoEdges(t *testing.T) {
	tx := testTx(t)

	survivor := seedCard(t, tx, uniqueTitle(t, "Survivor"), "I live")
	victim1 := seedCard(t, tx, uniqueTitle(t, "Victim1"), "I die")
	victim2 := seedCard(t, tx, uniqueTitle(t, "Victim2"), "I also die")

	result, err := NewMergeService(tx).Merge(context.Background(), MergeRequest{
		SurvivorID: survivor.ID,
		VictimIDs:  []string{victim1.ID, victim2.ID},
	})
	require.NoError(t, err)
	assert.Equal(t, 2, result.NodesDeleted)
	assert.Equal(t, 0, result.EdgesMigrated)

	assertCardExists(t, tx, survivor.ID, true)
	assertCardExists(t, tx, victim1.ID, false)
	assertCardExists(t, tx, victim2.ID, false)
}

func TestPG_MergeCards_HappyPath_WithIncomingEdges(t *testing.T) {
	tx := testTx(t)

	survivor := seedCard(t, tx, uniqueTitle(t, "Survivor"), "live")
	victim := seedCard(t, tx, uniqueTitle(t, "Victim"), "die")
	external := seedCard(t, tx, uniqueTitle(t, "External"), "bystander")

	// external → victim (incoming edge to victim)
	seedEdge(t, tx, external.ID, victim.ID, "reference")
	// victim → external (outgoing edge from victim)
	seedEdge(t, tx, victim.ID, external.ID, "sequence")

	result, err := NewMergeService(tx).Merge(context.Background(), MergeRequest{
		SurvivorID: survivor.ID,
		VictimIDs:  []string{victim.ID},
	})
	require.NoError(t, err)
	assert.Equal(t, 1, result.NodesDeleted)
	assert.Equal(t, 2, result.EdgesMigrated)

	// Victim deleted
	assertCardExists(t, tx, victim.ID, false)

	// Edges migrated: external → survivor and survivor → external
	assertEdgeExists(t, tx, external.ID, survivor.ID, "reference", true)
	assertEdgeExists(t, tx, survivor.ID, external.ID, "sequence", true)

	// Old edges gone
	assertEdgeExists(t, tx, external.ID, victim.ID, "reference", false)
	assertEdgeExists(t, tx, victim.ID, external.ID, "sequence", false)
}

// ============================================================================
// MergeCards — Validation Errors
// ============================================================================

func TestPG_MergeCards_SurvivorInVictims(t *testing.T) {
	tx := testTx(t)

	card := seedCard(t, tx, uniqueTitle(t, "SelfMerge"), "content")

	result, err := NewMergeService(tx).Merge(context.Background(), MergeRequest{
		SurvivorID: card.ID,
		VictimIDs:  []string{card.ID},
	})
	assert.Nil(t, result)
	assert.EqualError(t, err, "survivor_id cannot be in victim_ids")

	// Card should still exist
	assertCardExists(t, tx, card.ID, true)
}

func TestPG_MergeCards_CardNotFound(t *testing.T) {
	tx := testTx(t)

	survivor := seedCard(t, tx, uniqueTitle(t, "Real"), "content")

	result, err := NewMergeService(tx).Merge(context.Background(), MergeRequest{
		SurvivorID: survivor.ID,
		VictimIDs:  []string{"00000000-0000-0000-0000-00000000dead"},
	})
	assert.Nil(t, result)
	assert.EqualError(t, err, "one or more card IDs not found")

	// Survivor should still exist (transaction rolled back)
	assertCardExists(t, tx, survivor.ID, true)
}

// ============================================================================
// MergeCards — Self-loop detection
// ============================================================================

func TestPG_MergeCards_IncomingSelfLoop(t *testing.T) {
	tx := testTx(t)

	survivor := seedCard(t, tx, uniqueTitle(t, "Survivor"), "live")
	victim := seedCard(t, tx, uniqueTitle(t, "Victim"), "die")

	// survivor → victim edge (will become survivor → survivor = self-loop)
	seedEdge(t, tx, survivor.ID, victim.ID, "reference")

	result, err := NewMergeService(tx).Merge(context.Background(), MergeRequest{
		SurvivorID: survivor.ID,
		VictimIDs:  []string{victim.ID},
	})
	require.NoError(t, err)
	assert.Equal(t, 1, result.NodesDeleted)
	assert.Equal(t, 0, result.EdgesMigrated) // self-loop skipped
	assert.Contains(t, result.Warnings, "skipped incoming self-loop edge")

	// No self-loop should exist
	assertEdgeExists(t, tx, survivor.ID, survivor.ID, "reference", false)
}

func TestPG_MergeCards_OutgoingSelfLoop_Reference(t *testing.T) {
	tx := testTx(t)

	survivor := seedCard(t, tx, uniqueTitle(t, "Survivor"), "live")
	victim := seedCard(t, tx, uniqueTitle(t, "Victim"), "die")

	// victim → survivor edge (will become survivor → survivor = self-loop)
	seedEdge(t, tx, victim.ID, survivor.ID, "reference")

	result, err := NewMergeService(tx).Merge(context.Background(), MergeRequest{
		SurvivorID: survivor.ID,
		VictimIDs:  []string{victim.ID},
	})
	require.NoError(t, err)
	assert.Equal(t, 0, result.EdgesMigrated)
	assert.Contains(t, result.Warnings, "skipped outgoing self-loop edge")
}

func TestPG_MergeCards_OutgoingSelfLoop_Sequence(t *testing.T) {
	tx := testTx(t)

	survivor := seedCard(t, tx, uniqueTitle(t, "Survivor"), "live")
	victim := seedCard(t, tx, uniqueTitle(t, "Victim"), "die")

	// victim → survivor sequence edge (self-loop removed)
	seedEdge(t, tx, victim.ID, survivor.ID, "sequence")

	result, err := NewMergeService(tx).Merge(context.Background(), MergeRequest{
		SurvivorID: survivor.ID,
		VictimIDs:  []string{victim.ID},
	})
	require.NoError(t, err)
	assert.Contains(t, result.Warnings, "removed sequence self-loop edge")
}

// ============================================================================
// MergeCards — Edge deduplication
// ============================================================================

func TestPG_MergeCards_DeduplicateMigratedEdges(t *testing.T) {
	tx := testTx(t)

	survivor := seedCard(t, tx, uniqueTitle(t, "Survivor"), "live")
	victim := seedCard(t, tx, uniqueTitle(t, "Victim"), "die")
	external := seedCard(t, tx, uniqueTitle(t, "External"), "bystander")

	// Both survivor and victim have edges to external with same type
	seedEdge(t, tx, survivor.ID, external.ID, "reference")
	seedEdge(t, tx, victim.ID, external.ID, "reference")

	_, err := NewMergeService(tx).Merge(context.Background(), MergeRequest{
		SurvivorID: survivor.ID,
		VictimIDs:  []string{victim.ID},
	})
	require.NoError(t, err)

	// After merge + dedup, there should be exactly ONE survivor → external reference edge
	edgeCount := countEdges(t, tx, "source_id = ? AND target_id = ? AND relation_type = ?",
		survivor.ID, external.ID, "reference")
	assert.Equal(t, int64(1), edgeCount, "duplicate edge should be deduplicated")
}

// ============================================================================
// MergeCards — Complex topology
// ============================================================================

func TestPG_MergeCards_ComplexGraph(t *testing.T) {
	tx := testTx(t)

	// Create a graph: A → B → C → D, with cross-refs
	A := seedCard(t, tx, uniqueTitle(t, "A"), "a")
	B := seedCard(t, tx, uniqueTitle(t, "B"), "b")
	C := seedCard(t, tx, uniqueTitle(t, "C"), "c")
	D := seedCard(t, tx, uniqueTitle(t, "D"), "d")

	seedEdge(t, tx, A.ID, B.ID, "sequence")
	seedEdge(t, tx, B.ID, C.ID, "sequence")
	seedEdge(t, tx, C.ID, D.ID, "reference")
	seedEdge(t, tx, A.ID, C.ID, "reference") // cross-ref

	// Merge B and C into A (survivor)
	result, err := NewMergeService(tx).Merge(context.Background(), MergeRequest{
		SurvivorID: A.ID,
		VictimIDs:  []string{B.ID, C.ID},
	})
	require.NoError(t, err)
	assert.Equal(t, 2, result.NodesDeleted)

	// A and D survive
	assertCardExists(t, tx, A.ID, true)
	assertCardExists(t, tx, D.ID, true)
	assertCardExists(t, tx, B.ID, false)
	assertCardExists(t, tx, C.ID, false)

	// A → D should exist (migrated from C → D)
	assertEdgeExists(t, tx, A.ID, D.ID, "reference", true)
}

// ============================================================================
// MergeCards — Concurrent merge (P0-1 regression test)
// ============================================================================

func TestPG_MergeCards_ConcurrentNoDeadlock(t *testing.T) {
	db := ensureTestDB(t)

	// Create 4 cards with unique titles
	type cardRef struct{ ID, Title string }
	cards := make([]*cardRef, 4)
	for i := 0; i < 4; i++ {
		title := uniqueTitle(t, fmt.Sprintf("Concurrent-%d", i))
		if err := db.Exec(`INSERT INTO cards (title, raw_md, excerpt, ast_data) VALUES (?, 'c', 'c', '{}')`,
			title).Error; err != nil {
			t.Fatalf("failed to create card: %v", err)
		}
		var card cardRef
		db.Raw(`SELECT id, title FROM cards WHERE title = ?`, title).Scan(&card)
		cards[i] = &card
	}
	t.Cleanup(func() {
		for _, c := range cards {
			db.Exec("DELETE FROM card_edges WHERE source_id = ? OR target_id = ?", c.ID, c.ID)
			db.Exec("DELETE FROM cards WHERE id = ?", c.ID)
		}
	})

	// Two concurrent merges: merge [1,2] into 0, and merge [3] into 0
	// Both target card 0 — tests row-level locking
	var wg sync.WaitGroup
	errors := make([]error, 2)

	wg.Add(2)
	go func() {
		defer wg.Done()
		_, errors[0] = NewMergeService(db).Merge(context.Background(), MergeRequest{
			SurvivorID: cards[0].ID,
			VictimIDs:  []string{cards[1].ID, cards[2].ID},
		})
	}()
	go func() {
		defer wg.Done()
		_, errors[1] = NewMergeService(db).Merge(context.Background(), MergeRequest{
			SurvivorID: cards[0].ID,
			VictimIDs:  []string{cards[3].ID},
		})
	}()
	wg.Wait()

	// At least one should succeed, neither should deadlock
	successCount := 0
	for i, err := range errors {
		if err == nil {
			successCount++
		} else {
			t.Logf("merge %d error (may be expected due to race): %v", i, err)
		}
	}
	assert.GreaterOrEqual(t, successCount, 1, "at least one merge should succeed")
}
