# Memory Stream 管理解耦与体验闭环优化

## TL;DR

> **Quick Summary**: 将分类管理从侧边栏解耦为全屏管理面板，实现多级树形分类（Go CTE + Vue 递归组件），升级 JWT 为 Rust 层主动刷新（防惊群），引入 Zod 校验 API 响应，优化 Vite 分包，增强 Web 端视觉动效。
> 
> **Deliverables**:
> - Rust 层 JWT 主动刷新（Arc<RwLock> + 拦截器 + 并发安全）
> - Go 后端 parent_id + CTE 递归查询 + 环引用保护
> - 全屏覆盖式分类管理面板（左右双栏 Split View）
> - Vue 递归树形组件（CategoryTreeNode）
> - Zod Schema 校验层（5 个 CRUD 端点）
> - Vite manualChunks 分包 + Gzip/Brotli 压缩
> - Web 端 Genesis Node 能量流动画 + StatsWidget 聚光灯联动
> 
> **Estimated Effort**: Large
> **Parallel Execution**: YES - 5 waves
> **Critical Path**: JWT Refresh → Multi-level Backend → Category Panel Frontend

---

## Context

### Original Request
基于 UI/UX 审计报告，对 Memory Stream 进行"管理解耦"与"体验闭环"优化，涵盖 6 个维度。

### Interview Summary
**Key Discussions**:
- 面板形态：用户选择 **全屏覆盖式管理面板**（类似 macOS 系统偏好设置）
- 多级分类：**本期实现**（Go 后端 parent_id + CTE + 前端树形组件）
- JWT 刷新：**Rust 层定时器**（Arc<RwLock> + 拦截器模式，防惊群效应）

**Research Findings**:
- Go 后端已有完整 CRUD API（5 个端点）、AppError 统一错误、JWT 中间件
- Rust 层已有被动 401 刷新（api.rs 第 149-192 行），需升级为主动 + 并发安全
- Web-reader 已有 manualChunks 分包（vendor-core/vendor-graph/vendor-virtual）
- useCategoryStore 已完善（CRUD），可直接复用
- @memory-stream/types Category 接口已有 parent_id 字段
- Zod v4 推荐（生态最成熟，vee-validate 官方支持）

### Metis Review
**Identified Gaps** (addressed):
- 分类深度限制：设定为 **5 级**，防止无限嵌套
- 环引用保护：Go 后端必须验证 parent_id 不形成环
- 删除保护：父分类有子分类时拒绝删除（返回错误）
- JWT 刷新失败处理：3 次重试 → 通知前端退出登录
- 拖拽排序/搜索：**明确排除**（防止范围蔓延）

---

## Work Objectives

### Core Objective
将分类管理升级为独立的全屏管理面板 + 多级树形结构，同时加固 JWT 认证和 API 类型安全。

### Concrete Deliverables
- `auth.rs` 新增主动刷新定时器 + 并发安全拦截器
- `categories` 表新增 `parent_id` + `sort_order` 字段
- Go 后端新增 CTE 递归查询 + 环引用验证
- `CategoryPanel.vue` 全屏覆盖式面板组件
- `CategoryTreeNode.vue` 递归树形组件
- `src/schemas/` 目录下的 Zod schema 定义
- Vite manualChunks 配置优化
- ListView.vue Genesis Node 能量流动画增强

### Definition of Done
- [ ] `cargo test` 全部通过
- [ ] `go test ./...` 全部通过
- [ ] `vue-tsc --noEmit` 两应用零错误
- [ ] 分类面板打开 < 300ms，树形渲染 5 级深度正常
- [ ] JWT 刷新：并发 5 请求仅触发 1 次 refresh
- [ ] Vite 构建：vendor-graph chunk < 200KB

### Must Have
- JWT 刷新的并发安全（防惊群效应）
- 环引用保护（后端验证）
- 深度限制 5 级
- 父分类删除保护（有子分类时拒绝）
- 数据迁移脚本（现有分类 parent_id = NULL）
- 向后兼容（现有单层分类消费者不受影响）

### Must NOT Have (Guardrails)
- ❌ 拖拽排序功能（明确排除，后续迭代）
- ❌ 分类图标/颜色/视觉自定义
- ❌ 分类使用统计（卡片数量）
- ❌ 分类权限/授权
- ❌ 撤销/重做操作
- ❌ 分类导出/导入
- ❌ 新建数据库表（只修改现有 categories 表）
- ❌ 改变现有 API 响应结构（只新增 parent_id 字段）
- ❌ 破坏现有 useCategoryStore API

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: YES (Go + Rust + Vue test setups)
- **Automated tests**: Tests-after
- **Framework**: Go test / cargo test / vitest
- **E2E**: Playwright for UI verification

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — backend foundation):
├── Task 1: JWT 主动刷新 — Rust Arc<RwLock> + 拦截器 + 定时器 [deep]
├── Task 2: Categories 表迁移 — parent_id + sort_order + 环引用保护 [quick]

Wave 2 (After Wave 1 — data layer):
├── Task 3: Go CTE 递归查询 — 全量树形查询 API [deep]
├── Task 4: 数据迁移脚本 — 现有分类 parent_id = NULL [quick]

