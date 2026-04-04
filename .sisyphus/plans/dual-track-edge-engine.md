# 双轨制连线引擎 (Dual-Track Edge Engine)

## TL;DR

> **Quick Summary**: 为知识图谱实现双轨连线机制 — Sequence Track（手动图谱拖拽）和 Reference Track（[[wikilink]] 自动解析）。横跨 Rust/Go/Vue 三层，采用 TDD 模式构建。
> 
> **Deliverables**:
> - Rust md-parser: `extract_wikilinks()` 函数 + `RenderResult` 新增 `extracted_links` 字段
> - Go backend: `SyncReferenceEdges()` 原子事务（edge diff + ghost card 创建）+ backlinks context_snippet 增强
> - Admin-tauri TheForge: CodeMirror 6 替换 textarea + [[wikilink]] 霓虹高亮 decorator
> - Admin-tauri RightAstrolabe: 禁止手动创建 reference edge 的 UI 守卫
> - Admin-tauri Radar: backlinks 面板增强上下文摘要显示
> - Web-reader: 边突变交互禁用 + backlinks 面板
> - Rust ast-renderer: [[wikilink]] → `<a class="reference-link">` HTML 渲染
> 
> **Estimated Effort**: Large
> **Parallel Execution**: YES — 5 waves
> **Critical Path**: T1(Rust提取) → T4(Go原子事务) → T9(集成测试) → FINAL

---

## Context

### Original Request
实现"双轨制连线引擎"：图谱用来搭建系统的骨架（Sequence，手动拖拽），文本+雷达用来生长系统的神经（Reference，[[wikilink]] 自动解析）。两种边存入同一张 card_edges 表，通过 relation_type 区分。

### Interview Summary
**Key Discussions**:
- 编辑器策略：**CodeMirror 6**（非 Tiptap/WYSIWYG），保留纯文本 + 正则 decorator 高亮
- 提取管道：Rust 正则提取 → 前端 payload 透传 → Go 原子事务（ghost card + edge diff）
- 雷达增强：Go 后端 ±30 chars context_snippet 提取
- Web-reader 定位：**纯消费器**，禁用所有边突变操作
- 测试策略：**TDD 模式**（RED → GREEN → REFACTOR）

**Research Findings**:
- card_edges 表已有 `relation_type CHECK IN ('sequence','reference')` — 无需改表
- Go EdgeService 已有 CreateEdge/DeleteEdge/UpdateEdgeType — 需新增 SyncReferenceEdges 批量事务方法
- 前端视觉区分已存在：sequence=青色动画实线，reference=灰色虚线
- TheForge 是纯 textarea，无富编辑器
- Backlinks 面板已有基础，返回 `{ source_id, source_title, relation_type }`
- **[[wikilink]] 解析不存在** — 需要从零实现
- RenderResult 返回 `{ html, ast_json, excerpt }` — 无 toc 字段（TOC 单独提取）
- EdgeService 无事务支持 — 需参照 CardService.DeleteCard 事务模式

### Metis Review
**Identified Gaps** (addressed):
- RenderResult 返回结构理解偏差：不含 toc，只有 html/ast_json/excerpt → 已修正
- EdgeService 无事务支持：需新建 SyncReferenceEdges 批量方法，不修改现有单行方法
- 图片粘贴处理器需要完全重写（textarea DOM API → CodeMirror state/dispatch API）
- Ghost card 无 is_ghost 字段：使用空内容 + 标题约定代替（不改 schema）
- Wikilink 语法范围：MVP 仅支持 `[[plain-card-name]]`，不支持 `[[card|display]]` 或 `[[#section]]`
- 自引用/循环引用：允许（合法用例）
- Unicode 卡片名：保留 UTF-8
- 并发编辑：Last-writer-wins（现有行为）
- 去重策略：Rust 提取时去重

---

## Work Objectives

### Core Objective
实现双轨制连线引擎的完整数据管道：[[wikilink]] 在 Rust 层提取 → Go 后端原子同步 → 前端双向展示（图谱视觉 + 雷达上下文），同时保持 Sequence Track 的纯手动特性。

### Concrete Deliverables
- `rust-workspace/md-parser/src/lib.rs` — `extract_wikilinks()` 公共函数
- `frontend-workspace/apps/admin-tauri/src-tauri/src/lib.rs` — `RenderResult` 新增 `extracted_links: Vec<String>`
- `go-server/internal/services/edge.go` — `SyncReferenceEdges()` 事务方法
- `go-server/internal/services/card.go` — `FindOrCreateByTitle()` ghost card 创建
- `go-server/internal/services/backlink.go` 或 handlers — context_snippet 增强
- `frontend-workspace/apps/admin-tauri/src/components/TheForge.vue` — CodeMirror 6 替换
- `frontend-workspace/apps/admin-tauri/src/components/RightAstrolabe.vue` — reference edge 创建守卫
- `frontend-workspace/apps/admin-tauri/src/components/TheForge.vue` (底部面板) — 雷达增强
- `frontend-workspace/apps/web-reader/src/views/GraphView.vue` — 边突变禁用
- `rust-workspace/ast-renderer/src/lib.rs` — [[wikilink]] → `<a>` 渲染

### Definition of Done
- [ ] `cargo test` 全部通过（含 extract_wikilinks 测试）
- [ ] `go test ./...` 全部通过（含 SyncReferenceEdges 测试）
- [ ] `vue-tsc --noEmit` 两应用零错误
- [ ] CodeMirror 中 `[[wikilink]]` 显示霓虹青色高亮
- [ ] PUT /cards 含 extracted_links → Go 原子同步 reference edges
- [ ] Ghost card 自动创建，图谱中可见
- [ ] Backlinks 返回 context_snippet 字段
- [ ] Web-reader 禁用所有边突变交互
- [ ] RightAstrolabe 禁止手动创建 reference edge

### Must Have
- `extract_wikilinks()` 函数：正则 `\[\[([^\]]+)\]\]`，返回去重 `Vec<String>`
- `SyncReferenceEdges()` 事务：edge diff（INSERT 新/DELETE 废）+ ghost card 创建
- CodeMirror 6 集成：替换 textarea，保留图片粘贴 + 快捷键 + 脏追踪
- [[wikilink]] 语法高亮：`.text-neon-cyan` CSS class
- Backlinks context_snippet：±30 chars，首次出现
- RightAstrolabe reference edge 创建守卫：tooltip 引导用户回文本编辑
- Web-reader 边突变禁用：隐藏拖拽手柄 + 删除按钮
- Rust ast-renderer：[[wikilink]] → `<a href="/cards/UUID" class="reference-link">` HTML
- 每层 TDD 测试（RED → GREEN → REFACTOR）

### Must NOT Have (Guardrails)
- ❌ 修改 card_edges 表结构（schema 已满足）
- ❌ 修改现有 EdgeService.CreateEdge/DeleteEdge/UpdateEdgeType 方法
- ❌ WYSIWYG 编辑器（Quill/TinyMCE 禁止）
- ❌ Wikilink 自动补全（MVP 排除）
- ❌ `[[card|display]]` 或 `[[card#section]]` 语法（MVP 仅支持 `[[plain-name]]`）
- ❌ Ghost card 添加 is_ghost 数据库字段（用空内容约定）
- ❌ 改变边视觉样式（使用现有 sequence=cyan/reference=gray）
- ❌ Web-reader 任何边突变能力
- ❌ AI Slop：多余注释、无关重构、过度抽象

---

## Verification Strategy (MANDATORY)

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: YES (Go test + Cargo test + vitest)
- **Automated tests**: TDD (RED → GREEN → REFACTOR)
- **Framework**: cargo test / go test / vitest

