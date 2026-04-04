//go:build integration

package handlers

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/DATA-DOG/go-sqlmock"
	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"go.uber.org/zap"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

func setupSearchTest(t *testing.T) (*gorm.DB, sqlmock.Sqlmock, *gin.Engine, *services.SearchService) {
	t.Helper()
	gin.SetMode(gin.TestMode)

	if logger.Log == nil {
		l, _ := zap.NewProduction()
		logger.Log = l.Sugar()
	}

	sqlDB, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create sqlmock: %v", err)
	}

	dialector := postgres.New(postgres.Config{
		Conn: sqlDB,
	})
	db, err := gorm.Open(dialector, &gorm.Config{
		SkipDefaultTransaction: true,
	})
	if err != nil {
		t.Fatalf("failed to open gorm db: %v", err)
	}

	searchSvc := services.NewSearchService(db)
	searchHandler := NewSearchHandler(searchSvc)

	router := gin.New()
	router.Use(appErr.ErrorHandler())

	v1 := router.Group("/api/v1")
	{
		v1.GET("/search", searchHandler.Search)
	}

	return db, mock, router, searchSvc
}

func TestSearchHandler_Search_Success(t *testing.T) {
	_, mock, router, _ := setupSearchTest(t)

	countRows := sqlmock.NewRows([]string{"count"}).AddRow(2)
	mock.ExpectQuery("SELECT count.*").WithArgs("golang").WillReturnRows(countRows)

	searchRows := sqlmock.NewRows([]string{"id", "title", "excerpt", "rank"}).
		AddRow("uuid-1", "Golang Tutorial", "Learn Go programming", 0.9).
		AddRow("uuid-2", "Go Basics", "Introduction to Go", 0.7)
	mock.ExpectQuery("SELECT id.*rank").WithArgs("golang", "golang", 20, 0).WillReturnRows(searchRows)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/search?q=golang&limit=20&offset=0", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var response map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.Equal(t, float64(2), response["total"])
	assert.Len(t, response["results"], 2)
	assert.Equal(t, "golang", response["query"])

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSearchHandler_Search_EmptyQuery(t *testing.T) {
	_, _, router, _ := setupSearchTest(t)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/search", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)
	assert.Contains(t, w.Body.String(), "search query cannot be empty")
}

func TestSearchHandler_Search_InvalidLimit(t *testing.T) {
	_, _, router, _ := setupSearchTest(t)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/search?q=test&limit=150", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)
	assert.Contains(t, w.Body.String(), "limit cannot exceed 100")
}

func TestSearchHandler_Search_NegativeOffset(t *testing.T) {
	_, _, router, _ := setupSearchTest(t)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/search?q=test&offset=-1", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)
	assert.Contains(t, w.Body.String(), "offset cannot be negative")
}

func TestSearchHandler_Search_NoResults(t *testing.T) {
	_, mock, router, _ := setupSearchTest(t)

	countRows := sqlmock.NewRows([]string{"count"}).AddRow(0)
	mock.ExpectQuery("SELECT count.*").WithArgs("nonexistent").WillReturnRows(countRows)

	searchRows := sqlmock.NewRows([]string{"id", "title", "excerpt", "rank"})
	mock.ExpectQuery("SELECT id.*rank").WithArgs("nonexistent", "nonexistent", 20, 0).WillReturnRows(searchRows)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/search?q=nonexistent", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var response map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.Equal(t, float64(0), response["total"])

	results, ok := response["results"].([]interface{})
	if ok {
		assert.Empty(t, results)
	} else {
		assert.Nil(t, response["results"])
	}

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSearchHandler_Search_DatabaseError(t *testing.T) {
	_, mock, router, _ := setupSearchTest(t)

	mock.ExpectQuery("SELECT count.*").WithArgs("test").WillReturnError(gorm.ErrInvalidDB)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/search?q=test", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusInternalServerError, w.Code)
	assert.NoError(t, mock.ExpectationsWereMet())
}
