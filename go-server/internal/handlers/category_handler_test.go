//go:build integration

package handlers

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"go.uber.org/zap"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

func setupCategoryTestDB(t *testing.T) *gorm.DB {
	t.Helper()
	db, err := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{
		DisableForeignKeyConstraintWhenMigrating: true,
	})
	if err != nil {
		t.Fatalf("failed to open sqlite: %v", err)
	}
	if err := db.AutoMigrate(&models.Category{}); err != nil {
		t.Fatalf("failed to migrate: %v", err)
	}
	return db
}

func setupCategoryHandlerValidationTest(t *testing.T) *gin.Engine {
	t.Helper()
	gin.SetMode(gin.TestMode)

	if logger.Log == nil {
		l, _ := zap.NewProduction()
		logger.Log = l.Sugar()
	}

	handler := NewCategoryHandler(nil)

	router := gin.New()
	router.Use(appErr.ErrorHandler())
	v1 := router.Group("/api/v1")
	{
		v1.GET("/categories", handler.List)
		v1.POST("/categories", handler.Create)
		v1.PUT("/categories/:id", handler.Update)
		v1.DELETE("/categories/:id", handler.Delete)
	}

	return router
}

func setupCategoryHandlerTest(t *testing.T) (*gorm.DB, *gin.Engine) {
	t.Helper()
	gin.SetMode(gin.TestMode)

	if logger.Log == nil {
		l, _ := zap.NewProduction()
		logger.Log = l.Sugar()
	}

	db := setupCategoryTestDB(t)
	catSvc := services.NewCategoryService(db)
	handler := NewCategoryHandler(catSvc)

	router := gin.New()
	router.Use(appErr.ErrorHandler())
	v1 := router.Group("/api/v1")
	{
		v1.GET("/categories", handler.List)
		v1.GET("/categories/tree", handler.GetTree)
		v1.POST("/categories", handler.Create)
		v1.PUT("/categories/:id", handler.Update)
		v1.DELETE("/categories/:id", handler.Delete)
	}

	return db, router
}

func TestCategoryHandler_Create_EmptyBody(t *testing.T) {
	router := setupCategoryHandlerValidationTest(t)

	req := httptest.NewRequest(http.MethodPost, "/api/v1/categories", strings.NewReader(`{}`))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)
}

func TestCategoryHandler_Delete_InvalidID(t *testing.T) {
	router := setupCategoryHandlerValidationTest(t)

	req := httptest.NewRequest(http.MethodDelete, "/api/v1/categories/abc", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)
	assert.Contains(t, w.Body.String(), "无效的分类 ID")
}

func TestCategoryHandler_Update_InvalidID(t *testing.T) {
	router := setupCategoryHandlerValidationTest(t)

	body := `{"name":"test"}`
	req := httptest.NewRequest(http.MethodPut, "/api/v1/categories/abc", strings.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)
	assert.Contains(t, w.Body.String(), "无效的分类 ID")
}

func TestCategoryHandler_List_Success(t *testing.T) {
	db, router := setupCategoryHandlerTest(t)

	cat1 := &models.Category{Name: "Go", Description: "Go language"}
	cat2 := &models.Category{Name: "Rust", Description: "Rust language"}
	db.Create(cat1)
	db.Create(cat2)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/categories", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var response map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(t, err)

	categories, ok := response["categories"].([]interface{})
	assert.True(t, ok)
	assert.Len(t, categories, 2)
}

func TestCategoryHandler_GetTree_Success(t *testing.T) {
	db, router := setupCategoryHandlerTest(t)

	parentCat := &models.Category{Name: "Parent", Description: "parent cat"}
	db.Create(&parentCat)

	childCat := &models.Category{
		Name:        "Child",
		Description: "child cat",
		ParentID:    &parentCat.ID,
		SortOrder:   1,
	}
	db.Create(&childCat)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/categories/tree", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var response map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(t, err)

	categories, ok := response["categories"].([]interface{})
	assert.True(t, ok)
	assert.Len(t, categories, 1)

	parentMap, ok := categories[0].(map[string]interface{})
	assert.True(t, ok)
	assert.Equal(t, "Parent", parentMap["name"])

	children, ok := parentMap["children"].([]interface{})
	assert.True(t, ok)
	assert.Len(t, children, 1)
}
