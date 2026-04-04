# 机械祭坛 — 工业控制台设计系统重构

## TL;DR

> **Quick Summary**: 将 Memory Stream 前端从"Web 网页感"全面重构为"机械祭坛"工业控制台美学：全局等宽字体、消灭 >4px 圆角、线框化图谱节点、终端风格输入框、霓虹发光交互态。
> 
> **Deliverables**:
> - admin-tauri 全部组件工业化改造（~11 文件）
> - web-reader 全部组件工业化改造（~13 文件）
> - Tailwind 设计 Token 统一升级（2 个 config）
> - 全局样式工业化（2 个 style.css）
> 
> **Estimated Effort**: Medium
> **Parallel Execution**: YES — 4 waves
> **Critical Path**: Token Foundation → Core Components → Web-Reader → Final QA

---

## Context

### Original Request
用户要求以"机械祭坛"为设计理念重新思考并重构整个 UI。此前用户已精准诊断出 4 个"打破沉浸感的罪魁祸首"：(1) 右侧图谱节点太圆太亮 (2) 顶部切换控件是胶囊形状 (3) 搜索框残留输入框错觉 (4) 内边距不统一。用户明确要求**设计系统级重构**，不满足于局部修补。

### Interview Summary
**Key Discussions**:
- **改造范围**: 用户选择"设计系统重构"而非"只改 4 个点"
- **圆角策略**: 全局规范，所有 >4px 圆角消灭（头像等极特殊状态除外）
- **交互态**: 霓虹发光（box-shadow），不用纯边框切换
- **字体策略**: 全局 monospace，整个界面统一等宽字体

**Research Findings**:
- ~28 个 rounded-* >4px 实例分布在 ~7 个文件
- admin-tauri 已有 23 处 font-mono 使用，body 字体需全局覆盖
- 设计 Token 完全在 tailwind.config.js（无独立 token 文件）
- VueFlow 节点使用**内联 style** 设置 borderRadius，需特殊处理
- RightAstrolabe.vue 节点 active 态为实心青色背景（#00e5ff），需改为线框
- web-reader CardNode.vue 有 scoped CSS `border-radius: 8px`

### Metis Review
**Identified Gaps** (addressed):
- VueFlow 连接 Handle 必须保持圆形（可用性）→ 保留 rounded-full
- 状态指示器（小圆点、spinner）保持圆形 → 保留 rounded-full
- 全局 mono 字体比 Inter 更宽，可能需要微调间距 → 加入验收标准
- 滚动条 thumb 保持胶囊形状（可用性）→ 保留 border-radius: 9999px
- 上下文菜单浮层可保留极小圆角（2px）用于 UX → 使用 rounded-sm
- Markdown 预览区代码块圆角→尖角 → 统一改为 2px

---

## Work Objectives

### Core Objective
将 Memory Stream 两个前端应用（admin-tauri + web-reader）的视觉语言从"现代 Web App"彻底转换为"工业控制台 / 机械祭坛"美学，通过设计 Token 统一、组件逐个改造、全局字体切换来实现。

### Concrete Deliverables
- 2 个 Tailwind config 新增工业设计 Token（borderRadius scale、neon glow shadow）
- 2 个 style.css 全局基底升级（font-mono、sharp corners）
- admin-tauri 8 个组件 Vue 文件全部工业化
- web-reader ~10 个组件 Vue 文件全部工业化
- 右侧图谱节点从实心胶囊→线框全息锁定框
- 顶部切换控件从圆角 Pill→硬切角矩阵分段器
- 左侧搜索框从常规输入框→终端风格底部线条
- 中心编辑区统一呼吸空间 p-6/p-8

### Definition of Done
- [ ] `pnpm --filter admin-tauri build` 零错误
- [ ] `pnpm --filter web-reader build` 零错误
- [ ] `grep -rn "rounded-lg\|rounded-xl\|rounded-2xl\|rounded-3xl"` 在 .vue 文件中返回 0 结果（排除 scrollbar、rounded-full 例外）
- [ ] 右侧图谱节点视觉：线框 + 等宽字体 + 透明底
- [ ] 顶部切换控件视觉：直角分段 + 霓虹指示线
- [ ] 左侧搜索框视觉：底部线条 + `>` 前缀
- [ ] 所有 UI 文字使用等宽字体

### Must Have
- 所有 UI 元素圆角 ≤4px（除明确例外列表）
- 全局 font-mono 字体族
- 图谱节点线框化（透明底 + 1px 边框 + 等宽文字）
- 切换控件硬切角（0px 或 2px 圆角）
- 搜索框终端化（底部线条 + `>` 前缀）
- 霓虹发光 hover/active 状态
- 统一内边距规范

### Must NOT Have (Guardrails)
- **不改变功能行为**：纯视觉重构，零逻辑变更
- **不创建过度抽象**：不搞 borderRadius 工具类封装，直接用 Tailwind 原生类
- **不添加新 Token**：除工业设计所需的 borderRadius scale 和 neon glow shadow 外
- **不改 Rust 后端**：不动 go-server/、rust-workspace/ 下的任何文件
- **不改 VueFlow Handle**：连接点保持圆形（可用性）
- **不改滚动条 thumb**：保持胶囊形状（可用性）
- **不改动 rounded-full 的小圆点**：状态指示器、spinner 保持圆形
- **不改第三方库覆盖**：只改项目自有样式
- **AI Slop 防御**：不加多余的注释、不做无关重构、不创建 README/文档

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: YES (Vue 3 + Vite build pipeline)
- **Automated tests**: None (pure visual refactor, no logic changes)
- **Framework**: Visual QA via Playwright + build verification + grep audit

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Visual**: Use Playwright (playwright skill) — Navigate, screenshot, assert computed styles
- **Build**: Use Bash — Run build commands, verify zero errors
- **Style Audit**: Use Bash (grep) — Verify no rounded-lg/xl/2xl/3xl remaining
- **Font Audit**: Use Playwright — Verify computed font-family is monospace

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — 4 parallel tasks, ~15 min each):
├── Task 1: admin-tauri Tailwind Token 升级 [quick]
├── Task 2: admin-tauri style.css 全局基底升级 [quick]
├── Task 3: web-reader Tailwind Token 升级 [quick]
└── Task 4: web-reader style.css 全局基底升级 [quick]