### QA Policy
Every task follows TDD: write failing test first → implement minimum to pass → refactor.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Rust**: `cargo test` — unit tests for extraction, rendering
- **Go**: `go test` — unit tests for edge sync, ghost card, context snippet
- **Vue**: `vitest` / `vue-tsc` — component tests, type checks
- **API**: `curl` — endpoint verification
- **UI**: Playwright — visual verification where needed

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — Rust + Go foundation, MAX PARALLEL):
├── Task 1: Rust extract_wikilinks() 函数 [deep] — TDD
├── Task 2: Rust RenderResult + process_markdown 改造 [quick] — TDD
├── Task 3: Go FindOrCreateByTitle() ghost card 服务方法 [quick] — TDD
├── Task 4: Go SyncReferenceEdges() 原子事务 [deep] — TDD

Wave 2 (After Wave 1 — Go enhancements + Rust rendering):
├── Task 5: Go backlinks context_snippet 增强 [quick] — TDD
├── Task 6: Go PUT /cards handler 集成 extracted_links [quick]
├── Task 7: Rust ast-renderer [[wikilink]] → <a> HTML 渲染 [quick] — TDD

Wave 3 (After Wave 1 — Frontend, MAX PARALLEL):
├── Task 8: Admin TheForge → CodeMirror 6 替换 + wikilink decorator [visual-engineering] — TDD
├── Task 9: Admin RightAstrolabe reference edge 创建守卫 [quick]
├── Task 10: Web-reader 边突变交互禁用 [quick]
├── Task 11: Admin TheForge save 流程集成 extracted_links [quick]

Wave 4 (After Wave 2+3 — UI enhancement):
├── Task 12: Admin Radar backlinks 面板增强 context_snippet [visual-engineering]
├── Task 13: Web-reader backlinks 面板 [visual-engineering]
├── Task 14: Rust ast-renderer wikilink 解析器增强 (resolve UUID) [deep]

Wave FINAL (After ALL tasks — 4 parallel reviews):
├── Task F1: Plan compliance audit [oracle]
├── Task F2: Code quality review [unspecified-high]
├── Task F3: Real manual QA [unspecified-high]
├── Task F4: Scope fidelity check [deep]
-> Present results -> Get explicit user okay

Critical Path: T1 → T2 → T11 → T12 → F1-F4
                T1 → T4 → T6 → T11 → F1-F4
                T8 → T11 → F1-F4
Parallel Speedup: ~65% faster than sequential
Max Concurrent: 4 (Waves 1 & 3)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| 1 | — | 2, 4, 7, 8 | 1 |
| 2 | 1 | 8, 11 | 1 |
| 3 | — | 4 | 1 |
| 4 | 1, 3 | 6 | 1 |
| 5 | 4 | 12, 13 | 2 |
| 6 | 4 | 11 | 2 |
| 7 | 1 | 14 | 2 |
| 8 | 2 | 11 | 3 |
| 9 | — | — | 3 |
| 10 | — | — | 3 |
| 11 | 2, 6, 8 | 12 | 3 |
| 12 | 5, 11 | — | 4 |
| 13 | 5 | — | 4 |
| 14 | 7 | — | 4 |
| F1-F4 | ALL | user | FINAL |

### Agent Dispatch Summary

