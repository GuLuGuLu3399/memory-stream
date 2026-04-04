package handlers

import (
	"net/http"
	"strconv"

	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/middleware"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/ws"
	"github.com/gin-gonic/gin"
)

type CardHandler struct {
	cardSvc     *services.CardService
	edgeSvc     *services.EdgeService
	graphSvc    *services.GraphService
	rateLimiter *middleware.ViewRateLimiter
	hub         *ws.Hub
}

func NewCardHandler(cardSvc *services.CardService, edgeSvc *services.EdgeService, graphSvc *services.GraphService, rateLimiter *middleware.ViewRateLimiter, hub *ws.Hub) *CardHandler {
	return &CardHandler{cardSvc: cardSvc, edgeSvc: edgeSvc, graphSvc: graphSvc, rateLimiter: rateLimiter, hub: hub}
}

type CreateCardReq struct {
	Title        string       `json:"title"`
	RawMd        string       `json:"raw_md" binding:"required"`
	Excerpt      string       `json:"excerpt"`
	AstData      models.JSONB `json:"ast_data"`
	TocData      models.JSONB `json:"toc_data"`
	ParentID     *string      `json:"parent_id,omitempty"`
	RelationType string       `json:"relation_type,omitempty"`
}

func (h *CardHandler) Create(c *gin.Context) {
	var req CreateCardReq
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequestWithLog("参数解析失败", err.Error()))
		return
	}

	astData := req.AstData
	if astData == nil {
		astData = models.JSONB("{}")
	}

	tocData := req.TocData
	card, err := h.cardSvc.CreateCard(req.Title, req.RawMd, req.Excerpt, astData, tocData)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	if req.ParentID != nil && *req.ParentID != "" {
		relationType := req.RelationType
		if relationType == "" {
			relationType = "sequence"
		}
		if err := h.edgeSvc.CreateEdge(*req.ParentID, card.ID, relationType); err != nil {
			appErr.Respond(c, appErr.NewInternal(err))
			return
		}
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "卡片已存入记忆流",
		"card_id": card.ID,
	})

	// WS 增量推送：通知所有在线客户端有新卡片
	if h.hub != nil {
		h.hub.BroadcastEvent(ws.WSEvent{
			Event: "CARD_CREATED",
			Payload: ws.CardEventPayload{
				CardID:  card.ID,
				Title:   req.Title,
				Excerpt: req.Excerpt,
			},
		})
	}
}

func (h *CardHandler) List(c *gin.Context) {
	cursor := c.Query("cursor")
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))

	result, err := h.cardSvc.ListCards(services.CursorPage{Cursor: cursor, Limit: limit})
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, result)
}

func (h *CardHandler) Discover(c *gin.Context) {
	sort := c.DefaultQuery("sort", "latest")
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))

	result, err := h.cardSvc.GetDiscover(sort, services.OffsetPage{Page: page, PageSize: pageSize})
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, result)
}

func (h *CardHandler) GetByID(c *gin.Context) {
	cardID := c.Param("id")

	card, err := h.cardSvc.GetCardByID(cardID)
	if err != nil {
		appErr.Respond(c, appErr.NewNotFound("卡片未找到"))
		return
	}

	c.JSON(http.StatusOK, card)
}

func (h *CardHandler) Graph(c *gin.Context) {
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

func (h *CardHandler) IncrementView(c *gin.Context) {
	cardID := c.Param("id")
	clientIP := c.ClientIP()

	if !h.rateLimiter.Allow(clientIP, cardID) {
		c.JSON(http.StatusOK, gin.H{"message": "热度已更新", "card_id": cardID})
		return
	}

	if err := h.cardSvc.IncrementView(cardID); err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "热度已更新", "card_id": cardID})
}

type UpdateCardReq struct {
	Title          string       `json:"title" binding:"required"`
	RawMd          string       `json:"raw_md" binding:"required"`
	Excerpt        string       `json:"excerpt"`
	AstData        models.JSONB `json:"ast_data" binding:"required"`
	TocData        models.JSONB `json:"toc_data"`
	CategoryID     *uint        `json:"category_id,omitempty"`
	ExtractedLinks []string     `json:"extracted_links"`
}

func (h *CardHandler) Update(c *gin.Context) {
	id := c.Param("id")
	var req UpdateCardReq
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequestWithLog("参数解析失败", err.Error()))
		return
	}
	if err := h.cardSvc.UpdateCard(id, req.Title, req.RawMd, req.Excerpt, req.AstData, req.TocData, req.CategoryID); err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	// Sync reference edges if extracted_links is provided
	if len(req.ExtractedLinks) > 0 {
		var resolvedIDs []string
		for _, title := range req.ExtractedLinks {
			if title == "" {
				continue
			}
			card, err := h.cardSvc.FindOrCreateByTitle(title)
			if err != nil {
				logger.Log.Warnf("Failed to resolve title '%s' to card ID: %v", title, err)
				continue
			}
			resolvedIDs = append(resolvedIDs, card.ID)
		}
		if len(resolvedIDs) > 0 {
			if err := h.edgeSvc.SyncReferenceEdges(id, resolvedIDs); err != nil {
				logger.Log.Warnf("Failed to sync reference edges for card %s: %v", id, err)
				// Don't fail the card save - edge sync is secondary
			}
		}
	}

	c.JSON(http.StatusOK, gin.H{"message": "卡片已更新", "card_id": id})

	// WS 增量推送：通知所有在线客户端卡片已更新
	if h.hub != nil {
		h.hub.BroadcastEvent(ws.WSEvent{
			Event: "CARD_UPDATED",
			Payload: ws.CardEventPayload{
				CardID: id,
				Title:  req.Title,
			},
		})
	}
}

// GetBacklinks 获取指向当前卡片的所有反向引用。
// GET /cards/:id/backlinks
func (h *CardHandler) GetBacklinks(c *gin.Context) {
	cardID := c.Param("id")

	backlinks, err := h.cardSvc.GetBacklinks(cardID)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"card_id":   cardID,
		"backlinks": backlinks,
	})
}

func (h *CardHandler) Delete(c *gin.Context) {
	id := c.Param("id")
	if err := h.cardSvc.DeleteCard(id); err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "卡片已删除", "card_id": id})

	// WS 增量推送：通知所有在线客户端卡片已删除
	if h.hub != nil {
		h.hub.BroadcastEvent(ws.WSEvent{
			Event: "CARD_DELETED",
			Payload: ws.CardEventPayload{
				CardID: id,
			},
		})
	}
}
