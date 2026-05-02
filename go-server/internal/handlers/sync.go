// Package handlers SyncHandler handles synchronization endpoints.
// BatchUpsert accepts a batch of cards and returns accepted/conflicts/rejected lists.
// GetManifest returns incremental card changes since the given cursor timestamp.
// DeleteCard soft-deletes a card by UUID.
// GetCard returns a single card by UUID.
package handlers

import (
	"fmt"
	"net/http"
	"time"

	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/gin-gonic/gin"
)

// SyncHandler handles synchronization endpoints.
type SyncHandler struct {
	syncSvc *services.SyncService
}

// NewSyncHandler creates a new SyncHandler.
func NewSyncHandler(syncSvc *services.SyncService) *SyncHandler {
	return &SyncHandler{syncSvc: syncSvc}
}

// BatchUpsert accepts a batch of cards and returns accepted/conflicts/rejected lists.
func (h *SyncHandler) BatchUpsert(c *gin.Context) {
	var req models.SyncBatchRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequest("无效的请求体: "+err.Error()))
		return
	}

	const maxBatchSize = 100
	if len(req.Cards) > maxBatchSize {
		appErr.Respond(c, appErr.NewBadRequest(
			fmt.Sprintf("batch size %d exceeds limit %d", len(req.Cards), maxBatchSize),
		))
		return
	}

	resp, err := h.syncSvc.BatchUpsert(c.Request.Context(), req)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	c.JSON(http.StatusOK, resp)
}

// GetManifest returns incremental card changes since the given cursor timestamp.
func (h *SyncHandler) GetManifest(c *gin.Context) {
	var since time.Time
	if raw := c.Query("since"); raw != "" {
		parsed, err := time.Parse(time.RFC3339, raw)
		if err != nil {
			appErr.Respond(c, appErr.NewBadRequest("invalid since parameter, expected RFC3339"))
			return
		}
		since = parsed
	} else {
		since = time.Time{} // zero value = Unix epoch → fetch all
	}

	resp, err := h.syncSvc.GetManifest(c.Request.Context(), since)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	c.JSON(http.StatusOK, resp)
}

// DeleteCard soft-deletes a card by UUID.
func (h *SyncHandler) DeleteCard(c *gin.Context) {
	uuid := c.Param("uuid")
	if uuid == "" {
		appErr.Respond(c, appErr.NewBadRequest("uuid is required"))
		return
	}

	if err := h.syncSvc.DeleteCard(c.Request.Context(), uuid); err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	appErr.RespondSuccess(c, gin.H{
		"id": uuid,
		"op": "delete",
	})
}

// GetCard returns a single card by UUID with its edges.
func (h *SyncHandler) GetCard(c *gin.Context) {
	uuid := c.Param("uuid")
	if uuid == "" {
		appErr.Respond(c, appErr.NewBadRequest("uuid is required"))
		return
	}

	card, err := h.syncSvc.GetCard(c.Request.Context(), uuid)
	if err != nil {
		appErr.Respond(c, appErr.NewNotFound("卡片不存在"))
		return
	}

	appErr.RespondSuccess(c, card)
}

// SyncRelations accepts a full set of trunk relations and replaces existing ones.
func (h *SyncHandler) SyncRelations(c *gin.Context) {
	var req models.RelationsSyncRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequest("无效的请求体: "+err.Error()))
		return
	}

	resp, err := h.syncSvc.SyncRelations(c.Request.Context(), req)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	c.JSON(http.StatusOK, resp)
}

// GetAllTrunks returns all trunk relations for full pull.
func (h *SyncHandler) GetAllTrunks(c *gin.Context) {
	trunks, err := h.syncSvc.GetAllTrunks(c.Request.Context())
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	appErr.RespondSuccess(c, trunks)
}