Wave 3 (After Wave 2 — frontend, MAX PARALLEL):
├── Task 5: CategoryTreeNode.vue — 递归树形组件 [visual-engineering]
├── Task 6: CategoryPanel.vue — 全屏覆盖式管理面板 [visual-engineering]
├── Task 7: Zod Schema 校验层 — 5 个 CRUD 端点 [quick]

Wave 4 (Independent — optimization):
├── Task 8: Vite 分包优化 — manualChunks + compression [quick]
├── Task 9: Genesis Node 能量流动画 + StatsWidget 聚光灯联动 [visual-engineering]

Wave FINAL (After ALL tasks):
├── Task F1: Plan compliance audit [oracle]
├── Task F2: Code quality review [unspecified-high]
├── Task F3: Real manual QA [unspecified-high]
├── Task F4: Scope fidelity check [deep]
-> Present results -> Get explicit user okay

Critical Path: Task 1 → Task 3 → Task 5 → Task 6 → F1-F4
Parallel Speedup: ~60% faster than sequential
Max Concurrent: 3 (Wave 3)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| 1 | — | 3 | 1 |
| 2 | — | 3, 4 | 1 |
| 3 | 1, 2 | 5, 6, 7 | 2 |
| 4 | 2 | 5, 6 | 2 |
| 5 | 3, 4 | 6 | 3 |
| 6 | 5 | — | 3 |
| 7 | 3 | — | 3 |
| 8 | — | — | 4 |
| 9 | — | — | 4 |

### Agent Dispatch Summary

