# 品质加固 × 新能注入 — 下一次迭代计划

## TL;DR

> **Quick Summary**: 三方向迭代 — (A) WebSocket 同步性能优化 + 并发安全修复 + 类型安全； (B) 全文搜索（Go tsvector + Web Reader）+ 知识库导入（Tauri MD/ZIP）； (C) UI 过渡动效打磨。全程 TDD。
> 
> **Deliverables**:
> - Go Hub 并发修复 + WS AUTH 超时对齐 + 性能基准测试
> - Vue shallowRef 大图谱优化 + catch(any)→unknown 类型安全
> - PostgreSQL tsvector 全文索引 + Go 搜索端点 + Web Reader 搜索 UI
> - Admin Tauri Markdown/ZIP 导入管道（gray_matter + zip crate）
> - 双端 UI 过渡动效系统
> 
> **Estimated Effort**: Large
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: P1(Hub基准) → P4(shallowRef) → S2(搜索集成) → I3(导入集成) → F1-F4

---

## Context

### Original Request
基于项目全量分析（4 模块、8 个已完成计划、零 TODO 代码库），用户要求下一迭代聚焦：代码质量加固、新增功能特性、UX 打磨。

### Interview Summary
**Key Discussions**:
- 性能优化方向：**实时同步性能**（非大图谱渲染），聚焦 WebSocket 层
- 搜索范围：**全文搜索**（前后端），Go 端 PostgreSQL tsvector + Web Reader 搜索 UI
- 知识库导入：**Markdown 文件 + ZIP** 都支持，通过 Admin Tauri 文件系统
- 动画范围：**UI 过渡动效**（抽屉、加载骨架屏、状态切换），非图谱动画
- 测试策略：**TDD 模式**

**Research Findings** (Metis + Explore):
- Hub RLock→Lock 升级窗口：hub.go:67-78，需释放 RLock 再获取 Lock
- AUTH 超时：仅 ws_client.rs:11 需修复（Rust 5s→3s），Vue 端已是 3s
- 4 处 catch(err: any) 需改为 unknown
- useGraph.ts:33-34 使用 ref() 而非 shallowRef，大图谱性能隐患
- CodemirrorEditor.vue:27 已有 shallowRef 使用模式可参考
- tsvector 推荐方案：GIN 索引 + generated stored column + trigger
- 导入依赖：gray_matter（frontmatter）、zip（归档）、encoding_rs（编码）

### Metis Review
**Identified Gaps** (addressed):
- Vue AUTH 超时实际已是 3s，仅 Rust 端需修复 → 缩小了 P2 范围
- 项目含中文内容，tsvector 应使用 `simple` 配置（非 english-only）→ 已调整
- 导入需处理 UTF-8 BOM、二进制文件跳过、重复标题检测 → 已加入 I2 验收标准
- 无现有性能基准测试 → P1/P4 首步创建 benchmark
- prefers-reduced-motion 应纳入动画系统 → 已加入 A1

---

## Work Objectives

### Core Objective
在不破坏现有稳定功能的前提下，通过并发安全修复和性能优化加固代码质量，新增全文搜索和知识库导入两个核心功能，并打磨 UI 过渡动效体验。

### Concrete Deliverables
- `go-server/internal/ws/hub.go` — Hub 并发修复（RLock→Lock 升级消除）
- `frontend-workspace/apps/admin-tauri/src-tauri/src/ws_client.rs:11` — AUTH 超时 5s→3s
- 4 处 Vue catch(err: any) → catch(err: unknown) 修复
- `frontend-workspace/apps/web-reader/src/composables/useGraph.ts` — shallowRef 优化
- `go-server/internal/handlers/search.go` — 全文搜索端点
- `go-server/internal/services/search.go` — tsvector 查询服务
- `frontend-workspace/apps/web-reader/src/components/SearchBar.vue` — 搜索 UI
- `frontend-workspace/apps/admin-tauri/src-tauri/src/importer/` — 导入模块
- `frontend-workspace/apps/admin-tauri/src/components/ImportPanel.vue` — 导入 UI
- 双端 `composables/useTransitions.ts` — 动画系统

### Definition of Done
- [ ] `go test -race ./internal/ws/...` 零 data race
- [ ] `go test -bench=. ./internal/ws/...` 性能不低于修复前
- [ ] `go test ./internal/handlers/... ./internal/services/...` 全部通过
- [ ] `vue-tsc --noEmit` 两应用零错误
- [ ] 全文搜索返回结果 < 200ms（1000 卡片数据集）
- [ ] 导入 100 个 Markdown 文件 < 10s
- [ ] 所有动画遵守 prefers-reduced-motion
- [ ] 每个任务 RED→GREEN→REFACTOR 完成

### Must Have
- Hub 并发安全（零 data race）
- AUTH 超时全局一致（3s）
- tsvector GIN 索引 + generated column
- 搜索端点支持标题 + 内容全文搜索
- 导入支持 MD（frontmatter）+ ZIP
- shallowRef 大图谱优化
- TDD 模式（测试先行）

### Must NOT Have (Guardrails)
- ❌ 不修改 Rust Workspace 10 个 crate（全部生产就绪，不碰）
- ❌ 不加未列出的 npm/cargo 依赖（仅 gray_matter, zip, encoding_rs）
- ❌ 不实现导入进度条 UI（仅基础 loading 状态）
- ❌ 不添加搜索筛选/分面功能（MVP 仅文本搜索）
- ❌ 不创建自定义动画库（仅 CSS transitions + Vue Transition）
- ❌ 不改动现有 API 接口签名（向后兼容）
- ❌ 不添加 i18n 框架

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (Go test + vitest + cargo test)
- **Automated tests**: TDD (RED → GREEN → REFACTOR)
- **Framework**: go test / vitest / cargo test
- **TDD Flow**: 每个 TODO 先写失败测试 → 最小实现通过 → 重构

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — 性能基准 + 并发修复 + 搜索索引):
├── P1: Hub 并发修复 + benchmark [deep]
├── P2: WS AUTH 超时对齐 (ws_client.rs) [quick]
├── P3: catch(any)→unknown 类型安全 [quick]
├── P4: shallowRef 大图谱优化 + benchmark [unspecified-high]
├── S1: PostgreSQL tsvector 索引 + 搜索服务 [deep]
├── I1: 导入解析器 (gray_matter + zip) [deep]
└── A1: UI 过渡动效系统 [visual-engineering]

Wave 2 (Integration — 搜索前端 + 导入管道 + 动效应用):
├── S2: Go 搜索端点 + Web Reader 搜索 UI [unspecified-high]
├── I2: Admin Tauri 导入命令 + 前端面板 [unspecified-high]
└── A2: 双端动效批量应用 [visual-engineering]

Wave 3 (Integration — 端到端):
├── S3: 搜索端到端集成测试 [unspecified-high]
├── I3: 导入端到端集成测试 [unspecified-high]
└── A3: 动效 + prefers-reduced-motion 验证 [quick]

