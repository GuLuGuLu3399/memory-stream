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

type GlossaryHandler struct {
	glossarySvc *services.GlossaryService
}

func NewGlossaryHandler(glossarySvc *services.GlossaryService) *GlossaryHandler {
	return &GlossaryHandler{glossarySvc: glossarySvc}
}

func (h *GlossaryHandler) List(c *gin.Context) {
	items, err := h.glossarySvc.GetAll(c.Request.Context())
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	appErr.RespondSuccess(c, gin.H{"glossary": items})
}

func (h *GlossaryHandler) Slim(c *gin.Context) {
	data, err := h.glossarySvc.GetAllSlim(c.Request.Context())
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	appErr.RespondSuccess(c, data)
}

func (h *GlossaryHandler) SyncManifest(c *gin.Context) {
	var since time.Time
	if raw := c.Query("since"); raw != "" {
		parsed, err := time.Parse(time.RFC3339, raw)
		if err != nil {
			appErr.Respond(c, appErr.NewBadRequest("invalid since parameter, expected RFC3339"))
			return
		}
		since = parsed
	}

	resp, err := h.glossarySvc.GetManifest(c.Request.Context(), since)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	c.JSON(http.StatusOK, resp)
}

func (h *GlossaryHandler) SyncBatchUpsert(c *gin.Context) {
	var req models.GlossaryBatchRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequest("无效的请求体: "+err.Error()))
		return
	}

	const maxBatchSize = 100
	if len(req.Items) > maxBatchSize {
		appErr.Respond(c, appErr.NewBadRequest(
			fmt.Sprintf("batch size %d exceeds limit %d", len(req.Items), maxBatchSize),
		))
		return
	}

	resp, err := h.glossarySvc.BatchUpsert(c.Request.Context(), req)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	c.JSON(http.StatusOK, resp)
}