Wave 2 (Core Components — 6 parallel tasks after Wave 1):
├── Task 5: TheForge.vue 工业化改造 (depends: 1, 2) [visual-engineering]
├── Task 6: RightAstrolabe.vue 线框节点改造 (depends: 1, 2) [visual-engineering]
├── Task 7: LeftSidebar.vue 终端化改造 (depends: 1, 2) [quick]
├── Task 8: CategoryPanel.vue + CategoryTreeNode.vue 工业化 (depends: 1, 2) [quick]
├── Task 9: ConfirmDialog.vue + App.vue 工业化 (depends: 1, 2) [quick]
└── Task 10: GlobalToolbar.vue + TitleBar.vue 工业化 (depends: 1, 2) [quick]

Wave 3 (Web-Reader Components — 3 parallel tasks after Wave 3):
├── Task 11: web-reader/App.vue + DetailDrawer.vue 工业化 (depends: 3, 4) [quick]
├── Task 12: web-reader/LeftDock.vue + CommandPalette.vue 工业化 (depends: 3, 4) [quick]
└── Task 13: web-reader 其余组件工业化 (depends: 3, 4) [quick]

Wave FINAL (Verification — 4 parallel reviews):
├── Task F1: Plan Compliance Audit (oracle)
├── Task F2: Code Quality Review (unspecified-high)
├── Task F3: Visual QA — Playwright 全组件截图验证 (unspecified-high)
└── Task F4: Scope Fidelity Check (deep)

Critical Path: Task 1/2 → Task 5 → F1-F4 → user okay
Parallel Speedup: ~65% faster than sequential
Max Concurrent: 6 (Wave 2)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| 1 | — | 5-10 | 1 |
| 2 | — | 5-10 | 1 |
| 3 | — | 11-13 | 1 |
| 4 | — | 11-13 | 1 |
| 5 | 1, 2 | F1-F4 | 2 |
| 6 | 1, 2 | F1-F4 | 2 |
| 7 | 1, 2 | F1-F4 | 2 |
| 8 | 1, 2 | F1-F4 | 2 |
| 9 | 1, 2 | F1-F4 | 2 |
| 10 | 1, 2 | F1-F4 | 2 |
| 11 | 3, 4 | F1-F4 | 3 |
| 12 | 3, 4 | F1-F4 | 3 |
| 13 | 3, 4 | F1-F4 | 3 |
| F1-F4 | ALL | user okay | FINAL |

### Agent Dispatch Summary

