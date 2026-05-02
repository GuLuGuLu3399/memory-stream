
// ────────────────────────────────────────────────────────────────
// search.go — Search management handlers
// search.go — 搜索管理处理器
// ────────────────────────────────────────────────────────────────


package handlers

import (
	"strconv"

	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/gin-gonic/gin"
)

type SearchHandler struct {
	searchSvc *services.SearchService
}

func NewSearchHandler(searchSvc *services.SearchService) *SearchHandler {
	return &SearchHandler{searchSvc: searchSvc}
}

func (h *SearchHandler) Search(c *gin.Context) {
	query := c.Query("q")
	if query == "" {
		appErr.Respond(c, appErr.NewBadRequest("搜索关键词不能为空"))
		return
	}

	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))
	offset, _ := strconv.Atoi(c.DefaultQuery("offset", "0"))

	hits, err := h.searchSvc.SearchCards(c.Request.Context(), query, limit, offset)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	appErr.RespondSuccess(c, gin.H{"results": hits})
}
