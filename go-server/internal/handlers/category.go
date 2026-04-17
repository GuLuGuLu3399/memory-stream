package handlers

import (
	"errors"
	"net/http"
	"strconv"

	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/gin-gonic/gin"
)

// CategoryHandler handles category CRUD HTTP requests.
type CategoryHandler struct {
	service *services.CategoryService
}

// NewCategoryHandler creates a new CategoryHandler instance.
func NewCategoryHandler(service *services.CategoryService) *CategoryHandler {
	return &CategoryHandler{service: service}
}

// List returns all categories as a flat list.
// GET /categories
func (h *CategoryHandler) List(c *gin.Context) {
	categories, err := h.service.ListAll(c.Request.Context())
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, gin.H{"categories": categories})
}

// GetTree returns categories organized as a tree structure.
// GET /categories/tree
func (h *CategoryHandler) GetTree(c *gin.Context) {
	tree, err := h.service.GetTree(c.Request.Context())
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, gin.H{"categories": tree})
}

// Create adds a new category.
// POST /categories
func (h *CategoryHandler) Create(c *gin.Context) {
	var req struct {
		Name        string  `json:"name" binding:"required"`
		Description string  `json:"description"`
		ParentID    *uint   `json:"parent_id"`
		ThemeColor  *string `json:"theme_color"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequestWithLog("参数解析失败", err.Error()))
		return
	}
	cat, err := h.service.Create(c.Request.Context(), req.Name, req.Description, req.ThemeColor, req.ParentID)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, gin.H{"category": cat})
}

// Update modifies an existing category.
// PUT /categories/:id
func (h *CategoryHandler) Update(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		appErr.Respond(c, appErr.NewBadRequest("无效的分类 ID"))
		return
	}
	var req struct {
		Name        string  `json:"name" binding:"required"`
		Description string  `json:"description"`
		ParentID    *uint   `json:"parent_id"`
		ThemeColor  *string `json:"theme_color"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequestWithLog("参数解析失败", err.Error()))
		return
	}
	if err := h.service.Update(c.Request.Context(), uint(id), req.Name, req.Description, req.ThemeColor, req.ParentID); err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "分类已更新"})
}

// Delete removes a category by ID.
// DELETE /categories/:id
func (h *CategoryHandler) Delete(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		appErr.Respond(c, appErr.NewBadRequest("无效的分类 ID"))
		return
	}
	if err := h.service.Delete(c.Request.Context(), uint(id)); err != nil {
		// If the category has children, return HTTP 409 per contract
		if errors.Is(err, services.ErrCategoryHasChildren) {
			appErr.Respond(c, appErr.Wrap(err, 409, 40901, err.Error()))
			return
		}
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "分类已删除"})
}

// GetClusters returns graph clusters for a category.
// GET /categories/:id/clusters
func (h *CategoryHandler) GetClusters(c *gin.Context) {
	idStr := c.Param("id")
	id, err := strconv.ParseUint(idStr, 10, 64)
	if err != nil {
		appErr.Respond(c, appErr.NewBadRequest("无效的分类 ID"))
		return
	}

	clusters, err := h.service.GetClusters(c.Request.Context(), uint(id))
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, gin.H{"clusters": clusters})
}