- **Wave 1**: **4** — T1-T4 → `quick`
- **Wave 2**: **6** — T5-T6 → `visual-engineering`, T7-T10 → `quick`
- **Wave 3**: **3** — T11-T13 → `quick`
- **FINAL**: **4** — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high` (+ `playwright`), F4 → `deep`

---

## TODOs

- [ ] 1. admin-tauri Tailwind Token 升级

  **What to do**:
  - 在 `tailwind.config.js` 中新增工业设计 Token：
    - `borderRadius` 扩展：添加 `'sharp': '0px'`, `'industrial': '2px'`
    - `boxShadow` 扩展：添加 `'neon-glow': '0 0 8px rgba(0, 229, 255, 0.3), 0 0 20px rgba(0, 229, 255, 0.1)'`, `'neon-glow-sm': '0 0 4px rgba(0, 229, 255, 0.2)'`, `'neon-glow-lg': '0 0 12px rgba(0, 229, 255, 0.4), 0 0 30px rgba(0, 229, 255, 0.15)'`
    - `fontFamily` 更新：将 `body` 改为与 `mono` 相同的值 `['"JetBrains Mono"', '"Fira Code"', 'Consolas', 'Monaco', 'monospace']`
    - 保留 `display` 字体族不变（标题仍可用 display）
  - 不删除现有 Token，只新增和更新

  **Must NOT do**:
  - 不改动 colors 部分（已完美）
  - 不改 zIndex（已合理）
  - 不添加不需要的 Token（如 spacing、width 等）

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 单文件配置修改，范围明确
  - **Skills**: [`coding-standards`]
    - `coding-standards`: 确保 Tailwind 配置符合最佳实践
  - **Skills Evaluated but Omitted**:
    - `frontend-design`: 不涉及 UI 设计，只是配置

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3, 4)
  - **Blocks**: Tasks 5-10
  - **Blocked By**: None (can start immediately)

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/admin-tauri/tailwind.config.js` — 当前完整配置，包含所有颜色 Token、fontFamily、zIndex。理解当前结构以正确扩展

  **WHY Each Reference Matters**:
  - tailwind.config.js: 这是唯一的修改目标，需要理解现有结构以避免覆盖

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Tailwind 配置文件包含新的工业设计 Token
    Tool: Bash (grep)
    Preconditions: 文件已修改
    Steps:
      1. grep "sharp.*0px\|industrial.*2px" frontend-workspace/apps/admin-tauri/tailwind.config.js
      2. grep "neon-glow" frontend-workspace/apps/admin-tauri/tailwind.config.js
      3. grep "fontFamily" -A 4 frontend-workspace/apps/admin-tauri/tailwind.config.js | grep "JetBrains Mono"
    Expected Result: 三个 grep 均有匹配输出
    Failure Indicators: 任何一个 grep 返回空
    Evidence: .sisyphus/evidence/task-1-tailwind-tokens.txt

  Scenario: 构建成功验证
    Tool: Bash
    Preconditions: Token 已添加
    Steps:
      1. cd frontend-workspace && pnpm --filter admin-tauri build
    Expected Result: 构建成功，零错误
    Failure Indicators: 构建失败或报错
    Evidence: .sisyphus/evidence/task-1-build.txt
  ```

  **Commit**: NO (groups with Wave 1)
  - Message: `refactor(design): industrial-altar design tokens and global base styles`
  - Files: `tailwind.config.js`, `style.css`

- [ ] 2. admin-tauri style.css 全局基底升级

  **What to do**:
  - 在 `src/style.css` 中进行以下修改：
    - `@layer base` body 样式：`font-body` → `font-mono`（全局等宽字体）
    - `.preview-content code`：`rounded` → `rounded-sm`（代码内联元素）
    - `.preview-content pre`：`rounded-lg` → `rounded-sm`（代码块）
    - `.vue-flow__controls`：`border-radius: 4px` 保持（已 ≤4px）
    - 滚动条 thumb：保持 `border-radius: 9999px`（可用性例外）
  - 添加新的 `@layer components` 工具类：
    - `.neon-glow-hover:hover` — 霓虹发光 hover 效果
    - `.terminal-input` — 终端风格输入框（底部线条 + `>` 前缀样式）

  **Must NOT do**:
  - 不改滚动条 thumb 圆角（用户体验）
  - 不删 grid-texture / seam 类（核心视觉）
  - 不添加过多工具类（只加 2 个必要的）

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 单文件样式修改，范围明确
  - **Skills**: [`coding-standards`]
    - `coding-standards`: CSS 最佳实践
  - **Skills Evaluated but Omitted**:
    - `frontend-design`: 不涉及 UI 设计，只是样式更新

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 3, 4)
  - **Blocks**: Tasks 5-10
  - **Blocked By**: None (can start immediately)

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/admin-tauri/src/style.css` — 当前完整样式，包含 base layer、components layer、VueFlow 覆盖、滚动条。需要修改 rounded 值和 font

  **WHY Each Reference Matters**:
  - style.css: 唯一修改目标，需理解 layer 结构以正确插入新工具类

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: 全局字体已切换为等宽
    Tool: Bash (grep)
    Steps:
      1. grep "font-mono" frontend-workspace/apps/admin-tauri/src/style.css
    Expected Result: body 基础样式中使用 font-mono
    Evidence: .sisyphus/evidence/task-2-style-base.txt

  Scenario: rounded-lg 已从 style.css 中移除
    Tool: Bash (grep)
    Steps:
      1. grep "rounded-lg" frontend-workspace/apps/admin-tauri/src/style.css
    Expected Result: 0 匹配（代码块已改为 rounded-sm）
    Evidence: .sisyphus/evidence/task-2-no-rounded-lg.txt

  Scenario: 构建成功
    Tool: Bash
    Steps:
      1. cd frontend-workspace && pnpm --filter admin-tauri build
    Expected Result: 零错误
    Evidence: .sisyphus/evidence/task-2-build.txt
  ```

  **Commit**: NO (groups with Wave 1)

- [ ] 3. web-reader Tailwind Token 升级

  **What to do**:
  - 在 `web-reader/tailwind.config.js` 中同步 admin-tauri 的工业设计 Token：
    - `borderRadius` 扩展：`'sharp': '0px'`, `'industrial': '2px'`
    - `boxShadow` 扩展：与 admin-tauri 相同的 neon-glow 系列
    - `fontFamily` 更新：`mono` 改为 `['"JetBrains Mono"', '"Fira Code"', 'Consolas', 'Monaco', 'monospace']`
    - 添加 `body` 字体族（与 mono 相同）
    - 添加 `display` 字体族（与 admin-tauri 一致）
  - 添加缺失的颜色 Token（与 admin-tauri 对齐）：`ms-void`, `ms-carbon`, `ms-border-light`, `ms-border-active`

  **Must NOT do**:
  - 不删除 graph-specific colors（ms-spine 等）
  - 不改 content 路径

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 单文件配置修改
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2, 4)
  - **Blocks**: Tasks 11-13
  - **Blocked By**: None (can start immediately)

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/web-reader/tailwind.config.js` — 当前配置，需对齐 admin-tauri Token
  - `frontend-workspace/apps/admin-tauri/tailwind.config.js` — 参考源，复制 Token 定义

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Token 与 admin-tauri 一致
    Tool: Bash (diff)
    Steps:
      1. diff <(grep -A 20 "borderRadius\|boxShadow\|fontFamily" frontend-workspace/apps/admin-tauri/tailwind.config.js) <(grep -A 20 "borderRadius\|boxShadow\|fontFamily" frontend-workspace/apps/web-reader/tailwind.config.js)
    Expected Result: borderRadius/boxShadow/fontFamily 部分一致
    Evidence: .sisyphus/evidence/task-3-token-sync.txt

  Scenario: 构建成功
    Tool: Bash
    Steps:
      1. cd frontend-workspace && pnpm --filter web-reader build
    Expected Result: 零错误
    Evidence: .sisyphus/evidence/task-3-build.txt
  ```

  **Commit**: NO (groups with Wave 1)

- [ ] 4. web-reader style.css 全局基底升级

  **What to do**:
  - 在 `web-reader/src/style.css` 中进行以下修改：
    - `.prose code`：`border-radius: 4px` → `border-radius: 2px`
    - `.prose pre`：`border-radius: 8px` → `border-radius: 2px`
    - `.prose img`：`border-radius: 8px` → `border-radius: 2px`
    - `body` 字体：`-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif` → `"JetBrains Mono", "Fira Code", Consolas, Monaco, monospace`
    - 添加与 admin-tauri 相同的 `@layer components` 工具类（`.neon-glow-hover:hover`, `.terminal-input`）
  - 滚动条 thumb：保持胶囊形状

  **Must NOT do**:
  - 不改 prose 排版层级（字体大小、行高）
  - 不改滚动条
  - 不删任何现有样式规则

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 单文件样式修改
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2, 3)
  - **Blocks**: Tasks 11-13
  - **Blocked By**: None (can start immediately)

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/web-reader/src/style.css` — 当前样式，需修改 rounded 和 font
  - `frontend-workspace/apps/admin-tauri/src/style.css` — 参考源，复制工具类定义

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: body 字体已切换
    Tool: Bash (grep)
    Steps:
      1. grep "JetBrains Mono\|monospace" frontend-workspace/apps/web-reader/src/style.css
    Expected Result: body 选择器使用 monospace 字体族
    Evidence: .sisyphus/evidence/task-4-font-switch.txt

  Scenario: 无 >4px border-radius
    Tool: Bash (grep)
    Steps:
      1. grep "border-radius: [5-9]px\|border-radius: [0-9][0-9]px" frontend-workspace/apps/web-reader/src/style.css
    Expected Result: 0 匹配
    Evidence: .sisyphus/evidence/task-4-no-large-radius.txt

  Scenario: 构建成功
    Tool: Bash
    Steps:
      1. cd frontend-workspace && pnpm --filter web-reader build
    Expected Result: 零错误
    Evidence: .sisyphus/evidence/task-4-build.txt
  ```

  **Commit**: NO (groups with Wave 1)

- [ ] 5. TheForge.vue 工业化改造

  **What to do**:
  - **View Mode Toggle（第 266-273 行）**：从圆角 Pill 改为硬切角矩阵分段器
    - 外层容器：`bg-ms-deep rounded-lg p-0.5` → `border border-ms-border rounded-none p-0`
    - 选中态按钮：`bg-ms-surface text-neon rounded-md` → `text-neon border-b-2 border-neon bg-transparent rounded-none`
    - 未选中态：保持 `text-slate-500 hover:text-slate-300`，去掉圆角
  - **Save Button（第 280-287 行）**：`rounded-lg` → `rounded-sm`
  - **未保存标记（第 247 行）**：`rounded-full` 保持（这是状态指示小点，属于例外）
  - **空状态十字准星（第 381-397 行）**：
    - 外圈 `rounded-full` → `rounded-none`（改为方形准星框）
    - 中心点 `rounded-full` 保持（小圆点例外）
  - **编辑区内容 padding**：
    - 标题输入框（第 305 行）：`px-8 pt-6` → `px-6 pt-8`（统一呼吸空间）
    - Textarea（第 308 行）：`px-8 py-4` → `px-6 py-6`
    - 预览区（第 314 行）：`px-8 pt-6 pb-2` → `px-6 pt-8 pb-2`
    - 预览内容（第 319 行）：`px-8 py-4` → `px-6 py-6`
  - **搜索输入框（第 357 行，添加关系面板）**：`rounded` → `rounded-sm`
  - **卡片列表按钮（第 366 行）**：`rounded` → `rounded-sm`
  - **Select 下拉框（第 260 行）**：`rounded` → `rounded-sm`
  - **Save neon pulse 动画（第 459 行 CSS）**：`border-radius: 12px` → `border-radius: 4px`

  **Must NOT do**:
  - 不改功能逻辑（保存、渲染、快捷键等）
  - 不改 Transition 动画名称和时序
  - 不改 v-model 数据绑定
  - 不动 script setup 部分

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: 核心编辑器组件，涉及视觉设计决策（分段器样式、准星形态）
  - **Skills**: [`frontend-design`, `coding-standards`]
    - `frontend-design`: 分段器和准星的工业美学设计
    - `coding-standards`: Vue 模板最佳实践

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 6, 7, 8, 9, 10)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 1, 2

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/admin-tauri/src/components/TheForge.vue` — 完整 487 行，核心编辑器组件。所有需修改的行号已在上方列出
  - `frontend-workspace/apps/admin-tauri/src/components/GlobalToolbar.vue:52-72` — `.console-btn::before` 霓虹指示线模式，View Mode Toggle 的选中态可以参考这种底部指示线风格

  **WHY Each Reference Matters**:
  - TheForge.vue: 唯一修改目标，包含所有需改的元素
  - GlobalToolbar.vue: 提供霓虹指示线的成熟模式（左侧 2px 发光线），可复用到分段器的底部指示线

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: View Mode Toggle 是硬切角分段器
    Tool: Bash (grep)
    Steps:
      1. grep "rounded-lg\|rounded-md" frontend-workspace/apps/admin-tauri/src/components/TheForge.vue
    Expected Result: 0 匹配
    Failure Indicators: 仍有 rounded-lg 或 rounded-md
    Evidence: .sisyphus/evidence/task-5-no-soft-corners.txt

  Scenario: 编辑区 padding 统一为 p-6
    Tool: Bash (grep)
    Steps:
      1. grep "px-8\|px-6" frontend-workspace/apps/admin-tauri/src/components/TheForge.vue
    Expected Result: 所有内容区 padding 使用 px-6，无 px-8
    Evidence: .sisyphus/evidence/task-5-padding.txt

  Scenario: 构建成功
    Tool: Bash
    Steps:
      1. cd frontend-workspace && pnpm --filter admin-tauri build
    Expected Result: 零错误
    Evidence: .sisyphus/evidence/task-5-build.txt
  ```

  **Commit**: NO (groups with Wave 2)
  - Message: `refactor(design): admin-tauri components industrial console conversion`

- [ ] 6. RightAstrolabe.vue 线框节点改造

  **What to do**:
  - **Active 节点样式（第 90-98 行内联 style）**：从实心胶囊改为线框全息锁定框
    - `background: "#00e5ff"` → `background: "transparent"` 或 `"rgba(0, 229, 255, 0.05)"`
    - `color: "#0d0d0d"` → `color: "#00e5ff"`（文字改为霓虹色）
    - `fontWeight: "bold"` 保持
    - `borderRadius: "6px"` → `borderRadius: "2px"`
    - 添加 `border: "1px solid #00e5ff"`
    - 添加 `boxShadow: "0 0 8px rgba(0, 229, 255, 0.2)"`（霓虹发光）
    - 添加 `fontFamily: '"JetBrains Mono", "Fira Code", Consolas, monospace'`
  - **Inactive 节点样式（第 100-108 行内联 style）**：
    - `background: "#2a2a2a"` → `background: "transparent"` 或 `"rgba(42, 42, 42, 0.3)"`
    - `borderRadius: "6px"` → `borderRadius: "2px"`
    - `border: "1px solid #333333"` → `border: "1px solid #2a2a2a"`
    - 添加 `fontFamily: '"JetBrains Mono", "Fira Code", Consolas, monospace'`
  - **Header 中选中连线标记（第 249 行）**：`rounded-full` → `rounded-sm`
  - **召唤按钮（第 268 行）**：`rounded-md` → `rounded-sm`
  - **召唤弹出面板（第 275 行）**：`rounded-lg` → `rounded-sm`
  - **召唤搜索框（第 277 行）**：`rounded` → `rounded-sm`
  - **召唤卡片按钮（第 280 行）**：`rounded` → `rounded-sm`
  - **新建按钮（第 289 行）**：`rounded-md` → `rounded-sm`
  - **Context Menu（第 301 行）**：`rounded-lg` → `rounded-sm`
  - **Context Menu 中小圆点（第 306, 312 行）**：`rounded-full` 保持（状态指示例外）

  **Must NOT do**:
  - 不改 VueFlow 交互逻辑（drag, connect, select）
  - 不改 dagre layout 配置
  - 不改边（edges）样式（已经合理）
  - 不改 Handle 组件（连接点必须保持圆形）
  - 不动 script setup 中的数据处理逻辑

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: 图谱节点是核心视觉元素，需要精心设计线框全息风格
  - **Skills**: [`frontend-design`, `coding-standards`]
    - `frontend-design`: 线框全息节点的美学设计
    - `coding-standards`: Vue 内联样式最佳实践

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 7, 8, 9, 10)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 1, 2

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/admin-tauri/src/components/RightAstrolabe.vue` — 完整 327 行，图谱面板组件。所有需修改的行号和当前值已在上方列出
  - `frontend-workspace/apps/admin-tauri/src/style.css:106-129` — VueFlow 暗色覆盖样式，需确保节点新样式与这些覆盖兼容

  **WHY Each Reference Matters**:
  - RightAstrolabe.vue: 唯一修改目标
  - style.css VueFlow 覆盖: 确保节点样式与全局 VueFlow 暗色主题不冲突

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Active 节点是线框风格
    Tool: Bash (grep)
    Steps:
      1. grep 'background.*#00e5ff' frontend-workspace/apps/admin-tauri/src/components/RightAstrolabe.vue
      2. grep 'borderRadius.*6px' frontend-workspace/apps/admin-tauri/src/components/RightAstrolabe.vue
    Expected Result: grep 1 返回空（无实心青色背景），grep 2 返回空（无 6px 圆角）
    Failure Indicators: 仍有实心背景或大圆角
    Evidence: .sisyphus/evidence/task-6-wireframe-nodes.txt

  Scenario: 无 rounded-lg/xl/2xl
    Tool: Bash (grep)
    Steps:
      1. grep "rounded-lg\|rounded-xl\|rounded-2xl" frontend-workspace/apps/admin-tauri/src/components/RightAstrolabe.vue
    Expected Result: 0 匹配
    Evidence: .sisyphus/evidence/task-6-no-soft-corners.txt

  Scenario: 构建成功
    Tool: Bash
    Steps:
      1. cd frontend-workspace && pnpm --filter admin-tauri build
    Expected Result: 零错误
    Evidence: .sisyphus/evidence/task-6-build.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 7. LeftSidebar.vue 终端化改造

  **What to do**:
  - **搜索输入框（第 89-91 行）**：终端化改造
    - 外层 div：保持 `border-b border-ms-border` 容器
    - input 样式：移除 `border border-ms-border rounded-sm`，改为 `border-0 border-b border-ms-border bg-transparent rounded-none`
    - 添加 `>` 前缀：在 input 外层 div 中，input 之前添加 `<span class="text-slate-600 font-mono text-xs mr-1.5 select-none">&gt;</span>`
    - placeholder 从 `"搜索... (Ctrl+K)"` → `"search..."`
    - 保持 `font-mono text-xs`
  - **View Tabs（第 95-103 行）**：已经使用 `border-b border-neon/50` 底部指示线模式，保持不变
  - **Card Items（第 109 行）**：已经使用 `rounded-sm`，保持不变
  - **Category Badge（第 147 行）**：已经使用 `rounded-sm`，保持不变

  **Must NOT do**:
  - 不改卡片列表逻辑（过滤、排序、选中）
  - 不改删除功能
  - 不改数据绑定
  - 不添加新的 UI 元素（除了 `>` 前缀）

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 改动小且明确，主要是搜索框终端化
  - **Skills**: [`coding-standards`]
    - `coding-standards`: 确保改动不影响无障碍

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 6, 8, 9, 10)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 1, 2

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/admin-tauri/src/components/LeftSidebar.vue` — 完整 186 行，左侧卡片库。需修改搜索框样式

  **WHY Each Reference Matters**:
  - LeftSidebar.vue: 唯一修改目标

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: 搜索框是终端风格
    Tool: Bash (grep)
    Steps:
      1. grep "border-0\|border-b border-ms-border" frontend-workspace/apps/admin-tauri/src/components/LeftSidebar.vue
    Expected Result: input 使用底部线条样式，无全包围边框
    Evidence: .sisyphus/evidence/task-7-terminal-input.txt

  Scenario: 搜索框有 > 前缀
    Tool: Bash (grep)
    Steps:
      1. grep "&gt;\|>`" frontend-workspace/apps/admin-tauri/src/components/LeftSidebar.vue
    Expected Result: 存在 > 前缀标记
    Evidence: .sisyphus/evidence/task-7-prefix.txt

  Scenario: 构建成功
    Tool: Bash
    Steps:
      1. cd frontend-workspace && pnpm --filter admin-tauri build
    Expected Result: 零错误
    Evidence: .sisyphus/evidence/task-7-build.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 8. CategoryPanel.vue + CategoryTreeNode.vue 工业化

  **What to do**:
  - **CategoryPanel.vue 面板容器（第 145 行）**：`rounded-2xl` → `rounded-sm`
  - **关闭按钮（第 155 行）**：`rounded-lg` → `rounded-sm`
  - **新建分类输入框（第 172 行）**：`rounded-lg` → `rounded-sm`
  - **新建按钮（第 178 行）**：`rounded-lg` → `rounded-sm`
  - **子分类提示（第 188 行）**：`rounded-lg` → `rounded-sm`
  - **分类名称输入框（第 242 行）**：`rounded-lg` → `rounded-sm`
  - **分类描述 textarea（第 255 行）**：`rounded-lg` → `rounded-sm`
  - **保存按钮（第 264 行）**：`rounded-lg` → `rounded-sm`
  - **删除按钮（第 275 行）**：`rounded-lg` → `rounded-sm`
  - **CategoryTreeNode.vue 节点行（第 62 行）**：`rounded` → `rounded-sm`
  - **操作按钮（第 98, 104, 110 行）**：`rounded` → `rounded-sm`
  - **小圆点指示器（第 83 行）**：`rounded-full` → `rounded-none`（改为小方形指示器，匹配工业风）

  **Must NOT do**:
  - 不改分类 CRUD 逻辑
  - 不改树形递归组件结构
  - 不改 panel 动画

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 批量 rounded 替换，范围明确
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 6, 7, 9, 10)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 1, 2

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/admin-tauri/src/components/CategoryPanel.vue` — 完整 352 行，分类管理面板
  - `frontend-workspace/apps/admin-tauri/src/components/CategoryTreeNode.vue` — 完整 158 行，递归树节点

  **WHY Each Reference Matters**:
  - CategoryPanel.vue: 包含最多 rounded-lg 实例（9 处），需全部替换
  - CategoryTreeNode.vue: 包含 rounded-full 小圆点需改为方形

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: 无 rounded-lg/xl/2xl
    Tool: Bash (grep)
    Steps:
      1. grep "rounded-lg\|rounded-xl\|rounded-2xl" frontend-workspace/apps/admin-tauri/src/components/CategoryPanel.vue frontend-workspace/apps/admin-tauri/src/components/CategoryTreeNode.vue
    Expected Result: 0 匹配
    Evidence: .sisyphus/evidence/task-8-no-soft-corners.txt

  Scenario: 构建成功
    Tool: Bash
    Steps:
      1. cd frontend-workspace && pnpm --filter admin-tauri build
    Expected Result: 零错误
    Evidence: .sisyphus/evidence/task-8-build.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 9. ConfirmDialog.vue + App.vue 工业化

  **What to do**:
  - **ConfirmDialog.vue 对话卡片（第 59 行）**：`rounded-xl` → `rounded-sm`
  - **取消按钮（第 73 行）**：`rounded-lg` → `rounded-sm`
  - **确认按钮（第 76 行）**：`rounded-lg` → `rounded-sm`
  - **App.vue 加载 spinner（第 31 行）**：`rounded-full` 保持（spinner 是旋转动画，必须圆形）
  - **App.vue Toast 通知（第 82 行）**：`rounded-sm` 已是，保持
  - **App.vue Toast 小圆点（第 88 行）**：`rounded-full` 保持（状态指示例外）

  **Must NOT do**:
  - 不改焦点管理（focus trap）
  - 不改键盘导航
  - 不改 toast 数据绑定
  - 不改 Transition 动画

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 少量 rounded 替换
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 6, 7, 8, 10)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 1, 2

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/admin-tauri/src/components/ConfirmDialog.vue` — 完整 102 行
  - `frontend-workspace/apps/admin-tauri/src/App.vue` — 完整 125 行

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: 无 rounded-xl/lg
    Tool: Bash (grep)
    Steps:
      1. grep "rounded-xl\|rounded-lg" frontend-workspace/apps/admin-tauri/src/components/ConfirmDialog.vue
    Expected Result: 0 匹配
    Evidence: .sisyphus/evidence/task-9-dialog-sharp.txt

  Scenario: 构建成功
    Tool: Bash
    Steps:
      1. cd frontend-workspace && pnpm --filter admin-tauri build
    Expected Result: 零错误
    Evidence: .sisyphus/evidence/task-9-build.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 10. GlobalToolbar.vue + TitleBar.vue 工业化

  **What to do**:
  - **GlobalToolbar.vue**：审查所有样式
    - `.console-btn`（第 45 行）：已使用 `rounded-sm`，保持
    - `.console-btn::before`（第 84 行）：`border-radius: 4px` → `border-radius: 2px`
    - `.console-btn-create`（第 75 行）：已使用 `rounded-sm`，保持
    - `.console-btn-create::before`（第 87 行）：`border-radius: 4px` → `border-radius: 2px`
  - **TitleBar.vue**：审查所有样式
    - 已无 rounded 类，保持不变
    - 确认无其他圆角残留

  **Must NOT do**:
  - 不改霓虹指示线动画效果（核心视觉特征）
  - 不改呼吸动画
  - 不改窗口控制按钮逻辑

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 极少改动，只是 CSS 微调
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 6, 7, 8, 9)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 1, 2

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/admin-tauri/src/components/GlobalToolbar.vue` — 完整 101 行，左侧工具栏
  - `frontend-workspace/apps/admin-tauri/src/components/TitleBar.vue` — 完整 53 行，标题栏

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: 内部元素 border-radius ≤2px
    Tool: Bash (grep)
    Steps:
      1. grep "border-radius.*[3-9]px\|border-radius.*[0-9][0-9]px" frontend-workspace/apps/admin-tauri/src/components/GlobalToolbar.vue
    Expected Result: 0 匹配
    Evidence: .sisyphus/evidence/task-10-toolbar-sharp.txt

  Scenario: 构建成功
    Tool: Bash
    Steps:
      1. cd frontend-workspace && pnpm --filter admin-tauri build
    Expected Result: 零错误
    Evidence: .sisyphus/evidence/task-10-build.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 11. web-reader/App.vue + DetailDrawer.vue 工业化

  **What to do**:
  - **App.vue**：
    - `font-sans`（第 61 行）→ `font-mono`（全局等宽字体）
    - Skip to content 按钮（第 65 行）：`focus:rounded-lg` → `focus:rounded-sm`
    - 错误边界图标容器（第 73 行）：`rounded-full` → `rounded-none`（方形错误图标框）
    - 错误恢复按钮（第 83 行）：`rounded-lg` → `rounded-sm`
  - **DetailDrawer.vue**：查找并替换所有 `rounded-lg/xl/2xl` → `rounded-sm`，`rounded-full`（小圆点）保持

  **Must NOT do**:
  - 不改视图切换逻辑
  - 不改错误边界恢复逻辑
  - 不改 DetailDrawer 滑动关闭手势

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 批量 rounded + font 替换
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 12, 13)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 3, 4

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/web-reader/src/App.vue` — 完整 132 行
  - `frontend-workspace/apps/web-reader/src/components/DetailDrawer.vue` — 阅读抽屉组件

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: font-sans 已移除
    Tool: Bash (grep)
    Steps:
      1. grep "font-sans" frontend-workspace/apps/web-reader/src/App.vue
    Expected Result: 0 匹配
    Evidence: .sisyphus/evidence/task-11-no-font-sans.txt

  Scenario: 无 rounded-lg/xl/2xl
    Tool: Bash (grep)
    Steps:
      1. grep "rounded-lg\|rounded-xl\|rounded-2xl" frontend-workspace/apps/web-reader/src/App.vue frontend-workspace/apps/web-reader/src/components/DetailDrawer.vue
    Expected Result: 0 匹配
    Evidence: .sisyphus/evidence/task-11-no-soft-corners.txt

  Scenario: 构建成功
    Tool: Bash
    Steps:
      1. cd frontend-workspace && pnpm --filter web-reader build
    Expected Result: 零错误
    Evidence: .sisyphus/evidence/task-11-build.txt
  ```

  **Commit**: NO (groups with Wave 3)

- [ ] 12. web-reader/LeftDock.vue + CommandPalette.vue 工业化

  **What to do**:
  - **LeftDock.vue**：查找并替换所有 `rounded-lg/xl/2xl` → `rounded-sm`，`rounded-full`（小圆点、状态灯）保持
  - **CommandPalette.vue**：查找并替换所有 `rounded-lg/xl/2xl` → `rounded-sm`，`rounded-full`（spinner）保持
  - 检查内联 style 中的 `borderRadius` 值，确保 ≤4px

  **Must NOT do**:
  - 不改键盘导航逻辑
  - 不改搜索功能
  - 不改 WebSocket 状态显示

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 批量 rounded 替换
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 11, 13)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 3, 4

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/web-reader/src/components/LeftDock.vue` — 左侧导航中枢
  - `frontend-workspace/apps/web-reader/src/components/CommandPalette.vue` — Cmd+K 搜索面板

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: 无 rounded-lg/xl/2xl
    Tool: Bash (grep)
    Steps:
      1. grep "rounded-lg\|rounded-xl\|rounded-2xl" frontend-workspace/apps/web-reader/src/components/LeftDock.vue frontend-workspace/apps/web-reader/src/components/CommandPalette.vue
    Expected Result: 0 匹配
    Evidence: .sisyphus/evidence/task-12-no-soft-corners.txt

  Scenario: 构建成功
    Tool: Bash
    Steps:
      1. cd frontend-workspace && pnpm --filter web-reader build
    Expected Result: 零错误
    Evidence: .sisyphus/evidence/task-12-build.txt
  ```

  **Commit**: NO (groups with Wave 3)

- [ ] 13. web-reader 其余组件工业化

  **What to do**:
  - 对以下每个文件执行相同操作：查找并替换所有 `rounded-lg/xl/2xl` → `rounded-sm`，内联 `borderRadius > 4px` → `2px`，`rounded-full`（spinner、小圆点）保持
  - 文件列表：
    - `StatsWidget.vue`
    - `ZenReader.vue`
    - `EntranceAnimation.vue`
    - `FloatingCompass.vue`
    - `TimelineTrack.vue`
    - `RightDock.vue`
    - `FloatingCommandBar.vue`
  - 同时检查 `views/GraphView.vue` 和 `views/ListView.vue` 中的 rounded 类
  - 检查 `components/ui/CardNode.vue` 的 scoped CSS：`border-radius: 8px` → `border-radius: 2px`
  - 检查 `components/ui/SkeletonLine.vue` 的 rounded 类

  **Must NOT do**:
  - 不改 VueFlow 图谱交互逻辑
  - 不改入场动画时序
  - 不改禅模式阅读功能

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 批量扫描替换，模式统一
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 11, 12)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 3, 4

  **References**:

  **Pattern References**:
  - `frontend-workspace/apps/web-reader/src/components/` — 所有剩余组件
  - `frontend-workspace/apps/web-reader/src/components/ui/CardNode.vue` — 图谱节点，scoped CSS 需修改 border-radius

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: 全部 web-reader 组件无 rounded-lg/xl/2xl
    Tool: Bash (grep)
    Steps:
      1. grep -rn "rounded-lg\|rounded-xl\|rounded-2xl\|rounded-3xl" frontend-workspace/apps/web-reader/src --include="*.vue"
    Expected Result: 0 匹配
    Failure Indicators: 任何文件仍有大圆角
    Evidence: .sisyphus/evidence/task-13-full-audit.txt

  Scenario: CardNode 无 >4px border-radius
    Tool: Bash (grep)
    Steps:
      1. grep "border-radius.*[5-9]px\|border-radius.*[0-9][0-9]px" frontend-workspace/apps/web-reader/src/components/ui/CardNode.vue
    Expected Result: 0 匹配
    Evidence: .sisyphus/evidence/task-13-card-node.txt

  Scenario: 构建成功
    Tool: Bash
    Steps:
      1. cd frontend-workspace && pnpm --filter web-reader build
    Expected Result: 零错误
    Evidence: .sisyphus/evidence/task-13-build.txt
  ```

  **Commit**: NO (groups with Wave 3)
  - Message: `refactor(design): web-reader components industrial console conversion`

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.
> Do NOT auto-proceed after verification. Wait for user's explicit approval.

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, grep pattern, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `pnpm --filter admin-tauri build` + `pnpm --filter web-reader build`. Review all changed files for: `as any`/`@ts-ignore`, empty catches, console.log in prod, commented-out code, unused imports. Check AI slop: excessive comments, over-abstraction, generic names. Verify no functional logic was changed.
  Output: `Build [PASS/FAIL] | Lint [PASS/FAIL] | Files [N clean/N issues] | VERDICT`

