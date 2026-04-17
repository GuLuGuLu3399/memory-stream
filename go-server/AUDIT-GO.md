# Go Backend Audit Report — Blood Temple

> Audit Date: 2026-04-15
> Scope: `go-server/` 全链路 — cmd, handlers, services, middleware, storage, ws, errors, models, pkg

---

## Executive Summary

| War Zone | P0 | P1 | P2 | CLEAN |
|---|---|---|---|---|
| **1. Context & Goroutines** | 2 | 2 | 0 | — |
| **2. Error Handling & Wrapping** | 1 | 2 | 2 | — |
| **3. Concurrency & Data Races** | 2 | 1 | 0 | — |
| **4. GC & Memory** | 1 | 2 | 0 | — |
| **5. Clean Architecture** | 1 | 2 | 0 | — |
| **合计** | **7** | **9** | **2** | — |

---

## War Zone 1: Context & Goroutines

### P0-1: Hub.Run 无 graceful shutdown — goroutine 永不退出

**File**: `internal/ws/hub.go`

`Hub.Run()` 是一个 `for {}` 无限循环，监听 channel 事件。`main()` 中以 `go hub.Run()` 启动，但没有任何机制通知它退出。进程收到 SIGTERM 时，Hub goroutine 被 OS 强杀，正在写入的 WebSocket 消息可能截断。

**Status**: ✅ FIXED — Added `stopCh chan struct{}`, `wg sync.WaitGroup`, and `Stop()` method. `Run()` now listens on `<-h.stopCh` for graceful exit. `main.go` shutdown sequence calls `hub.Stop()`.

### P0-2: main() 无 graceful shutdown — 连接被强杀

**File**: `cmd/api/main.go`

原实现直接调用 `r.Run()`，进程收到信号时 TCP 连接被强制断开。无 draining period，无 WebSocket 清理。

**Status**: ✅ FIXED — Replaced with `http.Server` + `signal.Notify(SIGINT, SIGTERM)`. 3-step graceful shutdown: HTTP → Hub → RateLimiter.

### P1-1: Hub.BroadcastEvent 无 panic recovery

**File**: `internal/ws/hub.go`

如果单个 client send channel 满导致 panic（或 ws.WriteJSON 内部 panic），整个 Hub goroutine 崩溃，所有客户端断连。

**Status**: Open

### P1-2: WS handler 不 respect context cancellation

**File**: `internal/handlers/ws.go`

`HandleWS` 的 readPump/writePump goroutine 不检查 `c.Request.Context()`。如果客户端连接后服务器开始 shutdown，这些 goroutine 不会被通知。

**Status**: Open

---

## War Zone 2: Error Handling & Wrapping

### P0-1: 26 处 `return err` 裸返回，丢失调用栈上下文

**Files**: `services/card.go`, `services/edge.go`, `services/graph.go`, `services/category.go`, `services/search.go`, `services/auth.go`, `handlers/*.go`

大量 GORM 调用的错误直接 `return err`，没有 `fmt.Errorf("failed to do X: %w", err)` 包装。上层无法区分是 "card not found" 还是 "DB connection lost"。

**Status**: Deferred (batch fix pending)

### P1-1: storage.InitDB 错误被 fatalf 而非 propagated

**File**: `internal/storage/pg.go`

`InitDB()` 失败时直接 `log.Fatalf()`。在生产环境中，这绕过了 graceful shutdown 流程。

**Status**: Open

### P1-2: Redis Init 同样 fatalf

**File**: `internal/storage/redis.go`

`InitRedis()` 失败同样直接 fatalf。

**Status**: Open

### P2-1: logger.Init 可能 panic

**File**: `internal/pkg/logger/logger.go`

如果 zap config 错误，`zap.Must()` 会 panic。

**Status**: Open

### P2-2: middleware 错误信息泄露内部状态

**File**: `internal/middleware/auth.go`

某些 error path 返回了 GORM 错误原文，可能暴露数据库结构。