- **Wave 1**: 4 tasks — T1 → `deep`, T2 → `quick`, T3 → `quick`, T4 → `deep`
- **Wave 2**: 3 tasks — T5 → `quick`, T6 → `quick`, T7 → `quick`
- **Wave 3**: 4 tasks — T8 → `visual-engineering`, T9 → `quick`, T10 → `quick`, T11 → `quick`
- **Wave 4**: 3 tasks — T12 → `visual-engineering`, T13 → `visual-engineering`, T14 → `deep`
- **FINAL**: 4 tasks — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [ ] 1. Rust `extract_wikilinks()` 函数 — TDD

  **What to do**:
  - **RED**: Write failing tests in `rust-workspace/md-parser/tests/wikilink_test.rs`:
    - 5 wikilinks → returns 5 unique names
    - `[[Duplicate]] [[Duplicate]]` → returns `["Duplicate"]` (deduped)
    - `[[Card With Spaces]]` → returns `"Card With Spaces"` (preserves spaces)
    - `[[Émoji 🎉]]` → returns `"Émoji 🎉"` (UTF-8 preserved)
    - Empty markdown → returns empty vec
    - `[[Outer [[Inner]]]]` → returns only outer match `"Outer [[Inner]]"`
    - No wikilinks → returns empty vec
    - Self-reference `[[Same Card]]` inside card named "Same Card" → returns `["Same Card"]` (allowed)
  - **GREEN**: Implement in `rust-workspace/md-parser/src/lib.rs`:
    - `pub fn extract_wikilinks(md: &str) -> Vec<String>`
    - Use `Regex::new(r"\[\[([^\]]+)\]\]")` (lazy regex crate)
    - Collect matches, deduplicate (preserve order, use `IndexSet` or manual dedup)
    - Return `Vec<String>` of unique card names
  - **REFACTOR**: Extract regex to `lazy_static!` constant

  **Must NOT do**:
  - 不支持 `[[card|display]]` 语法
  - 不支持 `[[card#section]]` 语法
  - 不做卡片名规范化（保留原始大小写和空格）

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Rust 正则 + 边界情况处理需要深度理解
  - **Skills**: [`coding-standards`]
    - `coding-standards`: Rust 最佳实践

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3, 4)
  - **Blocks**: Tasks 2, 4, 7, 8
  - **Blocked By**: None

  **References**:
  - `D:\dev\memory-stream\rust-workspace\md-parser\src\lib.rs` — 现有 parse_markdown 函数，extract_wikilinks 需在此文件中新增
  - `D:\dev\memory-stream\rust-workspace\md-parser\Cargo.toml` — 依赖配置，需确认 regex crate 是否已引入
  - `D:\dev\memory-stream\rust-workspace\ast-core\src\lib.rs` — AstNode::Link 定义，理解现有链接处理模式

  **WHY Each Reference Matters**:
  - md-parser/src/lib.rs: 这是要添加 extract_wikilinks 的文件，需要了解现有函数签名和代码风格
  - Cargo.toml: 确认 regex 依赖是否可用，如需添加则知道在哪添加
  - ast-core/src/lib.rs: 理解现有 Link 节点类型，未来可能需要添加 Wikilink 变体

  **Acceptance Criteria**:

  **TDD Phase**:
  - [ ] Test file `md-parser/tests/wikilink_test.rs` created with 8 test cases
  - [ ] `cargo test --manifest-path rust-workspace/md-parser/Cargo.toml` → 8 tests fail (RED)

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Wikilink extraction — happy path
    Tool: Bash (cargo test)
    Preconditions: md-parser crate compiled
    Steps:
      1. cargo test extract_wikilinks --manifest-path rust-workspace/md-parser/Cargo.toml
      2. Verify output contains "test result: ok. 8 passed"
    Expected Result: All 8 test cases pass
    Failure Indicators: Any test failure, compilation error
    Evidence: .sisyphus/evidence/task-1-rust-wikilink-test.txt

  Scenario: Wikilink extraction — edge cases
    Tool: Bash (cargo test)
    Preconditions: Tests written
    Steps:
      1. cargo test extract_wikilinks --manifest-path rust-workspace/md-parser/Cargo.toml -- --nocapture
      2. Verify deduplication, UTF-8, nested brackets, empty input tests pass
    Expected Result: Dedup works, UTF-8 preserved, nested brackets handled, empty returns []
    Evidence: .sisyphus/evidence/task-1-rust-wikilink-edge.txt
  ```

  **Commit**: YES (groups with Task 2)
  - Message: `feat(rust): implement extract_wikilinks function with TDD`
  - Files: `md-parser/src/lib.rs`, `md-parser/tests/wikilink_test.rs`

- [ ] 2. Rust RenderResult 改造 + process_markdown 集成 — TDD

  **What to do**:
  - **RED**: Write test verifying process_markdown returns extracted_links
  - **GREEN**: Modify `frontend-workspace/apps/admin-tauri/src-tauri/src/lib.rs`:
    - Add `extracted_links: Vec<String>` field to `RenderResult` struct
    - In `process_markdown` Tauri command: call `extract_wikilinks(&md)` BEFORE `parse_markdown`
    - Include result in `RenderResult { html, ast_json, excerpt, extracted_links }`
  - **REFACTOR**: Ensure backward compatibility — existing consumers of RenderResult unaffected

  **Must NOT do**:
  - 不改变 parse_markdown 函数本身（只在外层包装）
  - 不改变 extract_toc 的逻辑
  - 不删除 RenderResult 的任何现有字段

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 结构体字段添加 + 函数调用包装，模式清晰
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 3, 4)
  - **Blocks**: Tasks 8, 11
  - **Blocked By**: Task 1

  **References**:
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src-tauri\src\lib.rs` — RenderResult struct 和 process_markdown command 定义
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src-tauri\src\main.rs` — Tauri command 注册，确认 process_markdown 注册位置
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\composables\useMarkdown.ts` 或 TheForge.vue — 前端消费 RenderResult 的位置，理解调用模式

  **Acceptance Criteria**:
  - [ ] `RenderResult` struct 包含 `extracted_links: Vec<String>` 字段
  - [ ] `cargo test` 通过
  - [ ] 前端调用 `process_markdown` 后能获取 `extracted_links` 数组

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: RenderResult contains extracted_links
    Tool: Bash (cargo test)
    Steps:
      1. cargo test --manifest-path frontend-workspace/apps/admin-tauri/src-tauri/Cargo.toml
      2. Verify RenderResult serialization includes extracted_links field
    Expected Result: Tests pass, extracted_links field present in JSON output
    Evidence: .sisyphus/evidence/task-2-render-result.txt

  Scenario: Backward compatibility — existing consumers unaffected
    Tool: Bash (grep)
    Steps:
      1. grep -r "RenderResult" frontend-workspace/apps/admin-tauri/src-tauri/src/
      2. Verify all usages still compile
    Expected Result: No compilation errors, existing fields untouched
    Evidence: .sisyphus/evidence/task-2-backward-compat.txt
  ```

  **Commit**: YES
  - Message: `feat(tauri): add extracted_links to RenderResult`
  - Files: `src-tauri/src/lib.rs`

- [ ] 3. Go `FindOrCreateByTitle()` ghost card 服务方法 — TDD

  **What to do**:
  - **RED**: Write failing tests in `go-server/internal/services/card_test.go`:
    - Existing card title → returns existing card, no new creation
    - Unknown card title → creates new card with empty content, returns it
    - Second call with same unknown title → returns previously created ghost
    - Concurrent calls with same title → only 1 card created (upsert protection)
  - **GREEN**: Implement in `go-server/internal/services/card.go`:
    - `func (s *CardService) FindOrCreateByTitle(title string) (*models.Card, error)`
    - First: `SELECT * FROM cards WHERE title = ? LIMIT 1`
    - If found: return existing
    - If not found: `INSERT INTO cards (id, title, content, created_at, updated_at) VALUES (uuid, title, '', now, now)`
    - Use `db.Transaction()` for SELECT+INSERT atomicity
    - Ghost card convention: content = "" (empty string), rest is normal card
  - **REFACTOR**: Consider adding `IsGhost()` helper method on Card model (content == "")

  **Must NOT do**:
  - 不添加 `is_ghost` 数据库字段
  - 不创建特殊的 card status/type
  - 不修改 Card struct 的现有字段

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: SELECT + INSERT 模式，Go GORM 事务模式清晰
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2, 4)
  - **Blocks**: Task 4
  - **Blocked By**: None

  **References**:
  - `D:\dev\memory-stream\go-server\internal\services\card.go` — 现有 CardService，包含 CreateCard 等方法，参照事务模式
  - `D:\dev\memory-stream\go-server\internal\models\schema.go` — Card struct 定义（ID, Title, Content, RawMd 等字段）
  - `D:\dev\memory-stream\go-server\internal\services\edge.go` — EdgeService 模式参照

  **Acceptance Criteria**:
  - [ ] `go test ./internal/services/ -run TestFindOrCreateByTitle` → PASS
  - [ ] Ghost card content 为空字符串
  - [ ] 幂等性：多次调用同一 title 不重复创建

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Ghost card creation — unknown title
    Tool: go test
    Steps:
      1. go test ./internal/services/ -run TestFindOrCreateByTitle -v
      2. Verify test creates ghost card with empty content
      3. Verify second call returns same card (no duplicate)
    Expected Result: 4 tests pass, ghost card has empty content
    Failure Indicators: Duplicate card created, non-empty content
    Evidence: .sisyphus/evidence/task-3-ghost-card.txt

  Scenario: Existing card lookup — known title
    Tool: go test
    Steps:
      1. Pre-create card with title "Known Card"
      2. Call FindOrCreateByTitle("Known Card")
      3. Verify returns existing card (same ID)
    Expected Result: Returns existing card, no new INSERT
    Evidence: .sisyphus/evidence/task-3-existing-card.txt
  ```

  **Commit**: YES (groups with Task 4)
  - Message: `feat(go): implement FindOrCreateByTitle for ghost card creation`
  - Files: `services/card.go`, `services/card_test.go`

