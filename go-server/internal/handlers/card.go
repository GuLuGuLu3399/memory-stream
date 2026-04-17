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

// CardHandler handles card CRUD and graph-related HTTP requests.
type CardHandler struct {
	cardSvc     *services.CardService
	edgeSvc     *services.EdgeService
	graphSvc    *services.GraphService
	rateLimiter *middleware.ViewRateLimiter
	hub         *ws.Hub
}

// NewCardHandler creates a new CardHandler instance.
func NewCardHandler(cardSvc *services.CardService, edgeSvc *services.EdgeService, graphSvc *services.GraphService, rateLimiter *middleware.ViewRateLimiter, hub *ws.Hub) *CardHandler {
	return &CardHandler{cardSvc: cardSvc, edgeSvc: edgeSvc, graphSvc: graphSvc, rateLimiter: rateLimiter, hub: hub}
}

type CreateCardReq struct {
	Title        string       `json:"title"`
	RawMd        string       `json:"raw_md" binding:"required"`
	Excerpt      string       `json:"excerpt"`
	AstData      models.JSONB `json:"ast_data"`
	TocData      models.JSONB `json:"toc_data"`
	CategoryID   *uint        `json:"category_id,omitempty"`
	ParentID     *string      `json:"parent_id,omitempty"`
	RelationType string       `json:"relation_type,omitempty"`
	ExtractedLinks []string   `json:"extracted_links"`
}

// Create creates a new card and optionally links it to a parent via an edge.
// POST /cards
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
	card, err := h.cardSvc.CreateCard(c.Request.Context(), req.Title, req.RawMd, req.Excerpt, astData, tocData, req.CategoryID)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	if req.ParentID != nil && *req.ParentID != "" {
		relationType := req.RelationType
		if relationType == "" {
			relationType = "sequence"
		}
		if err := h.edgeSvc.CreateEdge(c.Request.Context(), *req.ParentID, card.ID, relationType); err != nil {
			appErr.Respond(c, appErr.NewInternal(err))
			return
		}
	}

	// 首次创建即同步 wikilink reference 边，避免“新卡片初始无连接”
	if len(req.ExtractedLinks) > 0 {
		var resolvedIDs []string
		for _, title := range req.ExtractedLinks {
			if title == "" {
				continue
			}
			resolvedCard, err := h.cardSvc.FindOrCreateByTitle(c.Request.Context(), title)
			if err != nil {
				logger.Log.Warnf("Failed to resolve title '%s' to card ID: %v", title, err)
				continue
			}
			resolvedIDs = append(resolvedIDs, resolvedCard.ID)
		}
		if len(resolvedIDs) > 0 {
			if err := h.edgeSvc.SyncReferenceEdges(c.Request.Context(), card.ID, resolvedIDs); err != nil {
				logger.Log.Warnf("Failed to sync reference edges for card %s: %v", card.ID, err)
			}
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

// List returns a cursor-paginated list of cards.
// GET /cards
func (h *CardHandler) List(c *gin.Context) {
	cursor := c.Query("cursor")
	limit, err := strconv.Atoi(c.DefaultQuery("limit", "20"))
	if err != nil {
		appErr.Respond(c, appErr.NewBadRequest("limit 必须是整数"))
		return
	}

	result, err := h.cardSvc.ListCards(c.Request.Context(), services.CursorPage{Cursor: cursor, Limit: limit})
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, result)
}

// Discover returns cards sorted by various criteria with offset pagination.
// GET /cards/discover
func (h *CardHandler) Discover(c *gin.Context) {
	sort := c.DefaultQuery("sort", "latest")
	page, err := strconv.Atoi(c.DefaultQuery("page", "1"))
	if err != nil {
		appErr.Respond(c, appErr.NewBadRequest("page 必须是整数"))
		return
	}
	pageSize, err := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if err != nil {
		appErr.Respond(c, appErr.NewBadRequest("page_size 必须是整数"))
		return
	}

	result, err := h.cardSvc.GetDiscover(c.Request.Context(), sort, services.OffsetPage{Page: page, PageSize: pageSize})
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, result)
}

// GetByID returns a single card by its ID.
// GET /cards/:id?view=compact — excludes raw_md, ast_data, toc_data for sidebar/preview.
// GET /cards/:id — returns full card (default).
func (h *CardHandler) GetByID(c *gin.Context) {
	cardID := c.Param("id")
	compact := c.Query("view") == "compact"

	var card interface{}
	var err error
	if compact {
		card, err = h.cardSvc.GetCardCompact(c.Request.Context(), cardID)
	} else {
		card, err = h.cardSvc.GetCardByID(c.Request.Context(), cardID)
	}

	if err != nil {
		appErr.Respond(c, appErr.NewNotFound("卡片未找到"))
		return
	}

	c.JSON(http.StatusOK, card)
}

// Graph returns the neighborhood graph around a card and increments its view count.
// GET /cards/:id/graph
func (h *CardHandler) Graph(c *gin.Context) {
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

// IncrementView increments a card's view count with IP-based rate limiting.
// POST /cards/:id/view
func (h *CardHandler) IncrementView(c *gin.Context) {
	cardID := c.Param("id")
	clientIP := c.ClientIP()

	if !h.rateLimiter.Allow(clientIP, cardID) {
		c.JSON(http.StatusOK, gin.H{"message": "热度已更新", "card_id": cardID})
		return
	}

	if err := h.cardSvc.IncrementView(c.Request.Context(), cardID); err != nil {
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

// Update modifies a card's content and syncs wikilink reference edges.
// PUT /cards/:id
func (h *CardHandler) Update(c *gin.Context) {
	id := c.Param("id")
	var req UpdateCardReq
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequestWithLog("参数解析失败", err.Error()))
		return
	}
	if err := h.cardSvc.UpdateCard(c.Request.Context(), id, req.Title, req.RawMd, req.Excerpt, req.AstData, req.TocData, req.CategoryID); err != nil {
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
			card, err := h.cardSvc.FindOrCreateByTitle(c.Request.Context(), title)
			if err != nil {
				logger.Log.Warnf("Failed to resolve title '%s' to card ID: %v", title, err)
				continue
			}
			resolvedIDs = append(resolvedIDs, card.ID)
		}
		if len(resolvedIDs) > 0 {
			if err := h.edgeSvc.SyncReferenceEdges(c.Request.Context(), id, resolvedIDs); err != nil {
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

	backlinks, err := h.cardSvc.GetBacklinks(c.Request.Context(), cardID)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"card_id":   cardID,
		"backlinks": backlinks,
	})
}

// Delete removes a card by ID and broadcasts the deletion via WebSocket.
// DELETE /cards/:id
func (h *CardHandler) Delete(c *gin.Context) {
	id := c.Param("id")
	if err := h.cardSvc.DeleteCard(c.Request.Context(), id); err != nil {
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
