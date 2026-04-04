package services

import (
	"errors"
	"time"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"gorm.io/gorm"
)

// EdgeService 处理卡片关系（边）的 CRUD 操作。
//
// 边是知识图谱的核心连接，表示两张卡片之间的有向关系。
// 支持的关系类型：sequence（时序）、reference（引用）。
type EdgeService struct {
	db                   *gorm.DB
	invalidateGraphCache func(cardID string)
}

// NewEdgeService 创建 EdgeService 实例
func NewEdgeService(db *gorm.DB, invalidateGraphCache func(cardID string)) *EdgeService {
	return &EdgeService{db: db, invalidateGraphCache: invalidateGraphCache}
}

// invalidateCache 使边的两端卡片图谱缓存失效
func (s *EdgeService) invalidateCache(sourceID, targetID string) {
	if s.invalidateGraphCache != nil {
		s.invalidateGraphCache(sourceID)
		s.invalidateGraphCache(targetID)
	}
}

// CreateEdge 在两张卡片之间创建有向边。
//
// 参数：
//   - sourceID: 源卡片 UUID（必填）
//   - targetID: 目标卡片 UUID（必填）
//   - relationType: 关系类型，仅支持 "sequence" 或 "reference"
func (s *EdgeService) CreateEdge(sourceID, targetID, relationType string) error {
	if sourceID == "" || targetID == "" {
		return errors.New("source_id and target_id are required")
	}
	if relationType != "sequence" && relationType != "reference" {
		return errors.New("relation_type must be 'sequence' or 'reference'")
	}

	edge := models.CardEdge{
		SourceID:     sourceID,
		TargetID:     targetID,
		RelationType: relationType,
		CreatedAt:    time.Now(),
	}

	if err := s.db.Create(&edge).Error; err != nil {
		return err
	}

	s.invalidateCache(sourceID, targetID)
	logger.Log.Infof("[EdgeService] Edge created: %s -> %s (%s)", sourceID, targetID, relationType)
	return nil
}

// DeleteEdge 删除指定源→目标的有向边。
func (s *EdgeService) DeleteEdge(sourceID, targetID string) error {
	if err := s.db.Where("source_id = ? AND target_id = ?", sourceID, targetID).
		Delete(&models.CardEdge{}).Error; err != nil {
		return err
	}
	s.invalidateCache(sourceID, targetID)
	logger.Log.Infof("[EdgeService] Edge deleted: %s -> %s", sourceID, targetID)
	return nil
}

// UpdateEdgeType 更新指定边的关系类型。
// 如果边不存在返回 "edge not found" 错误。
func (s *EdgeService) UpdateEdgeType(sourceID, targetID, newRelation string) error {
	if newRelation != "sequence" && newRelation != "reference" {
		return errors.New("relation_type must be 'sequence' or 'reference'")
	}
	result := s.db.Model(&models.CardEdge{}).
		Where("source_id = ? AND target_id = ?", sourceID, targetID).
		Update("relation_type", newRelation)
	if result.Error != nil {
		return result.Error
	}
	if result.RowsAffected == 0 {
		return errors.New("edge not found")
	}
	s.invalidateCache(sourceID, targetID)
	logger.Log.Infof("[EdgeService] Edge updated: %s -> %s (%s)", sourceID, targetID, newRelation)
	return nil
}

// FindRoot 沿 sequence 边反向遍历，找到卡片的根节点（知识流起点）。
// 使用递归 CTE 替代逐行查询，避免 N+1 问题。
func (s *EdgeService) FindRoot(cardID string) string {
	var rootID string
	cte := `
WITH RECURSIVE chain AS (
    SELECT ? AS id, 0 AS depth
    UNION ALL
    SELECT e.source_id, c.depth + 1
    FROM chain c
    JOIN card_edges e ON e.target_id = c.id AND e.relation_type = 'sequence'
    WHERE c.depth < 100
)
SELECT id FROM chain ORDER BY depth DESC LIMIT 1`
	if err := s.db.Raw(cte, cardID).Scan(&rootID).Error; err != nil {
		logger.Log.Warnf("[EdgeService] FindRoot CTE failed for %s: %v, fallback to self", cardID, err)
		return cardID
	}
	if rootID == "" {
		return cardID
	}
	return rootID
}

