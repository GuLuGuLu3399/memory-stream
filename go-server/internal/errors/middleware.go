// Package errors provides error types, response formatting, and recovery middleware for the API.
package errors

import (
	"errors"
	"net/http"
	"runtime/debug"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"
	"github.com/gin-gonic/gin"
)

func ErrorHandler() gin.HandlerFunc {
	return func(c *gin.Context) {
		defer func() {
			if r := recover(); r != nil {
				logger.Log.Errorf("panic recovered: %v\n%s", r, debug.Stack())
				c.AbortWithStatusJSON(http.StatusInternalServerError, gin.H{
					"code":    50001,
					"message": "服务器开小差了",
				})
			}
		}()
		c.Next()
	}
}

func Respond(c *gin.Context, err error) {
	var e *AppError
	switch {
	case errors.As(err, &e):
		if e.LogDetails != "" {
			logger.Log.Errorf("[AppError] http=%d biz=%d msg=%s detail=%s", e.HTTPCode, e.BizCode, e.Message, e.LogDetails)
		}
		c.JSON(e.HTTPCode, e)
	default:
		logger.Log.Errorf("[InternalError] %v", err)
		c.JSON(http.StatusInternalServerError, gin.H{
			"code":    50001,
			"message": "服务器开小差了",
		})
	}
}

// RespondSuccess wraps data in the standard API envelope.
func RespondSuccess(c *gin.Context, data interface{}) {
	c.JSON(http.StatusOK, gin.H{
		"code":    0,
		"message": "success",
		"data":    data,
	})
}
