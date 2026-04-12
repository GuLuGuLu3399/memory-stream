package handlers

import (
	"net/http"

	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/ws"
	"github.com/gin-gonic/gin"
	"gorm.io/gorm"
)

// MergeHandler handles card merge HTTP requests.
type MergeHandler struct {
	DB  *gorm.DB
	Hub *ws.Hub
}

// NewMergeHandler creates a new MergeHandler instance.
func NewMergeHandler(db *gorm.DB, hub *ws.Hub) *MergeHandler {
	return &MergeHandler{DB: db, Hub: hub}
}

type mergeCardsReq struct {
	SurvivorID string   `json:"survivor_id" binding:"required"`
	VictimIDs  []string `json:"victim_ids" binding:"required,min=1"`
}

// MergeCards merges multiple victim cards into a survivor card, migrating edges atomically.
// POST /merge
func (h *MergeHandler) MergeCards(c *gin.Context) {
	var req mergeCardsReq
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequestWithLog("invalid request body", err.Error()))
		return
	}

	result, err := services.MergeCards(c.Request.Context(), h.DB, services.MergeRequest{
		SurvivorID: req.SurvivorID,
		VictimIDs:  req.VictimIDs,
	})
	if err != nil {
		if err.Error() == "survivor_id cannot be in victim_ids" {
			appErr.Respond(c, appErr.NewBadRequest(err.Error()))
			return
		}
		if err.Error() == "one or more card IDs not found" {
			appErr.Respond(c, appErr.NewNotFound(err.Error()))
			return
		}
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	// Broadcast WS events after successful merge
	if h.Hub != nil && result != nil {
		payload := gin.H{
			"survivor_id":    req.SurvivorID,
			"victim_ids":     req.VictimIDs,
			"edges_migrated": result.EdgesMigrated,
			"nodes_deleted":  result.NodesDeleted,
		}
		h.Hub.BroadcastEvent(ws.WSEvent{Event: "CARDS_MERGED", Payload: payload})

		for _, vid := range req.VictimIDs {
			h.Hub.BroadcastEvent(ws.WSEvent{Event: "CARD_DELETED", Payload: gin.H{"card_id": vid}})
		}

		h.Hub.BroadcastEvent(ws.WSEvent{Event: "CARD_UPDATED", Payload: gin.H{"card_id": req.SurvivorID}})
	}

	c.JSON(http.StatusOK, gin.H{
		"message":        "cards merged successfully",
		"edges_migrated": result.EdgesMigrated,
		"nodes_deleted":  result.NodesDeleted,
		"warnings":       result.Warnings,
	})
}
