package services

import (
	"context"
	"errors"
	"strconv"
	"time"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"gorm.io/gorm"
)

// GraphResult 图谱查询结果 — 不含坐标信息（布局由前端负责）
type GraphResult struct {
	Nodes []GraphNode `json:"nodes"`
	Edges []GraphEdge `json:"edges"`
}

// GraphNode 图谱节点（卡片摘要）
type GraphNode struct {
	ID    string `json:"id"`
	Title string `json:"title"`
}

// GraphEdge 图谱边（卡片关联关系）
type GraphEdge struct {
	Source   string `json:"source"`
	Target   string `json:"target"`
	Relation string `json:"relation"`
}

// OutlineResult 大纲视图数据 — Topic/Cluster 层级结构
type OutlineResult struct {
	Topics   []OutlineTopic   `json:"topics"`
	Clusters []OutlineCluster `json:"clusters"`
}

// OutlineTopic 大纲主题（对应 Category）
type OutlineTopic struct {
	ID        string `json:"id"`
	Label     string `json:"label"`
	CardCount int    `json:"card_count"`
}

// OutlineCluster 大纲聚类（对应 Card）
type OutlineCluster struct {
	ID        string    `json:"id"`
	Title     string    `json:"title"`
	TopicID   string    `json:"topic_id"`
	CreatedAt time.Time `json:"created_at"`
}

// GraphService 处理图谱遍历和大纲生成。
//
// 使用递归 CTE 实现深度受限的子图查询，避免 N+1 问题。
// 所有查询使用批量加载策略，性能与节点数量线性相关。
type GraphService struct {
	db *gorm.DB
}

// NewGraphService 创建 GraphService 实例
func NewGraphService(db *gorm.DB) *GraphService {
	return &GraphService{db: db}
}

// resolveIdentifier 将虚拟标识符 "root" 解析为实际的根卡片 ID。
// 根卡片定义为：没有入边（未被任何边指向）的最早创建的卡片。
func (s *GraphService) resolveIdentifier(ctx context.Context, id string) (string, error) {
	if id != "root" {
		return id, nil
	}

	var realID string
	err := s.db.WithContext(ctx).Raw(`
			SELECT c.id FROM cards c
			LEFT JOIN card_edges e ON c.id = e.target_id
			WHERE e.target_id IS NULL
			ORDER BY c.created_at LIMIT 1
		`).Scan(&realID).Error

	if err != nil || realID == "" {
		s.db.WithContext(ctx).Raw(`SELECT id FROM cards ORDER BY created_at LIMIT 1`).Scan(&realID)
	}

	if realID == "" {
		return "", errors.New("knowledge base is empty")
	}

	return realID, nil
}

// GetGraph 获取以指定卡片为中心、指定深度的子图。
//
// 使用 PostgreSQL 递归 CTE 进行深度受限遍历，批量加载节点标题，
// 消除 N+1 查询问题。边的去重通过 seen map 保证。
//
// 参数：
//   - cardID: 中心卡片 ID（"root" 自动解析为根卡片）
//   - depth: 遍历深度（1-5，默认 2）
func (s *GraphService) GetGraph(ctx context.Context, cardID string, depth int) (*GraphResult, error) {
	// 深度硬上限：防止递归爆炸（服务层兜底，handler 已限制 1-5）
	const maxDepth = 5
	if depth < 1 {
		depth = 1
	} else if depth > maxDepth {
		depth = maxDepth
	}

	realID, err := s.resolveIdentifier(ctx, cardID)
	if err != nil {
		return nil, err
	}

	// Step 1: Batch-fetch all relevant edges using recursive CTE (depth-limited).
	var allEdges []models.CardEdge
	err = s.db.WithContext(ctx).Raw(`
		WITH RECURSIVE reachable AS (
			SELECT ?::uuid AS id, 0 AS depth
			UNION ALL
			SELECT CASE
				WHEN e.source_id = r.id THEN e.target_id
				ELSE e.source_id
			END AS id, r.depth + 1
			FROM reachable r
			JOIN card_edges e ON (e.source_id = r.id OR e.target_id = r.id)
			WHERE r.depth + 1 <= ?
		)
		SELECT e.* FROM card_edges e
		WHERE e.source_id IN (SELECT id FROM reachable)
		   OR e.target_id IN (SELECT id FROM reachable)
	`, realID, depth).Scan(&allEdges).Error
	if err != nil {
		return nil, err
	}

	// Step 2: Collect all unique node IDs from edges + root.
	nodeIDSet := map[string]bool{realID: true}
	for _, e := range allEdges {
		nodeIDSet[e.SourceID] = true
		nodeIDSet[e.TargetID] = true
	}
	nodeIDs := make([]string, 0, len(nodeIDSet))
	for id := range nodeIDSet {
		nodeIDs = append(nodeIDs, id)
	}

	// Step 3: Batch-fetch all card titles.
	var cards []struct {
		ID    string
		Title string
	}
	s.db.WithContext(ctx).Table("cards").Select("id, title").Where("id IN ?", nodeIDs).Scan(&cards)

	titleMap := make(map[string]string, len(cards))
	for _, c := range cards {
		titleMap[c.ID] = c.Title
	}

	// Step 4: Build result — nodes with titles + deduplicated edges.
	result := &GraphResult{
		Nodes: make([]GraphNode, 0, len(nodeIDs)),
		Edges: make([]GraphEdge, 0, len(allEdges)),
	}

	for _, id := range nodeIDs {
		result.Nodes = append(result.Nodes, GraphNode{
			ID:    id,
			Title: titleMap[id],
		})
	}

	seen := make(map[string]bool, len(allEdges))
	for _, e := range allEdges {
		key := e.SourceID + ":" + e.TargetID
		if !seen[key] {
			seen[key] = true
			result.Edges = append(result.Edges, GraphEdge{
				Source:   e.SourceID,
				Target:   e.TargetID,
				Relation: e.RelationType,
			})
		}
	}

	return result, nil
}

