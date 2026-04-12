package services

import (
	"context"
	"errors"
	"fmt"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"gorm.io/gorm"
)

// CategoryService 处理知识分类的 CRUD 操作。
type CategoryService struct {
	db *gorm.DB
}

// NewCategoryService 创建 CategoryService 实例
func NewCategoryService(db *gorm.DB) *CategoryService {
	return &CategoryService{db: db}
}

// ListAll 获取全部分类列表，按名称排序。
func (s *CategoryService) ListAll(ctx context.Context) ([]models.Category, error) {
	var categories []models.Category
	if err := s.db.Order("name").Find(&categories).Error; err != nil {
		return nil, err
	}
	return categories, nil
}

// GetTree 获取分类树结构，按 sort_order 排序。
func (s *CategoryService) GetTree(ctx context.Context) ([]models.CategoryTreeNode, error) {
	var categories []models.Category
	// Query all categories ordered by sort_order, then name
	if err := s.db.Order("sort_order, name").Find(&categories).Error; err != nil {
		return nil, err
	}

	// Build tree in memory
	nodeMap := make(map[uint]*models.CategoryTreeNode)
	for _, cat := range categories {
		nodeMap[cat.ID] = &models.CategoryTreeNode{
			ID:          cat.ID,
			Name:        cat.Name,
			Description: cat.Description,
			ParentID:    cat.ParentID,
			SortOrder:   cat.SortOrder,
			CreatedAt:   cat.CreatedAt,
			ThemeColor:  cat.ThemeColor,
			Children:    []models.CategoryTreeNode{},
		}
	}

	// Attach children to parents and collect roots
	var roots []models.CategoryTreeNode
	for _, cat := range categories {
		node := nodeMap[cat.ID]
		if cat.ParentID == nil {
			// Root node
			roots = append(roots, *node)
		} else {
			// Attach to parent
			if parent, exists := nodeMap[*cat.ParentID]; exists {
				parent.Children = append(parent.Children, *node)
			}
		}
	}

	return roots, nil
}

// Create 创建新的知识分类。名称不能为空；可选传 parentID。
func (s *CategoryService) Create(ctx context.Context, name string, description string, themeColor *string, parentID *uint) (*models.Category, error) {
	if name == "" {
		return nil, fmt.Errorf("分类名称不能为空")
	}
	// 进行父级相关的验证（若提供某个父级）
	if parentID != nil {
		if err := s.validateDepth(*parentID); err != nil {
			return nil, err
		}
		if err := s.validateCircularReference(0, *parentID); err != nil {
			return nil, err
		}
	}
	cat := &models.Category{
		Name:        name,
		Description: description,
		ThemeColor:  themeColor,
		ParentID:    parentID,
		SortOrder:   0,
	}
	if err := s.db.Create(cat).Error; err != nil {
		return nil, err
	}
	return cat, nil
}

// Update 更新分类的名称、描述及父级。名称不能为空。
func (s *CategoryService) Update(ctx context.Context, id uint, name string, description string, themeColor *string, parentID *uint) error {
	if name == "" {
		return fmt.Errorf("分类名称不能为空")
	}
	// 验证父级关系（如果提供了父ID）
	if parentID != nil {
		if err := s.validateDepth(*parentID); err != nil {
			return err
		}
		if err := s.validateCircularReference(id, *parentID); err != nil {
			return err
		}
		if id == *parentID {
			return fmt.Errorf("invalid circular reference")
		}
	}
	return s.db.Model(&models.Category{}).Where("id = ?", id).Updates(map[string]interface{}{
		"name":        name,
		"description": description,
		"theme_color": themeColor,
		"parent_id":   parentID,
	}).Error
}

// Delete 删除分类，并将该分类下的所有卡片 category_id 置为 NULL。
// 使用事务确保原子性。
func (s *CategoryService) Delete(ctx context.Context, id uint) error {
	// 1) 必须先检查子分类是否存在
	hasChild, err := s.HasChildren(id)
	if err != nil {
		return err
	}
	if hasChild {
		return fmt.Errorf("category has children")
	}
	return s.db.Transaction(func(tx *gorm.DB) error {
		// 1. 将该分类下所有卡片的 category_id 置为 NULL
		if err := tx.Model(&models.Card{}).Where("category_id = ?", id).
			Update("category_id", nil).Error; err != nil {
			return err
		}
		// 2. 删除分类本身
		return tx.Where("id = ?", id).Delete(&models.Category{}).Error
	})
}

// HasChildren 判断指定分类是否存在子分类
func (s *CategoryService) HasChildren(id uint) (bool, error) {
	var count int64
	if err := s.db.Model(&models.Category{}).Where("parent_id = ?", id).Count(&count).Error; err != nil {
		return false, err
	}
	return count > 0, nil
}

// validateCircularReference 检查在将节点放到 parentID 下时，是否会产生循环引用
func (s *CategoryService) validateCircularReference(id uint, parentID uint) error {
	current := parentID
	for {
		var c models.Category
		if err := s.db.Select("id", "parent_id").Where("id = ?", current).First(&c).Error; err != nil {
			// 不存在的父级视为无循环
			if errors.Is(err, gorm.ErrRecordNotFound) {
				break
			}
			return err
		}
		if c.ParentID == nil {
			break
		}
		if *c.ParentID == id {
			return fmt.Errorf("circular reference detected")
		}
		current = *c.ParentID
	}
	return nil
}

// validateDepth 验证在 parentID 下的层级深度不能超过 5 层（不包含待新增节点）
func (s *CategoryService) validateDepth(parentID uint) error {
	depth := 1
	current := parentID
	for {
		var c models.Category
		if err := s.db.Select("parent_id").Where("id = ?", current).First(&c).Error; err != nil {
			if errors.Is(err, gorm.ErrRecordNotFound) {
				break
			}
			return err
		}
		if c.ParentID == nil {
			break
		}
		depth++
		if depth > 5 {
			return fmt.Errorf("depth limit exceeded")
		}
		current = *c.ParentID
	}
	return nil
}

// GetClusters 获取指定分类下的卡片聚类（按热度和更新时间排序）。
func (s *CategoryService) GetClusters(ctx context.Context, categoryID uint) ([]ClusterResult, error) {
	var clusters []ClusterResult

	err := s.db.Raw(`
		SELECT 
			c.id as card_id,
			c.title,
			c.updated_at,
			COALESCE(m.view_count, 0) as view_count
		FROM cards c
		LEFT JOIN card_metrics m ON m.card_id = c.id
		WHERE c.category_id = ?
		ORDER BY COALESCE(m.hot_score, 0) DESC, c.updated_at DESC
		LIMIT 20
	`, categoryID).Scan(&clusters).Error

	if err != nil {
		return nil, err
	}
	return clusters, nil
}

// ClusterResult 分类聚类结果 — 包含卡片摘要和浏览量
type ClusterResult struct {
	CardID    string `json:"card_id"`
	Title     string `json:"title"`
	UpdatedAt string `json:"updated_at"`
	ViewCount int64  `json:"view_count"`
}