- [ ] 4. Go `SyncReferenceEdges()` 原子事务 — TDD

  **What to do**:
  - **RED**: Write failing tests in `go-server/internal/services/edge_test.go`:
    - new_links=["A","B"], existing=[] → creates 2 reference edges to resolved card IDs
    - new_links=["A"], existing=["A","B"] → deletes edge to B, keeps A
    - new_links=[], existing=["A"] → deletes all reference out-edges
    - Unknown title "Ghost" → creates ghost card + reference edge to it
    - Transaction rollback on ghost creation failure → no partial edges
    - Does NOT touch sequence edges for same source-target pair
    - Deduplication: new_links=["A","A"] → only 1 edge created
  - **GREEN**: Implement in `go-server/internal/services/edge.go`:
    - `func (s *EdgeService) SyncReferenceEdges(sourceCardID string, targetTitles []string) error`
    - Use `s.db.Transaction(func(tx *gorm.DB) error { ... })` (参照 CardService.DeleteCard 事务模式)
    - Inside transaction:
      1. Deduplicate targetTitles
      2. For each title: call CardService.FindOrCreateByTitle → get cardID
      3. Query current reference out-edges: `SELECT target_id FROM card_edges WHERE source_id = ? AND relation_type = 'reference'`
      4. Compute diff: `toAdd` (in new, not in existing), `toRemove` (in existing, not in new)
      5. Batch INSERT new edges (relation_type = 'reference')
      6. Batch DELETE removed edges (WHERE source_id AND target_id IN removed AND relation_type = 'reference')
      7. Invalidate graph cache for source card and all affected targets
    - On any error: transaction rolls back, no partial state
  - **REFACTOR**: Extract edge diff computation to helper function

  **Must NOT do**:
  - 不修改现有 CreateEdge/DeleteEdge/UpdateEdgeType 方法
  - 不触碰 relation_type = 'sequence' 的边
  - 不修改 card_edges 表结构
  - 不在事务外做任何写入

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: 事务 + 批量 diff + ghost card 集成，数据完整性关键
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2, 3)
  - **Blocks**: Tasks 5, 6
  - **Blocked By**: Task 1, Task 3

  **References**:
  - `D:\dev\memory-stream\go-server\internal\services\edge.go` — 现有 EdgeService，SyncReferenceEdges 需在此文件新增，参照现有方法风格
  - `D:\dev\memory-stream\go-server\internal\services\card.go` — CardService.DeleteCard 事务模式（约 line 78-95），直接复制此事务结构
  - `D:\dev\memory-stream\go-server\internal\models\schema.go:CardEdge` — CardEdge struct 字段：SourceID, TargetID, RelationType, CreatedAt
  - `D:\dev\memory-stream\go-server\internal\services\graph.go` — Graph cache invalidation 方法，理解如何清除缓存

  **WHY Each Reference Matters**:
  - edge.go: SyncReferenceEdges 添加位置，需匹配现有代码风格（错误处理、日志模式）
  - card.go: CardService.DeleteCard 展示了 GORM 事务的正确写法，避免重复造轮子
  - schema.go: CardEdge 是操作的核心数据结构
  - graph.go: 每次 edge 变更都需要清除 graph cache，否则图谱不刷新

  **Acceptance Criteria**:
  - [ ] `go test ./internal/services/ -run TestSyncReferenceEdges` → PASS (7 tests)
  - [ ] Transaction rollback 测试通过（模拟 ghost 创建失败）
  - [ ] 不触碰 sequence edges 的测试通过

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Edge diff — add new, remove old
    Tool: go test
    Preconditions: source card exists, target cards "A" and "B" exist
    Steps:
      1. SyncReferenceEdges(sourceID, ["A", "B"]) → creates 2 edges
      2. SyncReferenceEdges(sourceID, ["A", "C"]) → adds C, removes B, keeps A
      3. Query edges: SELECT * FROM card_edges WHERE source_id = sourceID AND relation_type = 'reference'
    Expected Result: 2 edges (A, C), no B edge
    Evidence: .sisyphus/evidence/task-4-edge-diff.txt

  Scenario: Transaction rollback on failure
    Tool: go test
    Preconditions: Mock FindOrCreateByTitle to fail on second call
    Steps:
      1. SyncReferenceEdges(sourceID, ["A", "FAIL"])
      2. Verify NO edges created (rollback)
    Expected Result: 0 edges, error returned
    Evidence: .sisyphus/evidence/task-4-txn-rollback.txt

  Scenario: Sequence edges untouched
    Tool: go test
    Preconditions: source→target has BOTH sequence and reference edges
    Steps:
      1. SyncReferenceEdges(sourceID, []) → removes reference edge
      2. Query: SELECT * FROM card_edges WHERE source_id = sourceID AND relation_type = 'sequence'
    Expected Result: sequence edge still exists
    Evidence: .sisyphus/evidence/task-4-sequence-preserve.txt
  ```

  **Commit**: YES
  - Message: `feat(go): implement SyncReferenceEdges atomic transaction with TDD`
  - Files: `services/edge.go`, `services/edge_test.go`

- [ ] 5. Go backlinks context_snippet 增强 — TDD

  **What to do**:
  - **RED**: Write failing tests:
    - `text [[target]] more text` → context = `"text [[target]] more tex"` (±30 chars)
    - `[[target]]` at content start → no negative index, returns `"[[target]]"`
    - Short content `a[[target]]b` → returns full content `"a[[target]]b"`
    - Multiple `[[target]]` occurrences → returns first context only
    - No `[[target]]` in content → returns empty string ""
    - Unicode content → handles multibyte correctly
  - **GREEN**: Implement:
    - Add `ContextSnippet string` field to `BacklinkItem` struct (or response struct)
    - In `GetBacklinks` handler/service:
      1. After finding source_id, fetch source card's `raw_md`
      2. Call `extractContextSnippet(rawMd, targetCardTitle)` helper
      3. Helper: `strings.Index(rawMd, "[["+targetTitle+"]]")` → ±30 chars with bounds checking
      4. Include context_snippet in response
    - Update `BacklinkItem` struct: `{ source_id, source_title, relation_type, context_snippet }`
  - **REFACTOR**: Consider reusing extract_wikilinks logic from Rust for consistency

  **Must NOT do**:
  - 不修改现有 backlinks 查询的 SQL 结构
  - 不做跨段落智能截断
  - 不返回多个 context snippet（只返回首次出现）

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 字符串截取 + API 字段添加，模式清晰
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 6, 7)
  - **Blocks**: Tasks 12, 13
  - **Blocked By**: Task 4

  **References**:
  - `D:\dev\memory-stream\go-server\internal\handlers\card.go` — 现有 GetBacklinks handler，需在此增强返回结构
  - `D:\dev\memory-stream\go-server\internal\services\card.go` — CardService，用于获取 source card 的 raw_md
  - `D:\dev\memory-stream\go-server\internal\models\schema.go` — Card model，确认 RawMd 或 Content 字段名称

  **Acceptance Criteria**:
  - [ ] `go test ./internal/services/ -run TestExtractContextSnippet` → PASS (6 tests)
  - [ ] `GET /api/v1/cards/:id/backlinks` 返回 context_snippet 字段
  - [ ] context_snippet 最大 63 chars (±30 + target name)

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Context snippet in backlinks API response
    Tool: Bash (curl)
    Preconditions: Card A contains "Some text [[Card B]] more content here" and links to Card B
    Steps:
      1. curl -s http://localhost:8080/api/v1/cards/{cardB_id}/backlinks -H "Authorization: Bearer $TOKEN"
      2. Parse JSON response
      3. Verify context_snippet field exists and contains "...Some text [[Card B]] more conte..."
    Expected Result: context_snippet ≈ "Some text [[Card B]] more conte" (±30 chars around link)
    Evidence: .sisyphus/evidence/task-5-context-api.txt

  Scenario: Edge case — wikilink at content start
    Tool: go test
    Steps:
      1. Test with raw_md = "[[Target]] is the first thing"
      2. Verify snippet starts at index 0, no negative bounds
    Expected Result: snippet = "[[Target]] is the first thing" (truncated at +30)
    Evidence: .sisyphus/evidence/task-5-start-boundary.txt
  ```

  **Commit**: YES
  - Message: `feat(go): add context_snippet to backlinks API response`
  - Files: `handlers/card.go`, `services/card.go` or new helper