- **Wave 1**: 2 tasks — T1 → `deep`, T2 → `quick`
- **Wave 2**: 2 tasks — T3 → `deep`, T4 → `quick`
- **Wave 3**: 3 tasks — T5 → `visual-engineering`, T6 → `visual-engineering`, T7 → `quick`
- **Wave 4**: 2 tasks — T8 → `quick`, T9 → `visual-engineering`
- **FINAL**: 4 tasks — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [ ] 1. JWT 主动刷新 — Rust 层 Arc<RwLock> + 拦截器 + 定时器

  **What to do**:
  - 修改 `src-tauri/src/auth.rs`，添加 `Arc<RwLock<TokenState>>` 包装全局认证状态
  - 在 `src-tauri/src/api.rs` 的 `api_request` 函数中实现拦截器模式：收到 401 → 获取写锁 → 二次校验 token 是否已变 → 未变则执行 refresh → 成功后重试原请求
  - 添加主动刷新定时器：在 `TokenState` 中记录 token 获取时间和过期时间，启动 tokio spawn 定时任务，在过期前 5 分钟主动刷新
  - 刷新失败处理：3 次指数退避重试 → 全部失败则通过 Tauri IPC 事件 `auth:logout` 通知 Vue 层
  - 添加 `auth:logout` 事件监听，前端收到后重定向到登录界面
  - 编写单元测试：模拟并发 5 个 401 请求，验证只触发 1 次 refresh

  **Must NOT do**:
  - 修改 Go 后端 JWT 生成逻辑（已有双 token 机制，无需改动）
  - 改变现有 token 过期时间（保持 Access 2h / Refresh 7d）
  - 在 Vue 层实现刷新逻辑（全部在 Rust 层）

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Rust 异步编程 + Arc<RwLock> 并发控制需要深度理解
  - **Skills**: [`coding-standards`]
    - `coding-standards`: Rust + Tauri 最佳实践

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 2)
  - **Blocks**: Task 3
  - **Blocked By**: None

  **References**:
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src-tauri\src\auth.rs` — 当前认证状态管理，包含 token 持久化、try_refresh_token、get_access_token 等方法
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src-tauri\src\api.rs:149-192` — 现有被动 401 刷新逻辑，需要升级为主动 + 并发安全
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\composables\useAuth.ts` — Vue 层认证 composable，需要添加 `auth:logout` 事件监听
  - `D:\dev\memory-stream\go-server\internal\handlers\auth.go` — Go 后端 `/auth/refresh` 端点，验证 refresh token 并返回新 token 对

  **Acceptance Criteria**:
  - [ ] `cargo test` 包含 `test_concurrent_refresh` 测试通过
  - [ ] 并发 5 个 401 请求只触发 1 次 refresh 调用
  - [ ] 定时器在 token 过期前 5 分钟主动刷新
  - [ ] 刷新失败 3 次后发送 `auth:logout` Tauri 事件

  **QA Scenarios**:
  ```
  Scenario: 并发请求触发 401 — 仅刷新一次
    Tool: cargo test
    Preconditions: auth.rs 中 mock refresh endpoint
    Steps:
      1. 设置 token 在 1 秒后过期
      2. 并发发送 5 个 API 请求
      3. 所有请求收到 401
      4. 验证 refresh endpoint 只被调用 1 次
      5. 验证所有 5 个请求用新 token 重试成功
    Expected Result: 5/5 请求成功，refresh 调用计数 = 1
    Evidence: .sisyphus/evidence/task-1-concurrent-refresh.txt

  Scenario: 刷新失败 — 退出登录通知
    Tool: cargo test
    Preconditions: refresh endpoint 返回 401
    Steps:
      1. 设置 token 已过期
      2. 发送 API 请求触发刷新
      3. 模拟 refresh 失败 3 次
      4. 验证 `auth:logout` 事件被 emit
    Expected Result: Tauri event "auth:logout" 已发送
    Evidence: .sisyphus/evidence/task-1-refresh-failure.txt
  ```

  **Commit**: YES
  - Message: `feat(jwt): implement proactive token refresh with concurrent safety`
  - Files: `src-tauri/src/auth.rs`, `src-tauri/src/api.rs`, `src/composables/useAuth.ts`

- [ ] 2. Categories 表迁移 — parent_id + sort_order + 环引用保护

  **What to do**:
  - 创建 SQL migration 文件 `migration/sql/007_category_hierarchy.sql`
  - 添加 `parent_id` 列（INTEGER REFERENCES categories(id) ON DELETE SET NULL, 默认 NULL）
  - 添加 `sort_order` 列（INTEGER DEFAULT 0）
  - 添加 CHECK 约束防止自引用：`CHECK (id != parent_id)`
  - 修改 Go model `Category` struct，添加 `ParentID *uint` 和 `SortOrder int` 字段
  - 更新 `@memory-stream/types` 的 `Category` 接口，确认已有 `parent_id`
  - 在 Go 后端 `category.go` service 中添加环引用验证函数：沿 parent_id 链向上遍历，确保不会形成环
  - 在 `CreateCategory` 和 `UpdateCategory` handler 中调用环引用验证
  - 在 `DeleteCategory` 中检查是否有子分类，有则返回错误
  - 添加深度限制验证：新分类深度不得超过 5 级

  **Must NOT do**:
  - 创建新的数据库表（只修改现有 categories 表）
  - 改变现有 API 响应结构（只新增 parent_id 和 sort_order 字段）
  - 破坏现有 useCategoryStore 的 CRUD API

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 主要是 SQL migration + Go struct 字段添加，模式清晰
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 1)
  - **Blocks**: Task 3, Task 4
  - **Blocked By**: None

  **References**:
  - `D:\dev\memory-stream\go-server\migration\sql\001_schema.sql:1-20` — 现有 categories 表定义，需要添加 parent_id 和 sort_order
  - `D:\dev\memory-stream\go-server\internal\models\schema.go` — Go GORM Category struct，需要添加 ParentID 和 SortOrder
  - `D:\dev\memory-stream\go-server\internal\services\category.go` — 现有 category CRUD 服务，需要添加环引用验证
  - `D:\dev\memory-stream\go-server\internal\handlers\category.go` — 现有 category handlers，需要添加验证调用
  - `D:\dev\memory-stream\frontend-workspace\packages\types\index.ts` — Category TypeScript 类型，确认 parent_id 字段

  **Acceptance Criteria**:
  - [ ] `migration/sql/007_category_hierarchy.sql` 存在且语法正确
  - [ ] `go test ./internal/services/ -run TestCircular` 通过
  - [ ] `go test ./internal/services/ -run TestDepthLimit` 通过
  - [ ] 删除有子分类的父分类返回 409 Conflict

  **QA Scenarios**:
  ```
  Scenario: 环引用检测 — 创建 A→B→A 循环
    Tool: go test
    Steps:
      1. 创建分类 A
      2. 创建分类 B (parent_id = A.id)
      3. 尝试更新 A (parent_id = B.id)
    Expected Result: 返回错误 "circular reference detected"
    Evidence: .sisyphus/evidence/task-2-circular-ref.txt

  Scenario: 深度限制 — 超过 5 级
    Tool: go test
    Steps:
      1. 创建 5 级嵌套分类链
      2. 尝试创建第 6 级
    Expected Result: 返回错误 "maximum depth exceeded"
    Evidence: .sisyphus/evidence/task-2-depth-limit.txt

  Scenario: 删除保护 — 父分类有子分类
    Tool: curl
    Steps:
      1. 创建父分类和子分类
      2. DELETE /api/v1/categories/{parent_id}
    Expected Result: HTTP 409, message "category has children"
    Evidence: .sisyphus/evidence/task-2-delete-protection.txt
  ```

  **Commit**: YES
  - Message: `feat(categories): add parent_id and circular reference validation`

- [ ] 3. Go CTE 递归查询 — 全量树形查询 API

  **What to do**:
  - 在 `CategoryService` 中新增 `GetTree()` 方法，使用 PostgreSQL CTE 递归查询返回树形结构
  - CTE 查询逻辑：从 parent_id IS NULL 的根节点开始，递归向下查找所有子节点，按 sort_order 排序
  - 返回结构为 `CategoryTreeNode`（包含 `children: CategoryTreeNode[]`），而非扁平列表
  - 在 `CategoryHandler` 中新增 `GET /api/v1/categories/tree` 端点，调用 `GetTree()` 返回树形 JSON
  - 在 `cmd/api/main.go` 中注册新路由 `categories.GET("/tree", categoryHandler.GetTree)`
  - 保留现有 `GET /api/v1/categories` 端点不变（向后兼容），新端点是附加的
  - 确保空数据库时返回空数组 `[]` 而非 null

  **Must NOT do**:
  - 修改现有 `ListAll()` 方法的返回结构（保持扁平列表）
  - 删除或替换现有 `/categories` 端点
  - 在 CTE 查询中返回卡片数量统计（明确排除）

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: PostgreSQL CTE 递归查询 + Go GORM 原生 SQL 需要深度理解
  - **Skills**: [`coding-standards`]
    - `coding-standards`: Go SQL 最佳实践，错误处理

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Task 4)
  - **Blocks**: Task 5, Task 6, Task 7
  - **Blocked By**: Task 1, Task 2

  **References**:
  - `D:\dev\memory-stream\go-server\internal\services\category.go:21-27` — 现有 `ListAll()` 方法，新的 `GetTree()` 方法需在同文件中新增
  - `D:\dev\memory-stream\go-server\internal\handlers\category.go:20-27` — 现有 `List` handler，新 `GetTree` handler 需参照此模式
  - `D:\dev\memory-stream\go-server\internal\models\schema.go:43-48` — Category struct，Task 2 添加 ParentID/SortOrder 后用于 CTE 查询结果映射
  - `D:\dev\memory-stream\go-server\cmd\api\main.go` — 路由注册位置，需要添加 `/tree` 端点
  - `D:\dev\memory-stream\frontend-workspace\packages\types\index.ts:107-118` — Category 接口定义，前端将直接使用 parent_id 构建树

  **Acceptance Criteria**:
  - [ ] `GET /api/v1/categories/tree` 返回嵌套 JSON 树结构
  - [ ] `GET /api/v1/categories` 仍返回扁平列表（向后兼容）
  - [ ] 空数据库时返回 `{"categories": []}`
  - [ ] 子节点按 `sort_order` 排序
  - [ ] `go test ./internal/services/ -run TestGetTree` 通过

  **QA Scenarios**:
  ```
  Scenario: 多级树形结构查询
    Tool: curl
    Preconditions: 数据库有 3 级嵌套分类（Root → Child → GrandChild）
    Steps:
      1. curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/categories/tree
      2. 解析 JSON 响应
      3. 验证根节点 children 非空
      4. 验证子节点 children 非空
      5. 验证叶子节点 children 为空数组
    Expected Result: HTTP 200, 嵌套结构正确，每层按 sort_order 排序
    Evidence: .sisyphus/evidence/task-3-tree-query.json

  Scenario: 空数据库返回空数组
    Tool: curl
    Preconditions: categories 表为空
    Steps:
      1. curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/categories/tree
      2. 验证响应体
    Expected Result: HTTP 200, {"categories": []}
    Evidence: .sisyphus/evidence/task-3-empty-tree.json
  ```

  **Commit**: YES
  - Message: `feat(categories): implement CTE recursive query for tree retrieval`
  - Files: `internal/services/category.go`, `internal/handlers/category.go`, `cmd/api/main.go`

- [ ] 4. 数据迁移脚本 — 现有分类 parent_id = NULL

  **What to do**:
  - 创建 SQL migration 文件 `migration/sql/008_category_data_migration.sql`
  - 内容：`UPDATE categories SET parent_id = NULL WHERE parent_id IS NULL;`（确保现有数据兼容）
  - 添加 `UPDATE categories SET sort_order = 0 WHERE sort_order IS NULL;`
  - 添加注释说明：此迁移确保所有现有分类默认为顶级分类
  - 如果 Task 2 的 migration 已包含 DEFAULT NULL / DEFAULT 0，此脚本作为幂等安全检查
  - 验证迁移可以重复执行不报错（幂等性）

  **Must NOT do**:
  - 删除或修改现有数据
  - 创建新的分类或修改分类名称
  - 在迁移中添加复杂业务逻辑

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 简单 SQL 迁移脚本，模式清晰
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Task 3)
  - **Blocks**: Task 5, Task 6
  - **Blocked By**: Task 2

  **References**:
  - `D:\dev\memory-stream\go-server\migration\sql\` — 现有迁移文件目录，了解命名规范
  - `D:\dev\memory-stream\go-server\migration\sql\001_schema.sql` — 初始 schema，了解 categories 表定义

  **Acceptance Criteria**:
  - [ ] `migration/sql/008_category_data_migration.sql` 存在且 SQL 语法正确
  - [ ] 迁移可以重复执行不报错
  - [ ] 现有分类的 parent_id 均为 NULL, sort_order 均为 0

  **QA Scenarios**:
  ```
  Scenario: 迁移脚本幂等执行
    Tool: psql
    Preconditions: PostgreSQL 运行中，categories 表有数据
    Steps:
      1. 执行 008_category_data_migration.sql
      2. 查询 SELECT COUNT(*) FROM categories WHERE parent_id IS NOT NULL
      3. 再次执行 008_category_data_migration.sql
    Expected Result: 两次执行均成功，COUNT = 0（现有分类无 parent）
    Evidence: .sisyphus/evidence/task-4-migration-idempotent.txt
  ```

  **Commit**: YES
  - Message: `chore(categories): migrate existing data to root level`
  - Files: `migration/sql/008_category_data_migration.sql`

- [ ] 5. CategoryTreeNode.vue — 递归树形组件

  **What to do**:
  - 创建 `frontend-workspace/apps/admin-tauri/src/components/CategoryTreeNode.vue`
  - 组件 props: `node: CategoryTreeNode`, `depth: number` (默认 0), `activeId: number | null`
  - 组件 emits: `select(id)`, `create-child(parentId)`, `delete(id, name)`, `edit(id, name)`
  - 实现递归渲染：每个节点显示名称 + 展开/折叠箭头（有子节点时） + 操作按钮（hover 时显示编辑/删除/添加子分类）
  - 层级缩进：每层 `padding-left: 20px`，最多 5 级
  - 展开/折叠动画：`<Transition name="tree-expand">` 高度过渡 200ms
  - 选中态：activeId 匹配时左边框 + 背景 `bg-neon/10`
  - 子节点递归调用 `<CategoryTreeNode>` 自身
  - 叶子节点无箭头，用圆点占位
  - 使用 Lucide icons: `ChevronRight`（折叠）、`ChevronDown`（展开）、`Plus`（添加子分类）、`Pencil`（编辑）、`Trash2`（删除）

  **Must NOT do**:
  - 实现拖拽排序功能
  - 添加分类图标/颜色选择器
  - 实现搜索/过滤功能
  - 修改 useCategoryStore 的现有 API

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Vue 递归组件 + 动画过渡 + Tailwind 样式需要视觉工程能力
  - **Skills**: [`frontend-design`]
    - `frontend-design`: 高质量 UI 组件设计，避免 AI 通用审美

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 6, 7)
  - **Blocks**: Task 6
  - **Blocked By**: Task 3, Task 4

  **References**:
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\components\LeftSidebar.vue:153-188` — 现有分类 tab 区域，了解当前分类列表的样式和交互模式（需被 CategoryPanel 替代）
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\stores\useCategoryStore.ts` — Category CRUD store，组件将通过此 store 操作数据
  - `D:\dev\memory-stream\frontend-workspace\packages\types\index.ts:107-118` — Category 接口，含 parent_id 字段
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\components\ConfirmDialog.vue` — 删除确认弹窗组件，删除操作需使用此组件

  **Acceptance Criteria**:
  - [ ] `CategoryTreeNode.vue` 文件存在且 `vue-tsc --noEmit` 无报错
  - [ ] 组件递归渲染 5 级深度树形结构正常
  - [ ] 展开/折叠动画流畅（< 200ms）
  - [ ] 选中态高亮正确
  - [ ] 操作按钮 hover 时显示，非 hover 时隐藏

  **QA Scenarios**:
  ```
  Scenario: 递归树形组件渲染 5 级深度
    Tool: interactive_bash (tmux)
    Preconditions: admin-tauri dev 模式运行中，数据库有 5 级嵌套分类
    Steps:
      1. 打开分类管理面板
      2. 逐级展开第 1 → 2 → 3 → 4 → 5 级节点
      3. 验证每级缩进增加 20px
      4. 验证叶子节点无展开箭头
    Expected Result: 5 级节点全部可见，缩进正确，无渲染错误
    Evidence: .sisyphus/evidence/task-5-tree-render.png

  Scenario: 节点操作（添加子分类/编辑/删除）
    Tool: interactive_bash (tmux)
    Preconditions: 树形组件已渲染
    Steps:
      1. hover 第 2 级节点，点击 "+" 添加子分类
      2. 输入名称 "测试子分类" 并确认
      3. hover 新创建的节点，点击编辑图标
      4. 修改名称为 "已修改" 并确认
      5. hover 该节点，点击删除图标
      6. 确认删除弹窗
    Expected Result: 添加/编辑/删除操作正常，树形结构实时更新
    Evidence: .sisyphus/evidence/task-5-node-ops.png
  ```

  **Commit**: YES
  - Message: `feat(ui): add recursive CategoryTreeNode component`
  - Files: `src/components/CategoryTreeNode.vue`

