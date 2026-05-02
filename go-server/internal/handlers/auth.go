// Package handlers implements HTTP request handlers for the Memory Stream API.
package handlers

import (
	"errors"
	"net/http"

	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/gin-gonic/gin"
)

// AuthHandler handles authentication and authorization endpoints.
type AuthHandler struct {
	authSvc *services.AuthService
}

// NewAuthHandler creates a new AuthHandler.
func NewAuthHandler(authSvc *services.AuthService) *AuthHandler {
	return &AuthHandler{authSvc: authSvc}
}

type loginRequest struct {
	Username string `json:"username" binding:"required"`
	Password string `json:"password" binding:"required"`
}

type loginResponse struct {
	AccessToken  string `json:"access_token"`
	RefreshToken string `json:"refresh_token"`
	User         struct {
		ID       string `json:"id"`
		Username string `json:"username"`
		Role     string `json:"role"`
	} `json:"user"`
}

// Login authenticates a user and returns access/refresh tokens.
func (h *AuthHandler) Login(c *gin.Context) {
	var req loginRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequest("用户名和密码不能为空"))
		return
	}

	accessToken, refreshToken, user, err := h.authSvc.Login(c.Request.Context(), req.Username, req.Password)
	if err != nil {
		appErr.Respond(c, appErr.NewUnauthorized(err.Error()))
		return
	}

	resp := loginResponse{
		AccessToken:  accessToken,
		RefreshToken: refreshToken,
	}
	resp.User.ID = user.ID.String()
	resp.User.Username = user.Username
	resp.User.Role = user.Role

	appErr.RespondSuccess(c, resp)
}

type refreshRequest struct {
	RefreshToken string `json:"refresh_token" binding:"required"`
}

// Refresh rotates access/refresh tokens.
func (h *AuthHandler) Refresh(c *gin.Context) {
	var req refreshRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequest("refresh_token 不能为空"))
		return
	}

	access, refresh, err := h.authSvc.RefreshTokens(c.Request.Context(), req.RefreshToken)
	if err != nil {
		appErr.Respond(c, appErr.NewUnauthorized(err.Error()))
		return
	}

	appErr.RespondSuccess(c, gin.H{
		"access_token":  access,
		"refresh_token": refresh,
	})
}

type registerRequest struct {
	Username string `json:"username" binding:"required"`
	Password string `json:"password" binding:"required"`
}

// Register creates a new user account (admin only).
func (h *AuthHandler) Register(c *gin.Context) {
	var req registerRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequest("用户名和密码不能为空"))
		return
	}

	user, err := h.authSvc.Register(c.Request.Context(), req.Username, req.Password)
	if err != nil {
		appErr.Respond(c, appErr.NewBadRequest(err.Error()))
		return
	}

	appErr.RespondSuccess(c, gin.H{
		"id":       user.ID.String(),
		"username": user.Username,
		"role":     user.Role,
	})
}

type genesisRequest struct {
	Username string `json:"username" binding:"required"`
	Password string `json:"password" binding:"required"`
}

// Genesis creates the initial admin account (one-time).
func (h *AuthHandler) Genesis(c *gin.Context) {
	var req genesisRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		appErr.Respond(c, appErr.NewBadRequest("用户名和密码不能为空"))
		return
	}

	admin, err := h.authSvc.GenesisAdmin(c.Request.Context(), req.Username, req.Password)
	if err != nil {
		if errors.Is(err, services.ErrGenesisSealed) {
			appErr.Respond(c, appErr.NewBadRequest("创世大门已关闭"))
			return
		}
		appErr.Respond(c, appErr.NewBadRequest(err.Error()))
		return
	}

	c.JSON(http.StatusCreated, gin.H{
		"code":    0,
		"message": "admin created",
		"data": gin.H{
			"id":       admin.ID.String(),
			"username": admin.Username,
			"role":     admin.Role,
		},
	})
}
