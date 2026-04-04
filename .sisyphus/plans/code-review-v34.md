# Memory Stream V3.4 — 全栈深度代码审查计划

## TL;DR

> **Quick Summary**: 对 Memory Stream 四端（Go Server / Vue Web Reader / Vue Admin Tauri / Rust 10 crates）进行系统性代码审查，覆盖架构模式、性能优化、并发安全、错误处理、安全、类型安全、可扩展性 7 大维度。
> 
> **Deliverables**:
> - 每个模块的《代码审查报告》（总体评价 + 关键发现 + 逐段重构建议 + 进阶建议）
> - 具体代码问题的修复建议（含重构后代码示例）
> - 跨端一致性与架构级改进建议
> 
> **Estimated Effort**: Large
> **Parallel Execution**: YES — 5 waves
> **Critical Path**: Wave 1 (Go) → Wave 2/3/4/5 (并行) → Final (综合报告)

---

## Context

### Original Request
用户希望以资深架构师视角，对 Memory Stream V3.4 全栈代码进行深度 Review。按照两阶段流程：先制定审查计划（阶段一），确认后逐模块生成审查报告（阶段二）。

### Interview Summary
**确认的审查维度**: 架构模式、性能优化、并发安全、错误处理、安全、代码规范/类型安全、可扩展性
**审查范围**: 全栈（Go + Vue + Rust），按模块分批
**审查标准**: 已确认 7 维度"优秀"定义
**执行方式**: 按 5 批次并行审查，每批次产出独立报告

### Research Findings (Explore Agents)

**Go Server**:
- handler→service→storage 三层清晰，handler 负责 HTTP 语义，service 负责业务逻辑
- WebSocket Hub 使用 sync.RWMutex + channel 模式，单 goroutine 处理事件循环
- GORM Preload 避免 N+1，但存在 fallback SQL 错误静默忽略（card.go:250, graph.go:81）
- Hub 广播路径存在 RLock→Lock 升级窗口（hub.go:64-69）
- ViewRateLimiter cleanup goroutine 持锁遍历可能阻塞

**Vue Frontend**:
- 10 个 composable（web-reader）+ 4 个（admin-tauri），职责分明
- WASM 懒加载 + LRU(30) + IndexedDB(100) 三级缓存策略
- 4 处 `catch (err: any)` 应改为 `unknown`
- Zod runtime validation 覆盖主要 API 响应
- nodes/edges 使用 ref() 而非 shallowRef，1000+ 节点有性能隐患
- WebSocket AUTH 超时：后端 3s vs 前端 5s 不一致

**Rust Workspace**:
- 10 crates DAG 依赖，无循环，ast-core 为纯类型基础层
- 零 unsafe blocks，thiserror 统一错误处理
- AstNode 使用 Cow<'a, str> 零拷贝，serde tag 序列化
- 14 处 .clone() 中 13 处必要，1 处（image-compressor:146）可优化
- ast-renderer 有 XSS 防护（escape_html + sanitize_url）
- wasm-engine release profile: LTO + opt-level="z" + strip

---

## Work Objectives

### Core Objective
对 Memory Stream V3.4 全栈代码进行 7 维度系统性审查，产出可执行的审查报告与重构建议。

### Concrete Deliverables
- 5 份模块审查报告（Go Server / Web Reader / Admin Tauri / Rust Core / Rust Utils）
- 1 份跨端综合审查报告
- 每个问题含：优先级（P0-P3）、代码定位、问题描述、重构示例

### Definition of Done
- [ ] 所有 7 个审查维度在每个模块均完成检查
- [ ] 每个模块报告包含：总体评价 + 关键发现（2-3 个）+ 逐段点评 + 进阶建议
- [ ] P0/P1 问题均提供重构后代码示例
- [ ] 跨端一致性问题汇总

### Must Have
- 按用户确认的 7 维度审查
- 关键问题含代码示例
- 优先级排序

### Must NOT Have (Guardrails)
- 不做纯主观审美评价（如"命名不好看"），必须有可量化标准
- 不提出无实际影响的微优化建议
- 不建议引入项目中不存在的新框架/库（除非安全漏洞必须修补）
- 不脱离 AGENT.md 已定义的编码规范另立标准

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: N/A（审查任务，非实现任务）
- **Automated tests**: N/A
- **Framework**: N/A

### QA Policy
- 每个审查任务完成后，验证发现的准确性（通过 Read 工具再次确认代码）
- 跨任务引用的代码路径必须实际存在
- 优先级判定需给出理由

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — Go Server 全量审查):
├── Task 1: Go 架构与分层审查 [deep]
├── Task 2: Go 性能与数据库查询审查 [deep]
├── Task 3: Go 并发安全审查 [deep]
├── Task 4: Go 错误处理审查 [unspecified-high]
└── Task 5: Go 安全审查 [unspecified-high]

Wave 2 (After Wave 1 — 前端 + Rust 并行):
├── Task 6: Web Reader 架构与性能审查 [deep]
├── Task 7: Web Reader 实时同步与离线审查 [deep]
├── Task 8: Web Reader 类型安全与规范审查 [unspecified-high]
├── Task 9: Admin Tauri 审查 [unspecified-high]
├── Task 10: Rust Core Pipeline 审查 (ast-core/md-parser/ast-renderer/wasm-engine) [deep]
└── Task 11: Rust Utility Crates 审查 (6 crates) [unspecified-high]

Wave FINAL (After ALL tasks — 综合报告):
├── Task F1: 跨端一致性审查 [deep]
├── Task F2: 综合审查报告生成 [writing]
└── Task F3: P0/P1 问题汇总与优先级排序 [unspecified-high]
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| 1-5  | — | 6-11, F1-F3 | 1 |
| 6-11 | 1-5 (findings inform cross-cutting) | F1-F3 | 2 |
| F1-F3 | 1-11 | User review | FINAL |

### Agent Dispatch Summary

- **Wave 1**: **5 agents** — T1-T3 → `deep`, T4-T5 → `unspecified-high`
- **Wave 2**: **6 agents** — T6-T7,T10 → `deep`, T8-T9,T11 → `unspecified-high`
- **FINAL**: **3 agents** — F1 → `deep`, F2 → `writing`, F3 → `unspecified-high`

---

## TODOs