- [ ] 6. CategoryPanel.vue — 全屏覆盖式管理面板

  **What to do**:
  - 创建 `frontend-workspace/apps/admin-tauri/src/components/CategoryPanel.vue`
  - 面板形态：**全屏覆盖式**（类似 macOS 系统偏好设置），非抽屉/弹窗
  - 打开动画：主界面 `scale(0.95)` + `blur(4px)` + `opacity(0.7)` 退后，面板从中心展开覆盖
  - 关闭动画：反向恢复，300ms Expo-Out
  - 面板布局：**左右双栏 Split View**
    - 左栏（40% 宽度）：分类树形结构（使用 Task 5 的 CategoryTreeNode），顶部搜索栏（标题栏），底部"新建顶级分类"按钮
    - 右栏（60% 宽度）：选中分类的详情/编辑面板，显示名称、描述、子分类数量、创建时间
  - 关闭方式：左上角关闭按钮 + Esc 键 + 面板外点击（blur 区域）
  - 背景遮罩：`bg-black/60 backdrop-blur-sm`
  - 在 `App.vue` 或 `GlobalToolbar.vue` 中添加打开按钮（齿轮图标）
  - 面板打开/关闭状态可通过 `useLayoutStore` 管理（新增 `isCategoryPanelOpen` ref）

  **Must NOT do**:
  - 使用抽屉（Drawer）或弹窗（Modal）形态
  - 实现拖拽排序
  - 添加分类图标/颜色/统计
  - 修改 LeftSidebar.vue 的现有结构（本轮只添加面板，不动侧边栏）

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: 全屏覆盖面板 + Split View + 复杂动画过渡，需要高水准视觉工程
  - **Skills**: [`frontend-design`]
    - `frontend-design`: 创建高质量的全屏面板界面，避免通用 AI 审美

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 5, 7)
  - **Blocks**: None
  - **Blocked By**: Task 5 (需要 CategoryTreeNode 组件)

  **References**:
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\components\LeftSidebar.vue:105-280` — 现有侧边栏模板结构，了解当前布局和 z-index 层级
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\stores\useCategoryStore.ts` — 分类 CRUD store，面板通过此 store 操作数据
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\components\ConfirmDialog.vue` — 删除确认弹窗，面板中删除分类需调用
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\stores\layout.ts` — 布局 store，需添加 `isCategoryPanelOpen` 状态
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\components\GlobalToolbar.vue` — 全局工具栏，添加齿轮按钮入口

  **Acceptance Criteria**:
  - [ ] `CategoryPanel.vue` 文件存在且 `vue-tsc --noEmit` 无报错
  - [ ] 面板打开/关闭动画流畅（scale + blur + opacity 过渡 < 300ms）
  - [ ] 左栏树形结构渲染正确（使用 CategoryTreeNode）
  - [ ] 右栏显示选中分类详情
  - [ ] Esc 键和关闭按钮均可关闭面板
  - [ ] 面板打开时主界面退后效果正常

  **QA Scenarios**:
  ```
  Scenario: 面板打开/关闭动画 + 键盘交互
    Tool: interactive_bash (tmux) + playwright
    Preconditions: admin-tauri 运行中
    Steps:
      1. 点击 GlobalToolbar 齿轮按钮
      2. 验证主界面 scale(0.95) + blur 效果
      3. 验证面板从中心展开
      4. 按 Esc 键
      5. 验证面板关闭 + 主界面恢复
      6. 再次点击齿轮按钮打开
      7. 点击面板外 blur 区域
      8. 验证面板关闭
    Expected Result: 动画流畅 < 300ms，两种关闭方式均有效
    Evidence: .sisyphus/evidence/task-6-panel-anim.png

  Scenario: Split View 左右栏交互
    Tool: interactive_bash (tmux)
    Preconditions: 面板已打开，数据库有多级分类
    Steps:
      1. 在左栏点击第 2 级分类节点
      2. 验证右栏显示该分类详情（名称、描述、子分类数、时间）
      3. 在右栏修改分类名称为 "面板编辑测试"
      4. 点击保存
      5. 验证左栏树中名称同步更新
    Expected Result: 左右栏数据同步，编辑实时反映
    Evidence: .sisyphus/evidence/task-6-split-view.png
  ```

  **Commit**: YES
  - Message: `feat(ui): add full-screen CategoryPanel with split-view`
  - Files: `src/components/CategoryPanel.vue`, `src/stores/layout.ts`, `src/components/GlobalToolbar.vue`

- [ ] 7. Zod Schema 校验层 — 5 个 CRUD 端点

  **What to do**:
  - 在 `frontend-workspace/apps/admin-tauri/src/` 下创建 `schemas/` 目录
  - 创建 `schemas/category.ts`：定义 Category CRUD 的 Zod schema
    - `CategorySchema`: `z.object({ id: z.number(), name: z.string(), parent_id: z.number().nullable(), created_at: z.string(), updated_at: z.string() })`
    - `CreateCategoryRequestSchema`: `z.object({ name: z.string().min(1).max(100), description: z.string().optional(), parent_id: z.number().nullable().optional() })`
    - `UpdateCategoryRequestSchema`: 同 Create
    - `CategoryListResponseSchema`: `z.object({ categories: z.array(CategorySchema) })`
    - `CategoryTreeResponseSchema`: 递归 schema 使用 `z.lazy(() => z.object({ ... children: z.array(TreeNodeSchema) }))`
  - 在 `useCategoryStore.ts` 的 `loadCategories()` 中使用 Zod 解析 API 响应
  - 解析失败时 console.error + toast 提示，不阻断应用
  - 安装 `zod` 依赖（`pnpm --filter admin-tauri add zod`）

  **Must NOT do**:
  - 在 Go 后端添加 Zod（Go 端用原生 struct 验证）
  - 改变 API 响应结构
  - 将 Zod 校验用于表单输入（本轮只校验 API 响应）

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Zod schema 定义是模式清晰的类型安全工作
  - **Skills**: [`coding-standards`]
    - `coding-standards`: TypeScript 类型安全最佳实践

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 5, 6)
  - **Blocks**: None
  - **Blocked By**: Task 3 (需要了解 CTE 查询的返回结构来定义 TreeSchema)

  **References**:
  - `D:\dev\memory-stream\frontend-workspace\packages\types\index.ts:107-124` — Category 和 CreateCategoryPayload 接口，Zod schema 需与此对齐
  - `D:\dev\memory-stream\frontend-workspace\apps\admin-tauri\src\stores\useCategoryStore.ts:22-38` — `loadCategories()` 方法，需要在此添加 Zod 解析
  - `D:\dev\memory-stream\go-server\internal\handlers\category.go:20-27` — List 端点返回结构 `{ "categories": [...] }`，Zod schema 需匹配
  - Zod v4 docs: `https://zod.dev` — 递归 schema 使用 `z.lazy()`

  **Acceptance Criteria**:
  - [ ] `src/schemas/category.ts` 文件存在
  - [ ] `zod` 在 `admin-tauri/package.json` 的 dependencies 中
  - [ ] `vue-tsc --noEmit` 无报错
  - [ ] API 响应通过 Zod 解析后类型安全

  **QA Scenarios**:
  ```
  Scenario: Zod 校验正确解析 API 响应
    Tool: bun REPL
    Preconditions: schemas/category.ts 已创建
    Steps:
      1. import { CategoryListResponseSchema } from './src/schemas/category'
      2. const mockResponse = { categories: [{ id: 1, name: "test", parent_id: null, created_at: "2024-01-01", updated_at: "2024-01-01" }] }
      3. const result = CategoryListResponseSchema.parse(mockResponse)
      4. 验证 result.categories[0].name === "test"
    Expected Result: 解析成功，类型推断正确
    Evidence: .sisyphus/evidence/task-7-zod-parse.txt

  Scenario: Zod 校验拒绝非法数据
    Tool: bun REPL
    Steps:
      1. const badResponse = { categories: [{ id: "not-a-number" }] }
      2. try { CategoryListResponseSchema.parse(badResponse) } catch(e) { verify ZodError }
    Expected Result: 抛出 ZodError，包含字段级错误信息
    Evidence: .sisyphus/evidence/task-7-zod-reject.txt
  ```

  **Commit**: YES
  - Message: `feat(types): add Zod schemas for Category API response validation`
  - Files: `src/schemas/category.ts`, `src/stores/useCategoryStore.ts`, `package.json`

