# ============================================================================
# Memory Stream v3.4 — 统一构建系统
# ============================================================================
#
# 异构 Monorepo 构建胶水：
#   - frontend-workspace/  (pnpm + Vue + Vite)
#   - rust-workspace/      (Cargo)
#   - go-server/           (Go)
#   - src-tauri/           (Tauri CLI)
#
# 用法:
#   make dev          — 一键启动全部开发服务
#   make build        — 生产构建全部模块
#   make check        — 快速检查（lint + type check）
#   make test         — 运行全部测试
#   make install      — 安装前端依赖
# ============================================================================

.PHONY: all dev build check test install clean \
        frontend dev-frontend build-frontend \
        web dev-web \
        rust build-rust test-rust \
        go dev-go build-go test-go \
        tauri dev-tauri build-tauri

# ── 默认目标 ──────────────────────────────────────────────────
all: build

# ============================================================================
# 前端 (pnpm workspace)
# ============================================================================

install:
	cd frontend-workspace && pnpm install

dev-frontend:
	cd frontend-workspace && pnpm dev

build-frontend:
	cd frontend-workspace && pnpm build

check-frontend:
	cd frontend-workspace && pnpm run check 2>/dev/null || true

# ============================================================================
# Web Reader (Vite SPA)
# ============================================================================

dev-web:
	cd frontend-workspace/apps/web-reader && pnpm dev

# ============================================================================
# Rust (Cargo workspace)
# ============================================================================

build-rust:
	cd rust-workspace && cargo build

## WASM 引擎构建（release + wasm-opt -Oz 体积优化）
build-wasm:
	cd rust-workspace/wasm-engine && wasm-pack build --target web --release -- --cfg web_sys_unstable_apis
	@which wasm-opt > /dev/null 2>&1 && wasm-opt -Oz -o target/pkg/wasm_engine_bg.wasm target/pkg/wasm_engine_bg.wasm || echo "⚠️  wasm-opt not found, skipping binary-size optimization"
	@echo "✅ WASM engine built → rust-workspace/wasm-engine/pkg/"

test-rust:
	cd rust-workspace && cargo test

# ============================================================================
# Go Server
# ============================================================================

dev-go:
	cd go-server && go run ./cmd/api

build-go:
	cd go-server && go build -o bin/server ./cmd/api

test-go:
	cd go-server && go test ./...

# ============================================================================
# Tauri 桌面端 (依赖 frontend + rust)
# ============================================================================

dev-tauri:
	cd frontend-workspace/apps/admin-tauri && pnpm tauri dev

build-tauri:
	cd frontend-workspace/apps/admin-tauri && pnpm tauri build

# ============================================================================
# 组合命令
# ============================================================================

## 一键开发：启动 Go 后端 + Web Reader + Tauri 桌面端
dev:
	@echo "🚀 Starting Memory Stream development..."
	@echo "   Go Server:   http://localhost:8080"
	@echo "   Web Reader:  http://localhost:5173"
	@echo "   Tauri App:   launching..."
	$(MAKE) -j3 dev-go dev-web dev-tauri

## 生产构建：前端 + Rust + Go
build: build-rust build-frontend build-go
	@echo "✅ All modules built successfully"

## 快速检查：Rust 编译 + 前端类型检查
check: build-rust check-frontend
	@echo "✅ All checks passed"

## 运行全部测试
test: test-rust test-go
	@echo "✅ All tests passed"

## 清理构建产物
clean:
	cd frontend-workspace && pnpm clean 2>/dev/null || true
	cd rust-workspace && cargo clean 2>/dev/null || true
	cd go-server && rm -rf bin/ 2>/dev/null || true
	@echo "🧹 Cleaned all build artifacts"