- [ ] 1. Go Server — 架构与分层审查

  **What to do**:
  - 审查 `cmd/api/main.go` 路由注册结构：Public / Authenticated / Admin-only 三层路由分组的完整性
  - 审查 handler → service → storage 三层调用链：是否存在 handler 直接访问 DB、service 直接操作 HTTP context 等越级调用
  - 审查中间件链执行顺序：CORSConfig → ErrorHandler → AuthMiddleware → RequireRole，验证每层职责单一
  - 审查 WebSocket 模块（`internal/ws/`）与业务模块（handlers/services）的边界：Hub 是否只负责连接管理和广播，不包含业务逻辑
  - 审查 `internalerrors/` 和 `internalpkg/logger/` 的包组织：是否遵循 Go internal 包约定
  - 验证 service 层的依赖注入方式：是否通过接口注入便于测试，还是直接依赖具体实现

  **Must NOT do**:
  - 不评判"应该用 Echo 替代 Gin"等框架替换建议
  - 不提出无实际影响的目录重命名建议

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: 架构审查需要跨文件理解调用链和分层关系
  - **Skills**: [`coding-standards`]
    - `coding-standards`: 用于对照 Go 惯用模式和分层规范

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3, 4, 5)
  - **Blocks**: Tasks 6-11, F1-F3
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `go-server/cmd/api/main.go` — 入口：路由注册 + 中间件链 + 服务初始化，理解整体组装方式
  - `go-server/internal/middleware/auth.go` — AuthMiddleware + RequireRole + CORSConfig，验证三层中间件职责
  - `go-server/internal/errors/middleware.go` — ErrorHandler 中间件，验证 panic recovery 策略

  **API/Type References**:
  - `go-server/internal/models/schema.go` — Card, CardEdge, CardMetrics, CardLayout, Category GORM 模型定义
  - `go-server/internal/models/user.go` — User 模型
  - `go-server/internal/ws/protocol.go` — WSAction, WSEvent 事件类型定义

  **Test References**:
  - N/A

  **WHY Each Reference Matters**:
  - `main.go`: 理解路由分组和中间件挂载顺序，是架构审查的起点
  - `middleware/auth.go`: 中间件职责边界审查的核心文件
  - `errors/middleware.go`: 错误处理架构的关键环节
  - `models/`: 理解数据模型定义，验证 service 层是否正确使用
  - `ws/protocol.go`: 理解 WebSocket 协议定义，验证与业务层的边界

  **Acceptance Criteria**:
  - [ ] 路由三层分组验证完成（Public / Authenticated / Admin）
  - [ ] handler→service→storage 调用链无越级调用，或越级调用已标记为 P0/P1
  - [ ] 中间件链顺序正确性验证完成
  - [ ] WebSocket 模块与业务模块边界清晰度评估完成
  - [ ] 产出：架构维度审查报告片段（含关键发现 + 重构建议）

  **QA Scenarios**:

  ```
  Scenario: 验证 handler 层无直接 DB 访问
    Tool: Grep (ast_grep_search)
    Preconditions: Go Server 源码完整
    Steps:
      1. 在 go-server/internal/handlers/*.go 中搜索 db.Raw, db.Find, db.Create, db.Where 等 GORM 调用
      2. 对每个匹配验证是否通过 service 层间接调用
    Expected Result: handlers/ 目录下零直接 GORM 调用，或标记为 P0 issue
    Evidence: .sisyphus/evidence/task-1-handler-db-access.txt

  Scenario: 验证 service 层无 HTTP context 依赖
    Tool: Grep
    Preconditions: Go Server 源码完整
    Steps:
      1. 在 go-server/internal/services/*.go 中搜索 *gin.Context, c.JSON, c.Param 等
    Expected Result: services/ 目录下零 gin.Context 引用，或标记为 P0 issue
    Evidence: .sisyphus/evidence/task-1-service-http-dep.txt
  ```

  **Commit**: NO

---

- [ ] 2. Go Server — 性能与数据库查询审查

  **What to do**:
  - 审查 `internal/services/card.go` 的 ListCards：验证 Preload("Category"), Preload("Metrics") 覆盖所有必要关联
  - 审查 `internal/services/graph.go` 的 GetGraph：验证递归 CTE 的 depth limit 是否在 handler 和 service 双重防护
  - 审查 `internal/services/card.go` 的 IncrementView：验证 raw SQL `INSERT ... ON CONFLICT DO UPDATE` 的正确性
  - 审查 `internal/services/category.go` 的 GetClusters：raw SQL 是否参数化，是否有 SQL 注入风险
  - 搜索所有 GORM 查询路径，标记 N+1 风险（未使用 Preload/Joins 的关联查询）
  - 验证 Redis 缓存策略：`card.go` 中缓存 key 设计、TTL 设置、更新/删除路径的缓存失效
  - 审查 `internal/pkg/logger/slow_query.go`：200ms 阈值是否合理，日志格式是否包含足够诊断信息
  - 验证连接池配置（pg.go: 10 idle, 25 max, 5min lifetime）是否适合预期负载

  **Must NOT do**:
  - 不提出"换用 SQL 替代 GORM"等框架替换建议
  - 不做纯理论性能优化（无实际瓶颈证据）

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: 性能审查需要理解 GORM 查询生成、CTE 语义、缓存策略等深层逻辑
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 3, 4, 5)
  - **Blocks**: Tasks 6-11, F1-F3
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `go-server/internal/services/card.go` — Card CRUD、Redis 缓存、Preload 模式、分页、raw SQL，核心审查目标
  - `go-server/internal/services/graph.go` — 递归 CTE 遍历、batch loading 避免N+1，复杂查询审查
  - `go-server/internal/services/category.go` — Raw SQL cluster 查询，参数化验证
  - `go-server/internal/storage/pg.go` — GORM 连接池配置（10/25/5min），验证合理性
  - `go-server/internal/storage/redis.go` — Redis 客户端初始化，验证连接管理
  - `go-server/internal/pkg/logger/slow_query.go` — 慢查询日志插件，200ms 阈值审查

  **WHY Each Reference Matters**:
  - `card.go`: Card 是核心实体，其查询模式直接决定系统性能
  - `graph.go`: CTE 递归查询是最复杂的 SQL，需验证 depth limit 和性能
  - `category.go`: Raw SQL 是 SQL 注入风险点
  - `pg.go`: 连接池配置决定并发容量
  - `slow_query.go`: 慢查询监控的有效性

  **Acceptance Criteria**:
  - [ ] 所有 GORM 查询路径已审查，N+1 风险已标记
  - [ ] 递归 CTE depth limit 双重防护验证完成
  - [ ] Raw SQL 参数化验证完成
  - [ ] Redis 缓存失效策略验证完成
  - [ ] 连接池配置合理性评估完成
  - [ ] 产出：性能维度审查报告片段

  **QA Scenarios**:

  ```
  Scenario: N+1 查询风险扫描
    Tool: Grep
    Steps:
      1. 在 services/*.go 中搜索 .Find(&, .First(&, .Where( 等查询模式
      2. 对每个查询检查是否有对应的 Preload/Joins
      3. 对缺失 Preload 的关联查询标记为 N+1 风险
    Expected Result: 所有查询路径已分类为"安全"或"N+1 风险"
    Evidence: .sisyphus/evidence/task-2-n-plus-1-scan.txt

  Scenario: Raw SQL 参数化验证
    Tool: Grep
    Steps:
      1. 搜索 db.Raw(, db.Exec( 调用
      2. 验证每个 raw SQL 使用 ? 占位符而非字符串拼接
    Expected Result: 所有 raw SQL 使用参数化查询，或标记为安全风险
    Evidence: .sisyphus/evidence/task-2-raw-sql-params.txt
  ```

  **Commit**: NO

