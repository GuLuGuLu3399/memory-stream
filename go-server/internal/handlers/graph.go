
// ────────────────────────────────────────────────────────────────
// graph.go — Graph management handlers
// graph.go — 图管理处理器
// ────────────────────────────────────────────────────────────────


package handlers

import (
	"strconv"

	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/gin-gonic/gin"
)

type GraphHandler struct {
	graphSvc *services.GraphService
}

func NewGraphHandler(graphSvc *services.GraphService) *GraphHandler {
	return &GraphHandler{graphSvc: graphSvc}
}

func (h *GraphHandler) All(c *gin.Context) {
	graph, err := h.graphSvc.GetFullGraph(c.Request.Context())
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	appErr.RespondSuccess(c, graph)
}

func (h *GraphHandler) Neighborhood(c *gin.Context) {
	uuid := c.Param("uuid")
	if uuid == "" {
		appErr.Respond(c, appErr.NewBadRequest("uuid is required"))
		return
	}

	depth, _ := strconv.Atoi(c.DefaultQuery("depth", "2"))

	graph, err := h.graphSvc.GetNeighborhood(c.Request.Context(), uuid, depth)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	appErr.RespondSuccess(c, graph)
}
