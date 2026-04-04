package handlers

import (
	"net/http"
	"strconv"

	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/gin-gonic/gin"
)

type GraphHandler struct {
	graphSvc *services.GraphService
	cardSvc  *services.CardService
}

func NewGraphHandler(graphSvc *services.GraphService, cardSvc *services.CardService) *GraphHandler {
	return &GraphHandler{graphSvc: graphSvc, cardSvc: cardSvc}
}

func (h *GraphHandler) Outline(c *gin.Context) {
	categoryIDStr := c.Query("category_id")

	outline, err := h.graphSvc.GetOutline(categoryIDStr)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	c.JSON(http.StatusOK, outline)
}

func (h *GraphHandler) All(c *gin.Context) {
	graph, err := h.graphSvc.GetAllGraph()
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	c.JSON(http.StatusOK, graph)
}

func (h *GraphHandler) Detail(c *gin.Context) {
	cardID := c.Param("id")
	depth, _ := strconv.Atoi(c.DefaultQuery("depth", "2"))

	if depth < 1 || depth > 5 {
		depth = 2
	}

	graph, err := h.graphSvc.GetGraph(cardID, depth)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	err = h.cardSvc.IncrementView(cardID)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	c.JSON(http.StatusOK, graph)
}