---

- [ ] 3. Go Server — 并发安全审查

  **What to do**:
  - 深入审查 `internal/ws/hub.go` 的 Hub.Run 广播路径：RLock→Lock 升级窗口（line 64-69），评估在实际并发场景下的数据一致性风险
  - 审查 `internal/ws/hub.go` 的 Client.Send/SendError：非阻塞 send + panic recover 是否掩盖了真实问题（closed channel write）
  - 审查 `internal/middleware/ratelimit.go` 的 ViewRateLimiter：cleanup goroutine 持 Lock 遍历 map 对 Allow() 的阻塞影响
  - 审查 WebSocket client 的 ReadPump/WritePump goroutine 生命周期：是否存在 goroutine 泄漏风险
  - 审查 handlers/ws.go 的 AUTH deadline timer：timer 清理是否在所有退出路径执行
  - 搜索 `go func` 启动的所有 goroutine：验证是否有未处理的 panic、是否正确处理 context cancellation

  **Must NOT do**:
  - 不建议引入新的并发库（如 ants pool）
  - 不提出无证据的"可能存在 race condition"猜测

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: 并发安全审查需要精确理解 goroutine 生命周期、锁语义、channel 时序
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2, 4, 5)
  - **Blocks**: Tasks 6-11, F1-F3
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `go-server/internal/ws/hub.go` — Hub struct + Run loop + Client 管理 + broadcast，并发安全审查核心
  - `go-server/internal/handlers/ws.go` — WebSocket upgrade、AUTH 握手、ReadPump/WritePump 启动
  - `go-server/internal/middleware/ratelimit.go` — ViewRateLimiter 并发 map + cleanup goroutine

  **WHY Each Reference Matters**:
  - `hub.go`: Hub 是 WebSocket 并发安全的核心，所有客户端连接共享同一个 Hub 实例
  - `ws.go`: Client goroutine 生命周期管理，AUTH timer 清理是泄漏风险点
  - `ratelimit.go`: 独立的并发模式，cleanup goroutine 的锁行为影响请求延迟

  **Acceptance Criteria**:
  - [ ] Hub RLock→Lock 升级窗口风险评估完成（含具体竞态场景分析）
  - [ ] Client.Send panic recover 合理性评估完成
  - [ ] RateLimiter cleanup 阻塞影响评估完成
  - [ ] 所有 goroutine 泄漏风险点已标记
  - [ ] 产出：并发安全维度审查报告片段

  **QA Scenarios**:

  ```
  Scenario: Goroutine 泄漏风险扫描
    Tool: Grep
    Steps:
      1. 搜索 go-server/ 下所有 "go func" 和 "go " 启动的 goroutine
      2. 对每个 goroutine 检查退出条件（context cancel / channel close / done signal）
      3. 验证 defer recover 是否覆盖所有 panic 路径
    Expected Result: 所有 goroutine 有明确退出条件，或标记为泄漏风险
    Evidence: .sisyphus/evidence/task-3-goroutine-leak.txt

  Scenario: Hub broadcast 竞态条件分析
    Tool: Read
    Steps:
      1. 精读 hub.go Run() 方法的 select 分支
      2. 追踪 broadcast 分支中 RLock→unlock→Lock 的代码路径
      3. 分析在 unlock 到 Lock 之间是否有其他 goroutine 修改 clients map
    Expected Result: 竞态窗口确认或排除，给出具体场景
    Evidence: .sisyphus/evidence/task-3-hub-race.txt
  ```

  **Commit**: NO

---

- [ ] 4. Go Server — 错误处理审查

  **What to do**:
  - 审查 `internal/errors/errors.go` 的 AppError 结构：HTTP code / biz code / message / logDetails 字段是否足够
  - 审查 `internal/errors/middleware.go` 的 Respond 函数：类型分支是否覆盖所有 error 类型
  - 搜索所有 handler 中 `appErr.Respond(c, ...)` 的使用：验证一致性
  - 审查已知的静默忽略错误：`card.go:250` 和 `graph.go:81` 的 fallback SQL 错误是否应至少记录日志
  - 审查 `hub.go` 中 `json.Marshal` 错误处理：BroadcastEvent 中 marshal 失败仅 log 是否足够
  - 审查 WebSocket 错误传播：客户端发送无效 action 时是否返回有意义的 ERROR 事件

  **Must NOT do**:
  - 不提出"引入 error chain 库"等新依赖建议

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: 错误处理审查涉及跨文件的一致性检查，需要细心但不需深度架构推理
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2, 3, 5)
  - **Blocks**: Tasks 6-11, F1-F3
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `go-server/internal/errors/errors.go` — AppError 定义和构造函数
  - `go-server/internal/errors/middleware.go` — ErrorHandler + Respond 函数
  - `go-server/internal/services/card.go` — 已知静默错误点 (line 250)
  - `go-server/internal/services/graph.go` — 已知静默错误点 (line 81)
  - `go-server/internal/ws/hub.go` — BroadcastEvent marshal 错误处理
  - `go-server/internal/handlers/ws.go` — WebSocket 错误响应

  **WHY Each Reference Matters**:
  - `errors.go`: 错误体系的基础定义
  - `middleware.go`: 统一错误响应的入口
  - `card.go:250`, `graph.go:81`: 已知的静默错误需评估严重性
  - `hub.go`: 广播场景的错误处理是否会导致事件丢失

  **Acceptance Criteria**:
  - [ ] AppError 结构体完整性评估完成
  - [ ] 所有 handler 错误处理一致性验证完成
  - [ ] 静默错误点严重性评级完成
  - [ ] WebSocket 错误传播路径验证完成
  - [ ] 产出：错误处理维度审查报告片段

  **QA Scenarios**:

  ```
  Scenario: 静默错误扫描
    Tool: Grep
    Steps:
      1. 搜索 go-server/ 中 _ = db., if err != nil { /* 空 */ }, err = xxx (无 if 检查) 模式
      2. 对每个匹配评估：是否应该处理、是否应该记录日志
    Expected Result: 所有静默错误已分类（合理忽略 / 应添加日志 / 应返回错误）
    Evidence: .sisyphus/evidence/task-4-silent-errors.txt

  Scenario: Handler 错误响应一致性
    Tool: Grep
    Steps:
      1. 搜索 handlers/*.go 中 c.JSON( 和 appErr.Respond( 调用
      2. 验证是否统一使用 appErr.Respond，无直接 c.JSON 错误响应
    Expected Result: 所有错误响应通过 appErr.Respond，或标记为不一致
    Evidence: .sisyphus/evidence/task-4-error-consistency.txt
  ```

  **Commit**: NO