- [ ] 6. Go PUT /cards handler 集成 extracted_links

  **What to do**:
  - Modify `go-server/internal/handlers/card.go` UpdateCard handler:
    - Accept `extracted_links` field in PUT request body: `type UpdateCardRequest struct { ..., ExtractedLinks []string }`
    - After updating card content, if `extracted_links` is non-nil:
      1. Call `edgeService.SyncReferenceEdges(cardID, extractedLinks)`
      2. On error: log but don't fail the card update (card content is source of truth)
    - Ensure WS broadcast still fires for both CARD_UPDATED and edge changes
  - Modify request struct/types to include `extracted_links` field
  - Add input validation: each link title should be non-empty, trim whitespace

  **Must NOT do**:
  - 不改变现有 card update 的核心逻辑
  - 不让 edge sync 失败阻断 card save
  - 不修改 PATCH /cards 或其他端点

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: API 字段添加 + 服务调用集成
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 7)
  - **Blocks**: Task 11
  - **Blocked By**: Task 4

  **References**:
  - `D:\dev\memory-stream\go-server\internal\handlers\card.go` — UpdateCard handler，需在此添加 extracted_links 处理
  - `D:\dev\memory-stream\go-server\internal\models\schema.go` — Card model，理解 UpdateCardRequest 结构
  - `D:\dev\memory-stream\go-server\cmd\api\main.go` — 路由注册，确认 PUT /cards 端点位置

  **Acceptance Criteria**:
  - [ ] PUT /cards 接受 `extracted_links` 字段
  - [ ] extracted_links 触发 SyncReferenceEdges 调用
  - [ ] Edge sync 失败不阻断 card save
  - [ ] `go test ./internal/handlers/` → PASS

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: PUT /cards with extracted_links syncs reference edges
    Tool: Bash (curl)
    Preconditions: Card A exists, Card B exists with title "Linked Card"
    Steps:
      1. curl -X PUT http://localhost:8080/api/v1/cards/{cardA_id} -H "Authorization: Bearer $TOKEN" -d '{"content":"See [[Linked Card]]", "extracted_links":["Linked Card"]}'
      2. Query card_edges WHERE source_id = cardA_id AND relation_type = 'reference'
    Expected Result: Reference edge from A to B created, card content updated
    Evidence: .sisyphus/evidence/task-6-put-sync.txt

  Scenario: Edge sync failure doesn't block card save
    Tool: Bash (curl)
    Preconditions: Mock edge service failure
    Steps:
      1. curl -X PUT ... -d '{"content":"[[Bad]]", "extracted_links":["Bad"]}'
      2. Verify card content still updated despite edge failure
    Expected Result: Card saved, edges may be missing, no 500 error
    Evidence: .sisyphus/evidence/task-6-resilient-save.txt
  ```

  **Commit**: YES
  - Message: `feat(go): integrate extracted_links in PUT /cards handler`
  - Files: `handlers/card.go`

- [ ] 7. Rust ast-renderer `[[wikilink]] → <a>` HTML 渲染 — TDD

  **What to do**:
  - **RED**: Write failing test:
    - Input markdown `Check [[Card Name]] for details` → rendered HTML contains `<a class="reference-link" data-card-name="Card Name">Card Name</a>`
    - Multiple wikilinks → multiple anchors
    - Wikilink with special chars `[[C++ Notes]]` → properly escaped in attribute
  - **GREEN**: Implement in `rust-workspace/ast-renderer/src/lib.rs`:
    - Before HTML rendering pass: regex replace `\[\[([^\]]+)\]\]` with a sentinel/marker
    - OR: Add wikilink handling in the AST rendering pipeline
    - Rendered output: `<a class="reference-link" data-card-name="Card Name">Card Name</a>`
    - Note: `href` attribute will be resolved by the frontend (UUID lookup) or set to `#` for now
  - **REFACTOR**: Ensure wikilink rendering is separate from regular link rendering

  **Must NOT do**:
  - 不在渲染时做卡片 UUID 查询（那是 Go 后端或前端的事）
  - 不改变现有 Link 节点的渲染逻辑
  - 不添加 JavaScript 行为到 anchor 标签

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 正则替换 + HTML 生成，模式清晰
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 6)
  - **Blocks**: Task 14
  - **Blocked By**: Task 1

  **References**:
  - `D:\dev\memory-stream\rust-workspace\ast-renderer\src\lib.rs` — 现有 AST→HTML 渲染逻辑，需在此添加 wikilink 处理
  - `D:\dev\memory-stream\rust-workspace\ast-core\src\lib.rs` — AstNode 枚举定义，理解 Link 变体
  - `D:\dev\memory-stream\rust-workspace\md-parser\src\lib.rs` — extract_wikilinks 函数（Task 1），渲染器需匹配相同正则

  **Acceptance Criteria**:
  - [ ] `cargo test --manifest-path rust-workspace/ast-renderer/Cargo.toml` → PASS
  - [ ] `[[Card Name]]` 渲染为 `<a class="reference-link" data-card-name="Card Name">Card Name</a>`

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Wikilink renders as anchor tag
    Tool: Bash (cargo test)
    Steps:
      1. cargo test --manifest-path rust-workspace/ast-renderer/Cargo.toml -- --nocapture
      2. Verify output HTML contains reference-link anchor
    Expected Result: HTML contains '<a class="reference-link" data-card-name="Card Name">Card Name</a>'
    Evidence: .sisyphus/evidence/task-7-wikilink-render.txt
  ```

  **Commit**: YES
  - Message: `feat(rust): render [[wikilink]] as reference-link anchor in ast-renderer`
  - Files: `ast-renderer/src/lib.rs`

- [ ] 8. Admin TheForge → CodeMirror 6 替换 + wikilink decorator — TDD

  **What to do**:
  This is the largest and most complex frontend task. Replace the plain `<textarea>` in TheForge.vue with CodeMirror 6, preserving all existing functionality.

  **Phase A — CodeMirror 6 Integration**:
  - Install dependencies: `pnpm --filter admin-tauri add codemirror @codemirror/view @codemirror/state @codemirror/lang-markdown @codemirror/language`
  - Create `src/components/CodemirrorEditor.vue` wrapper component:
    - Props: `modelValue: string` (v-model), `placeholder: string`
    - Emits: `update:modelValue`, `save`
    - Uses `EditorView` and `EditorState` from @codemirror/view and @codemirror/state
    - Extensions: `markdown()`, `lineNumbers()`, `highlightActiveLine()`, `keymap` (Ctrl+S → emit save)
    - Sync: `view.dispatch({ changes })` on external modelValue change, `@update` listener on internal change
  - Replace `<textarea>` in TheForge.vue with `<CodemirrorEditor v-model="activeCard.content" />`
  - **Preserve**: split view toggle, preview mode, save button behavior, dirty tracking

  **Phase B — Image Paste Handler Rewrite**:
  - Replace `editorRef.value.selectionStart/selectionEnd` (textarea DOM API) with CodeMirror API:
    ```typescript
    const { from, to } = view.state.selection.main
    view.dispatch({ changes: { from, to, insert: placeholder } })
    ```
  - After upload completes, find placeholder and replace with `![](url)`:
    ```typescript
    const pos = view.state.doc.toString().indexOf(placeholder)
    if (pos !== -1) {
      view.dispatch({ changes: { from: pos, to: pos + placeholder.length, insert: `![](${url})` } })
    }
    ```

  **Phase C — Wikilink Highlighting (TDD)**:
  - **RED**: Write test verifying `[[wikilink]]` renders with `.text-neon-cyan` class
  - **GREEN**: Create `src/composables/wikilinkHighlight.ts`:
    ```typescript
    import { Decoration, DecorationSet, EditorView, ViewPlugin, ViewUpdate } from '@codemirror/view'
    import { RangeSetBuilder } from '@codemirror/state'
    
    const WIKILINK_REGEX = /\[\[([^\]]+)\]\]/g
    
    const wikilinkDecoration = Decoration.mark({ class: 'text-neon-cyan' })
    
    const wikilinkPlugin = ViewPlugin.fromClass(class {
      decorations: DecorationSet
      constructor(view: EditorView) { this.decorations = this.buildDecorations(view) }
      update(update: ViewUpdate) { if (update.docChanged || update.viewportChanged) this.decorations = this.buildDecorations(update.view) }
      buildDecorations(view: EditorView): DecorationSet {
        const builder = new RangeSetBuilder<Decoration>()
        const doc = view.state.doc.toString()
        let match
        while ((match = WIKILINK_REGEX.exec(doc)) !== null) {
          const from = match.index
          const to = from + match[0].length
          if (builder.add(from, to, wikilinkDecoration)) break // one range
        }
        return builder.finish()
      }
    }, { decorations: v => v.decorations })
    ```
  - Add `EditorView.baseTheme({ '.text-neon-cyan': { color: '#00e5ff', fontWeight: 'bold' } })`
  - Add plugin to CodeMirror extensions

  **Must NOT do**:
  - 不使用 WYSIWYG 编辑器（Quill/TinyMCE）
  - 不添加 wikilink 自动补全
  - 不改变 save 工作流（process_markdown → PUT 仍为流程）
  - 不删除现有的 preview/split view 功能

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: CodeMirror 6 集成 + 自定义 decorator + 图片粘贴重写是复杂的前端视觉工程
  - **Skills**: [`frontend-design`, `coding-standards`]
    - `frontend-design`: 高质量编辑器 UI 设计
    - `coding-standards`: TypeScript + Vue3 最佳实践

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 9, 10, 11)
  - **Blocks**: Task 11
  - **Blocked By**: Task 2

  **References**:
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\components\TheForge.vue` — 现有编辑器组件，包含 textarea、save、image paste、split view、preview 全部逻辑
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\stores\knowledge.ts` — activeCard 状态管理，理解 v-model 绑定和脏追踪
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src-tauri\src\lib.rs` — RenderResult struct（Task 2 修改后），前端需消费 extracted_links
  - CodeMirror 6 docs: `https://codemirror.net/docs/guide/` — ViewPlugin, Decoration, MatchDecorator 模式

  **WHY Each Reference Matters**:
  - TheForge.vue: 这是被替换的核心文件，必须逐行理解 textarea 绑定、图片粘贴、save 流程，确保 CodeMirror 替换不丢失任何功能
  - knowledge.ts: activeCard.content 的响应式绑定必须与 CodeMirror 状态双向同步
  - lib.rs: Task 2 新增的 extracted_links 字段，前端 save 时需包含在 payload 中

  **Acceptance Criteria**:
  - [ ] CodeMirror 渲染正常，无 textarea 残留
  - [ ] `[[wikilink]]` 文本显示霓虹青色高亮
  - [ ] 图片粘贴（Ctrl+V）正常工作
  - [ ] Ctrl+S 保存正常工作
  - [ ] Preview/Split view 正常工作
  - [ ] 脏追踪（dirty state）正常触发
  - [ ] `vue-tsc --noEmit` 零错误

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Wikilink syntax highlighting
    Tool: Playwright / interactive_bash (tmux)
    Preconditions: admin-tauri dev 模式运行
    Steps:
      1. 打开一个卡片
      2. 在编辑器中输入 "Check [[Test Card]] for reference"
      3. 验证 "[[Test Card]]" 文本显示为霓虹青色
    Expected Result: "[[Test Card]]" has color #00e5ff, bold
    Failure Indicators: wikilink 显示为普通文本
    Evidence: .sisyphus/evidence/task-8-wikilink-highlight.png

  Scenario: Image paste still works after CodeMirror upgrade
    Tool: interactive_bash (tmux)
    Preconditions: admin-tauri dev 运行，剪贴板有图片
    Steps:
      1. 在 CodeMirror 编辑器中 Ctrl+V
      2. 等待上传完成
      3. 验证编辑器中显示 ![](uploaded-url)
    Expected Result: 图片上传成功，markdown 图片语法插入
    Failure Indicators: 粘贴无反应，或图片 URL 未替换占位符
    Evidence: .sisyphus/evidence/task-8-image-paste.txt

  Scenario: Save workflow preserves extracted_links
    Tool: Bash (grep)
    Steps:
      1. grep "extracted_links" frontend-workspace/apps/admin-tauri/src/components/TheForge.vue
      2. Verify PUT /cards payload includes extracted_links from RenderResult
    Expected Result: save payload 包含 extracted_links 数组
    Evidence: .sisyphus/evidence/task-8-save-payload.txt

  Scenario: vue-tsc passes
    Tool: Bash
    Steps:
      1. cd frontend-workspace && npx vue-tsc --noEmit --project apps/admin-tauri/tsconfig.json
    Expected Result: 零错误
    Evidence: .sisyphus/evidence/task-8-tsc.txt
  ```

  **Commit**: YES
  - Message: `feat(vue): replace textarea with CodeMirror 6 + wikilink highlight decorator`
  - Files: `src/components/TheForge.vue`, `src/components/CodemirrorEditor.vue`, `src/composables/wikilinkHighlight.ts`

- [ ] 9. Admin RightAstrolabe reference edge 创建守卫

  **What to do**:
  - In `RightAstrolabe.vue`, modify the `onConnect` handler:
    - Current: creates "reference" edge by default via `store.createEdgeHttp(...)`
    - New behavior:
      1. When user drags to connect → show confirmation dialog with edge type choice
      2. Choices: "Sequence (主干拓扑)" or cancel
      3. **Remove the "Reference" option entirely** — reference edges can ONLY be created via [[wikilink]]
      4. If user tries to create an edge with relation_type = 'reference' → show tooltip: "Reference edges are created automatically from [[wikilinks]] in card content. Please use [[Card Name]] syntax in the editor."
  - In edge context menu: remove "Set as Reference" option
  - Add a visual indicator in the graph legend: "Solid = Manual (Sequence) · Dashed = Auto (Reference from [[links]])"

  **Must NOT do**:
  - 不禁用 sequence edge 的拖拽创建
  - 不改变边的视觉样式
  - 不修改 onConnect 之外的交互

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: UI 守卫逻辑简单，主要是移除选项 + 添加 tooltip
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 8, 10, 11)
  - **Blocks**: None
  - **Blocked By**: None

  **References**:
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\components\RightAstrolabe.vue` — onConnect handler（约 line 150+），edge context menu，flowEdges computed
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\stores\useEdgeStore.ts` — createEdgeHttp 方法

  **Acceptance Criteria**:
  - [ ] 拖拽连线只能创建 sequence edge
  - [ ] Reference edge 选项已移除
  - [ ] 图谱底部显示图例说明

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Drag-to-connect creates only sequence edge
    Tool: Bash (grep)
    Steps:
      1. grep -A5 "onConnect" frontend-workspace/apps/admin-tauri/src/components/RightAstrolabe.vue
      2. Verify relation_type defaults to "sequence" (not "reference")
    Expected Result: onConnect creates sequence edge only
    Evidence: .sisyphus/evidence/task-9-sequence-only.txt

  Scenario: Edge context menu has no "Reference" option
    Tool: Bash (grep)
    Steps:
      1. grep -i "reference\|relation_type" frontend-workspace/apps/admin-tauri/src/components/RightAstrolabe.vue
      2. Verify context menu doesn't offer reference edge creation
    Expected Result: No "set as reference" option in context menu
    Evidence: .sisyphus/evidence/task-9-no-ref-menu.txt
  ```

  **Commit**: YES
  - Message: `feat(vue): enforce sequence-only edge creation in astrolabe graph`
  - Files: `src/components/RightAstrolabe.vue`

