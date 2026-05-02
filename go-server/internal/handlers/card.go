
// ────────────────────────────────────────────────────────────────
// card.go — 卡片管理处理器
// ────────────────────────────────────────────────────────────────

package handlers

import (
	"errors"
	"strconv"
	"strings"

	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/gin-gonic/gin"
	"gorm.io/gorm"
)

type CardHandler struct {
	cardSvc *services.CardService
}

func NewCardHandler(cardSvc *services.CardService) *CardHandler {
	return &CardHandler{cardSvc: cardSvc}
}

func (h *CardHandler) List(c *gin.Context) {
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))
	offset, _ := strconv.Atoi(c.DefaultQuery("offset", "0"))
	category := c.Query("category")

	cards, err := h.cardSvc.ListCards(c.Request.Context(), category, limit, offset)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	appErr.RespondSuccess(c, gin.H{"cards": cards})
}

func (h *CardHandler) ListCategories(c *gin.Context) {
	categories, err := h.cardSvc.ListCategories(c.Request.Context())
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	appErr.RespondSuccess(c, gin.H{"categories": categories})
}

func (h *CardHandler) GetRandom(c *gin.Context) {
	count, _ := strconv.Atoi(c.DefaultQuery("count", "5"))

	cards, err := h.cardSvc.GetRandomCards(c.Request.Context(), count)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	appErr.RespondSuccess(c, gin.H{"cards": cards})
}

func (h *CardHandler) ResolveByTitle(c *gin.Context) {
	title := strings.TrimSpace(c.Query("title"))
	if title == "" {
		appErr.Respond(c, appErr.NewBadRequest("标题参数不能为空"))
		return
	}

	id, err := h.cardSvc.ResolveByTitle(c.Request.Context(), title)
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			appErr.Respond(c, appErr.NewNotFound("卡片不存在"))
		} else {
			appErr.Respond(c, appErr.NewInternal(err))
		}
		return
	}

	appErr.RespondSuccess(c, gin.H{"uuid": id.String()})
}

func (h *CardHandler) GetBacklinks(c *gin.Context) {
	uuid := c.Param("uuid")
	if uuid == "" {
		appErr.Respond(c, appErr.NewBadRequest("uuid 不能为空"))
		return
	}

	items, err := h.cardSvc.GetBacklinks(c.Request.Context(), uuid)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	appErr.RespondSuccess(c, gin.H{"backlinks": items})
}

func (h *CardHandler) GetByID(c *gin.Context) {
	uuid := c.Param("uuid")
	if uuid == "" {
		appErr.Respond(c, appErr.NewBadRequest("uuid 不能为空"))
		return
	}

	card, err := h.cardSvc.GetCardRead(c.Request.Context(), uuid)
	if err != nil {
		appErr.Respond(c, appErr.NewNotFound("卡片不存在"))
		return
	}

	appErr.RespondSuccess(c, card)
}
