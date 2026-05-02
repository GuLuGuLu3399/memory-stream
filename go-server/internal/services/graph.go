package services

import (
	"context"
	"fmt"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type GraphService struct {
	db *gorm.DB
}

func NewGraphService(db *gorm.DB) *GraphService {
	return &GraphService{db: db}
}

func (s *GraphService) GetFullGraph(ctx context.Context) (*models.FullGraph, error) {
	graph := &models.FullGraph{
		Nodes: []models.GraphNode{},
		Edges: []models.GraphEdge{},
	}

	var cards []models.Card
	if err := s.db.WithContext(ctx).
		Select("uuid, title").
		Where("deleted_at IS NULL").
		Find(&cards).Error; err != nil {
		return nil, fmt.Errorf("failed to query cards: %w", err)
	}

	for _, c := range cards {
		graph.Nodes = append(graph.Nodes, models.GraphNode{
			ID:    c.UUID.String(),
			Title: c.Title,
		})
	}

	var relations []models.Relation
	if err := s.db.WithContext(ctx).Find(&relations).Error; err != nil {
		return nil, fmt.Errorf("failed to query relations: %w", err)
	}

	for _, r := range relations {
		graph.Edges = append(graph.Edges, models.GraphEdge{
			Source:       r.SourceUUID.String(),
			Target:       r.TargetUUID.String(),
			RelationType: string(r.RelationType),
		})
	}

	return graph, nil
}

func (s *GraphService) GetNeighborhood(ctx context.Context, rawUUID string, depth int) (*models.FullGraph, error) {
	if depth <= 0 || depth > 5 {
		depth = 2
	}

	centerUUID, err := uuid.Parse(rawUUID)
	if err != nil {
		return nil, fmt.Errorf("invalid uuid: %w", err)
	}

	visited := map[uuid.UUID]bool{centerUUID: true}
	currentLevel := []uuid.UUID{centerUUID}

	var allNodes []models.GraphNode
	var allEdges []models.GraphEdge

	// BFS
	for d := 0; d < depth; d++ {
		if len(currentLevel) == 0 {
			break
		}

		// Find all edges where source or target is in currentLevel
		var rels []models.Relation
		if err := s.db.WithContext(ctx).
			Where("source_uuid IN ? OR target_uuid IN ?", currentLevel, currentLevel).
			Find(&rels).Error; err != nil {
			return nil, fmt.Errorf("failed to query relations: %w", err)
		}

		var nextLevel []uuid.UUID
		for _, r := range rels {
			allEdges = append(allEdges, models.GraphEdge{
				Source:       r.SourceUUID.String(),
				Target:       r.TargetUUID.String(),
				RelationType: string(r.RelationType),
			})

			for _, u := range []uuid.UUID{r.SourceUUID, r.TargetUUID} {
				if !visited[u] {
					visited[u] = true
					nextLevel = append(nextLevel, u)
				}
			}
		}

		currentLevel = nextLevel
	}

	// Fetch node data for all visited UUIDs
	allUUIDs := make([]uuid.UUID, 0, len(visited))
	for u := range visited {
		allUUIDs = append(allUUIDs, u)
	}

	var cards []models.Card
	if len(allUUIDs) > 0 {
		if err := s.db.WithContext(ctx).
			Select("uuid, title").
			Where("uuid IN ? AND deleted_at IS NULL", allUUIDs).
			Find(&cards).Error; err != nil {
			return nil, fmt.Errorf("failed to query cards: %w", err)
		}
	}

	for _, c := range cards {
		allNodes = append(allNodes, models.GraphNode{
			ID:    c.UUID.String(),
			Title: c.Title,
		})
	}

	return &models.FullGraph{Nodes: allNodes, Edges: allEdges}, nil
}
