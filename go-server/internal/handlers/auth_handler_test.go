//go:build integration

package handlers

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"strings"
	"testing"

	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"go.uber.org/zap"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

func setupAuthTestDB(t *testing.T) *gorm.DB {
	t.Helper()
	db, err := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{
		DisableForeignKeyConstraintWhenMigrating: true,
	})
	if err != nil {
		t.Fatalf("failed to open sqlite: %v", err)
	}
	if err := db.AutoMigrate(&models.User{}); err != nil {
		t.Fatalf("failed to migrate: %v", err)
	}
	return db
}

func setupAuthHandlerTest(t *testing.T) (*gorm.DB, *gin.Engine) {
	t.Helper()
	gin.SetMode(gin.TestMode)

	if logger.Log == nil {
		l, _ := zap.NewProduction()
		logger.Log = l.Sugar()
	}

	origEnv := os.Getenv("JWT_SECRET")
	err := os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")
	if err != nil {
		t.Fatalf("failed to set JWT_SECRET: %v", err)
	}
	t.Cleanup(func() { os.Setenv("JWT_SECRET", origEnv) })

	db := setupAuthTestDB(t)
	authSvc := services.NewAuthService(db)
	authHandler := NewAuthHandler(authSvc)

	router := gin.New()
	router.Use(appErr.ErrorHandler())
	v1 := router.Group("/api/v1")
	{
		v1.POST("/auth/login", authHandler.Login)
		v1.POST("/auth/refresh", authHandler.Refresh)
		v1.POST("/auth/register", authHandler.Register)
		v1.POST("/auth/genesis", authHandler.Genesis)
	}

	return db, router
}

func TestAuthHandler_Login_EmptyBody(t *testing.T) {
	_, router := setupAuthHandlerTest(t)

	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/login", strings.NewReader(`{}`))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)
	assert.Contains(t, w.Body.String(), "参数解析失败")
}

func TestAuthHandler_Login_Success(t *testing.T) {
	db, router := setupAuthHandlerTest(t)

	// Create user with bcrypt hashed password in SQLite
	password := "password123"
	hash, _ := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	user := &models.User{
		Username:     "admin",
		PasswordHash: string(hash),
		Role:         "admin",
	}
	if err := db.Create(user).Error; err != nil {
		t.Fatalf("failed to create user: %v", err)
	}

	body := `{"username":"admin","password":"password123"}`
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/login", strings.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var response map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.NotEmpty(t, response["access_token"])
	assert.NotEmpty(t, response["refresh_token"])

	userResp, ok := response["user"].(map[string]interface{})
	assert.True(t, ok)
	assert.Equal(t, user.ID, userResp["id"])
	assert.Equal(t, "admin", userResp["username"])
	assert.Equal(t, "admin", userResp["role"])
}

func TestAuthHandler_Login_InvalidPassword(t *testing.T) {
	db, router := setupAuthHandlerTest(t)

	// Create user with bcrypt hashed password
	hash, _ := bcrypt.GenerateFromPassword([]byte("correct-password"), bcrypt.DefaultCost)
	user := &models.User{
		Username:     "admin",
		PasswordHash: string(hash),
		Role:         "admin",
	}
	if err := db.Create(user).Error; err != nil {
		t.Fatalf("failed to create user: %v", err)
	}

	body := `{"username":"admin","password":"wrong-password"}`
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/login", strings.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusUnauthorized, w.Code)
}

func TestAuthHandler_Refresh_EmptyBody(t *testing.T) {
	_, router := setupAuthHandlerTest(t)

	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/refresh", strings.NewReader(`{}`))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)
	assert.Contains(t, w.Body.String(), "参数解析失败")
}

func TestAuthHandler_Register_EmptyBody(t *testing.T) {
	_, router := setupAuthHandlerTest(t)

	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/register", strings.NewReader(`{}`))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)
	assert.Contains(t, w.Body.String(), "参数解析失败")
}

func TestAuthHandler_Genesis_EmptyBody(t *testing.T) {
	_, router := setupAuthHandlerTest(t)

	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/genesis", strings.NewReader(`{}`))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)
	assert.Contains(t, w.Body.String(), "参数解析失败")
}