// GetAllEdges 获取数据库中的全部边记录。
// 用于图谱全量渲染和同步缓存。
func (s *EdgeService) GetAllEdges() ([]models.CardEdge, error) {
	var edges []models.CardEdge
	if err := s.db.Find(&edges).Error; err != nil {
		return nil, err
	}
	return edges, nil
}

// SyncReferenceEdges atomically synchronizes reference-type edges for a source card.
// It ensures the source card has exactly the specified reference edges, adding new ones
// and removing old ones as needed. Sequence edges are never touched.
//
// Parameters:
//   - sourceCardID: UUID of the source card
//   - targetCardIDs: slice of target card UUIDs (pre-resolved, will be deduplicated)
func (s *EdgeService) SyncReferenceEdges(sourceCardID string, targetCardIDs []string) error {
	deduplicated := deduplicatePreserveOrder(targetCardIDs)

	var affectedTargetIDs []string
	var addCount, removeCount int

	err := s.db.Transaction(func(tx *gorm.DB) error {
		var currentEdges []models.CardEdge
		if err := tx.Where("source_id = ? AND relation_type = ?", sourceCardID, "reference").
			Find(&currentEdges).Error; err != nil {
			return err
		}

		currentIDs := make([]string, len(currentEdges))
		for i, e := range currentEdges {
			currentIDs[i] = e.TargetID
		}

		var sequenceEdges []models.CardEdge
		if err := tx.Where("source_id = ? AND relation_type = ?", sourceCardID, "sequence").
			Find(&sequenceEdges).Error; err != nil {
			return err
		}

		sequenceSet := make(map[string]bool, len(sequenceEdges))
		for _, e := range sequenceEdges {
			sequenceSet[e.TargetID] = true
		}

		filtered := make([]string, 0, len(deduplicated))
		for _, id := range deduplicated {
			if !sequenceSet[id] {
				filtered = append(filtered, id)
			}
		}

		toAdd, toRemove := computeEdgeDiff(currentIDs, filtered)

		addCount = len(toAdd)
		removeCount = len(toRemove)

		affectedTargetIDs = append(affectedTargetIDs, toAdd...)
		affectedTargetIDs = append(affectedTargetIDs, toRemove...)

		if len(toAdd) > 0 {
			newEdges := make([]models.CardEdge, len(toAdd))
			now := time.Now()
			for i, targetID := range toAdd {
				newEdges[i] = models.CardEdge{
					SourceID:     sourceCardID,
					TargetID:     targetID,
					RelationType: "reference",
					CreatedAt:    now,
				}
			}
			if err := tx.Create(&newEdges).Error; err != nil {
				return err
			}
		}

		if len(toRemove) > 0 {
			if err := tx.Where("source_id = ? AND target_id IN ? AND relation_type = ?",
				sourceCardID, toRemove, "reference").
				Delete(&models.CardEdge{}).Error; err != nil {
				return err
			}
		}

		return nil
	})

	if err != nil {
		return err
	}

	for _, targetID := range affectedTargetIDs {
		s.invalidateCache(sourceCardID, targetID)
	}

	logger.Log.Infof("[EdgeService] Synced reference edges for %s: +%d -%d", sourceCardID, addCount, removeCount)
	return nil
}

func computeEdgeDiff(current []string, desired []string) (toAdd []string, toRemove []string) {
	currentSet := make(map[string]bool)
	for _, id := range current {
		currentSet[id] = true
	}

	desiredSet := make(map[string]bool)
	for _, id := range desired {
		desiredSet[id] = true
	}

	for _, id := range desired {
		if !currentSet[id] {
			toAdd = append(toAdd, id)
		}
	}

	for _, id := range current {
		if !desiredSet[id] {
			toRemove = append(toRemove, id)
		}
	}

	return toAdd, toRemove
}

// deduplicatePreserveOrder removes duplicates from a string slice while preserving order.
func deduplicatePreserveOrder(ids []string) []string {
	if len(ids) == 0 {
		return ids
	}

	seen := make(map[string]bool)
	result := make([]string, 0, len(ids))
	for _, id := range ids {
		if !seen[id] {
			seen[id] = true
			result = append(result, id)
		}
	}
	return result
}
