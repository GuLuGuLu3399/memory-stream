.PHONY: dev dev-tauri dev-go build build-rust build-go build-fe test test-rust test-go test-fe lint

# ── Dev ──────────────────────────────────────────────────────────
dev:
	@powershell -ExecutionPolicy Bypass -File scripts/dev.ps1

dev-tauri:
	cd frontend-workspace && pnpm --filter admin-tauri tauri dev

dev-go:
	cd go-server && go run ./cmd/api

# ── Build ─────────────────────────────────────────────────────────
build: build-rust build-go build-fe

build-rust:
	cd rust-workspace && cargo build

build-go:
	cd go-server && go build ./...

build-fe:
	cd frontend-workspace && pnpm --filter admin-tauri build

# ── Test ──────────────────────────────────────────────────────────
test: test-rust test-go test-fe

test-rust:
	cd rust-workspace && cargo test

test-go:
	cd go-server && go test -race ./...

test-fe:
	cd frontend-workspace && pnpm test

# ── Lint ──────────────────────────────────────────────────────────
lint:
	cd rust-workspace && cargo clippy -- -D warnings
	cd go-server && go vet ./...
	cd frontend-workspace/apps/admin-tauri && npx vue-tsc --noEmit
