package main

import (
	"os"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/errors"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/handlers"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/middleware"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/storage"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/ws"
	"github.com/gin-gonic/gin"
	"github.com/joho/godotenv"
)

func main() {
	_ = godotenv.Load()
	logger.Init()
	defer logger.Sync()

	db := storage.InitDB()
	logger.RegisterSlowQueryPlugin(db) // 慢查询监控：>200ms 的 SQL 输出到 zap

	rdb := storage.InitRedis()

	authSvc := services.NewAuthService(db)
	cardSvc := services.NewCardService(db, rdb)
	edgeSvc := services.NewEdgeService(db, cardSvc.InvalidateGraphCache)
	graphSvc := services.NewGraphService(db)
	categorySvc := services.NewCategoryService(db)
	searchSvc := services.NewSearchService(db)

	hub := ws.NewHub()
	go hub.Run()
	handlers.SetupWSHandlers(hub, edgeSvc, authSvc)

	rateLimiter := middleware.NewViewRateLimiter()
	defer rateLimiter.Stop()

	authHandler := handlers.NewAuthHandler(authSvc)
	cardHandler := handlers.NewCardHandler(cardSvc, edgeSvc, graphSvc, rateLimiter, hub)
	edgeHandler := handlers.NewEdgeHandler(edgeSvc, hub)
	mergeHandler := handlers.NewMergeHandler(db, hub)
	graphHandler := handlers.NewGraphHandler(graphSvc, cardSvc)
	categoryHandler := handlers.NewCategoryHandler(categorySvc)
	searchHandler := handlers.NewSearchHandler(searchSvc)

	r := gin.New()
	r.Use(middleware.CORSConfig(), errors.ErrorHandler())

	r.GET("/health", func(c *gin.Context) {
		c.JSON(200, gin.H{"status": "ok"})
	})

	v1 := r.Group("/api/v1")
	{
		v1.POST("/auth/login", authHandler.Login)
		v1.POST("/auth/refresh", authHandler.Refresh)
		v1.POST("/auth/init", authHandler.Genesis)
		v1.GET("/search", searchHandler.Search)

		authed := v1.Group("").Use(middleware.AuthMiddleware(authSvc))
		{
			// 只读：卡片浏览
			authed.GET("/cards", cardHandler.List)
			authed.GET("/cards/discover", cardHandler.Discover)
			authed.GET("/cards/:id", cardHandler.GetByID)
			authed.GET("/cards/:id/backlinks", cardHandler.GetBacklinks)
			authed.GET("/cards/:id/graph", cardHandler.Graph)
			authed.POST("/cards/:id/view", cardHandler.IncrementView)

			// 只读：分类 & 图谱
			authed.GET("/categories", categoryHandler.List)
			authed.GET("/categories/tree", categoryHandler.GetTree)
			authed.GET("/categories/:id/clusters", categoryHandler.GetClusters)
			authed.GET("/graph/all", graphHandler.All)
			authed.GET("/graph/outline", graphHandler.Outline)
			authed.GET("/graph/detail/:id", graphHandler.Detail)
		}

		// ── Admin 专属路由（第二道门：RequireRole("admin")） ──
		adminOnly := v1.Group("").Use(
			middleware.AuthMiddleware(authSvc),
			middleware.RequireRole("admin"),
		)
		{
			// 卡片写操作
			adminOnly.POST("/cards", cardHandler.Create)
			adminOnly.PUT("/cards/:id", cardHandler.Update)
			adminOnly.DELETE("/cards/:id", cardHandler.Delete)

			// 分类写操作
			adminOnly.POST("/categories", categoryHandler.Create)
			adminOnly.PUT("/categories/:id", categoryHandler.Update)
			adminOnly.DELETE("/categories/:id", categoryHandler.Delete)

			// 边写操作
			adminOnly.POST("/edges", edgeHandler.Create)
			adminOnly.DELETE("/edges", edgeHandler.Delete)
			adminOnly.PATCH("/edges", edgeHandler.Update)

			// 卡片合并
			adminOnly.POST("/cards/merge", mergeHandler.MergeCards)

			// 用户注册（仅 admin 可创建新用户）
			adminOnly.POST("/auth/register", authHandler.Register)

		}

		v1.GET("/ws", func(c *gin.Context) {
			handlers.HandleWS(c, hub)
		})
	}

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}
	logger.Log.Infof("Server starting on :%s", port)
	err := r.Run(":" + port)
	if err != nil {
		logger.Log.Fatal(err)
		return
	}
}