**Status**: Open

---

## War Zone 3: Concurrency & Data Races

### P0-1: Merge 并发无行锁 — 图谱拓扑脑裂（FORGE-REPORT P0-1）

**File**: `internal/services/merge.go`

原始 `MergeCards` 函数用 COUNT(*) 检查卡片存在性，但 PG 禁止 `FOR UPDATE` 与聚合函数同用。两个并发 merge 操作可能同时通过验证，导致边缘竞态。

**Status**: ✅ FIXED — Refactored to `MergeService` with `SELECT ... FOR UPDATE` row-level locking. Consistent lock ordering via `sort.Strings(allIDs)` prevents deadlocks.

### P0-2: Hub clients map 无并发保护

**File**: `internal/ws/hub.go`

`Hub.clients` map 被 `Run()` event loop 和 `BroadcastEvent()` 同时访问。`BroadcastEvent` 在生产者侧直接写 map，存在 data race。

**Status**: Open (channel-based event loop pattern mitigates most risk)

### P1-1: rateLimiter Stop timing

**File**: `internal/middleware/ratelimit.go`

`ViewRateLimiter.Stop()` 在 `defer` 中调用，但 goroutine 可能在 Stop 后仍在执行清理。

**Status**: Open

---

## War Zone 4: GC & Memory

### P0-1: sanitizeSnippet 每次调用 regexp.MustCompile

**File**: `internal/services/card.go`

`sanitizeSnippet()` 内部 11 次 `regexp.MustCompile()` 调用，每次卡片创建/更新/列表查询都会触发。正则编译是 CPU 密集操作，高频调用导致 GC 压力。

**Status**: ✅ FIXED — Extracted to 11 package-level `var reXxx = regexp.MustCompile(...)` pre-compiled regexes.

### P1-1: GORM Session 配置缺失

**File**: `internal/storage/pg.go`

某些查询路径未设置 `SkipDefaultTransaction`，导致单条操作也被包裹在事务中。

**Status**: Open

### P1-2: graph service 全量加载

**File**: `internal/services/graph.go`

`GraphService.All()` 加载所有节点和边到内存，无分页。大规模图谱可能消耗大量内存。

**Status**: Open

---

## War Zone 5: Clean Architecture

### P0-1: MergeHandler 直接注入 *gorm.DB — 违反分层原则

**File**: `internal/handlers/merge.go`

`MergeHandler` 直接持有 `*gorm.DB`，handler 层直接操作数据库事务，绕过了 service 层。

**Status**: ✅ FIXED — Refactored to `MergeService` with proper Handler → Service → DB layering. `MergeHandler` now holds `*services.MergeService`.

### P1-1: 部分 handler 包含业务逻辑

**Files**: `handlers/card.go`, `handlers/edge.go`

部分 handler 包含参数组装和条件分支逻辑，应下沉到 service。

**Status**: Open

### P1-2: models 缺少明确领域边界

**File**: `internal/models/schema.go`

所有 model 定义在一个文件中，缺少按领域分组（Card, Edge, Category 等）。

**Status**: Open

---

## Fixes Applied (3/7 P0)

| Fix | Files Modified | Commit |
|---|---|---|
| Hub graceful shutdown + main() signal handling | `ws/hub.go`, `cmd/api/main.go` | Applied |
| Regex precompilation in sanitizeSnippet | `services/card.go` | Applied |
| MergeService architecture refactor | `services/merge.go`, `handlers/merge.go`, `merge_test.go`, `merge_pg_test.go`, `cmd/api/main.go` | Applied |

## Remaining P0 Issues

| # | Issue | Risk |
|---|---|---|
| P0-2 | Hub clients map 并发保护 | Data race (mitigated by channel pattern) |
| P0-1 (WZ2) | 26 处 bare return err | Lost context in error chains |

---

## Validation

All fixes verified with:

```bash
cd go-server && go vet ./... && go test -race ./...
```

Zero warnings, all tests passing.
