// ────────────────────────────────────────────────────────────────
// maxbody.go — Middleware for limiting request body size
// maxbody.go — 限制请求体大小的中间件
// ────────────────────────────────────────────────────────────────

package middleware

import (
	"net/http"

	"github.com/gin-gonic/gin"
)

func MaxBodySize(maxBytes int64) gin.HandlerFunc {
	return func(c *gin.Context) {
		if c.Request.Body != nil {
			c.Request.Body = http.MaxBytesReader(c.Writer, c.Request.Body, maxBytes)
		}
		c.Next()
	}
}
