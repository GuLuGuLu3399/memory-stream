package handlers

import (
	"net/http"
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
		appErr.Respond(c, appErr.NewBadRequest("search query cannot be empty"))
		return
	}

	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))
	offset, _ := strconv.Atoi(c.DefaultQuery("offset", "0"))

	if limit > 100 {
		appErr.Respond(c, appErr.NewBadRequest("limit cannot exceed 100"))
		return
	}

	if offset < 0 {
		appErr.Respond(c, appErr.NewBadRequest("offset cannot be negative"))
		return
	}

	results, total, err := h.searchSvc.SearchCards(query, limit, offset)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"results": results,
		"total":   total,
		"query":   query,
	})
}