---

- [ ] 5. Go Server — 安全审查

  **What to do**:
  - 审查 `internal/services/auth.go`：JWT secret 长度验证、token 过期时间、refresh token 机制
  - 审查 `internal/handlers/auth.go`：Genesis Admin 首次启动是否存在竞态条件（多实例同时启动）
  - 审查 `internal/middleware/auth.go`：CORS 白名单是否包含生产域名（当前仅 localhost）
  - 审查所有 raw SQL（`card.go`, `graph.go`, `category.go`）：是否全部使用参数化查询
  - 审查 WebSocket AUTH 机制：token 是否通过消息体（非 URL）传递，AUTH 超时断开是否可靠
  - 审查 User model：PasswordHash 是否 json:"-"，bcrypt cost 因子是否合理
  - 搜索敏感信息泄露风险：日志中是否输出 JWT token、密码等

  **Must NOT do**:
  - 不提出"换用 OAuth2 替代 JWT"等架构变更建议
  - 不做渗透测试级别安全评估

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: 安全审查需要系统性检查但不需要架构深度推理
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2, 3, 4)
  - **Blocks**: Tasks 6-11, F1-F3
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `go-server/internal/services/auth.go` — JWT 签发/解析、bcrypt 密码处理、token claims
  - `go-server/internal/handlers/auth.go` — Login, Register, Refresh, Genesis 端点
  - `go-server/internal/middleware/auth.go` — CORS 白名单、JWT 验证
  - `go-server/internal/models/user.go` — User 模型，json:"-" 验证

  **WHY Each Reference Matters**:
  - `services/auth.go`: JWT 实现的安全性（secret、过期、claims）
  - `handlers/auth.go`: Genesis 竞态条件是高优先级安全风险
  - `middleware/auth.go`: CORS 配置决定哪些前端可以访问 API
  - `user.go`: 密码字段序列化安全性

  **Acceptance Criteria**:
  - [ ] JWT 实现安全性验证完成（secret 长度、过期时间、claims）
  - [ ] Genesis Admin 竞态条件评估完成
  - [ ] CORS 生产就绪性评估完成
  - [ ] SQL 注入风险扫描完成
  - [ ] WebSocket 认证安全性验证完成
  - [ ] 敏感信息泄露扫描完成
  - [ ] 产出：安全维度审查报告片段

  **QA Scenarios**:

  ```
  Scenario: 敏感信息泄露扫描
    Tool: Grep
    Steps:
      1. 搜索 logger.Log., log., fmt.Print 等输出语句
      2. 检查输出内容是否包含 token, password, secret 等敏感字段
    Expected Result: 零敏感信息泄露，或标记为 P0 安全漏洞
    Evidence: .sisyphus/evidence/task-5-sensitive-leak.txt

  Scenario: Genesis 竞态条件验证
    Tool: Read
    Steps:
      1. 精读 handlers/auth.go Genesis 函数
      2. 分析：查询 admin 是否存在 → 创建 admin 两步操作之间是否有并发保护
      3. 验证数据库层是否有唯一约束保护
    Expected Result: 确认竞态条件存在或已防护
    Evidence: .sisyphus/evidence/task-5-genesis-race.txt
  ```

  **Commit**: NO