// GetAllGraph 获取全量图谱数据（所有节点 + 所有边）。
//
// 不使用递归 CTE，直接全表扫描 cards（轻量字段）和 card_edges。
// 用于前端"上帝视角"星图展示，包含所有连通分量和孤岛节点。
func (s *GraphService) GetAllGraph(ctx context.Context) (*GraphResult, error) {
	const maxGraphNodes = 5000
	const maxGraphEdges = 10000

	// Step 1: 全量拉取节点（仅 id + title，排除 raw_md 等大文本）
	var cards []struct {
		ID    string
		Title string
	}
	if err := s.db.WithContext(ctx).Table("cards").Select("id, title").Limit(maxGraphNodes).Find(&cards).Error; err != nil {
		return nil, err
	}

	// Step 2: 全量拉取所有边（仅必要字段）
	var allEdges []models.CardEdge
	if err := s.db.WithContext(ctx).
		Select("source_id, target_id, relation_type").
		Limit(maxGraphEdges).
		Find(&allEdges).Error; err != nil {
		return nil, err
	}

	// Step 3: 构建结果
	result := &GraphResult{
		Nodes: make([]GraphNode, 0, len(cards)),
		Edges: make([]GraphEdge, 0, len(allEdges)),
	}

	for _, c := range cards {
		result.Nodes = append(result.Nodes, GraphNode{
			ID:    c.ID,
			Title: c.Title,
		})
	}

	seen := make(map[string]bool, len(allEdges))
	for _, e := range allEdges {
		key := e.SourceID + ":" + e.TargetID
		if !seen[key] {
			seen[key] = true
			result.Edges = append(result.Edges, GraphEdge{
				Source:   e.SourceID,
				Target:   e.TargetID,
				Relation: e.RelationType,
			})
		}
	}

	return result, nil
}

// GetOutline 生成大纲视图数据（Category → Topic → Card → Cluster）。
// 可选按 categoryID 过滤，返回指定分类下的主题和卡片聚类。
func (s *GraphService) GetOutline(ctx context.Context, categoryID string) (*OutlineResult, error) {
	result := &OutlineResult{
		Topics:   []OutlineTopic{},
		Clusters: []OutlineCluster{},
	}

	// Fetch categories.
	var categories []models.Category
	query := s.db.WithContext(ctx).Model(&models.Category{})
	if categoryID != "" {
		query = query.Where("id = ?", categoryID)
	}
	if err := query.Find(&categories).Error; err != nil {
		return nil, err
	}

	// Batch count cards per category.
	type countResult struct {
		CategoryID uint
		Count      int64
	}
	var counts []countResult
	cardCountQuery := s.db.WithContext(ctx).Model(&models.Card{}).
		Select("category_id, count(id) as count").
		Group("category_id")
	if categoryID != "" {
		cardCountQuery = cardCountQuery.Where("category_id = ?", categoryID)
	}
	cardCountQuery.Scan(&counts)

	countMap := make(map[uint]int64)
	for _, c := range counts {
		countMap[c.CategoryID] = c.Count
	}

	for _, cat := range categories {
		result.Topics = append(result.Topics, OutlineTopic{
			ID:        strconv.FormatUint(uint64(cat.ID), 10),
			Label:     cat.Name,
			CardCount: int(countMap[cat.ID]),
		})
	}

	// Fetch recent cards.
	var cards []models.Card
	cardQuery := s.db.WithContext(ctx).Model(&models.Card{}).Select("id, title, category_id, created_at").
		Order("created_at DESC").Limit(50)
	if categoryID != "" {
		cardQuery = cardQuery.Where("category_id = ?", categoryID)
	}
	if err := cardQuery.Find(&cards).Error; err != nil {
		return nil, err
	}

	for _, card := range cards {
		result.Clusters = append(result.Clusters, OutlineCluster{
			ID:    card.ID,
			Title: card.Title,
			TopicID: func(id *uint) string {
				if id == nil {
					return "0"
				}
				return strconv.FormatUint(uint64(*id), 10)
			}(card.CategoryID),
			CreatedAt: card.CreatedAt,
		})
	}

	return result, nil
}
