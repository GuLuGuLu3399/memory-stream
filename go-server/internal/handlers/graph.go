package handlers

import (
	"net/http"
	"strconv"

	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/gin-gonic/gin"
)

// GraphHandler handles graph query HTTP requests.
type GraphHandler struct {
	graphSvc *services.GraphService
	cardSvc  *services.CardService
}

// NewGraphHandler creates a new GraphHandler instance.
func NewGraphHandler(graphSvc *services.GraphService, cardSvc *services.CardService) *GraphHandler {
	return &GraphHandler{graphSvc: graphSvc, cardSvc: cardSvc}
}

// Outline returns the graph outline, optionally filtered by category.
// GET /graph/outline
func (h *GraphHandler) Outline(c *gin.Context) {
	categoryIDStr := c.Query("category_id")

	outline, err := h.graphSvc.GetOutline(c.Request.Context(), categoryIDStr)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	c.JSON(http.StatusOK, outline)
}

// All returns the complete graph with all nodes and edges.
// GET /graph
func (h *GraphHandler) All(c *gin.Context) {
	graph, err := h.graphSvc.GetAllGraph(c.Request.Context())
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	c.JSON(http.StatusOK, graph)
}

// Detail returns the N-degree neighborhood graph around a specific card.
// GET /graph/:id
func (h *GraphHandler) Detail(c *gin.Context) {
	cardID := c.Param("id")
	depth, err := strconv.Atoi(c.DefaultQuery("depth", "2"))
	if err != nil {
		appErr.Respond(c, appErr.NewBadRequest("depth 必须是整数"))
		return
	}

	if depth < 1 || depth > 5 {
		depth = 2
	}

	graph, err := h.graphSvc.GetGraph(c.Request.Context(), cardID, depth)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	err = h.cardSvc.IncrementView(c.Request.Context(), cardID)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	c.JSON(http.StatusOK, graph)
}