- [ ] 6. Web Reader — 架构与性能审查

  **What to do**:
  - 审查 Vue 组件层次结构：GraphView → DetailDrawer / ZenReader / LeftDock / FloatingCompass，验证组件职责单一性和解耦度
  - 审查 composable 职责边界：`useGraph`（数据加载）vs `useGraphSync`（实时同步）vs `useCards`（卡片 CRUD + WASM）是否有重叠
  - 审查 Pinia store（`useGraphStore`）状态设计：是否有过多的全局状态应该局部化
  - 审查 `utils/graphLayout.ts`（333 行）的布局算法复杂度：graphology 孤岛切割 + Dagre 布局 + potpack 打包的计算量
  - 审查 Vue Flow nodes/edges 的响应式策略：当前使用 `ref()`，评估 1000+ 节点场景下是否需要 `shallowRef`
  - 审查 `views/GraphView.vue` 的聚光灯模式实现：BFS 遍历 + opacity/blur 切换是否影响渲染性能
  - 审查 `components/EntranceAnimation.vue` 的动画性能：粒子场 + 光晕是否使用 GPU 加速（transform/opacity）
  - 审查 API 层（`api/index.ts`）：fetch wrapper 的 JWT 注入、guest 静默登录、refresh token 队列机制

  **Must NOT do**:
  - 不提出"换用 React/Virtual DOM 框架"建议
  - 不建议引入项目中不存在的虚拟滚动库（除非当前实现有实际瓶颈）

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: 前端架构审查需要理解 Vue 响应式系统、组件通信模式、算法复杂度
  - **Skills**: [`coding-standards`, `frontend-design`]
    - `coding-standards`: Vue/TS 编码规范对照
    - `frontend-design`: 动画性能、布局算法评估

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 7, 8, 9, 10, 11)
  - **Blocks**: F1-F3
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/web-reader/src/views/GraphView.vue` — 主视图，组件组合和图谱渲染入口
  - `frontend-workspace/apps/web-reader/src/composables/useGraph.ts` — 图谱数据加载，nodes/edges 转换
  - `frontend-workspace/apps/web-reader/src/utils/graphLayout.ts` — 布局算法核心（333 行），性能关键路径
  - `frontend-workspace/apps/web-reader/src/store/useGraphStore.ts` — 全局状态设计
  - `frontend-workspace/apps/web-reader/src/api/index.ts` — HTTP 客户端 + JWT + refresh token

  **API/Type References**:
  - `frontend-workspace/apps/web-reader/src/api/schemas.ts` — Zod runtime validation schemas

  **WHY Each Reference Matters**:
  - `GraphView.vue`: 组件组合方式反映整体架构质量
  - `useGraph.ts`: 数据加载策略直接影响首屏性能
  - `graphLayout.ts`: 布局算法是最重的 CPU 计算，需评估复杂度
  - `useGraphStore.ts`: 全局状态设计影响组件解耦度
  - `api/index.ts`: HTTP 层设计影响错误处理和认证安全
  - `schemas.ts`: 类型安全的关键保障层

  **Acceptance Criteria**:
  - [ ] 组件层次结构合理性评估完成
  - [ ] composable 职责边界清晰度评估完成
  - [ ] graphLayout 算法复杂度评估完成
  - [ ] nodes/edges 响应式策略评估完成
  - [ ] 动画 GPU 加速策略评估完成
  - [ ] API 层 JWT/refresh 机制审查完成
  - [ ] 产出：Web Reader 架构与性能审查报告片段

  **QA Scenarios**:

  ```
  Scenario: composable 职责重叠检测
    Tool: Read
    Steps:
      1. 并行读取 useGraph.ts, useGraphSync.ts, useCards.ts
      2. 提取每个 composable 的导出函数列表
      3. 检查是否有功能重叠（如两个 composable 都在管理 nodes/edges）
    Expected Result: 职责边界清晰无重叠，或标记为架构问题
    Evidence: .sisyphus/evidence/task-6-composable-overlap.txt

  Scenario: 响应式策略性能评估
    Tool: Grep
    Steps:
      1. 搜索 web-reader/src/ 中所有 ref<Node[]>, ref<Edge[]>, reactive(
      2. 评估这些数据结构的预期大小
      3. 标记潜在需要 shallowRef 的大数据结构
    Expected Result: 响应式策略合理性评估
    Evidence: .sisyphus/evidence/task-6-reactivity-strategy.txt
  ```

  **Commit**: NO

---

- [ ] 7. Web Reader — 实时同步与离线审查

  **What to do**:
  - 深入审查 `composables/useGraphSync.ts`（445 行）：
    - Auth-on-Connect 流程：连接→AUTH→5s 超时，与后端 3s 超时不一致的影响
    - Ping/Pong RTT：15s 间隔是否合理，延迟计算是否有竞态
    - 指数退避重连：3s→30s 策略是否正确实现，认证失败重试是否有上限
    - 增量更新处理：CARD_CREATED/UPDATED/DELETED、EDGE_CREATED/DELETED/UPDATED 事件处理是否有数据不一致风险
    - 模块级导出 `wsConnected`/`wsAuthenticated`/`wsLatency` 的响应式更新机制
  - 审查 `composables/useOfflineCache.ts`：
    - IndexedDB schema 设计（card-summaries, cached_at index）
    - 100 条上限自动裁剪策略的正确性
    - Promise 包装 IDB 操作的错误处理
  - 审查 `composables/useCardCache.ts`：LRU(30 items) 的实现正确性
  - 审查 `composables/useCards.ts`：WASM 懒加载策略、AST→HTML 渲染降级链（astData → rawMd → error）
  - 评估 WebSocket 断连期间的用户数据丢失风险

  **Must NOT do**:
  - 不建议引入 Service Worker（除非当前离线策略有实际缺陷）
  - 不提出"换用 Socket.IO"等库替换建议

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: WebSocket + IndexedDB + WASM 的交互涉及复杂的状态管理和时序问题
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 6, 8, 9, 10, 11)
  - **Blocks**: F1-F3
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/web-reader/src/composables/useGraphSync.ts` — WebSocket 全量实现（445 行），审查核心
  - `frontend-workspace/apps/web-reader/src/composables/useOfflineCache.ts` — IndexedDB 离线缓存
  - `frontend-workspace/apps/web-reader/src/composables/useCardCache.ts` — LRU 内存缓存
  - `frontend-workspace/apps/web-reader/src/composables/useCards.ts` — WASM 渲染 + 缓存集成

  **WHY Each Reference Matters**:
  - `useGraphSync.ts`: WebSocket 是实时同步的核心，445 行的单文件需要仔细审查时序问题
  - `useOfflineCache.ts`: 离线体验的可靠性取决于 IndexedDB 的正确使用
  - `useCardCache.ts`: LRU 缓存的正确性影响渲染性能
  - `useCards.ts`: WASM 集成的降级策略决定用户体验

  **Acceptance Criteria**:
  - [ ] WebSocket AUTH 超时不一致影响评估完成（后端 3s vs 前端 5s）
  - [ ] 重连策略正确性验证完成
  - [ ] 增量更新数据一致性风险评估完成
  - [ ] IndexedDB 缓存策略完整性验证完成
  - [ ] LRU 缓存实现正确性验证完成
  - [ ] WASM 降级链完整性验证完成
  - [ ] 产出：实时同步与离线审查报告片段

  **QA Scenarios**:

  ```
  Scenario: WebSocket AUTH 超时一致性
    Tool: Read
    Steps:
      1. 读取 useGraphSync.ts 中 AUTH 超时配置（前端 5s）
      2. 读取 go-server/internal/handlers/ws.go 中 AUTH 超时配置（后端 3s）
      3. 分析：前端 5s 等待 vs 后端 3s 断开的时序差异
    Expected Result: 确认不一致的具体影响，是否导致前端误判连接状态
    Evidence: .sisyphus/evidence/task-7-auth-timeout-mismatch.txt

  Scenario: IndexedDB 缓存裁剪正确性
    Tool: Read
    Steps:
      1. 精读 useOfflineCache.ts 的 auto-trim 逻辑
      2. 验证：是否使用 cached_at index 排序后删除最旧记录
      3. 验证：裁剪操作的事务完整性（是否可能在裁剪过程中丢失数据）
    Expected Result: 裁剪逻辑正确或有数据丢失风险标记
    Evidence: .sisyphus/evidence/task-7-idb-trim.txt
  ```

  **Commit**: NO

---

- [ ] 8. Web Reader — 类型安全与代码规范审查

  **What to do**:
  - 审查 4 处 `catch (err: any)` 使用（useGraph.ts:62,97 / useCards.ts:167 / TocNode.vue）：给出 `unknown` + narrowing 的重构示例
  - 审查 `api/schemas.ts` 的 Zod 验证覆盖度：是否所有 API 响应都有对应 schema
  - 搜索所有 TypeScript 文件中的 `any` 类型使用（超出已知 4 处）
  - 审查 `<script setup>` + Composition API 规范遵循情况
  - 审查 Tailwind CSS 使用：是否存在 inline style 混入
  - 审查组件 props/emits 类型定义完整性
  - 审查 `types/shims.d.ts` 是否有需要更新的类型声明
  - 验证 `packages/types/` 共享类型包的使用情况

  **Must NOT do**:
  - 不提出"换用 CSS-in-JS"等样式方案变更
  - 不对已规范的代码重复确认

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: 类型安全审查是系统性的扫描工作，需要仔细但不需要深度架构推理
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 6, 7, 9, 10, 11)
  - **Blocks**: F1-F3
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/web-reader/src/composables/useGraph.ts:62,97` — 已知 `any` 使用点
  - `frontend-workspace/apps/web-reader/src/composables/useCards.ts:167` — 已知 `any` 使用点
  - `frontend-workspace/apps/web-reader/src/api/schemas.ts` — Zod 验证覆盖度审查
  - `frontend-workspace/apps/web-reader/src/types/shims.d.ts` — 类型声明
  - `frontend-workspace/packages/types/` — 共享类型包

  **WHY Each Reference Matters**:
  - `useGraph.ts/useCards.ts`: 已知的类型安全薄弱点，需给出重构示例
  - `schemas.ts`: Zod 是运行时类型安全的最后一道防线
  - `shims.d.ts`: 全局类型声明的正确性
  - `packages/types/`: 跨应用共享类型的完整性

  **Acceptance Criteria**:
  - [ ] 所有 `any` 类型使用已定位和分类
  - [ ] 4 处已知 `any` 已给出重构示例
  - [ ] Zod schema 覆盖度评估完成
  - [ ] Composition API 规范遵循验证完成
  - [ ] Tailwind 规范遵循验证完成
  - [ ] 产出：类型安全与规范审查报告片段

  **QA Scenarios**:

  ```
  Scenario: 全面 `any` 扫描
    Tool: Grep
    Steps:
      1. 在 frontend-workspace/apps/web-reader/src/ 中搜索 : any, as any, <any>
      2. 对每个匹配分类：catch 块 / 函数参数 / 返回值 / 类型断言
      3. 给出优先级和重构建议
    Expected Result: 完整的 `any` 清单（含文件:行号:上下文:优先级）
    Evidence: .sisyphus/evidence/task-8-any-scan.txt
  ```

  **Commit**: NO

---

- [ ] 9. Admin Tauri — 全维度审查

  **What to do**:
  - **架构审查**：Vue 前端（components/stores/composables）与 Rust 后端（src-tauri/src/）的 IPC 边界是否清晰
  - **状态管理审查**：`stores/knowledge.ts` 是否承载过多逻辑（CRUD + 搜索 + 分类 + 草稿 + 导出），是否应拆分
  - **IPC 命令审查**：`src-tauri/src/lib.rs` 命令注册、`api.rs` HTTP 代理网关、`ws_client.rs` WebSocket 客户端
  - **认证审查**：`src-tauri/src/auth.rs` JWT 状态管理、`composables/useAuth.ts` 静默登录
  - **文件监控审查**：`src-tauri/src/watcher.rs` Markdown Vault 目录变更检测的可靠性
  - **图片管道审查**：`src-tauri/src/image.rs` WebP 压缩 + S3 上传的错误处理和进度反馈
  - **缓存审查**：`src-tauri/src/cache.rs` SQLite 缓存一致性
  - **导出审查**：`src-tauri/src/export.rs` ZIP 导出的大文件处理策略

  **Must NOT do**:
  - 不提出"换用 Electron"建议
  - 不做 Tauri v2 框架本身的 bug 追踪

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Admin Tauri 审查涉及 IPC 边界和 Rust/JS 交互，需要细心但模块相对独立
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 6, 7, 8, 10, 11)
  - **Blocks**: F1-F3
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/admin-tauri/src/stores/knowledge.ts` — 核心状态管理（CRUD + 搜索 + 分类 + 草稿 + 导出）
  - `frontend-workspace/apps/admin-tauri/src/components/TheForge.vue` — Markdown 编辑器组件
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/lib.rs` — Tauri 命令注册入口
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/api.rs` — HTTP 客户端网关
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/auth.rs` — JWT 认证状态
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/watcher.rs` — 文件系统监控
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/image.rs` — 图片压缩 + S3 管道
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/cache.rs` — SQLite 缓存
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/export.rs` — 知识库导出

  **WHY Each Reference Matters**:
  - `knowledge.ts`: 最核心的 Pinia store，职责是否过重
  - `TheForge.vue`: 编辑器是 Tauri 端的主要交互界面
  - `lib.rs`: 命令注册反映 IPC 架构设计
  - `api.rs`: HTTP 代理模式是否安全高效
  - `auth.rs`: JWT 状态管理的安全性
  - `watcher.rs`: 文件监控的可靠性影响用户体验
  - `image.rs`: 图片管道的错误处理影响创作流程
  - `cache.rs`/`export.rs`: 本地功能的数据安全

  **Acceptance Criteria**:
  - [ ] IPC 边界合理性评估完成
  - [ ] knowledge.ts 职责是否过重评估完成
  - [ ] 认证流程安全性验证完成
  - [ ] 文件监控可靠性评估完成
  - [ ] 图片管道错误处理评估完成
  - [ ] 产出：Admin Tauri 全维度审查报告片段

  **QA Scenarios**:

  ```
  Scenario: knowledge.ts 职责分析
    Tool: Read
    Steps:
      1. 读取 knowledge.ts 全文
      2. 提取所有 actions，按职责分类（CRUD / 搜索 / 分类 / 草稿 / 导出）
      3. 评估每个职责是否应在独立 composable/store 中
    Expected Result: 职责分类清单 + 拆分建议（如需要）
    Evidence: .sisyphus/evidence/task-9-store-responsibility.txt

  Scenario: IPC 命令安全性扫描
    Tool: Read
    Steps:
      1. 读取 lib.rs 中的 invoke_handler 命令注册
      2. 对每个命令检查：是否有权限检查、是否验证输入、是否处理错误
    Expected Result: 所有 IPC 命令安全性评估
    Evidence: .sisyphus/evidence/task-9-ipc-security.txt
  ```

  **Commit**: NO

- [ ] 10. Rust Core Pipeline — AST 管线审查 (ast-core / md-parser / ast-renderer / wasm-engine)

  **What to do**:
  - **ast-core 审查**：
    - 审查 `AstNode<'a>` enum 的 17 个变体：是否有遗漏的 Markdown 元素（如表格 table、脚注 footnote、定义列表 definition list）
    - 审查 `Cow<'a, str>` 的 zero-copy 策略：生命周期标注是否正确，是否存在悬垂引用风险
    - 审查 `#[serde(tag = "type")]` 序列化：与前端 JS 反序列化的兼容性
    - 审查 `push_child()` 方法的所有权语义
  - **md-parser 审查**：
    - 审查 pulldown-cmark 集成：是否正确处理所有 pulldown-cmark 事件（Event/Tag/HeadingLevel）
    - 审查 AST 构建逻辑：嵌套结构（blockquote > list > code）的正确性
    - 审查 wikilink 解析：`[[card-name]]` 语法的边界情况（空名称、特殊字符、嵌套方括号）
    - 审查代码块高亮：language hint 的传递方式
  - **ast-renderer 审查**：
    - 审查 XSS 防护覆盖度：`escape_html()` 和 `sanitize_url()` 的实现是否覆盖所有变体
    - 审查 HTML 输出语义化：heading → `<h1>`~`<h6>`、code → `<pre><code>` 等是否正确
    - 审查自定义渲染规则的可扩展性
    - 审查 Mermaid 代码块：`<pre class="mermaid">` 是否有 script injection 风险
  - **wasm-engine 审查**：
    - 审查 3 个公共函数的 JS API 设计：参数/返回值类型是否友好
    - 审查 `WasmRenderResult` 的 `#[wasm_bindgen(getter_with_clone)]` 是否引入不必要的 clone
    - 审查 release profile（LTO + opt-level="z" + strip）：对 WASM 运行时性能的影响
    - 审查错误传播：`Result<T, JsValue>` 是否包含足够诊断信息
  - **跨 crate 审查**：
    - 验证依赖图是严格的 DAG：ast-core → md-parser → wasm-engine, ast-core → ast-renderer → wasm-engine
    - 评估 ast-gen 为 binary-only（无 lib.rs）的设计是否合理
    - 评估新增 AST node 类型时需修改的文件数量

  **Must NOT do**:
  - 不提出"换用 markdown-it / unified.js"建议
  - 不建议引入 unsafe 优化（当前零 unsafe 是优势）

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Rust AST 管线审查需要理解生命周期、所有权、零拷贝、WASM FFI 等深层语义
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 6, 7, 8, 9, 11)
  - **Blocks**: F1-F3
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `rust-workspace/ast-core/src/lib.rs` — 17 个 AstNode 变体定义，zero-copy Cow 策略
  - `rust-workspace/ast-core/src/error.rs` — MSError 定义
  - `rust-workspace/md-parser/src/lib.rs` — pulldown-cmark 集成、AST 构建入口
  - `rust-workspace/ast-renderer/src/lib.rs` — HTML 渲染、XSS 防护、escape_html/sanitize_url
  - `rust-workspace/wasm-engine/src/lib.rs` — 3 个 wasm_bindgen 公共函数
  - `rust-workspace/wasm-engine/Cargo.toml` — release profile 配置

  **WHY Each Reference Matters**:
  - `ast-core/lib.rs`: AST 类型系统是整个管线的基石，影响所有下游 crate
  - `md-parser/lib.rs`: 解析正确性直接决定内容渲染质量
  - `ast-renderer/lib.rs`: XSS 防护是安全底线，HTML 输出质量影响阅读体验
  - `wasm-engine/lib.rs`: JS API 设计影响前端集成复杂度
  - `wasm-engine/Cargo.toml`: release profile 直接影响 WASM 体积和运行时性能

  **Acceptance Criteria**:
  - [ ] AstNode 变体完整性评估完成（是否覆盖常用 Markdown 元素）
  - [ ] Cow 生命周期正确性验证完成
  - [ ] pulldown-cmark 事件处理完整性验证完成
  - [ ] XSS 防护覆盖度验证完成
  - [ ] WASM API 设计合理性评估完成
  - [ ] 依赖图 DAG 验证完成
  - [ ] 新增 node 类型修改范围评估完成
  - [ ] 产出：Rust Core Pipeline 审查报告片段

  **QA Scenarios**:

  ```
  Scenario: AST node 类型完整性检查
    Tool: Read
    Steps:
      1. 读取 ast-core/src/lib.rs，提取所有 AstNode 变体
      2. 对照 CommonMark + GFM spec，列出缺失的元素类型（Table, Footnote, Strikethrough 等）
      3. 验证 md-parser 是否在 pulldown-cmark Options 中启用了对应的扩展
    Expected Result: 完整的覆盖度对照表（已支持 / 部分支持 / 未支持）
    Evidence: .sisyphus/evidence/task-10-ast-coverage.txt

  Scenario: XSS 防护边界测试
    Tool: Read
    Steps:
      1. 精读 ast-renderer/src/lib.rs 的 escape_html() 和 sanitize_url() 实现
      2. 构造测试用例：<script>, javascript:alert(1), data:text/html, onerror=, <img src=x
      3. 逐 case 验证渲染结果是否安全
    Expected Result: 所有 XSS 向量被正确转义/阻止
    Evidence: .sisyphus/evidence/task-10-xss-coverage.txt
  ```

  **Commit**: NO

