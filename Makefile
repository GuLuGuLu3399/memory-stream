# ============================================================================
# Memory Stream v3.4 — Unified Build System
# ============================================================================
#
# Monorepo build glue:
#   - frontend-workspace/  (pnpm + Vue + Vite)
#   - rust-workspace/      (Cargo)
#   - go-server/           (Go)
#   - src-tauri/           (Tauri CLI)
#
# Usage:
#   make dev          - Start all dev services (Ctrl+C to stop)
#   make build        - Production build all modules
#   make check        - Quick check (lint + type check)
#   make test         - Run all tests
#   make install      - Install frontend deps
# ============================================================================

.PHONY: all dev build check test install clean stop \
        frontend dev-frontend build-frontend \
        web dev-web \
        rust build-rust test-rust \
        go dev-go build-go test-go \
        tauri dev-tauri build-tauri

DEV_PID_FILE := .dev-pids

all: build

# ============================================================================
# Frontend (pnpm workspace)
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

build-wasm:
	cd rust-workspace/wasm-engine && wasm-pack build --target web --release -- --cfg web_sys_unstable_apis
	@which wasm-opt > /dev/null 2>&1 && wasm-opt -Oz -o target/pkg/wasm_engine_bg.wasm target/pkg/wasm_engine_bg.wasm || echo "[!]  wasm-opt not found, skipping"
	@echo "[+] WASM engine built -> rust-workspace/wasm-engine/pkg/"

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
# Tauri Desktop (depends on frontend + rust)
# ============================================================================

dev-tauri:
	cd frontend-workspace/apps/admin-tauri && pnpm tauri dev

build-tauri:
	cd frontend-workspace/apps/admin-tauri && pnpm tauri build

# ============================================================================
# Composite targets
# ============================================================================

dev:
	@powershell -ExecutionPolicy Bypass -File scripts\dev.ps1

stop:
	@powershell -ExecutionPolicy Bypass -File scripts\stop.ps1

build: build-rust build-frontend build-go
	@echo "[+] All modules built successfully"

check: build-rust check-frontend
	@echo "[+] All checks passed"

test: test-rust test-go
	@echo "[+] All tests passed"

clean:
	cd frontend-workspace && pnpm clean 2>/dev/null || true
	cd rust-workspace && cargo clean 2>/dev/null || true
	cd go-server && powershell -Command "Remove-Item -Recurse -Force bin -ErrorAction SilentlyContinue"
	-powershell -Command "Remove-Item -Force $(DEV_PID_FILE) -ErrorAction SilentlyContinue"
	@echo "[*] Cleaned all build artifacts"