Wave FINAL (After ALL tasks — 4 parallel reviews):
├── F1: Plan compliance audit (oracle)
├── F2: Code quality review (unspecified-high)
├── F3: Real manual QA (unspecified-high + playwright)
└── F4: Scope fidelity check (deep)
→ Present results → Get explicit user okay

Critical Path: P1 → P4 → S2 → S3 → F1-F4
Parallel Speedup: ~65% faster than sequential
Max Concurrent: 7 (Wave 1)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|------------|--------|------|
| P1 | — | S3 | 1 |
| P2 | — | S3 | 1 |
| P3 | — | — | 1 |
| P4 | — | A2 | 1 |
| S1 | — | S2, S3 | 1 |
| I1 | — | I2, I3 | 1 |
| A1 | P4 | A2, A3 | 1 |
| S2 | S1 | S3 | 2 |
| I2 | I1 | I3 | 2 |
| A2 | A1 | A3 | 2 |
| S3 | S2, P1, P2 | — | 3 |
| I3 | I2 | — | 3 |
| A3 | A2 | — | 3 |

### Agent Dispatch Summary
- **Wave 1**: **7** — P1→`deep`, P2,P3→`quick`, P4→`unspecified-high`, S1,I1→`deep`, A1→`visual-engineering`
- **Wave 2**: **3** — S2,I2→`unspecified-high`, A2→`visual-engineering`
- **Wave 3**: **3** — S3,I3→`unspecified-high`, A3→`quick`
- **FINAL**: **4** — F1→`oracle`, F2,F3→`unspecified-high`, F4→`deep`

---

## TODOs

- [ ] P1. Fix Hub RLock→Lock Race Condition + Benchmark

  **What to do**:
  - **TDD RED**: Write `hub_test.go` with `TestHub_ConcurrentBroadcast` that triggers the RLock→Lock race under `go test -race`
  - **TDD GREEN**: Fix `hub.go:67-78` — release RLock before acquiring Lock for client removal:
    ```go
    // BEFORE (race): holding rlock while trying to acquire lock
    h.mu.RLock()
    for client := range h.clients { ... }
    h.mu.RUnlock()
    
    // AFTER (safe): collect slow clients under rlock, remove under lock
    h.mu.RLock()
    slowClients := []Client{}
    for client := range h.clients {
        if !client.trySend(msg) { slowClients = append(slowClients, client) }
    }
    h.mu.RUnlock()
    if len(slowClients) > 0 {
        h.mu.Lock()
        for _, c := range slowClients { delete(h.clients, c); close(c.send) }
        h.mu.Unlock()
    }
    ```
  - Add benchmark `BenchmarkHub_Broadcast_100Clients` to establish baseline
  - Run `go test -race -bench=. ./internal/ws/...`

  **Must NOT do**:
  - Do NOT change the Hub's channel-based event loop pattern
  - Do NOT add new dependencies
  - Do NOT modify the WebSocket protocol

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Concurrent programming, race condition debugging, benchmark creation
  - **Skills**: [`coding-standards`]
    - `coding-standards`: Go concurrency patterns, proper mutex usage

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with P2, P3, P4, S1, I1, A1)
  - **Blocks**: S3 (integration test needs stable Hub)
  - **Blocked By**: None

  **References**:
  - `go-server/internal/ws/hub.go:67-78` — The RLock→Lock upgrade location (MUST fix)
  - `go-server/internal/ws/hub.go:17` — AUTH grace period constant
  - `go-server/internal/ws/protocol.go` — Message protocol types

  **Acceptance Criteria**:
  - [ ] `go test -race ./internal/ws/...` — ZERO data races
  - [ ] `go test -bench=. ./internal/ws/...` — Performance ≥ baseline
  - [ ] TDD: test written BEFORE fix, confirmed failing, then passing

  **QA Scenarios**:

  ```
  Scenario: Verify zero data race under concurrent broadcast
    Tool: Bash (go test -race)
    Preconditions: hub_test.go with concurrent broadcast test
    Steps:
      1. Run `go test -race -v -run TestHub_ConcurrentBroadcast ./internal/ws/...`
      2. Assert exit code 0
      3. Assert no "DATA RACE" in output
    Expected Result: Test passes without any race detection
    Failure Indicators: "DATA RACE" in output, exit code 1
    Evidence: .sisyphus/evidence/task-p1-race-test.txt

  Scenario: Verify broadcast performance maintained
    Tool: Bash (go test -bench)
    Preconditions: Benchmark function created
    Steps:
      1. Run `go test -bench=BenchmarkHub_Broadcast -benchmem ./internal/ws/...`
      2. Capture output showing ns/op and allocs/op
    Expected Result: Benchmark completes, performance not degraded
    Failure Indicators: Benchmark fails, significantly slower than before
    Evidence: .sisyphus/evidence/task-p1-benchmark.txt
  ```

  **Commit**: YES
  - Message: `fix(ws): resolve Hub RLock→Lock upgrade race condition`
  - Files: `go-server/internal/ws/hub.go`, `go-server/internal/ws/hub_test.go`
  - Pre-commit: `go test -race ./internal/ws/...`