---

- [ ] 11. Rust Utility Crates — 审查 (image-compressor / ms-storage / ms-local-draft / ms-toc-extractor / ms-kb-exporter)

  **What to do**:
  - **image-compressor 审查**：
    - 审查 `compress_to_webp()` 的 `png_data.clone()` 是否可优化为引用传递
    - 审查 CompressOptions 参数设计的合理性（质量/缩放/格式）
    - 审查错误处理：CompressError (DecodeError/EncodeError/ResizeError/TaskPanic) 的信息丰富度
    - 审查 `task::spawn_blocking` 的使用：是否正确处理 JoinError
  - **ms-storage 审查**：
    - 审查 `StorageProvider` trait 设计：是否覆盖必要操作（upload/delete/head/url）
    - 审查 `create_storage()` 工厂函数：config 验证是否充分
    - 审查 S3Backend 的并发安全性（多线程上传）
    - 审查 StorageError 变体的完整性
  - **ms-local-draft 审查**：
    - 审查 `Arc<Mutex<rusqlite::Connection>>` 的并发模型：高并发写入时的锁竞争
    - 审查 SQLite schema 设计：时间戳精度、索引覆盖
    - 审查 DraftError 变体的信息可操作性
    - 审查 `save_draft` 的 upsert 语义正确性
  - **ms-toc-extractor 审查**：
    - 审查 TOC 提取的边界情况：空文档、仅有一个 heading、heading 层级跳跃（h1→h4）
    - 审查 TocNode 数据结构的序列化格式
  - **ms-kb-exporter 审查**：
    - 审查 ZIP 导出的大文件策略：是否流式写入而非全量内存
    - 审查 ExportOptions 参数设计
    - 审查错误处理：ExportError 变体的信息可操作性
    - 审查 `export_with_fetcher` 的异步回调设计
  - **跨 crate 通用审查**：
    - 所有 .clone() 调用的必要性确认（14 处中 1 处已标记为可优化）
    - 所有 Result type alias 命名一致性
    - 所有 From trait 实现链的完整性

  **Must NOT do**:
  - 不提出"换用 SQLx 替代 rusqlite"等库替换建议
  - 不建议引入 async trait 库（如果 StorageProvider 已是手写 async）

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: 6 个 utility crate 审查需要系统性检查但每个 crate 相对独立且简单
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 6, 7, 8, 9, 10)
  - **Blocks**: F1-F3
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `rust-workspace/image-compressor/src/lib.rs` + `error.rs` — WebP 压缩 + 错误定义
  - `rust-workspace/ms-storage/src/lib.rs` + `error.rs` — S3 存储抽象 + 工厂函数
  - `rust-workspace/ms-local-draft/src/lib.rs` + `error.rs` — SQLite 草稿管理
  - `rust-workspace/ms-toc-extractor/src/lib.rs` — TOC 提取
  - `rust-workspace/ms-kb-exporter/src/lib.rs` + `error.rs` — ZIP 导出

  **WHY Each Reference Matters**:
  - `image-compressor`: 图片处理性能直接影响创作体验，clone 优化是已知改进点
  - `ms-storage`: S3 上传是图片管道的关键环节，并发安全性重要
  - `ms-local-draft`: 离线编辑的数据安全，SQLite 并发模型是核心关注点
  - `ms-toc-extractor`: TOC 正确性影响阅读导航体验
  - `ms-kb-exporter`: 导出功能的数据完整性和大文件处理能力

  **Acceptance Criteria**:
  - [ ] image-compressor clone 优化建议已产出（含代码示例）
  - [ ] StorageProvider trait 完整性评估完成
  - [ ] ms-local-draft 并发模型安全性评估完成
  - [ ] TOC 边界情况覆盖度评估完成
  - [ ] ms-kb-exporter 大文件策略评估完成
  - [ ] 所有 .clone() 必要性确认完成
  - [ ] 产出：Rust Utility Crates 审查报告片段

  **QA Scenarios**:

  ```
  Scenario: .clone() 必要性审计
    Tool: Grep
    Steps:
      1. 在 rust-workspace/ 中搜索 .clone()
      2. 对每个 clone 分析：是否可改为引用 (&T)、是否因所有权转移必要、是否因 async 边界必要
    Expected Result: 每个 clone 标记为 [必要/可优化/待定] + 理由
    Evidence: .sisyphus/evidence/task-11-clone-audit.txt

  Scenario: SQLite 并发写入安全性
    Tool: Read
    Steps:
      1. 读取 ms-local-draft/src/lib.rs
      2. 追踪 Arc<Mutex<Connection>> 的所有 acquire 点
      3. 评估：是否有死锁可能、是否因持锁时间过长影响并发
    Expected Result: 并发安全性评估 + 改进建议（如有）
    Evidence: .sisyphus/evidence/task-11-sqlite-concurrency.txt
  ```

  **Commit**: NO

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

