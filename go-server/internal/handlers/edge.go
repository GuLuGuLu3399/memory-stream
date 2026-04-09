package handlers

import (
	"net/http"

	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/ws"
	"github.com/gin-gonic/gin"
)

// EdgeHandler handles edge CRUD HTTP requests.
type EdgeHandler struct {
	edgeSvc *services.EdgeService
	hub     *ws.Hub
}

// NewEdgeHandler creates a new EdgeHandler instance.
func NewEdgeHandler(edgeSvc *services.EdgeService, hub *ws.Hub) *EdgeHandler {
	return &EdgeHandler{edgeSvc: edgeSvc, hub: hub}
}

type CreateEdgeReq struct {
	SourceID     string `json:"source_id" binding:"required"`
	TargetID     string `json:"target_id" binding:"required"`
	RelationType string `json:"relation_type"`
}

// Create adds a new directed edge between two cards.
// POST /edges
func (h *EdgeHandler) Create(c *gin.Context) {
	var req CreateEdgeReq
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequestWithLog("参数解析失败", err.Error()))
		return
	}
	relationType := req.RelationType
	if relationType == "" {
		relationType = "reference"
	}
	if err := h.edgeSvc.CreateEdge(req.SourceID, req.TargetID, relationType); err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "连线已创建"})

	// WS 增量推送：通知所有在线客户端有新边
	if h.hub != nil {
		h.hub.BroadcastEvent(ws.WSEvent{
			Event: "EDGE_CREATED",
			Payload: ws.EdgeEventPayload{
				SourceID:     req.SourceID,
				TargetID:     req.TargetID,
				RelationType: relationType,
			},
		})
	}
}

type DeleteEdgeReq struct {
	SourceID string `json:"source_id" binding:"required"`
	TargetID string `json:"target_id" binding:"required"`
}

// Delete removes a directed edge between two cards.
// DELETE /edges
func (h *EdgeHandler) Delete(c *gin.Context) {
	var req DeleteEdgeReq
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequestWithLog("参数解析失败", err.Error()))
		return
	}
	if err := h.edgeSvc.DeleteEdge(req.SourceID, req.TargetID); err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "连线已删除"})

	// WS 增量推送：通知所有在线客户端边已删除
	if h.hub != nil {
		h.hub.BroadcastEvent(ws.WSEvent{
			Event: "EDGE_DELETED",
			Payload: ws.EdgeEventPayload{
				SourceID: req.SourceID,
				TargetID: req.TargetID,
			},
		})
	}
}

type UpdateEdgeReq struct {
	SourceID     string `json:"source_id" binding:"required"`
	TargetID     string `json:"target_id" binding:"required"`
	RelationType string `json:"relation_type" binding:"required"`
}

// Update changes the relation type of an existing edge.
// PUT /edges
func (h *EdgeHandler) Update(c *gin.Context) {
	var req UpdateEdgeReq
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequestWithLog("参数解析失败", err.Error()))
		return
	}
	if err := h.edgeSvc.UpdateEdgeType(req.SourceID, req.TargetID, req.RelationType); err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "连线已更新"})

	// WS 增量推送：通知所有在线客户端边类型已变更
	if h.hub != nil {
		h.hub.BroadcastEvent(ws.WSEvent{
			Event: "EDGE_UPDATED",
			Payload: ws.EdgeEventPayload{
				SourceID:     req.SourceID,
				TargetID:     req.TargetID,
				RelationType: req.RelationType,
			},
		})
	}
}