- [ ] 10. Web-reader 边突变交互禁用

  **What to do**:
  - In `GraphView.vue`:
    - Remove or disable `onConnect` handler entirely (no drag-to-connect)
    - Remove edge context menu (no right-click delete/edit)
    - Add CSS to hide connection handles: `.vue-flow__handle { display: none; }`
    - Remove edge selection highlight (clicking edge does nothing)
  - In `useGraphSync.ts`: keep WS listeners for edge updates (read-only display still needs sync)
  - Ensure both edge types display correctly (existing visual differentiation should work)

  **Must NOT do**:
  - 不改变边的视觉样式
  - 不移除 WS 同步（仍需显示实时更新）
  - 不添加任何新的交互功能

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 纯 UI 移除/禁用操作
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 8, 9, 11)
  - **Blocks**: None
  - **Blocked By**: None

  **References**:
  - `D:\dev\memory-stream\frontend-workspace\apps\web-reader\src\views\GraphView.vue` — Vue Flow 配置，onConnect handler，edge rendering
  - `D:\dev\memory-stream\frontend-workspace\apps\web-reader\src\composables\useGraphSync.ts` — WS edge 同步，保留只读监听
  - `D:\dev\memory-stream\frontend-workspace\apps\web-reader\src\composables\useGraph.ts` — flowEdges 转换，验证视觉区分

  **Acceptance Criteria**:
  - [ ] 拖拽手柄不可见
  - [ ] 边右键菜单不存在
  - [ ] WS 同步仍正常（能看到实时边更新）
  - [ ] `vue-tsc --noEmit` 零错误

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: No drag handles in web-reader graph
    Tool: Playwright
    Preconditions: web-reader dev 运行，有图谱数据
    Steps:
      1. 导航到 GraphView
      2. 验证 .vue-flow__handle 元素不存在或 display:none
    Expected Result: No connection handles visible
    Evidence: .sisyphus/evidence/task-10-no-handles.png

  Scenario: WebSocket edge updates still sync
    Tool: Bash (grep)
    Steps:
      1. grep "edgeCreated\|edgeDeleted\|edgeUpdated" frontend-workspace/apps/web-reader/src/composables/useGraphSync.ts
      2. Verify WS handlers still active (read-only display)
    Expected Result: WS event handlers present and functional
    Evidence: .sisyphus/evidence/task-10-ws-sync.txt
  ```

  **Commit**: YES
  - Message: `feat(vue): disable edge mutations in web-reader graph view`
  - Files: `src/views/GraphView.vue`, possibly `src/composables/useGraphSync.ts`

- [ ] 11. Admin TheForge save 流程集成 extracted_links

  **What to do**:
  - Modify the save handler in TheForge.vue:
    - Current flow: `process_markdown(content)` → `extract_toc(ast_json)` → `PUT /cards/{id}`
    - New flow: `process_markdown(content)` → result contains `extracted_links` → `extract_toc(ast_json)` → `PUT /cards/{id}` with `extracted_links` in payload
  - In the PUT payload, add `extracted_links` field:
    ```typescript
    const payload = {
      raw_md: activeCard.content,
      ast_data: result.ast_json,
      toc_json: JSON.stringify(toc),
      excerpt: result.excerpt,
      extracted_links: result.extracted_links, // NEW
    }
    ```
  - Ensure auto-save (debounced) also includes extracted_links

  **Must NOT do**:
  - 不改变 process_markdown 的调用方式
  - 不改变 extract_toc 的调用方式
  - 不添加额外的 API 调用

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: payload 字段添加，模式清晰
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 8, 9, 10)
  - **Blocks**: Task 12
  - **Blocked By**: Task 2, Task 6, Task 8

  **References**:
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\components\TheForge.vue` — save handler（搜索 process_markdown 调用），auto-save debounce 逻辑
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\stores\knowledge.ts` — saveCard 方法，理解 PUT payload 结构

  **Acceptance Criteria**:
  - [ ] PUT /cards payload 包含 `extracted_links` 数组
  - [ ] Manual save (Ctrl+S) 包含 extracted_links
  - [ ] Auto-save 包含 extracted_links

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Save payload includes extracted_links
    Tool: Bash (grep)
    Steps:
      1. grep -A10 "extracted_links\|process_markdown" frontend-workspace/apps/admin-tauri/src/components/TheForge.vue
      2. Verify result.extracted_links is included in PUT payload
    Expected Result: extracted_links field present in save payload construction
    Evidence: .sisyphus/evidence/task-11-save-payload.txt

  Scenario: End-to-end — save card with wikilink creates reference edge
    Tool: Bash (curl) — integration test
    Preconditions: Card A and Card B exist
    Steps:
      1. PUT /cards/{A_id} with content "See [[Card B]]" and extracted_links=["Card B"]
      2. GET /cards/{A_id}/edges or query card_edges
    Expected Result: Reference edge from A to B exists
    Evidence: .sisyphus/evidence/task-11-e2e-edge.txt
  ```

  **Commit**: YES
  - Message: `feat(vue): integrate extracted_links in card save workflow`
  - Files: `src/components/TheForge.vue`

