package main

import (
	"context"
	"fmt"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

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
	// 开发环境优先加载 .env.local/.env，且覆盖系统同名变量，确保本地配置生效。
	// 仅当 GO_ENV=production（来自系统环境）时尝试加载 .env.production。
	var envLoaded string
	if os.Getenv("GO_ENV") == "production" {
		if err := godotenv.Load(".env.production"); err == nil {
			envLoaded = ".env.production"
		} else {
			envLoaded = "system environment variables"
		}
	} else if err := godotenv.Overload(".env.local"); err == nil {
		envLoaded = ".env.local"
	} else if err := godotenv.Overload(".env"); err == nil {
		envLoaded = ".env"
	} else {
		envLoaded = "system environment variables"
	}

	if err := logger.Init(); err != nil {
		fmt.Fprintf(os.Stderr, "Failed to initialize logger: %v\n", err)
		os.Exit(1)
	}
	defer logger.Sync()

	logger.Log.Infof("[config] Loaded from: %s", envLoaded)

	// 设置 Gin 运行模式：通过 GIN_MODE 或 DEBUG 环境变量控制
	// GIN_MODE=debug    -> 调试模式（详细日志）
	// GIN_MODE=release  -> 生产模式（精简日志）
	// 默认：从 DEBUG env 推断，否则默认 release
	ginMode := os.Getenv("GIN_MODE")
	debug := os.Getenv("DEBUG")
	goEnv := os.Getenv("GO_ENV")
	
	if ginMode != "" {
		gin.SetMode(ginMode)
		logger.Log.Infof("[gin] Mode set from GIN_MODE env: %s", ginMode)
	} else if debug == "true" || debug == "1" {
		gin.SetMode(gin.DebugMode)
		logger.Log.Infof("[gin] Mode set from DEBUG env: debug")
	} else if goEnv == "production" {
		gin.SetMode(gin.ReleaseMode)
		logger.Log.Infof("[gin] Mode set from GO_ENV=production: release")
	} else {
		gin.SetMode(gin.ReleaseMode)
		logger.Log.Infof("[gin] Using default mode: release")
	}

	db, err := storage.InitDB()
	if err != nil {
		logger.Log.Fatalf("Failed to initialize database: %v", err)
	}
	logger.RegisterSlowQueryPlugin(db) // 慢查询监控：>200ms 的 SQL 输出到 zap

	rdb, err := storage.InitRedis()
	if err != nil {
		logger.Log.Fatalf("Failed to initialize Redis: %v", err)
	}

	authSvc := services.NewAuthService(db)
	cardSvc := services.NewCardService(db, rdb)
	edgeSvc := services.NewEdgeService(db, cardSvc.InvalidateGraphCache)
	graphSvc := services.NewGraphService(db)
	categorySvc := services.NewCategoryService(db)
	searchSvc := services.NewSearchService(db)

	// ── 自动初始化流程：如果数据库为空，自动创建 admin + guest 账号 ──
	if !authSvc.IsGenesisSealed(context.Background()) {
		logger.Log.Warn("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
		logger.Log.Warn("⚠️  初次启动检测到数据库为空，自动执行初始化流程...")
		logger.Log.Warn("   正在创建 admin + guest 账号...")
		logger.Log.Warn("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
		
		admin, err := authSvc.GenesisAdmin(context.Background(), "admin", "admin123")
		if err != nil {
			logger.Log.Warnf("⚠️  初始化失败，但这可能不影响运行: %v", err)
		} else {
			logger.Log.Infof("✅ 初始化完成！")
			logger.Log.Infof("   Admin:  username=%s, password=admin123", admin.Username)
			logger.Log.Infof("   Guest:  username=guest, password=guest123")
			logger.Log.Infof("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
		}
	}

	hub := ws.NewHub()
	go hub.Run()
	handlers.SetupWSHandlers(hub, edgeSvc, authSvc)

	rateLimiter := middleware.NewViewRateLimiter()
	defer rateLimiter.Stop()

	authHandler := handlers.NewAuthHandler(authSvc)
	cardHandler := handlers.NewCardHandler(cardSvc, edgeSvc, graphSvc, rateLimiter, hub)
	edgeHandler := handlers.NewEdgeHandler(edgeSvc, hub)
	mergeSvc := services.NewMergeService(db)
	mergeHandler := handlers.NewMergeHandler(mergeSvc, hub)
	graphHandler := handlers.NewGraphHandler(graphSvc, cardSvc)
	categoryHandler := handlers.NewCategoryHandler(categorySvc)
	searchHandler := handlers.NewSearchHandler(searchSvc)

	r := gin.New()
	r.Use(gin.Logger(), middleware.CORSConfig(), middleware.MaxBodySize(10<<20), errors.ErrorHandler())

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
			handlers.HandleWS(c, hub, authSvc)
		})
	}

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	srv := &http.Server{
		Addr:    ":" + port,
		Handler: r,
	}

	// Graceful shutdown: listen for OS signals in a separate goroutine
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		logger.Log.Infof("Server starting on :%s", port)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			logger.Log.Fatalf("Server failed: %v", err)
		}
	}()

	sig := <-quit
	logger.Log.Infof("[shutdown] received signal %v, shutting down...", sig)

	// Step 1: Stop accepting new HTTP requests (5s timeout)
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	if err := srv.Shutdown(ctx); err != nil {
		logger.Log.Errorf("[shutdown] HTTP server forced shutdown: %v", err)
	}

	// Step 2: Close WebSocket hub (disconnect all clients gracefully)
	hub.Stop()
	logger.Log.Info("[shutdown] WebSocket hub stopped")

	// Step 3: Stop rate limiter cleanup goroutine
	rateLimiter.Stop()

	logger.Log.Info("[shutdown] server exited cleanly")
}