- [ ] P2. Align WS AUTH Timeout (Rust 5s → 3s)

  **What to do**:
  - **TDD RED**: Write test confirming AUTH_TIMEOUT_SECS is 3 (not 5)
  - **TDD GREEN**: Change `ws_client.rs:11` `const AUTH_TIMEOUT_SECS: u64 = 5` → `3`
  - Verify Go backend hub.go:17 uses `3 * time.Second`
  - Verify Vue frontend useGraphSync.ts uses `3000` ms (already correct per Metis)

  **Must NOT do**:
  - Do NOT change Go backend timeout (already 3s)
  - Do NOT change Vue frontend timeout (already 3s)
  - Do NOT change the AUTH protocol itself

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Single constant change with test
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: S3
  - **Blocked By**: None

  **References**:
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/ws_client.rs:11` — `AUTH_TIMEOUT_SECS = 5` (MUST change to 3)
  - `go-server/internal/ws/hub.go:17` — `authGrace = 3 * time.Second` (reference, do NOT change)
  - `frontend-workspace/apps/web-reader/src/composables/useGraphSync.ts:98` — `3000` ms (reference, do NOT change)

  **Acceptance Criteria**:
  - [ ] ws_client.rs AUTH_TIMEOUT_SECS = 3
  - [ ] Rust test confirms timeout value is 3
  - [ ] Go backend timeout remains 3s
  - [ ] Vue frontend timeout remains 3000ms

  **QA Scenarios**:

  ```
  Scenario: Verify AUTH timeout alignment
    Tool: Bash (cargo test)
    Preconditions: ws_client.rs updated
    Steps:
      1. Run `cargo test --manifest-path frontend-workspace/apps/admin-tauri/src-tauri/Cargo.toml auth_timeout`
      2. Assert test passes confirming AUTH_TIMEOUT_SECS == 3
    Expected Result: Test passes, timeout value confirmed as 3 seconds
    Failure Indicators: Test fails, value still 5
    Evidence: .sisyphus/evidence/task-p2-auth-timeout.txt
  ```

  **Commit**: YES
  - Message: `fix(ws): align AUTH timeout to 3s in Rust client`
  - Files: `frontend-workspace/apps/admin-tauri/src-tauri/src/ws_client.rs`
  - Pre-commit: `cargo test auth_timeout`

- [ ] P3. Fix catch(err: any) → catch(err: unknown)

  **What to do**:
  - **TDD RED**: Not applicable (no test for catch types; verified by tsc)
  - Find all 4 instances of `catch (err: any)` in Vue code
  - Replace with `catch (err: unknown)` + proper type narrowing:
    ```typescript
    // BEFORE
    catch (err: any) { console.error(err.message) }
    // AFTER
    catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err)
      console.error(message)
    }
    ```
  - Run `vue-tsc --noEmit` to verify type safety

  **Must NOT do**:
  - Do NOT add eslint rules (out of scope)
  - Do NOT refactor error handling beyond the catch clause
  - Do NOT silence any errors

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Mechanical type fix, well-defined pattern
  - **Skills**: [`coding-standards`]
    - `coding-standards`: TypeScript type narrowing best practices

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: None
  - **Blocked By**: None

  **References**:
  - Code review v34 report identified 4 instances (search for `catch (err: any)` or `catch(err: any)`)
  - `frontend-workspace/apps/web-reader/src/composables/useGraphSync.ts` — Likely location
  - `frontend-workspace/apps/web-reader/src/api/index.ts` — Likely location
  - `frontend-workspace/apps/admin-tauri/src/composables/useAuth.ts` — Likely location

  **Acceptance Criteria**:
  - [ ] Zero instances of `catch (err: any)` or `catch(err: any)` in codebase
  - [ ] `vue-tsc --noEmit` passes for both apps
  - [ ] All error messages still display correctly

  **QA Scenarios**:

  ```
  Scenario: Verify no catch-any remains
    Tool: Bash (grep)
    Preconditions: Changes applied
    Steps:
      1. Run grep -r "catch.*err.*any" frontend-workspace/apps/ --include="*.ts" --include="*.vue"
      2. Assert zero matches
    Expected Result: No catch(err: any) found
    Failure Indicators: Any matches returned
    Evidence: .sisyphus/evidence/task-p3-no-catch-any.txt

  Scenario: Verify TypeScript compilation
    Tool: Bash (vue-tsc)
    Preconditions: Changes applied
    Steps:
      1. Run `vue-tsc --noEmit` in both apps
      2. Assert zero errors
    Expected Result: Clean compilation
    Failure Indicators: Type errors
    Evidence: .sisyphus/evidence/task-p3-tsc.txt
  ```

  **Commit**: YES
  - Message: `fix(ts): replace catch(err: any) with unknown type narrowing`
  - Files: 4 Vue/TS files
  - Pre-commit: `vue-tsc --noEmit`

- [ ] P4. shallowRef Large Graph Optimization + Benchmark

  **What to do**:
  - **TDD RED**: Create `useGraph.bench.ts` benchmark simulating 1000 nodes + render
  - **TDD GREEN**: Convert `useGraph.ts:33-34` from `ref<Node[]>` / `ref<Edge[]>` to `shallowRef<Node[]>` / `shallowRef<Edge[]>`
  - Follow existing pattern from `CodemirrorEditor.vue:27`
  - Update all mutation sites to use `.value = newArray` (trigger replacement) instead of `.push()` (won't trigger shallowRef)
  - Also update `GraphView.vue:59-60` local copies to shallowRef
  - Verify WebSocket sync updates still trigger re-renders correctly

  **Must NOT do**:
  - Do NOT change the Vue Flow component props interface
  - Do NOT modify node/edge data structures
  - Do NOT optimize unrelated composables

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Performance optimization requires careful testing of all mutation paths
  - **Skills**: [`coding-standards`]
    - `coding-standards`: Vue reactivity system, shallowRef patterns

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: A2 (animation needs stable graph rendering)
  - **Blocked By**: None

  **References**:
  - `frontend-workspace/apps/web-reader/src/composables/useGraph.ts:33-34` — Current `ref<Node[]>` and `ref<Edge[]>` (MUST change to shallowRef)
  - `frontend-workspace/apps/web-reader/src/views/GraphView.vue:59-60` — Local node/edge copies (also change)
  - `frontend-workspace/apps/admin-tauri/src/components/CodemirrorEditor.vue:27` — Existing shallowRef pattern to follow
  - `frontend-workspace/apps/web-reader/src/composables/useGraphSync.ts` — WebSocket sync that mutates nodes/edges

  **Acceptance Criteria**:
  - [ ] useGraph.ts uses shallowRef for nodes and edges
  - [ ] All mutation sites use `.value =` assignment (not `.push()`)
  - [ ] WebSocket sync still triggers UI updates
  - [ ] Vue Flow renders correctly with 1000+ nodes
  - [ ] No regression in existing functionality

  **QA Scenarios**:

  ```
  Scenario: Verify shallowRef triggers on WebSocket update
    Tool: Playwright
    Preconditions: Web Reader running, graph loaded
    Steps:
      1. Load graph with 10+ nodes
      2. Trigger WebSocket EDGE_CREATED event via backend
      3. Assert new edge appears in graph within 2s
      4. Assert node count updated
    Expected Result: UI updates correctly despite shallowRef
    Failure Indicators: Stale UI, missing edges/nodes
    Evidence: .sisyphus/evidence/task-p4-ws-update.png

  Scenario: Verify no regression in node interaction
    Tool: Playwright
    Preconditions: Graph loaded
    Steps:
      1. Click on a node
      2. Assert detail drawer opens with correct content
      3. Assert spotlight mode activates
    Expected Result: All interactions work as before
    Failure Indicators: Click handler fails, drawer empty
    Evidence: .sisyphus/evidence/task-p4-interaction.png
  ```

  **Commit**: YES
  - Message: `perf(graph): use shallowRef for large graph performance`
  - Files: `useGraph.ts`, `GraphView.vue`
  - Pre-commit: `vue-tsc --noEmit`

- [ ] S1. PostgreSQL tsvector Full-Text Index + Search Service

  **What to do**:
  - **TDD RED**: Write `search_test.go` with test cases for: basic search, multi-word search, CJK search, empty result, special characters
  - **TDD GREEN — Database Migration**:
    ```sql
    ALTER TABLE cards ADD COLUMN search_vector tsvector
      GENERATED ALWAYS AS (
        setweight(to_tsvector('simple', coalesce(title, '')), 'A') ||
        setweight(to_tsvector('simple', coalesce(raw_md, '')), 'B')
      ) STORED;
    CREATE INDEX idx_cards_search ON cards USING GIN(search_vector);
    ```
    Note: Using `simple` config because project has Chinese content
  - **TDD GREEN — Service Layer**:
    - Create `go-server/internal/services/search.go`
    - Implement `SearchCards(query string, limit, offset int) ([]Card, int, error)`
    - Use `plainto_tsquery('simple', $1)` for user input safety
    - Use `ts_rank(search_vector, query)` for relevance ordering
    - Return cards with title, excerpt, and rank score
  - **TDD GREEN — Handler Layer**:
    - Create `go-server/internal/handlers/search.go`
    - `GET /api/v1/search?q=keyword&limit=20&offset=0` — public (no auth required)
    - Response: `{ results: [{id, title, excerpt, rank}], total, query }`

  **Must NOT do**:
  - Do NOT use `english` tsvector config (project has Chinese content, use `simple`)
  - Do NOT add search filters/facets (MVP only)
  - Do NOT modify existing card listing endpoints
  - Do NOT add Elasticsearch or external search engine

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Database schema change, new service layer, SQL query optimization
  - **Skills**: [`coding-standards`]
    - `coding-standards`: Go service patterns, SQL best practices

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: S2, S3
  - **Blocked By**: None

  **References**:
  - `go-server/internal/models/schema.go` — Card model with title, raw_md fields
  - `go-server/internal/services/card.go` — Existing service pattern (ListCards, GetCardByID)
  - `go-server/internal/handlers/card.go` — Existing handler pattern
  - `go-server/cmd/api/main.go` — Router registration location
  - `go-server/internal/storage/pg.go` — Database connection

  **Acceptance Criteria**:
  - [ ] `search_vector` column exists on cards table
  - [ ] GIN index `idx_cards_search` created
  - [ ] `GET /api/v1/search?q=test` returns matching cards
  - [ ] Chinese keyword search works (`q=知识`)
  - [ ] Empty query returns 400 with error message
  - [ ] Results ranked by relevance (ts_rank)
  - [ ] `go test ./internal/services/... ./internal/handlers/...` passes

  **QA Scenarios**:

  ```
  Scenario: Verify basic search returns results
    Tool: Bash (curl)
    Preconditions: Go server running with test data
    Steps:
      1. curl -s "http://localhost:8080/api/v1/search?q=test&limit=10" | jq .
      2. Assert HTTP 200
      3. Assert response has "results" array
      4. Assert each result has id, title, excerpt, rank
    Expected Result: Search results with ranked cards
    Failure Indicators: 404, empty results, missing fields
    Evidence: .sisyphus/evidence/task-s1-basic-search.txt

  Scenario: Verify CJK search works
    Tool: Bash (curl)
    Preconditions: Cards with Chinese titles exist
    Steps:
      1. curl -s "http://localhost:8080/api/v1/search?q=知识" | jq .
      2. Assert results contain Chinese-matching cards
    Expected Result: Chinese keywords match correctly
    Failure Indicators: Zero results for existing Chinese content
    Evidence: .sisyphus/evidence/task-s1-cjk-search.txt

  Scenario: Verify empty query rejected
    Tool: Bash (curl)
    Preconditions: Go server running
    Steps:
      1. curl -s "http://localhost:8080/api/v1/search?q=" | jq .
      2. Assert HTTP 400
    Expected Result: 400 Bad Request
    Failure Indicators: 200 OK with all results
    Evidence: .sisyphus/evidence/task-s1-empty-query.txt
  ```

  **Commit**: YES
  - Message: `feat(search): add PostgreSQL tsvector full-text index and search service`
  - Files: `go-server/internal/services/search.go`, `go-server/internal/handlers/search.go`, `go-server/internal/services/search_test.go`
  - Pre-commit: `go test ./internal/services/... ./internal/handlers/...`

- [ ] I1. Markdown/ZIP Import Parser (Rust)

  **What to do**:
  - Add dependencies to admin-tauri Cargo.toml: `gray_matter = "0.3"`, `zip = "0.6"`, `encoding_rs = "0.8"`
  - **TDD RED**: Write tests for: valid MD with frontmatter, MD without frontmatter, ZIP with multiple MDs, ZIP with images, UTF-8 BOM handling, malformed frontmatter, binary file skipping
  - **TDD GREEN**: Create `src-tauri/src/importer/` module:
    - `mod.rs` — Public API: `ImportResult`, `ImportError`
    - `markdown.rs` — `parse_markdown_file(path: &Path) -> Result<ImportCard>`:
      - Use gray_matter to extract YAML frontmatter (title, category, tags)
      - Fall back to first `# heading` as title if no frontmatter
      - Handle UTF-8 BOM via encoding_rs
      - Body = remaining content after frontmatter
    - `zip_reader.rs` — `extract_zip_archive(path: &Path) -> Result<Vec<ImportCard>>`:
      - Iterate ZIP entries, skip binary files (non-.md, non-.png/.jpg)
      - Parse each .md entry via markdown parser
      - Collect images as `Vec<ImportImage>` for separate upload
    - `detect_duplicate.rs` — `check_existing_titles(titles: &[String], existing: &[String]) -> Vec<DuplicateReport>`
  - Register Tauri commands: `import_markdown_files`, `import_zip_archive`, `check_import_duplicates`

  **Must NOT do**:
  - Do NOT upload images during import (that's a separate step)
  - Do NOT create cards via Go API in Rust (frontend orchestrates)
  - Do NOT implement progress tracking UI
  - Do NOT add dependencies beyond gray_matter, zip, encoding_rs

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: File format parsing, encoding handling, error taxonomy
  - **Skills**: [`coding-standards`]
    - `coding-standards`: Rust error handling patterns, module organization

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: I2, I3
  - **Blocked By**: None

  **References**:
  - `frontend-workspace/apps/admin-tauri/src-tauri/Cargo.toml` — Add dependencies here
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/lib.rs` — Register commands here
  - `rust-workspace/md-parser/src/lib.rs` — Existing markdown parsing (reference, don't modify)
  - `go-server/internal/services/card.go:43-75` — `FindOrCreateByTitle()` for duplicate detection reference
  - `rust-workspace/ms-kb-exporter/src/lib.rs` — KB export (reverse operation, reference for format)

  **Acceptance Criteria**:
  - [ ] `parse_markdown_file()` extracts frontmatter (title, category) + body
  - [ ] Falls back to `# heading` when no frontmatter
  - [ ] Handles UTF-8 BOM correctly
  - [ ] `extract_zip_archive()` parses multiple .md entries
  - [ ] Binary files in ZIP skipped silently
  - [ ] Malformed frontmatter: best-effort parse with defaults
  - [ ] `check_import_duplicates()` returns duplicate report
  - [ ] Tauri commands registered and callable
  - [ ] `cargo test importer` passes

  **QA Scenarios**:

  ```
  Scenario: Verify MD with frontmatter parsing
    Tool: Bash (cargo test)
    Preconditions: Test MD file with YAML frontmatter
    Steps:
      1. Run test with MD containing "---\ntitle: Test\ncategory: Notes\n---\nContent"
      2. Assert title="Test", category="Notes", body="Content"
    Expected Result: Frontmatter fields extracted correctly
    Failure Indicators: Missing fields, body includes frontmatter
    Evidence: .sisyphus/evidence/task-i1-frontmatter.txt

  Scenario: Verify ZIP with mixed content
    Tool: Bash (cargo test)
    Preconditions: Test ZIP with 3 .md files + 1 .png + 1 binary
    Steps:
      1. Run extract_zip_archive on test ZIP
      2. Assert 3 ImportCards returned
      3. Assert .png collected as ImportImage
      4. Assert binary file skipped
    Expected Result: 3 cards + 1 image, binary skipped
    Failure Indicators: Wrong count, panic on binary
    Evidence: .sisyphus/evidence/task-i1-zip-mixed.txt

  Scenario: Verify UTF-8 BOM handling
    Tool: Bash (cargo test)
    Preconditions: MD file with BOM prefix (\xEF\xBB\xBF)
    Steps:
      1. Parse file with BOM
      2. Assert content parsed without garbled characters
    Expected Result: Clean content, no BOM artifacts
    Failure Indicators: Garbled text, empty content
    Evidence: .sisyphus/evidence/task-i1-bom.txt
  ```

  **Commit**: YES
  - Message: `feat(import): add Markdown/ZIP import parser with frontmatter support`
  - Files: `src-tauri/src/importer/*.rs`, `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`
  - Pre-commit: `cargo test importer`

- [ ] A1. UI Transition Animation System

  **What to do**:
  - **TDD RED**: Write vitest test verifying animation utility respects prefers-reduced-motion
  - **TDD GREEN**: Create shared animation composable:
    - `frontend-workspace/packages/ui-shared/composables/useTransitions.ts`:
      ```typescript
      export function useTransitions() {
        const prefersReducedMotion = useMediaQuery('(prefers-reduced-motion: reduce)')
        const duration = computed(() => prefersReducedMotion.value ? '0ms' : '250ms')
        const drawerDuration = computed(() => prefersReducedMotion.value ? '0ms' : '300ms')
        
        return {
          // CSS transition strings
          fade: `opacity ${duration.value} ease`,
          slideRight: `transform ${drawerDuration.value} cubic-bezier(0.16, 1, 0.3, 1)`,
          slideUp: `transform ${duration.value} ease-out`,
          scale: `transform ${duration.value} ease, opacity ${duration.value} ease`,
          
          // Vue Transition name mapping
          transitionName: computed(() => prefersReducedMotion.value ? 'none' : 'default'),
          
          // Design tokens per ARCHITECTURE.md
          maxDuration: 250,  // ms — all transitions ≤ 250ms
          drawerDuration: 300, // ms — drawer slide
        }
      }
      ```
  - Create CSS transition classes in `ui-shared/styles/transitions.css`:
    - `.ms-fade-enter-active/leave-active` — opacity fade
    - `.ms-slide-right-enter/leave` — drawer slide from right
    - `.ms-slide-up-enter/leave` — panel slide up
    - `.ms-scale-enter/leave` — modal scale in/out
  - Design spec: all ≤ 250ms, drawers 300ms Expo-Out, sharp corners maintained

  **Must NOT do**:
  - Do NOT create custom animation library (CSS transitions + Vue Transition only)
  - Do NOT add anime.js/gsap or any animation dependency
  - Do NOT change graph animation (user chose UI transitions only)
  - Do NOT break industrial console aesthetic (sharp corners, mono font)

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Animation system design, CSS transitions, design system integration
  - **Skills**: [`frontend-design`, `coding-standards`]
    - `frontend-design`: Transition timing, easing curves, design system compliance
    - `coding-standards`: Vue composable patterns, TypeScript

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: A2, A3
  - **Blocked By**: None

  **References**:
  - `frontend-workspace/packages/ui-shared/` — Shared package location
  - `frontend-workspace/apps/admin-tauri/src/components/CodemirrorEditor.vue:27` — Existing composable pattern
  - ARCHITECTURE.md design tokens: "所有交互过渡 ≤ 250ms · 抽屉滑入 300ms Expo-Out"
  - `frontend-workspace/apps/web-reader/src/components/DetailDrawer.vue` — Drawer to animate
  - `frontend-workspace/apps/admin-tauri/src/components/TheForge.vue` — Panel to animate

  **Acceptance Criteria**:
  - [ ] `useTransitions()` composable created in ui-shared
  - [ ] CSS transition classes defined
  - [ ] `prefers-reduced-motion: reduce` → all durations become `0ms`
  - [ ] Transitions ≤ 250ms (drawers 300ms)
  - [ ] vitest test for reduced-motion behavior passes
  - [ ] No animation dependencies added

  **QA Scenarios**:

  ```
  Scenario: Verify prefers-reduced-motion support
    Tool: Bash (vitest)
    Preconditions: useTransitions composable created
    Steps:
      1. Run vitest test that sets prefers-reduced-motion: reduce
      2. Assert all durations resolve to '0ms'
    Expected Result: Durations are '0ms' when reduced motion preferred
    Failure Indicators: Non-zero durations when reduced motion active
    Evidence: .sisyphus/evidence/task-a1-reduced-motion.txt

  Scenario: Verify transition timing compliance
    Tool: Bash (vitest)
    Preconditions: Transition classes defined
    Steps:
      1. Assert fade duration ≤ 250ms
      2. Assert drawer duration ≤ 300ms
      3. Assert scale duration ≤ 250ms
    Expected Result: All within design spec
    Failure Indicators: Any duration exceeds spec
    Evidence: .sisyphus/evidence/task-a1-timing.txt
  ```

  **Commit**: YES
  - Message: `feat(ux): add UI transition animation system with reduced-motion support`
  - Files: `packages/ui-shared/composables/useTransitions.ts`, `packages/ui-shared/styles/transitions.css`
  - Pre-commit: `vitest run`

- [ ] S2. Go Search Endpoint + Web Reader Search UI

  **What to do**:
  - **TDD RED**: Write handler test for `GET /api/v1/search` edge cases
  - **TDD GREEN — Route Registration**:
    - Add `r.GET("/search", h.SearchCards)` in `cmd/api/main.go`
  - **TDD GREEN — Web Reader Frontend**:
    - Create `web-reader/src/components/SearchBar.vue`:
      - Fixed position search input in LeftDock header
      - Debounced input (300ms) → calls `GET /api/v1/search?q=...`
      - Results dropdown with title + excerpt preview
      - Click result → navigate to card in graph (select node + open drawer)
      - Escape key closes search
      - Cmd+K opens search (integrate with existing CommandPalette)
    - Create `web-reader/src/api/schemas.ts` — Zod schema for search response
    - Update `web-reader/src/composables/useCards.ts` — add `searchCards(query)` action
    - Update `web-reader/src/store/useGraphStore.ts` — add `searchMode` state
  - Industrial aesthetic: mono font input, sharp corners, neon accent on focus

  **Must NOT do**:
  - Do NOT add search filters/facets (MVP text search only)
  - Do NOT add search history or suggestions
  - Do NOT change the existing CommandPalette component (integrate alongside)
  - Do NOT add pagination beyond limit/offset params

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Cross-stack work (Go route + Vue UI + API integration), multiple files
  - **Skills**: [`coding-standards`]
    - `coding-standards`: REST API design, Vue composable patterns

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on S1)
  - **Parallel Group**: Wave 2
  - **Blocks**: S3
  - **Blocked By**: S1 (needs search service)

  **References**:
  - `go-server/cmd/api/main.go` — Route registration location
  - `go-server/internal/services/search.go` — Search service (created in S1)
  - `frontend-workspace/apps/web-reader/src/api/index.ts` — Axios HTTP client
  - `frontend-workspace/apps/web-reader/src/api/schemas.ts` — Existing Zod schemas
  - `frontend-workspace/apps/web-reader/src/components/LeftDock.vue` — Search bar placement
  - `frontend-workspace/apps/web-reader/src/components/CommandPalette.vue` — Cmd+K integration point
  - `frontend-workspace/apps/web-reader/src/store/useGraphStore.ts` — Graph state management

  **Acceptance Criteria**:
  - [ ] `GET /api/v1/search?q=keyword` returns results
  - [ ] SearchBar renders in LeftDock header
  - [ ] Debounced input triggers search after 300ms
  - [ ] Results show title + excerpt
  - [ ] Click result selects node in graph + opens drawer
  - [ ] Escape closes search
  - [ ] Sharp corners, mono font, neon focus accent

  **QA Scenarios**:

  ```
  Scenario: Verify search bar renders and searches
    Tool: Playwright
    Preconditions: Web Reader running with test data
    Steps:
      1. Navigate to web reader
      2. Click search input in LeftDock
      3. Type "test" and wait 400ms (debounce)
      4. Assert results dropdown appears
      5. Assert results contain matching cards
    Expected Result: Search results appear after debounce
    Failure Indicators: No dropdown, no results, immediate search
    Evidence: .sisyphus/evidence/task-s2-search-ui.png

  Scenario: Verify result click navigates to node
    Tool: Playwright
    Preconditions: Search results visible
    Steps:
      1. Click first search result
      2. Assert graph node is selected (highlighted)
      3. Assert detail drawer opens with correct card content
    Expected Result: Graph navigates to selected card
    Failure Indicators: No navigation, wrong card, drawer empty
    Evidence: .sisyphus/evidence/task-s2-result-nav.png
  ```

  **Commit**: YES
  - Message: `feat(search): add search endpoint route and Web Reader search UI`
  - Files: `cmd/api/main.go`, `SearchBar.vue`, `useCards.ts`, `useGraphStore.ts`, `schemas.ts`
  - Pre-commit: `go test ./internal/handlers/... && vue-tsc --noEmit`

