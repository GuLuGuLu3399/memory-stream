package handlers

import (
	"errors"
	"fmt"
	"net/http"

	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/gin-gonic/gin"
)

// AuthHandler handles authentication-related HTTP requests.
type AuthHandler struct {
	authSvc *services.AuthService
}

// NewAuthHandler creates a new AuthHandler instance.
func NewAuthHandler(authSvc *services.AuthService) *AuthHandler {
	return &AuthHandler{authSvc: authSvc}
}

type LoginReq struct {
	Username string `json:"username" binding:"required"`
	Password string `json:"password" binding:"required"`
}

// Login authenticates a user and returns access/refresh tokens.
// POST /auth/login
func (h *AuthHandler) Login(c *gin.Context) {
	var req LoginReq
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequestWithLog("参数解析失败", err.Error()))
		return
	}

	accessToken, refreshToken, user, err := h.authSvc.Login(req.Username, req.Password)
	if err != nil {
		appErr.Respond(c, appErr.NewUnauthorized(err.Error()))
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"access_token":  accessToken,
		"refresh_token": refreshToken,
		"user": gin.H{
			"id":       user.ID,
			"username": user.Username,
			"role":     user.Role,
		},
	})
}

type RefreshReq struct {
	RefreshToken string `json:"refresh_token" binding:"required"`
}

// Refresh rotates tokens using a valid refresh token.
// POST /auth/refresh
func (h *AuthHandler) Refresh(c *gin.Context) {
	var req RefreshReq
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequestWithLog("参数解析失败", err.Error()))
		return
	}

	accessToken, refreshToken, err := h.authSvc.RefreshTokens(req.RefreshToken)
	if err != nil {
		appErr.Respond(c, appErr.NewUnauthorized(err.Error()))
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"access_token":  accessToken,
		"refresh_token": refreshToken,
	})
}

type RegisterReq struct {
	Username string `json:"username" binding:"required"`
	Password string `json:"password" binding:"required"`
}

// Register creates a new user account with guest role.
// POST /auth/register
func (h *AuthHandler) Register(c *gin.Context) {
	var req RegisterReq
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequestWithLog("参数解析失败", err.Error()))
		return
	}

	user, err := h.authSvc.Register(req.Username, req.Password)
	if err != nil {
		appErr.Respond(c, appErr.NewBadRequest(err.Error()))
		return
	}

	c.JSON(http.StatusCreated, gin.H{
		"user": gin.H{
			"id":       user.ID,
			"username": user.Username,
			"role":     user.Role,
		},
	})
}

// ── 创世接口：一次性点火，创建 admin + guest 账号 ──

type GenesisReq struct {
	Username string `json:"username" binding:"required"`
	Password string `json:"password" binding:"required"`
}

// Genesis is the one-time bootstrap endpoint that creates the initial admin account.
// POST /auth/genesis
func (h *AuthHandler) Genesis(c *gin.Context) {
	var req GenesisReq
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequestWithLog("参数解析失败", err.Error()))
		return
	}

	admin, err := h.authSvc.GenesisAdmin(req.Username, req.Password)
	if err != nil {
		if errors.Is(err, services.ErrGenesisSealed) {
			appErr.Respond(c, appErr.NewForbidden("创世接口已关闭：admin 账号已存在"))
			return
		}
		appErr.Respond(c, appErr.NewBadRequest(err.Error()))
		return
	}

	// 创世成功后自动签发 admin 的 Token（免得还要再登录一次）
	accessToken, refreshToken, _, loginErr := h.authSvc.Login(req.Username, req.Password)
	if loginErr != nil {
		appErr.Respond(c, appErr.NewInternal(fmt.Errorf("genesis auto-login failed: %w", loginErr)))
		return
	}

	c.JSON(http.StatusCreated, gin.H{
		"message":       "创世完成：admin + guest 账号已就绪",
		"access_token":  accessToken,
		"refresh_token": refreshToken,
		"user": gin.H{
			"id":       admin.ID,
			"username": admin.Username,
			"role":     admin.Role,
		},
	})
}