- [ ] 12. Admin Radar backlinks 面板增强 context_snippet — TDD

  **What to do**:
  - **RED**: Write test verifying backlinks panel displays context_snippet text
  - **GREEN**: Modify the backlinks panel in TheForge.vue (bottom section, ~lines 328-355):
    - Current: displays `{ source_title } — { relation_type }`
    - Enhanced: display structure per backlink item:
      ```html
      <div class="py-1.5">
        <div class="flex items-center gap-2">
          <span class="text-xs font-mono text-slate-600 uppercase">{{ link.relation_type }}</span>
          <button class="text-sm text-slate-300 hover:text-neon" @click="loadCard(link.source_id)">
            {{ link.source_title }}
          </button>
        </div>
        <p v-if="link.context_snippet" class="text-xs text-slate-500 italic pl-4 mt-0.5 truncate">
          ...{{ link.context_snippet }}...
        </p>
      </div>
      ```
    - Context snippet styling: `text-xs text-slate-500 italic truncate` (gray italic, truncated)
    - Relation type badge: sequence vs reference with different colors (sequence=cyan badge, reference=gray badge)
  - Update `fetchBacklinks` to include context_snippet from API response (Task 5 modified the API)
  - Update BacklinkItem type to include `context_snippet?: string`

  **Must NOT do**:
  - 不添加 backlinks 分页
  - 不做 context_snippet 的语法高亮
  - 不修改后端 API（Task 5 已处理）

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: 面板重设计 + 工业控制台美学保持
  - **Skills**: [`frontend-design`, `coding-standards`]
    - `frontend-design`: 高质量面板 UI 设计
    - `coding-standards`: TypeScript 类型安全

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 13, 14)
  - **Blocks**: None
  - **Blocked By**: Task 5, Task 11

  **References**:
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\components\TheForge.vue:328-355` — 现有 backlinks 面板区域
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\stores\knowledge.ts:66-71` — backlinks ref 和 fetchBacklinks 方法，需更新类型
  - `D:\dev\memory-stream\frontend-workspace\packages\types\index.ts` — BacklinkItem 类型定义，需添加 context_snippet 字段

  **Acceptance Criteria**:
  - [ ] Backlinks 面板显示 context_snippet（灰色斜体，截断）
  - [ ] Relation type 区分 sequence/reference 视觉
  - [ ] `vue-tsc --noEmit` 零错误

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Context snippet displayed in radar panel
    Tool: Playwright / interactive_bash (tmux)
    Preconditions: admin-tauri 运行，Card B 有 backlink 到当前卡片，context_snippet 非空
    Steps:
      1. 打开 Card A（有来自 Card B 的 backlink）
      2. 查看底部 Radar 面板
      3. 验证 backlink 条目下方有灰色斜体文本
    Expected Result: Context snippet 显示为 "..." 包裹的灰色斜体文本
    Evidence: .sisyphus/evidence/task-12-radar-context.png

  Scenario: No new rounded-lg/xl/2xl in radar panel
    Tool: Bash (grep)
    Steps:
      1. grep "rounded-lg\|rounded-xl\|rounded-2xl" frontend-workspace/apps/admin-tauri/src/components/TheForge.vue
    Expected Result: 0 matches (工业控制台风格)
    Evidence: .sisyphus/evidence/task-12-no-soft-corners.txt
  ```

  **Commit**: YES
  - Message: `feat(vue): enhance radar backlinks with context snippets and type badges`
  - Files: `src/components/TheForge.vue`, `packages/types/index.ts`

- [ ] 13. Web-reader backlinks 面板 — TDD

  **What to do**:
  - Create `frontend-workspace/apps/web-reader/src/components/BacklinksPanel.vue`:
    - Read-only display of incoming links to current card
    - Fetch from `GET /api/v1/cards/:id/backlinks` (same API as admin)
    - Display: source_title, relation_type badge, context_snippet
    - Click on backlink → navigate to that card in reader
    - Style: match web-reader's existing dark theme (ms-deep/ms-panel design tokens)
    - Position: below card content in reading view (or as collapsible panel)
  - Integrate into web-reader's card detail view
  - Handle loading state and empty state gracefully

  **Must NOT do**:
  - 不添加 backlink 编辑/删除功能
  - 不分页（显示全部）
  - 不修改后端 API

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: 新组件设计 + web-reader 风格一致性
  - **Skills**: [`frontend-design`, `coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 12, 14)
  - **Blocks**: None
  - **Blocked By**: Task 5

  **References**:
  - `D:\dev\memory-stream\frontend-workspace\apps\web-reader\src\views\` — web-reader 页面视图，找到卡片阅读视图位置
  - `D:\dev\memory-stream\frontend-workspace\apps\web-reader\src\api\index.ts` — API 调用模式，添加 backlinks API
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\components\TheForge.vue:328-355` — admin 端 backlinks 面板作为 UI 参考

  **Acceptance Criteria**:
  - [ ] `BacklinksPanel.vue` 组件存在
  - [ ] 显示 backlinks 列表（title + type + context_snippet）
  - [ ] 点击 backlink 可导航到对应卡片
  - [ ] `vue-tsc --noEmit` 零错误

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Backlinks panel renders in web-reader
    Tool: Playwright
    Preconditions: web-reader dev 运行，当前卡片有 backlinks
    Steps:
      1. 导航到有 backlinks 的卡片
      2. 验证 BacklinksPanel 组件可见
      3. 验证 context_snippet 显示
    Expected Result: Backlinks 面板显示，context snippets 可见
    Evidence: .sisyphus/evidence/task-13-web-backlinks.png

  Scenario: Click backlink navigates to card
    Tool: Playwright
    Steps:
      1. 点击 backlink 列表中的第一条
      2. 验证 URL 变化，内容更新为目标卡片
    Expected Result: 导航成功，目标卡片内容显示
    Evidence: .sisyphus/evidence/task-13-navigate.txt
  ```

  **Commit**: YES
  - Message: `feat(vue): add backlinks panel to web-reader with context snippets`
  - Files: `src/components/BacklinksPanel.vue`, integrated into card detail view

- [ ] 14. Rust ast-renderer wikilink UUID 解析增强

  **What to do**:
  - Enhance the wikilink rendering from Task 7 to support actual card UUID resolution:
    - Task 7 output: `<a class="reference-link" data-card-name="Card Name">Card Name</a>`
    - This task: accept an optional `cardNameToId: HashMap<String, String>` parameter
    - When map provided and card name found: `<a href="/cards/{uuid}" class="reference-link">Card Name</a>`
    - When map not provided or name not found: fall back to data-card-name attribute
  - This is used by web-reader's WASM engine to render clickable links
  - The cardNameToId map would be injected by the Go backend (or frontend) before rendering

  **Must NOT do**:
  - 不在 Rust 中做数据库查询
  - 不改变 Task 7 的基础渲染逻辑（只是增强）
  - 不修改 ast-core 的 AstNode 类型

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Rust 泛型参数 + WASM 接口设计需要深度理解
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 12, 13)
  - **Blocks**: None
  - **Blocked By**: Task 7

  **References**:
  - `D:\dev\memory-stream\rust-workspace\ast-renderer\src\lib.rs` — Task 7 修改后的渲染器
  - `D:\dev\memory-stream\rust-workspace\wasm-engine\src\lib.rs` — WASM 打包层，理解 JS→Rust 接口

  **Acceptance Criteria**:
  - [ ] `cargo test --manifest-path rust-workspace/ast-renderer/Cargo.toml` → PASS
  - [ ] 有 UUID map 时渲染 `<a href="/cards/uuid">`
  - [ ] 无 UUID map 时回退到 `data-card-name`

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Wikilink resolves to UUID when map provided
    Tool: Bash (cargo test)
    Steps:
      1. cargo test --manifest-path rust-workspace/ast-renderer/Cargo.toml
      2. Verify test with HashMap {"Card Name": "uuid-123"} produces href="/cards/uuid-123"
    Expected Result: Anchor has correct href with UUID
    Evidence: .sisyphus/evidence/task-14-uuid-resolve.txt

  Scenario: Fallback when UUID not in map
    Tool: Bash (cargo test)
    Steps:
      1. Test with empty HashMap
      2. Verify output uses data-card-name="Card Name" (no href)
    Expected Result: Fallback rendering with data attribute
    Evidence: .sisyphus/evidence/task-14-fallback.txt
  ```

  **Commit**: YES
  - Message: `feat(rust): enhance wikilink renderer with UUID resolution`
  - Files: `ast-renderer/src/lib.rs`

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, curl endpoint, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo test` + `go test ./...` + `vue-tsc --noEmit`. Review all changed files for: `as any`/`@ts-ignore`, empty catches, console.log in prod, commented-out code, unused imports. Check AI slop: excessive comments, over-abstraction, generic names.
  Output: `Rust [PASS/FAIL] | Go [PASS/FAIL] | Vue [PASS/FAIL] | Files [N clean/N issues] | VERDICT`

- [ ] F3. **Real Manual QA** — `unspecified-high` (+ `playwright` skill if UI)
  Start from clean state. Execute EVERY QA scenario from EVERY task — follow exact steps, capture evidence. Test cross-task integration (features working together, not isolation). Test edge cases: self-reference, unicode names, ghost card lifecycle. Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (git log/diff). Verify 1:1 — everything in spec was built (no missing), nothing beyond spec was built (no creep). Check "Must NOT do" compliance. Detect cross-task contamination: Task N touching Task M's files. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

| Wave | Commit Message | Files |
|------|---------------|-------|
| 1 | test(rust): add failing tests for extract_wikilinks | md-parser/src/lib.rs, md-parser/tests/ |
| 1 | feat(rust): implement extract_wikilinks function | md-parser/src/lib.rs |
| 1 | feat(tauri): add extracted_links to RenderResult | admin-tauri/src-tauri/src/lib.rs |
| 1 | test(go): add failing tests for SyncReferenceEdges | services/edge_test.go |
| 1 | feat(go): implement SyncReferenceEdges with transaction | services/edge.go |
| 1 | feat(go): implement FindOrCreateByTitle ghost card | services/card.go |
| 2 | feat(go): add context_snippet to backlinks API | handlers/card.go, services/ |
| 2 | feat(go): integrate extracted_links in PUT /cards handler | handlers/card.go |
| 2 | feat(rust): render [[wikilink]] as reference-link anchor | ast-renderer/src/lib.rs |
| 3 | feat(vue): replace textarea with CodeMirror 6 + wikilink highlight | TheForge.vue |
| 3 | feat(vue): add reference edge creation guard to astrolabe | RightAstrolabe.vue |
| 3 | feat(vue): disable edge mutations in web-reader | GraphView.vue |
| 3 | feat(vue): integrate extracted_links in save workflow | TheForge.vue |
| 4 | feat(vue): enhance radar backlinks with context snippets | TheForge.vue |
| 4 | feat(vue): add backlinks panel to web-reader | web-reader component |
| 4 | feat(rust): resolve wikilink card names to UUIDs in renderer | ast-renderer |

---

## Success Criteria

### Verification Commands
```bash
cargo test --manifest-path rust-workspace/md-parser/Cargo.toml        # Expected: all pass
cargo test --manifest-path rust-workspace/ast-renderer/Cargo.toml     # Expected: all pass
go test ./internal/services/ -run TestSyncReferenceEdges              # Expected: PASS
go test ./internal/services/ -run TestExtractContextSnippet            # Expected: PASS
cd frontend-workspace && vue-tsc --noEmit --project apps/admin-tauri/tsconfig.json  # Expected: 0 errors
cd frontend-workspace && vue-tsc --noEmit --project apps/web-reader/tsconfig.json   # Expected: 0 errors
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass (Rust + Go + Vue)
- [ ] [[wikilink]] glows neon-cyan in CodeMirror
- [ ] PUT /cards with wikilinks → reference edges synced atomically
- [ ] Ghost cards auto-created for unknown targets
- [ ] Backlinks show context snippets
- [ ] Web-reader has zero edge mutation capabilities
- [ ] Graph UI blocks manual reference edge creation