- [ ] 8. Vite 分包优化 — manualChunks + compression

  **What to do**:
  - 优化 `web-reader/vite.config.ts` 的 `manualChunks` 配置：
    - 将 `lucide-vue-next` 独立为 `vendor-icons` chunk（图标库体积较大，使用频率低）
    - 将 `axios` 合并到 `vendor-core`（HTTP 客户端是核心依赖）
    - 将 `graphology` + `dagre` + `potpack` 统一为 `vendor-graph`（已存在，确认包含完整）
    - 添加 `vendor-markdown` chunk：`@aspect-build/rules_js` 或其他 WASM 相关依赖（如有）
  - 验证 `vite-plugin-compression` 已正确配置 Gzip + Brotli（已在 Phase 1 安装和配置）
  - 在 `build.rollupOptions.output` 中添加 `chunkFileNames: 'assets/[name]-[hash].js'` 确保长期缓存
  - 运行 `pnpm --filter web-reader build` 验证产出，确认各 chunk 大小合理
  - 目标：`vendor-graph` chunk < 200KB（gzip 前）

  **Must NOT do**:
  - 修改 admin-tauri 的 vite.config.ts（本轮只优化 web-reader）
  - 删除现有的 vendor-core/vendor-graph/vendor-virtual 分组
  - 添加代码分割框架（如 React.lazy 等概念不适用）

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Vite 配置修改是模式清晰的构建优化
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Task 9)
  - **Blocks**: None
  - **Blocked By**: None

  **References**:
  - `D:\dev\memory-stream\frontend-workspace\apps\web-reader\vite.config.ts:1-44` — 当前 Vite 配置，包含已有 manualChunks 和 compression 插件
  - `D:\dev\memory-stream\frontend-workspace\apps\web-reader\package.json` — 依赖列表，确认需要分割的包名

  **Acceptance Criteria**:
  - [ ] `pnpm --filter web-reader build` 成功
  - [ ] `vendor-graph` chunk < 200KB（gzip 前）
  - [ ] `vendor-icons` 独立 chunk 已生成
  - [ ] `.gz` 和 `.br` 压缩文件均已生成

  **QA Scenarios**:
  ```
  Scenario: 构建产出验证
    Tool: bash
    Preconditions: web-reader 项目依赖已安装
    Steps:
      1. cd frontend-workspace && pnpm --filter web-reader build
      2. ls -la apps/web-reader/dist/assets/ | grep vendor
      3. 检查各 vendor chunk 大小
      4. 验证 .gz 和 .br 文件存在
    Expected Result: 构建成功，vendor-graph < 200KB，所有压缩文件存在
    Evidence: .sisyphus/evidence/task-8-build-output.txt

  Scenario: 开发模式不受影响
    Tool: bash
    Steps:
      1. pnpm --filter web-reader dev
      2. curl http://localhost:5173
      3. 验证页面正常加载
    Expected Result: HTTP 200，页面正常渲染
    Evidence: .sisyphus/evidence/task-8-dev-mode.txt
  ```

  **Commit**: YES
  - Message: `perf(bundle): optimize Vite code splitting and add vendor-icons chunk`
  - Files: `apps/web-reader/vite.config.ts`

