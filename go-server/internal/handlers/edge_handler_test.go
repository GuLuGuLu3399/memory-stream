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

func setupEdgeTestDB(t *testing.T) *gorm.DB {
	t.Helper()
	db, err := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{
		DisableForeignKeyConstraintWhenMigrating: true,
	})
	if err != nil {
		t.Fatalf("failed to open sqlite: %v", err)
	}
	if err := db.AutoMigrate(&models.CardEdge{}); err != nil {
		t.Fatalf("failed to migrate: %v", err)
	}
	return db
}

func setupEdgeHandlerValidationTest(t *testing.T) *gin.Engine {
	t.Helper()
	gin.SetMode(gin.TestMode)

	if logger.Log == nil {
		l, _ := zap.NewProduction()
		logger.Log = l.Sugar()
	}

	handler := NewEdgeHandler(nil, nil)

	router := gin.New()
	router.Use(appErr.ErrorHandler())
	v1 := router.Group("/api/v1")
	{
		v1.POST("/edges", handler.Create)
		v1.DELETE("/edges", handler.Delete)
		v1.PUT("/edges", handler.Update)
	}

	return router
}

func setupEdgeHandlerTest(t *testing.T) (*gorm.DB, *gin.Engine) {
	t.Helper()
	gin.SetMode(gin.TestMode)

	if logger.Log == nil {
		l, _ := zap.NewProduction()
		logger.Log = l.Sugar()
	}

	db := setupEdgeTestDB(t)
	edgeSvc := services.NewEdgeService(db, nil)
	handler := NewEdgeHandler(edgeSvc, nil)

	router := gin.New()
	router.Use(appErr.ErrorHandler())
	v1 := router.Group("/api/v1")
	{
		v1.POST("/edges", handler.Create)
		v1.DELETE("/edges", handler.Delete)
		v1.PUT("/edges", handler.Update)
	}

	return db, router
}

func TestEdgeHandler_Create_EmptyBody(t *testing.T) {
	router := setupEdgeHandlerValidationTest(t)

	req := httptest.NewRequest(http.MethodPost, "/api/v1/edges", strings.NewReader(`{}`))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)
}

func TestEdgeHandler_Delete_EmptyBody(t *testing.T) {
	router := setupEdgeHandlerValidationTest(t)

	req := httptest.NewRequest(http.MethodDelete, "/api/v1/edges", strings.NewReader(`{}`))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)
}

func TestEdgeHandler_Update_EmptyBody(t *testing.T) {
	router := setupEdgeHandlerValidationTest(t)

	req := httptest.NewRequest(http.MethodPut, "/api/v1/edges", strings.NewReader(`{}`))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)
}

func TestEdgeHandler_Create_Success(t *testing.T) {
	_, router := setupEdgeHandlerTest(t)

	body := `{"source_id":"src-1","target_id":"tgt-1","relation_type":"reference"}`
	req := httptest.NewRequest(http.MethodPost, "/api/v1/edges", strings.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var response map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.Equal(t, "连线已创建", response["message"])
}

func TestEdgeHandler_Delete_Success(t *testing.T) {
	db, router := setupEdgeHandlerTest(t)

	edge := &models.CardEdge{
		SourceID:     "src-1",
		TargetID:     "tgt-1",
		RelationType: "reference",
	}
	if err := db.Create(edge).Error; err != nil {
		t.Fatalf("failed to create edge: %v", err)
	}

	body := `{"source_id":"src-1","target_id":"tgt-1"}`
	req := httptest.NewRequest(http.MethodDelete, "/api/v1/edges", strings.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var response map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.Equal(t, "连线已删除", response["message"])
}
