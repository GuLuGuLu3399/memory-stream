# ============================================================================
# Memory Stream — Unified Build System
# ============================================================================
#
# Monorepo: frontend-workspace (pnpm/Vue) | rust-workspace (Cargo) | go-server
#
# Primary targets:
#   make              Show help
#   make dev          Start all dev services
#   make build        Production build all
#   make lint         Lint & type check (no compile)
#   make test         Run all tests
#   make ci           Full CI pipeline (lint → test → build)
#   make fmt          Format all code
# ============================================================================

.DEFAULT_GOAL := help
SHELL := bash
DEV_PID_FILE := .dev-pids

.PHONY: help \
        dev stop \
        build build-rust build-frontend build-go build-wasm build-tauri \
        dist dist-linux dist-web dist-apk \
        lint lint-rust lint-go lint-frontend \
        test test-rust test-go test-frontend \
        fmt fmt-rust fmt-go \
        ci \
        install clean \
        docker-up docker-down

DIST_DIR := dist

# ============================================================================
# Help
# ============================================================================

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

# ============================================================================
# Install & Dev
# ============================================================================

install: ## Install frontend deps
	cd frontend-workspace && pnpm install

dev: ## Start all dev services (Go + Web + Tauri)
	@powershell -ExecutionPolicy Bypass -File scripts\dev.ps1

stop: ## Kill stray dev processes
	@powershell -ExecutionPolicy Bypass -File scripts\stop.ps1

# ============================================================================
# Lint (no compilation — matches CI Stage 1)
# ============================================================================

lint: lint-rust lint-go lint-frontend ## Lint & type check all

lint-rust: ## cargo fmt --check + clippy
	cd rust-workspace && cargo fmt --check
	cd rust-workspace && cargo clippy -- -D warnings

lint-go: ## go vet
	cd go-server && go vet ./...

lint-frontend: ## vue-tsc --noEmit
	cd frontend-workspace && npx vue-tsc --noEmit --project apps/admin-tauri/tsconfig.json

# ============================================================================
# Test (matches CI Stage 2)
# ============================================================================

test: test-rust test-go test-frontend ## Run all tests

test-rust: ## cargo test
	cd rust-workspace && cargo test

test-go: ## go test
	cd go-server && go test -count=1 ./...

test-frontend: ## vitest
	cd frontend-workspace && pnpm test

# ============================================================================
# Format
# ============================================================================

fmt: fmt-rust fmt-go ## Format all code

fmt-rust: ## cargo fmt
	cd rust-workspace && cargo fmt

fmt-go: ## gofmt
	cd go-server && gofmt -w .

# ============================================================================
# Build (matches CI Stage 3)
# ============================================================================

build: build-rust build-frontend build-go ## Production build all

build-rust: ## cargo build (release)
	cd rust-workspace && cargo build --release

build-frontend: ## Build web-reader SPA
	cd frontend-workspace/apps/web-reader && pnpm build

build-go: ## go build server binary
	cd go-server && go build -o bin/server ./cmd/api

build-wasm: ## Build WASM engine for web-reader
	cd rust-workspace/wasm-engine && wasm-pack build --target web --release -- --cfg web_sys_unstable_apis
	@which wasm-opt > /dev/null 2>&1 && wasm-opt -Oz -o rust-workspace/wasm-engine/pkg/wasm_engine_bg.wasm rust-workspace/wasm-engine/pkg/wasm_engine_bg.wasm || echo "[!] wasm-opt not found, skipping"
	@echo "[+] WASM engine -> rust-workspace/wasm-engine/pkg/"

build-tauri: ## Build Tauri desktop app
	cd frontend-workspace/apps/admin-tauri && pnpm tauri build

# ============================================================================
# Distribution (Linux server + Web SPA + Tauri desktop)
# ============================================================================

dist: test ## Build distributable Linux server + Web SPA + Tauri desktop (production config)
	@powershell -ExecutionPolicy Bypass -File scripts\dist.ps1

dist-linux: ## Cross-compile Go server for Linux amd64 into dist/go (via PowerShell)
	@powershell -ExecutionPolicy Bypass -Command "if(!(Test-Path dist/go)){New-Item -ItemType Directory -Path dist/go|Out-Null}; $$env:GOOS='linux'; $$env:GOARCH='amd64'; $$env:CGO_ENABLED='0'; cd go-server; go build -ldflags='-s -w' -o ../dist/go/server ./cmd/api; Write-Host '[+] Linux binary -> dist/go/server'"

dist-web: ## Build web-reader SPA with .env.production (via PowerShell)
	@powershell -ExecutionPolicy Bypass -Command "cd frontend-workspace/apps/web-reader; npx vite build --mode production; cd ../../../; if(!(Test-Path dist/web)){New-Item -ItemType Directory -Path dist/web|Out-Null}; Copy-Item frontend-workspace/apps/web-reader/dist/* dist/web/ -Recurse -Force; Write-Host '[+] Web SPA -> dist/web/'"

dist-apk: ## Build Android APK via Capacitor (web-reader -> dist/apk/)
	cd frontend-workspace/apps/web-reader && npx vite build --mode android && npx cap sync android
	@echo "[+] Android assets synced. Open with: npx cap open android"
	@echo "    Or build APK directly:"
	@echo "    cd frontend-workspace/apps/web-reader/android && ./gradlew assembleDebug"
	@mkdir -p dist/apk
	@echo "[+] APK output -> frontend-workspace/apps/web-reader/android/app/build/outputs/apk/"

# ============================================================================
# CI (full pipeline: lint → test → build)
# ============================================================================

ci: lint test build ## Full CI pipeline (lint → test → build)
	@echo "[+] CI pipeline passed"

# ============================================================================
# Docker (local infrastructure)
# ============================================================================

docker-up: ## Start PostgreSQL + Redis + MinIO
	docker compose up -d

docker-down: ## Stop and remove containers
	docker compose down

# ============================================================================
# Clean
# ============================================================================

clean: ## Remove all build artifacts
	cd rust-workspace && cargo clean 2>/dev/null || true
	rm -rf go-server/bin $(DIST_DIR)
	rm -f $(DEV_PID_FILE)
	@echo "[+] Cleaned all build artifacts"
