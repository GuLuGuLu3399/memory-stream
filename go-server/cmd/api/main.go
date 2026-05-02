package main

import (
	"context"
	stderrors "errors"
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
	"github.com/gin-gonic/gin"
	"github.com/joho/godotenv"
)

func main() {
	// Load environment
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
		if _, writeErr := fmt.Fprintf(os.Stderr, "Failed to initialize logger: %v\n", err); writeErr != nil {
			os.Exit(1)
		}
		os.Exit(1)
	}
	defer logger.Sync()

	logger.Log.Infof("[config] Loaded from: %s", envLoaded)

	// Gin mode
	ginMode := os.Getenv("GIN_MODE")
	debug := os.Getenv("DEBUG")
	goEnv := os.Getenv("GO_ENV")

	if ginMode != "" {
		gin.SetMode(ginMode)
	} else if debug == "true" || debug == "1" {
		gin.SetMode(gin.DebugMode)
	} else if goEnv == "production" {
		gin.SetMode(gin.ReleaseMode)
	} else {
		gin.SetMode(gin.ReleaseMode)
	}

	// Database
	db, err := storage.InitDB()
	if err != nil {
		logger.Log.Fatalf("Failed to initialize database: %v", err)
	}
	logger.RegisterSlowQueryPlugin(db)

	// Services
	authSvc := services.NewAuthService(db)
	cardSvc := services.NewCardService(db)
	graphSvc := services.NewGraphService(db)
	syncSvc := services.NewSyncService(db)
	searchSvc := services.NewSearchService(db)
	glossarySvc := services.NewGlossaryService(db)

	// Auto-init admin + guest on first run
	if !authSvc.IsGenesisSealed(context.Background()) {
		logger.Log.Warn("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
		logger.Log.Warn("First run detected, initializing admin + guest accounts...")
		logger.Log.Warn("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")

		adminPassword := os.Getenv("GENESIS_ADMIN_PASSWORD")
		if adminPassword == "" {
			adminPassword = "admin123"
		}
		guestPassword := os.Getenv("GUEST_PASSWORD")
		if guestPassword == "" {
			guestPassword = "guest123"
		}

		admin, err := authSvc.GenesisAdmin(context.Background(), "admin", adminPassword)
		if err != nil {
			logger.Log.Warnf("Init failed (may not affect runtime): %v", err)
		} else {
			logger.Log.Infof("Admin: username=%s, password=%s", admin.Username, adminPassword)
			logger.Log.Infof("Guest: username=guest, password=%s", guestPassword)
			logger.Log.Info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
		}
	}

	// Handlers
	healthHandler := handlers.NewHealthHandler(db)
	authHandler := handlers.NewAuthHandler(authSvc)
	cardHandler := handlers.NewCardHandler(cardSvc)
	graphHandler := handlers.NewGraphHandler(graphSvc)
	syncHandler := handlers.NewSyncHandler(syncSvc)
	searchHandler := handlers.NewSearchHandler(searchSvc)
	glossaryHandler := handlers.NewGlossaryHandler(glossarySvc)

	// Router
	r := gin.New()
	r.Use(gin.Logger(), middleware.CORSConfig(), middleware.MaxBodySize(10<<20), errors.ErrorHandler())

	v1 := r.Group("/api/v1")
	{
		// Public (unauthenticated)
		v1.GET("/health", healthHandler.HealthCheck)
		v1.POST("/auth/login", authHandler.Login)
		v1.POST("/auth/refresh", authHandler.Refresh)
		v1.POST("/auth/genesis", authHandler.Genesis)

		// Authenticated (read-only for web reader)
		authed := v1.Group("").Use(middleware.AuthMiddleware(authSvc))
		{
			authed.GET("/cards", cardHandler.List)
			authed.GET("/cards/random", cardHandler.GetRandom)
			authed.GET("/cards/resolve", cardHandler.ResolveByTitle)
			authed.GET("/cards/:uuid", cardHandler.GetByID)
			authed.GET("/cards/:uuid/backlinks", cardHandler.GetBacklinks)
			authed.GET("/graph/all", graphHandler.All)
			authed.GET("/graph/neighborhood/:uuid", graphHandler.Neighborhood)
			authed.GET("/search", searchHandler.Search)
			authed.GET("/categories", cardHandler.ListCategories)
			authed.GET("/glossary", glossaryHandler.List)
			authed.GET("/glossary/slim", glossaryHandler.Slim)
		}

		// Admin only (sync + user management)
		adminOnly := v1.Group("").Use(
			middleware.AuthMiddleware(authSvc),
			middleware.RequireRole("admin"),
		)
		{
			adminOnly.DELETE("/sync/card/:uuid", syncHandler.DeleteCard)
			adminOnly.GET("/sync/manifest", syncHandler.GetManifest)
			adminOnly.GET("/sync/card/:uuid", syncHandler.GetCard)
			adminOnly.POST("/sync/batch", syncHandler.BatchUpsert)
				adminOnly.POST("/sync/relations", syncHandler.SyncRelations)
				adminOnly.GET("/sync/relations", syncHandler.GetAllTrunks)
			adminOnly.GET("/sync/glossary/manifest", glossaryHandler.SyncManifest)
			adminOnly.POST("/sync/glossary/batch", glossaryHandler.SyncBatchUpsert)
			adminOnly.POST("/auth/register", authHandler.Register)
		}
	}

	// Start server with graceful shutdown
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	srv := &http.Server{
		Addr:    ":" + port,
		Handler: r,
	}

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		logger.Log.Infof("Server starting on :%s", port)
		if err := srv.ListenAndServe(); err != nil && !stderrors.Is(err, http.ErrServerClosed) {
			logger.Log.Fatalf("Server failed: %v", err)
		}
	}()

	sig := <-quit
	logger.Log.Infof("[shutdown] received signal %v, shutting down...", sig)

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	if err := srv.Shutdown(ctx); err != nil {
		logger.Log.Errorf("[shutdown] forced shutdown: %v", err)
	}

	logger.Log.Info("[shutdown] server exited cleanly")
}
