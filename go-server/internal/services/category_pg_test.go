//go:build integration

package services

import (
	"fmt"
	"math/rand"
	"testing"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// ──────────────────────────────────────────────
// ListAll
// ──────────────────────────────────────────────

func TestPG_ListAll_HappyPath(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	// Count before seeding
	var before int64
	tx.Model(&struct{}{}).Table("categories").Count(&before)

	seedCategory(t, tx, uniqueTitle(t, "CatA"), nil)
	seedCategory(t, tx, uniqueTitle(t, "CatB"), nil)

	cats, err := svc.ListAll()
	require.NoError(t, err)
	assert.Equal(t, int(before)+2, len(cats))
}

func TestPG_ListAll_ReturnsWithoutError(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	_, err := svc.ListAll()
	require.NoError(t, err)
}

// ──────────────────────────────────────────────
// GetTree
// ──────────────────────────────────────────────

func TestPG_GetTree_NestedStructure(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	parent := seedCategory(t, tx, uniqueTitle(t, "Parent"), nil)
	child := seedCategory(t, tx, uniqueTitle(t, "Child"), &parent.ID)

	roots, err := svc.GetTree()
	require.NoError(t, err)

	// Find our parent node among the roots
	var found *models.CategoryTreeNode
	for i := range roots {
		if roots[i].ID == parent.ID {
			found = &roots[i]
			break
		}
	}
	require.NotNil(t, found, "parent category should be a root node")
	assert.Len(t, found.Children, 1)
	assert.Equal(t, child.ID, found.Children[0].ID)
}

func TestPG_GetTree_FlatStructure(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	r1 := seedCategory(t, tx, uniqueTitle(t, "Root1"), nil)
	r2 := seedCategory(t, tx, uniqueTitle(t, "Root2"), nil)

	roots, err := svc.GetTree()
	require.NoError(t, err)

	// Our two roots should exist as root nodes with no children
	rootsByID := make(map[uint]models.CategoryTreeNode)
	for _, r := range roots {
		rootsByID[r.ID] = r
	}

	node1, ok1 := rootsByID[r1.ID]
	node2, ok2 := rootsByID[r2.ID]
	require.True(t, ok1, "Root1 should be in tree")
	require.True(t, ok2, "Root2 should be in tree")
	assert.Empty(t, node1.Children)
	assert.Empty(t, node2.Children)
}

// ──────────────────────────────────────────────
// Create
// ──────────────────────────────────────────────

func TestPG_Create_HappyPath(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	cat, err := svc.Create(uniqueTitle(t, "NewCat"), "desc", nil, nil)
	require.NoError(t, err)
	assert.NotZero(t, cat.ID)
	assert.Equal(t, "desc", cat.Description)
}

func TestPG_Create_WithParent(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	parent := seedCategory(t, tx, uniqueTitle(t, "Parent"), nil)

	child, err := svc.Create(uniqueTitle(t, "Child"), "child desc", nil, &parent.ID)
	require.NoError(t, err)
	assert.Equal(t, parent.ID, *child.ParentID)
}

func TestPG_Create_EmptyName(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	_, err := svc.Create("", "desc", nil, nil)
	assert.EqualError(t, err, "分类名称不能为空")
}

func TestPG_Create_DuplicateName(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	name := uniqueTitle(t, "Dup")
	_, err1 := svc.Create(name, "first", nil, nil)
	require.NoError(t, err1)

	_, err2 := svc.Create(name, "second", nil, nil)
	assert.Error(t, err2, "expected unique constraint violation on duplicate name")
}

func TestPG_Create_WithThemeColor(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	theme := "cyan"
	cat, err := svc.Create(uniqueTitle(t, "Themed"), "desc", &theme, nil)
	require.NoError(t, err)
	require.NotNil(t, cat.ThemeColor)
	assert.Equal(t, "cyan", *cat.ThemeColor)
}

// ──────────────────────────────────────────────
// Update
// ──────────────────────────────────────────────

func TestPG_Update_HappyPath(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	cat := seedCategory(t, tx, uniqueTitle(t, "Old"), nil)

	err := svc.Update(cat.ID, uniqueTitle(t, "New"), "new desc", nil, nil)
	require.NoError(t, err)

	// Verify updated
	var updated struct {
		Name        string
		Description string
	}
	tx.Model(&struct{}{}).Table("categories").Select("name, description").
		Where("id = ?", cat.ID).Scan(&updated)
	assert.Equal(t, uniqueTitle(t, "New"), updated.Name)
	assert.Equal(t, "new desc", updated.Description)
}

func TestPG_Update_EmptyName(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	cat := seedCategory(t, tx, uniqueTitle(t, "Cat"), nil)

	err := svc.Update(cat.ID, "", "desc", nil, nil)
	assert.EqualError(t, err, "分类名称不能为空")
}

func TestPG_Update_SelfReference(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	cat := seedCategory(t, tx, uniqueTitle(t, "SelfRef"), nil)

	err := svc.Update(cat.ID, uniqueTitle(t, "Updated"), "", nil, &cat.ID)
	assert.EqualError(t, err, "invalid circular reference")
}

func TestPG_Update_NonExistentID(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	// Update with non-existent ID should not error (GORM: 0 rows affected, no error)
	err := svc.Update(99999, uniqueTitle(t, "Ghost"), "desc", nil, nil)
	assert.NoError(t, err)
}

// ──────────────────────────────────────────────
// Delete
// ──────────────────────────────────────────────

func TestPG_Delete_HappyPath(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	cat := seedCategory(t, tx, uniqueTitle(t, "ToDelete"), nil)

	err := svc.Delete(cat.ID)
	require.NoError(t, err)

	// Verify category is gone
	var count int64
	tx.Model(&struct{}{}).Table("categories").Where("id = ?", cat.ID).Count(&count)
	assert.Equal(t, int64(0), count)
}

func TestPG_Delete_WithCards_NullifiesCategory(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	cat := seedCategory(t, tx, uniqueTitle(t, "CatWithCard"), nil)
	card := seedCard(t, tx, uniqueTitle(t, "CardInCat"), "# C")

	// Assign card to category
	tx.Model(&struct{}{}).Table("cards").Where("id = ?", card.ID).
		Update("category_id", cat.ID)

	err := svc.Delete(cat.ID)
	require.NoError(t, err)

	// Verify card's category_id is now nil
	var catID *uint
	tx.Model(&struct{}{}).Table("cards").Select("category_id").
		Where("id = ?", card.ID).Scan(&catID)
	assert.Nil(t, catID)
}

func TestPG_Delete_HasChildren_Blocked(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	parent := seedCategory(t, tx, uniqueTitle(t, "Parent"), nil)
	_ = seedCategory(t, tx, uniqueTitle(t, "Child"), &parent.ID)

	err := svc.Delete(parent.ID)
	assert.EqualError(t, err, "category has children")
}

// ──────────────────────────────────────────────
// HasChildren
// ──────────────────────────────────────────────

func TestPG_HasChildren_True(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	parent := seedCategory(t, tx, uniqueTitle(t, "Parent"), nil)
	_ = seedCategory(t, tx, uniqueTitle(t, "Child"), &parent.ID)

	has, err := svc.HasChildren(parent.ID)
	require.NoError(t, err)
	assert.True(t, has)
}

func TestPG_HasChildren_False(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	cat := seedCategory(t, tx, uniqueTitle(t, "Leaf"), nil)

	has, err := svc.HasChildren(cat.ID)
	require.NoError(t, err)
	assert.False(t, has)
}

func TestPG_HasChildren_NonExistent(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	has, err := svc.HasChildren(99999)
	require.NoError(t, err)
	assert.False(t, has)
}

// ──────────────────────────────────────────────
// GetClusters
// ──────────────────────────────────────────────

func TestPG_GetClusters_HappyPath(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	cat := seedCategory(t, tx, uniqueTitle(t, "ClusterCat"), nil)

	// Create cards assigned to this category
	card1 := seedCard(t, tx, uniqueTitle(t, "Card1"), "# C1")
	card2 := seedCard(t, tx, uniqueTitle(t, "Card2"), "# C2")
	tx.Model(&struct{}{}).Table("cards").Where("id = ?", card1.ID).Update("category_id", cat.ID)
	tx.Model(&struct{}{}).Table("cards").Where("id = ?", card2.ID).Update("category_id", cat.ID)

	clusters, err := svc.GetClusters(cat.ID)
	require.NoError(t, err)
	assert.Len(t, clusters, 2)
}

func TestPG_GetClusters_EmptyCategory(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	cat := seedCategory(t, tx, uniqueTitle(t, "EmptyCat"), nil)

	clusters, err := svc.GetClusters(cat.ID)
	require.NoError(t, err)
	assert.Empty(t, clusters)
}

func TestPG_GetClusters_NonExistentCategory(t *testing.T) {
	tx := testTx(t)
	svc := NewCategoryService(tx)

	clusters, err := svc.GetClusters(99999)
	require.NoError(t, err)
	assert.Empty(t, clusters)
}

// ──────────────────────────────────────────────
// Concurrent Create (unique name race)
// ──────────────────────────────────────────────

func TestPG_Category_Create_Concurrent(t *testing.T) {
	db := ensureTestDB(t)
	svc := NewCategoryService(db)

	rnd := rand.Int63()
	name := fmt.Sprintf("[concurrent:%d] RaceCat", rnd)

	// First create should succeed
	_, err := svc.Create(name, "desc", nil, nil)
	require.NoError(t, err)

	// Concurrent creates with same name should fail on unique constraint
	errCh := make(chan error, 5)
	for i := 0; i < 5; i++ {
		go func() {
			_, e := svc.Create(name, "dup", nil, nil)
			errCh <- e
		}()
	}
	for i := 0; i < 5; i++ {
		err := <-errCh
		assert.Error(t, err, "expected unique constraint violation")
	}
}