- [ ] I2. Admin Tauri Import Command + Frontend Panel

  **What to do**:
  - **TDD RED**: Write vitest test for import flow (file selection → parse → create)
  - **TDD GREEN — Tauri Commands**:
    - `import_markdown_files(paths: Vec<String>) -> Vec<ImportCard>` — Parse selected MD files
    - `import_zip_archive(path: String) -> Vec<ImportCard>` — Parse ZIP archive
    - `check_import_duplicates(titles: Vec<String>) -> Vec<DuplicateReport>` — Check existing
  - **TDD GREEN — Frontend**:
    - Create `admin-tauri/src/components/ImportPanel.vue`:
      - "Import Markdown" button → tauri-plugin-dialog file picker (multi-select .md)
      - "Import ZIP" button → tauri-plugin-dialog file picker (single .zip)
      - Preview table: filename, detected title, detected category, status (new/duplicate)
      - Duplicate rows highlighted orange with "Skip" toggle
      - "Import N cards" button → batch POST /api/v1/cards via knowledge store
      - Loading state during import
      - Success toast: "Imported N cards, skipped D duplicates"
    - Add import button to LeftSidebar (below existing actions)
    - Import panel as overlay (matching Settings panel pattern)

  **Must NOT do**:
  - Do NOT implement progress bar (basic loading spinner only)
  - Do NOT handle image upload in this task (follow-up)
  - Do NOT modify the Rust importer module (created in I1)
  - Do NOT change existing card creation logic

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: IPC integration, UI component, batch API orchestration
  - **Skills**: [`frontend-design`, `coding-standards`]
    - `frontend-design`: Industrial panel aesthetic matching Settings.vue
    - `coding-standards`: Tauri IPC patterns, Vue async flows

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on I1)
  - **Parallel Group**: Wave 2
  - **Blocks**: I3
  - **Blocked By**: I1 (needs import parser)

  **References**:
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/importer/` — Parser module (created in I1)
  - `frontend-workspace/apps/admin-tauri/src/views/Settings.vue` — Overlay panel pattern to follow
  - `frontend-workspace/apps/admin-tauri/src/components/LeftSidebar.vue` — Import button placement
  - `frontend-workspace/apps/admin-tauri/src/stores/knowledge.ts` — Card creation actions
  - `frontend-workspace/apps/admin-tauri/src/stores/useToast.ts` — Toast notifications
  - Tauri plugin-dialog docs: https://v2.tauri.app/plugin/dialog/

  **Acceptance Criteria**:
  - [ ] File picker opens for .md files (multi-select)
  - [ ] File picker opens for .zip files (single select)
  - [ ] Preview table shows parsed cards with title/category
  - [ ] Duplicates highlighted with skip toggle
  - [ ] "Import N cards" creates cards via Go API
  - [ ] Success toast shows import count
  - [ ] Loading state during import
  - [ ] Sharp corners, mono font, neon accents

  **QA Scenarios**:

  ```
  Scenario: Verify MD import flow
    Tool: Playwright
    Preconditions: Admin Tauri running, test MD files available
    Steps:
      1. Click "Import" button in sidebar
      2. Select 3 .md files in file dialog
      3. Assert preview table shows 3 rows
      4. Assert titles detected correctly
      5. Click "Import 3 cards"
      6. Assert success toast: "Imported 3 cards"
      7. Assert cards appear in sidebar card list
    Expected Result: 3 cards imported and visible
    Failure Indicators: Toast error, missing cards, empty preview
    Evidence: .sisyphus/evidence/task-i2-md-import.png

  Scenario: Verify duplicate detection
    Tool: Playwright
    Preconditions: Card "Test" already exists
    Steps:
      1. Import MD file with title "Test"
      2. Assert preview shows orange "Duplicate" badge
      3. Assert "Skip" toggle is ON by default
      4. Click "Import" 
      5. Assert existing card NOT overwritten
      6. Assert toast: "Imported 0 cards, skipped 1 duplicates"
    Expected Result: Duplicate detected and skipped
    Failure Indicators: Card overwritten, no duplicate warning
    Evidence: .sisyphus/evidence/task-i2-duplicates.png
  ```

  **Commit**: YES
  - Message: `feat(import): add Tauri import commands and import panel UI`
  - Files: `ImportPanel.vue`, `lib.rs` (command registration), `LeftSidebar.vue`
  - Pre-commit: `vue-tsc --noEmit`

- [ ] A2. Apply Transitions to Both Apps

  **What to do**:
  - Import `useTransitions` from ui-shared in both apps
  - **Web Reader transitions**:
    - `DetailDrawer.vue` — slide-right enter/leave
    - `LeftDock.vue` — fade in/out for panel content
    - `ZenReader.vue` — scale + fade enter/leave
    - `CommandPalette.vue` — fade + scale enter/leave
    - `BacklinksPanel.vue` — slide-up enter/leave
    - `StatsWidget.vue` — fade in data load
    - `EntranceAnimation.vue` — integrate with existing animation
  - **Admin Tauri transitions**:
    - `LeftSidebar.vue` — fade card list items
    - `RightAstrolabe.vue` — slide-right enter/leave (v-show toggle)
    - `CategoryPanel.vue` — slide-right overlay
    - `Settings.vue` — fade overlay enter/leave
    - `MergePanel.vue` — slide-up panel
    - `ConfirmDialog.vue` — scale + fade modal
  - Wrap each target with `<Transition :name="transitionName">` using composable

  **Must NOT do**:
  - Do NOT add graph node animations (user chose UI transitions only)
  - Do NOT add page transition effects (no vue-router)
  - Do NOT change transition timing beyond 250ms/300ms
  - Do NOT add new components

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Applying animation system across many components, maintaining design consistency
  - **Skills**: [`frontend-design`, `coding-standards`]
    - `frontend-design`: Consistent easing, timing, visual polish
    - `coding-standards`: Vue Transition component API

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on A1)
  - **Parallel Group**: Wave 2
  - **Blocks**: A3
  - **Blocked By**: A1 (needs transition system)

  **References**:
  - `frontend-workspace/packages/ui-shared/composables/useTransitions.ts` — Animation system (created in A1)
  - All Vue components listed above in both apps
  - ARCHITECTURE.md: "所有交互过渡 ≤ 250ms · 抽屉滑入 300ms Expo-Out · 图谱归位 800ms"

  **Acceptance Criteria**:
  - [ ] All listed components use `<Transition>` with composable
  - [ ] DetailDrawer slides in from right (300ms)
  - [ ] ZenReader scales + fades (250ms)
  - [ ] ConfirmDialog scales in (250ms)
  - [ ] All transitions smooth at 60fps
  - [ ] Industrial aesthetic preserved (sharp corners maintained)

  **QA Scenarios**:

  ```
  Scenario: Verify drawer transition
    Tool: Playwright
    Preconditions: Web Reader running
    Steps:
      1. Click a graph node
      2. Assert DetailDrawer slides in from right (not instant)
      3. Click close button
      4. Assert DetailDrawer slides out
    Expected Result: Smooth slide transition, not instant appear/disappear
    Failure Indicators: No animation, janky frame rate, visible clipping
    Evidence: .sisyphus/evidence/task-a2-drawer-transition.png

  Scenario: Verify modal transition
    Tool: Playwright
    Preconditions: Admin Tauri running
    Steps:
      1. Trigger ConfirmDialog (e.g., delete card)
      2. Assert dialog scales in from center
      3. Click cancel
      4. Assert dialog scales out
    Expected Result: Smooth scale transition
    Failure Indicators: Instant appear, no animation
    Evidence: .sisyphus/evidence/task-a2-modal-transition.png
  ```

  **Commit**: YES
  - Message: `feat(ux): apply transition animations to both apps`
  - Files: ~12 Vue files across both apps
  - Pre-commit: `vue-tsc --noEmit`

- [ ] S3. Search End-to-End Integration Test

  **What to do**:
  - **TDD RED → GREEN**: Write integration tests covering full search pipeline:
    - Create test cards with known titles/content via API
    - Search via `GET /api/v1/search?q=...`
    - Verify results match expected cards
    - Verify ranking order (title match > content match)
    - Verify Web Reader SearchBar displays results correctly
  - Test edge cases: special characters, very long queries, SQL injection attempts
  - Test performance: 1000-card dataset, search completes < 200ms

  **Must NOT do**:
  - Do NOT modify search implementation (test only)
  - Do NOT add load testing infrastructure

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Cross-stack integration testing, performance validation
  - **Skills**: [`coding-standards`]
    - `coding-standards`: Go testing patterns, HTTP test helpers

  **Parallelization**:
  - **Can Run In Parallel**: YES (with I3, A3)
  - **Parallel Group**: Wave 3
  - **Blocks**: None
  - **Blocked By**: S2 (needs search endpoint + UI), P1 (needs stable Hub)

  **References**:
  - `go-server/internal/services/search.go` — Service to test
  - `go-server/internal/handlers/search.go` — Handler to test
  - `frontend-workspace/apps/web-reader/src/components/SearchBar.vue` — UI to test

  **Acceptance Criteria**:
  - [ ] Integration test covers: create → search → verify → cleanup
  - [ ] Ranking test: title match ranks higher than content match
  - [ ] SQL injection attempt returns 400 (not error)
  - [ ] Performance: search on 1000 cards < 200ms
  - [ ] `go test ./internal/services/search_test.go` passes

  **QA Scenarios**:

  ```
  Scenario: Verify search ranking (title > content)
    Tool: Bash (go test + curl)
    Preconditions: Cards created with "Alpha" in title and "Alpha" in content only
    Steps:
      1. Create card A: title="Alpha Test", content="random"
      2. Create card B: title="Other", content="Alpha mentioned here"
      3. Search q="Alpha"
      4. Assert card A rank > card B rank
    Expected Result: Title match ranked first
    Failure Indicators: Content match ranked above title match
    Evidence: .sisyphus/evidence/task-s3-ranking.txt

  Scenario: Verify search performance
    Tool: Bash (go test -bench)
    Preconditions: 1000 test cards seeded
    Steps:
      1. Run benchmark search with 1000-card dataset
      2. Assert average query time < 200ms
    Expected Result: Sub-200ms search performance
    Failure Indicators: > 200ms average
    Evidence: .sisyphus/evidence/task-s3-performance.txt
  ```

  **Commit**: YES
  - Message: `test(search): add end-to-end search integration tests`
  - Files: `go-server/internal/services/search_integration_test.go`
  - Pre-commit: `go test ./internal/services/...`

- [ ] I3. Import End-to-End Integration Test

  **What to do**:
  - **TDD RED → GREEN**: Write integration tests covering full import pipeline:
    - Create test fixture files: valid MD, MD with frontmatter, ZIP with mixed content
    - Test import flow: file selection → parse → duplicate check → create cards → verify
    - Test error cases: corrupted ZIP, empty file, permission denied
    - Test batch import: 100 files < 10s
  - Verify imported cards appear in Go server via API
  - Verify WebSocket CARD_CREATED events fire for imported cards

  **Must NOT do**:
  - Do NOT modify import implementation (test only)
  - Do NOT test image upload (out of scope)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Cross-boundary integration testing (Tauri → Rust → Go API → WebSocket)
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with S3, A3)
  - **Parallel Group**: Wave 3
  - **Blocks**: None
  - **Blocked By**: I2 (needs import command + UI)

  **References**:
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/importer/` — Importer to test
  - `go-server/internal/handlers/card.go` — POST /cards endpoint (import target)
  - `go-server/internal/ws/protocol.go` — CARD_CREATED event verification

  **Acceptance Criteria**:
  - [ ] Full import pipeline test passes (MD + ZIP)
  - [ ] Imported cards queryable via GET /cards/:id
  - [ ] Duplicate detection prevents re-import
  - [ ] Batch 100 files completes < 10s
  - [ ] Error cases handled gracefully

  **QA Scenarios**:

  ```
  Scenario: Verify full MD import pipeline
    Tool: Bash (cargo test + curl)
    Preconditions: Test fixture MD files, Go server running
    Steps:
      1. Import 5 MD files via Tauri command
      2. Query GET /api/v1/cards for imported titles
      3. Assert all 5 cards exist in API response
    Expected Result: All imported cards queryable
    Failure Indicators: Missing cards, wrong content
    Evidence: .sisyphus/evidence/task-i3-full-pipeline.txt

  Scenario: Verify corrupted ZIP handling
    Tool: Bash (cargo test)
    Preconditions: Invalid ZIP file
    Steps:
      1. Attempt import with corrupted ZIP
      2. Assert error returned (not panic)
      3. Assert zero cards created
    Expected Result: Graceful error, no side effects
    Failure Indicators: Panic, partial import
    Evidence: .sisyphus/evidence/task-i3-corrupted-zip.txt
  ```

  **Commit**: YES
  - Message: `test(import): add end-to-end import integration tests`
  - Files: `import_integration_test.rs`, `fixtures/` test files
  - Pre-commit: `cargo test importer`

