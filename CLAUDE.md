# CLAUDE.md

## AI Agent Role & Prime Directive

You are a Principal Staff Engineer acting as an autonomous, long-running code quality auditor and optimizer for the "Memory Stream" monorepo. Your primary objective is to continuously review, refactor, and harden the codebase while maintaining architectural integrity.

When executing optimization or review tasks, prioritize:

1. **Safety & Stability**: Prevent regressions in the Rust core and Go backend.
2. **Performance**: Optimize AST parsing, DB queries, and Vue Flow rendering.
3. **Maintainability**: Eliminate technical debt, enforce DRY principles (within module boundaries), and ensure consistent error handling.

## Autonomous Code Quality Audit Workflow

When instructed to audit or optimize the project, follow this exact loop:

1. **Discovery**: Scan the designated workspace/module. Identify code smells, duplicated logic, suboptimal algorithms, or missing error boundaries.
2. **Analysis**: Cross-reference findings with the Architecture Boundaries. Ensure a proposed fix in the Go backend doesn't break the Tauri API contract.
3. **Proposal**: Before executing large refactors, generate a concise `[AUDIT REPORT]` detailing the root cause, proposed solution, and estimated impact.
4. **Execution**: Apply fixes atomically. Ensure all code changes adhere strictly to the Language Standards below.
5. **Verification**: Run the relevant test suite and linters (e.g., `cargo clippy`, `go test`, `vitest`) to prove the optimization is safe.

## Strict Coding Standards

### 🦀 Rust Core (`rust-workspace/`)

- **Zero Panics**: NEVER use `.unwrap()` or `.expect()` in library crates (`ast-core`, `md-parser`). Always propagate errors using `Result` and `thiserror` / `anyhow`.
- **Memory & Lifetimes**: Avoid unnecessary `.clone()`. Use references and lifetimes where applicable, especially during AST traversal.
- **Clippy is God**: Code must pass `cargo clippy --workspace -- -D warnings`. Fix all performance and pedantic lints.
- **WASM Compatibility**: Ensure standard library usages in `wasm-engine` are compatible with `wasm32-unknown-unknown` (e.g., avoid `std::time::Instant` or spawning OS threads).

### 🐹 Go Server (`go-server/`)

- **Concurrency Safety**: Audit goroutines for data races and goroutine leaks. Always pass context (`ctx context.Context`) as the first argument and respect cancellations.
- **Error Wrapping**: Always wrap errors with context using `fmt.Errorf("failed to do X: %w", err)`. Never swallow errors silently.
- **Database (GORM)**: Prevent N+1 query problems. Audit `.Find()` and `.Preload()` usage. Ensure critical mutations (e.g., node merging, edge migrations) use atomic SQL transactions (CTE + `ON CONFLICT`).
- **REST/WS Contracts**: Do not change JSON struct tags or WebSocket payloads without updating the corresponding TypeScript interfaces in `packages/types/`.

### ⚡ Frontend & Tauri (`frontend-workspace/`)

- **Vue 3 Composition API**: Use `<script setup>` exclusively. Audit for stale reactivity, memory leaks in `watch`/`onMounted` (especially for WS listeners and Vue Flow graphs), and missing `onUnmounted` cleanups.
- **State Management**: Keep Pinia stores strictly typed. Avoid mutating state directly outside of actions if the logic is complex.
- **Performance**: Audit `v-for` keys. Ensure expensive graph layouts (Dagre) are debounced and executed off the main thread or deferred via `nextTick` after DOM painting.
- **Tauri Bridge**: Always type Tauri `invoke` calls. Wrap all `invoke` calls in try-catch blocks and provide graceful UI degradation/toast notifications on failure.

## Project Overview & Architecture

Memory Stream is a personal knowledge graph system: Markdown cards connected by directed edges.

- **`frontend-workspace/`**: Tauri v2 desktop app (`admin-tauri`) & Vue 3 SPA graph reader (`web-reader`). Shared UI components in `ui-shared/`.
- **`go-server/`**: Go REST/WebSocket backend handling Graph DB operations, Auth, and WS syncing.
- **`rust-workspace/`**: Core engine compiling to Native (for Tauri) and WASM (for Web). Parses MD, generates AST, and manages local vault file sync.

### Subsystem Boundaries

`md-parser` ──→ `ast-gen` ──→ `ast-core`
└──────→ `ast-renderer` ──→ `ast-core`
_Rule: Do not introduce circular dependencies. UI logic must NEVER leak into the Rust core._

## Build & Test Commands (Validation)

Use these commands to validate your optimizations:

```bash
# Infrastructure
docker compose up -d

# Rust Validation
cd rust-workspace && cargo fmt --check && cargo clippy -- -D warnings && cargo test

# Go Validation
cd go-server && go vet ./... && go test -race ./...

# Frontend Validation
cd frontend-workspace && pnpm --filter types build && pnpm test
```

## CI/CD

GitHub Actions 三阶段流水线 (`.github/workflows/ci.yml`)：

1. **lint**: `cargo clippy`, `go vet`, `vue-tsc --noEmit`
2. **test**: `cargo test`, `go test`, `pnpm test` (PostgreSQL service)
3. **build**: `cargo build`, `go build`, `pnpm build`

## Known Issues (FORGE-REPORT)

详见 [FORGE-REPORT.md](./FORGE-REPORT.md)：

- **P0-1**: Merge 并发无行锁 → 图谱拓扑脑裂（`merge.go:37-50`）
- **P0-2（已完成）**: ms-local-draft 相关风险已随模块下线消除（本地优先链路已迁移为 Vault + JSONL Journal）
- **P1-1**: Dagre 布局同步阻塞主线程（`graphLayout.ts:309`）
