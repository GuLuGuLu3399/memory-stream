
// ────────────────────────────────────────────────────────────────
// auth.go — Authentication and authorization middleware
// auth.go — 认证和授权中间件
// ────────────────────────────────────────────────────────────────


package middleware

import (
	"net/http"
	"os"
	"strings"

	appErr "github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/gin-gonic/gin"
)

func AuthMiddleware(authSvc *services.AuthService) gin.HandlerFunc {
	return func(c *gin.Context) {
		token := extractBearerToken(c)
		if token == "" {
			appErr.Respond(c, appErr.NewUnauthorized("缺少认证令牌"))
			c.Abort()
			return
		}

		userID, role, err := authSvc.ParseAccessToken(token)
		if err != nil {
			appErr.Respond(c, appErr.NewUnauthorized("令牌无效或已过期"))
			c.Abort()
			return
		}

		c.Set("user_id", userID)
		c.Set("user_role", role)
		c.Next()
	}
}

func extractBearerToken(c *gin.Context) string {
	auth := c.GetHeader("Authorization")
	if auth == "" {
		return ""
	}
	parts := strings.SplitN(auth, " ", 2)
	if len(parts) != 2 || !strings.EqualFold(parts[0], "bearer") {
		return ""
	}
	return strings.TrimSpace(parts[1])
}

// RequireRole 角色拦截中间件 — 在 AuthMiddleware 之后使用。
// admin 角色永久通行（最高权限），其余角色必须在 allowedRoles 列表中。
func RequireRole(allowedRoles ...string) gin.HandlerFunc {
	return func(c *gin.Context) {
		userRole := c.GetString("user_role")
		if userRole == "" {
			appErr.Respond(c, appErr.NewUnauthorized("缺少认证令牌"))
			c.Abort()
			return
		}

		// admin 永久通行
		if userRole == "admin" {
			c.Next()
			return
		}

		for _, role := range allowedRoles {
			if userRole == role {
				c.Next()
				return
			}
		}

		appErr.Respond(c, appErr.NewForbidden("权限不足：该操作需要更高维度的授权"))
		c.Abort()
	}
}

func CORSConfig() gin.HandlerFunc {
	return func(c *gin.Context) {
		origin := c.GetHeader("Origin")

		defaultOrigins := "http://localhost:5173,http://localhost:1420,http://localhost:4173,https://tauri.localhost,http://tauri.localhost"
		allowedOrigins := strings.Split(defaultOrigins, ",")
		if raw := os.Getenv("CORS_ORIGINS"); raw != "" {
			allowedOrigins = strings.Split(raw, ",")
		}

		isAllowed := false
		for _, o := range allowedOrigins {
			if origin == o {
				isAllowed = true
				break
			}
		}

		if isAllowed {
			c.Header("Access-Control-Allow-Origin", origin)
		}

		c.Header("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE, OPTIONS")
		c.Header("Access-Control-Allow-Headers", "Content-Type, Authorization")
		c.Header("Access-Control-Allow-Credentials", "true")
		c.Header("Access-Control-Max-Age", "86400")

		if c.Request.Method == http.MethodOptions {
			c.AbortWithStatus(http.StatusNoContent)
			return
		}

		c.Next()
	}
}
