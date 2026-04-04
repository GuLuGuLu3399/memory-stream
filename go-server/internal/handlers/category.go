package handlers

import (
	"net/http"
	"strconv"

	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/gin-gonic/gin"
)

type CategoryHandler struct {
	service *services.CategoryService
}

func NewCategoryHandler(service *services.CategoryService) *CategoryHandler {
	return &CategoryHandler{service: service}
}

func (h *CategoryHandler) List(c *gin.Context) {
	categories, err := h.service.ListAll()
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, gin.H{"categories": categories})
}

func (h *CategoryHandler) GetTree(c *gin.Context) {
	tree, err := h.service.GetTree()
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, gin.H{"categories": tree})
}

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
	cat, err := h.service.Create(req.Name, req.Description, req.ThemeColor, req.ParentID)
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, gin.H{"category": cat})
}

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
	if err := h.service.Update(uint(id), req.Name, req.Description, req.ThemeColor, req.ParentID); err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "分类已更新"})
}

func (h *CategoryHandler) Delete(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		appErr.Respond(c, appErr.NewBadRequest("无效的分类 ID"))
		return
	}
	if err := h.service.Delete(uint(id)); err != nil {
		// If the category has children, return HTTP 409 per contract
		if err.Error() == "category has children" {
			appErr.Respond(c, appErr.Wrap(err, 409, 40901, err.Error()))
			return
		}
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "分类已删除"})
}

func (h *CategoryHandler) GetClusters(c *gin.Context) {
	idStr := c.Param("id")
	id, err := strconv.ParseUint(idStr, 10, 64)
	if err != nil {
		appErr.Respond(c, appErr.NewBadRequest("无效的分类 ID"))
		return
	}

	clusters, err := h.service.GetClusters(uint(id))
	if err != nil {
		appErr.Respond(c, appErr.NewInternal(err))
		return
	}
	c.JSON(http.StatusOK, gin.H{"clusters": clusters})
}