- [ ] F3. **Visual QA — Playwright 全组件截图验证** — `unspecified-high` (+ `playwright` skill)
  Start dev server for both apps. Navigate to every component area. Screenshot before/after. Verify: (1) no rounded-lg/xl/2xl in computed styles, (2) font-family is monospace on body, (3) graph nodes have transparent background, (4) toggle controls have sharp corners, (5) search input has bottom border only. Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Visual [N/N compliant] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (git log/diff). Verify 1:1 — everything in spec was built (no missing), nothing beyond spec was built (no creep). Check "Must NOT do" compliance. Detect cross-task contamination. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

- **Wave 1 (Foundation)**: `refactor(design): industrial-altar design tokens and global base styles` — tailwind.config.js, style.css ×2
- **Wave 2 (Core)**: `refactor(design): admin-tauri components industrial console conversion` — all admin-tauri .vue files
- **Wave 3 (Web-Reader)**: `refactor(design): web-reader components industrial console conversion` — all web-reader .vue files
- Each commit: verify build passes before committing

---

## Success Criteria

### Verification Commands
```bash
# Build both apps
cd frontend-workspace && pnpm --filter admin-tauri build  # Expected: zero errors
cd frontend-workspace && pnpm --filter web-reader build   # Expected: zero errors

# Rounded corner audit (>4px should be 0 except known exceptions)
grep -rn "rounded-lg\|rounded-xl\|rounded-2xl\|rounded-3xl" frontend-workspace/apps/*/src --include="*.vue"  # Expected: 0 results

# Inline borderRadius audit (should be ≤4px)
grep -rn "borderRadius.*[6-9]px\|borderRadius.*[12][0-9]px" frontend-workspace/apps --include="*.vue"  # Expected: 0 results

# Font audit (body should use monospace)
# Verified via Playwright computed style check
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] Both apps build successfully
- [ ] No functional logic changed
- [ ] Visual QA passes for all components
- [ ] VueFlow interactivity preserved (drag, select, connect)
- [ ] Mobile responsiveness preserved in web-reader