- [ ] 9. Genesis Node 能量流动画 + StatsWidget 聚光灯联动

  **What to do**:
  - **Genesis Node 能量流动画增强**（ListView.vue）：
    - 替换现有 `genesis-breathe` 简单呼吸动画为能量流动画效果
    - 使用 CSS `@keyframes` + `conic-gradient` 旋转 + `drop-shadow` 脉冲，模拟能量从节点中心向外流动
    - 动画节奏：3s 循环，Expo-In-Out 缓动
    - hover 时加速为 1.5s（已有，保持）
    - 选中态：添加外环旋转光环（`border` + `animation: spin`）
    - 能量连接线：hover 时从节点到卡片的连接线添加 `translateX` 扫光动画（光从左到右扫过）
  - **StatsWidget 聚光灯联动**：
    - 在 StatsWidget 展开态中添加"聚光灯模式"按钮（仅 web-reader）
    - 点击后调用 `useGraphStore` 的 `toggleSpotlight()` 方法
    - 当聚光灯激活时，StatsWidget 的霓虹小球添加旋转光环效果
    - 当聚光灯激活时，节点数量显示为高亮计数（显示聚焦的节点数而非总数）

  **Must NOT do**:
  - 修改 Go 后端
  - 修改 admin-tauri 组件（本轮只增强 web-reader）
  - 添加新的全局状态（复用 useGraphStore）

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: CSS 动画 + 视觉效果设计需要前端视觉工程专长
  - **Skills**: [`frontend-design`]
    - `frontend-design`: 高质量动画设计，避免通用效果

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Task 8)
  - **Blocks**: None
  - **Blocked By**: None

  **References**:
  - `D:\dev\memory-stream\frontend-workspace\apps\web-reader\src\views\ListView.vue:302-323` — 现有 Genesis Node 呼吸动画 CSS（`genesis-breathe` keyframes），需要替换为能量流动画
  - `D:\dev\memory-stream\frontend-workspace\apps\web-reader\src\views\ListView.vue:196-207` — Genesis Node 模板结构（双环：外圈 border + 内圈 bg-neon），动画增强基于此结构
  - `D:\dev\memory-stream\frontend-workspace\apps\web-reader\src\views\ListView.vue:224-230` — 能量连接线模板，hover 时显示的渐变线条，需要添加扫光动画
  - `D:\dev\memory-stream\frontend-workspace\apps\web-reader\src\components\StatsWidget.vue:33-43` — StatsWidget 折叠态霓虹小球，需要添加聚光灯激活态效果
  - `D:\dev\memory-stream\frontend-workspace\apps\web-reader\src\store\useGraphStore.ts` — 全局状态 store，包含 `toggleSpotlight()` 和聚光灯相关状态

  **Acceptance Criteria**:
  - [ ] Genesis Node 能量流动画流畅（无卡顿，3s 循环）
  - [ ] hover 时动画加速到 1.5s
  - [ ] 选中态外环旋转光环正常
  - [ ] 能量连接线扫光动画正常
  - [ ] StatsWidget 聚光灯按钮可点击，激活/关闭状态正确
  - [ ] `vue-tsc --noEmit` 无报错

  **QA Scenarios**:
  ```
  Scenario: Genesis Node 能量流动画效果
    Tool: playwright
    Preconditions: web-reader dev 模式运行中，有卡片数据
    Steps:
      1. 导航到 ListView
      2. 找到 Genesis Node（日期首卡的双环节点）
      3. 截图验证动画帧（3s 内至少 2 个不同状态）
      4. hover 该节点
      5. 验证动画加速
    Expected Result: 能量流动画可见，hover 后节奏加快
    Evidence: .sisyphus/evidence/task-9-genesis-anim.png

  Scenario: StatsWidget 聚光灯联动
    Tool: playwright
    Preconditions: web-reader 运行中，有图谱数据
    Steps:
      1. 在 ListView 展开 StatsWidget
      2. 点击聚光灯按钮
      3. 验证 StatsWidget 霓虹小球显示旋转光环
      4. 点击关闭聚光灯
      5. 验证效果消失
    Expected Result: 聚光灯激活/关闭切换正常，视觉反馈明显
    Evidence: .sisyphus/evidence/task-9-spotlight-link.png
  ```

  **Commit**: YES
  - Message: `feat(ui): enhance Genesis Node energy flow + StatsWidget spotlight link`
  - Files: `apps/web-reader/src/views/ListView.vue`, `apps/web-reader/src/components/StatsWidget.vue`

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists. For each "Must NOT Have": search codebase for forbidden patterns. Check evidence files. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo test`, `go test ./...`, `vue-tsc --noEmit`. Review all changed files for: `as any`, empty catches, console.log in prod, unused imports. Check AI slop.
  Output: `Rust [PASS/FAIL] | Go [PASS/FAIL] | Vue [PASS/FAIL] | Files [N clean/N issues] | VERDICT`

- [ ] F3. **Real Manual QA** — `unspecified-high` (+ `playwright` skill if UI)
  Start from clean state. Execute EVERY QA scenario from EVERY task. Test cross-task integration. Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff. Verify 1:1. Check "Must NOT do" compliance. Detect cross-task contamination. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

| Wave | Commit Message | Files |
|------|---------------|-------|
| 1 | feat(jwt): implement proactive token refresh with concurrent safety | auth.rs, api.rs |
| 1 | feat(categories): add parent_id and circular reference validation | migration, category.go |
| 2 | feat(categories): implement CTE recursive query for tree retrieval | category.go, handlers |
| 2 | chore(categories): migrate existing data to root level | migration script |
| 3 | feat(ui): add recursive CategoryTreeNode component | CategoryTreeNode.vue |
| 3 | feat(ui): add full-screen CategoryPanel with split-view | CategoryPanel.vue, App.vue |
| 3 | feat(types): add Zod schemas for Category API | schemas/*.ts |
| 4 | perf(bundle): optimize Vite code splitting and compression | vite.config.ts |
| 4 | feat(ui): enhance Genesis Node energy flow + spotlight link | ListView.vue, StatsWidget.vue |

---

## Success Criteria

### Verification Commands
```bash
cargo test                                          # Expected: all pass
go test ./...                                        # Expected: all pass
cd frontend-workspace/apps/admin-tauri && vue-tsc --noEmit  # Expected: 0 errors
cd frontend-workspace/apps/web-reader && vue-tsc --noEmit    # Expected: 0 errors
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass (Rust + Go + Vue)
- [ ] Category panel opens/closes smoothly
- [ ] JWT refresh handles concurrent requests safely
- [ ] No circular references possible in category tree