- [ ] F1. **跨端一致性审查** — `deep`
  检查 Go/Vue/Rust 三端的接口契约一致性：API 响应类型 ↔ Zod Schema ↔ TS Interface ↔ Go Struct ↔ Rust Serde 反序列化是否对齐。WebSocket 事件协议前后端是否匹配（事件名、payload 字段、超时配置）。JWT token 流程端到端是否闭环。

- [ ] F2. **综合审查报告生成** — `writing`
  汇总所有模块的审查结果，生成一份结构化综合报告：总体评价、P0-P1 关键问题清单（含代码定位和重构示例）、P2-P3 改进建议清单、架构级长期建议。输出为 `.sisyphus/drafts/review-report-comprehensive.md`。

- [ ] F3. **P0/P1 问题汇总与优先级排序** — `unspecified-high`
  从所有模块审查中提取 P0（阻断性）和 P1（高优先级）问题，按影响范围和修复成本排序。给出建议修复顺序和依赖关系。

---

## Commit Strategy

N/A — 审查任务不涉及代码修改，仅产出审查报告。

---

## Success Criteria

### Final Checklist
- [ ] Go Server 5 维度审查完成（架构/性能/并发/错误/安全）
- [ ] Web Reader 3 维度审查完成（架构性能/同步离线/类型安全）
- [ ] Admin Tauri 全维度审查完成
- [ ] Rust Core Pipeline 4 crates 审查完成
- [ ] Rust Utility 6 crates 审查完成
- [ ] 跨端一致性审查完成
- [ ] 综合审查报告生成
- [ ] P0/P1 问题清单产出