- [ ] A3. Verify prefers-reduced-motion Across Both Apps

  **What to do**:
  - Write Playwright test that sets `prefers-reduced-motion: reduce` in browser
  - Open all panels/drawers/modals in both apps
  - Verify all transitions are instant (duration=0ms)
  - Verify no visual glitches from disabled animations
  - Verify no performance regression from reduced-motion check

  **Must NOT do**:
  - Do NOT add new transitions
  - Do NOT modify existing transition system

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Focused verification test, clear scope
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with S3, I3)
  - **Parallel Group**: Wave 3
  - **Blocks**: None
  - **Blocked By**: A2 (needs transitions applied)

  **References**:
  - `frontend-workspace/packages/ui-shared/composables/useTransitions.ts` — Reduced-motion logic
  - All components with transitions (applied in A2)

  **Acceptance Criteria**:
  - [ ] Playwright test sets reduced-motion preference
  - [ ] All drawer/modal/panel transitions instant when reduced-motion active
  - [ ] No layout glitches without animations
  - [ ] Test passes on both apps

  **QA Scenarios**:

  ```
  Scenario: Verify reduced-motion disables all transitions
    Tool: Playwright
    Preconditions: Both apps running
    Steps:
      1. Set browser preference: prefers-reduced-motion: reduce
      2. Open DetailDrawer in Web Reader
      3. Assert drawer appears instantly (no slide animation)
      4. Open ConfirmDialog in Admin Tauri
      5. Assert dialog appears instantly (no scale animation)
    Expected Result: All transitions instant, content fully visible
    Failure Indicators: Visible animation, delayed appearance
    Evidence: .sisyphus/evidence/task-a3-reduced-motion.png
  ```

  **Commit**: YES
  - Message: `test(ux): verify prefers-reduced-motion across both apps`
  - Files: `tests/reduced-motion.spec.ts`
  - Pre-commit: `vitest run`

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, curl endpoint, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `go test -race ./...` + `go vet ./...` + `vue-tsc --noEmit` + `cargo test`. Review all changed files for: `as any`/`@ts-ignore`, empty catches, console.log in prod, commented-out code, unused imports. Check AI slop: excessive comments, over-abstraction, generic names.
  Output: `Build [PASS/FAIL] | Lint [PASS/FAIL] | Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [ ] F3. **Real Manual QA** — `unspecified-high` (+ `playwright` skill if UI)
  Start from clean state. Execute EVERY QA scenario from EVERY task — follow exact steps, capture evidence. Test cross-task integration. Test edge cases: empty search, import malformed file, disconnect during sync. Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff. Verify 1:1 — everything in spec was built, nothing beyond spec. Check "Must NOT do" compliance. Detect cross-task contamination. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

- **P1**: `fix(ws): resolve Hub RLock→Lock upgrade race condition` — hub.go, hub_test.go
- **P2**: `fix(ws): align AUTH timeout to 3s in Rust client` — ws_client.rs
- **P3**: `fix(ts): replace catch(err: any) with unknown` — 4 Vue files
- **P4**: `perf(graph): use shallowRef for large graph performance` — useGraph.ts, GraphView.vue
- **S1**: `feat(search): add PostgreSQL tsvector full-text index` — migration, search.go
- **S2**: `feat(search): add search endpoint and Web Reader UI` — handlers/search.go, SearchBar.vue
- **S3**: `test(search): end-to-end search integration tests` — search_test.go
- **I1**: `feat(import): add Markdown/ZIP import parser` — importer/*.rs
- **I2**: `feat(import): add Tauri import command and UI` — lib.rs, ImportPanel.vue
- **I3**: `test(import): end-to-end import integration tests` — import_test.go
- **A1**: `feat(ux): add UI transition animation system` — useTransitions.ts
- **A2**: `feat(ux): apply transitions to both apps` — multiple Vue files
- **A3**: `feat(ux): add prefers-reduced-motion support` — useTransitions.ts

---

## Success Criteria

### Verification Commands
```bash
make check          # Expected: all checks pass
make test           # Expected: all tests pass
go test -race ./... # Expected: zero data races
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass with TDD coverage
- [ ] No Rust workspace files modified
- [ ] WebSocket sync zero data race
- [ ] Search returns results < 200ms
- [ ] Import handles MD + ZIP
- [ ] All animations respect prefers-reduced-motion
