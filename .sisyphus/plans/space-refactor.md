# 空间减法 + 数据加法 — 三栏呼吸重构

## TL;DR

> **Quick Summary**: 三步递进重构：(1) RightAstrolabe 可折叠（Ctrl+\ 或按钮），编辑区瞬间拓宽 380px；(2) LeftSidebar 添加分类过滤标签栏，打通筛选闭环；(3) CategoryPanel 右侧重设计为"分类仪表盘"。
> 
> **Deliverables**:
> - RightAstrolabe 默认隐藏，快捷键/按钮可切换显示
> - LeftSidebar 搜索框下方添加 Category Ribbon 过滤标签
> - useCardListStore 新增 selectedCategoryId 过滤逻辑
> - CategoryPanel 右侧从"毛坯编辑框"升级为"分类仪表盘"
> 
> **Estimated Effort**: Medium-Large
> **Parallel Execution**: YES — 3 waves
> **Critical Path**: Store 改造 → 组件修改 → 验证

---

## Context

### Original Request
用户要求解决两大"死穴"：(1) 横向空间过度切割（5 栏→3 栏），(2) 分类体系"有表无里"（无筛选、无统计、无仪表盘）。用户明确要求按优先级顺序执行：Astrolabe 折叠 → 分类过滤标签 → 分类仪表盘。

### Key Discovery
**layout store 已经有 Astrolabe 折叠基础设施**：
- `isRightPanelOpen: ref(false)` — 默认关闭
- `toggleRightPanel()` — 切换函数
- `useGlobalShortcuts` 中 Ctrl+G 已绑定 `toggleRightPanel()`
- **但 App.vue 模板中没有使用 `isRightPanelOpen`**，RightAstrolabe 是硬编码常驻的
- **修复只需在 App.vue 中用 `v-show` 包裹 RightAstrolabe**

### Research Findings
- `useCardListStore.filteredOrphans/filteredRecent` 只按 `searchQuery` 过滤标题，**无分类过滤**
- `CardItem` 已有 `category_id` 字段，可直接用于过滤
- `Category` 类型：id, name, description, parent_id, sort_order, created_at, updated_at
- **无 color 字段**（Go 后端 model 不含 color），仪表盘的"主题色"需前端模拟或后续加字段
- **无 per-category card count API**，需前端从 recentCards/orphanCards 计算
- CategoryPanel 右侧目前只是 name + description 编辑框 + 保存/删除按钮
- `handleSelect` 有 bug：`editDescription.value = ""` 应为 `cat.description || ""`

---

## Work Objectives

### Core Objective
完成"空间减法 + 数据加法"：让编辑区获得最大呼吸空间，同时打通分类筛选闭环并丰满分类管理面板。

### Concrete Deliverables
- RightAstrolabe 默认隐藏，可通过 Ctrl+\（改绑快捷键）或 TheForge 头部按钮切换
- LeftSidebar 搜索框下方新增横向滚动 Category Ribbon
- useCardListStore 新增 selectedCategoryId + 分类过滤逻辑
- CategoryPanel 右侧升级为"分类仪表盘"：数据统计 + 快速索引 + 危险区
- App.vue 使用 isRightPanelOpen 控制右翼显示

### Definition of Done
- [ ] `vue-tsc --noEmit` 零错误
- [ ] RightAstrolabe 默认隐藏，Ctrl+\ 可切换，TheForge 有按钮
- [ ] 分类标签栏点击后卡片列表正确过滤
- [ ] CategoryPanel 选中分类后右侧显示统计信息
- [ ] 所有新增 UI 保持工业控制台美学（sharp corners, mono font, neon accents）

### Must Have
- RightAstrolabe 默认隐藏，有快捷键和按钮切换
- 分类过滤标签栏在搜索框下方
- filteredOrphans/filteredRecent 支持按 category_id 过滤
- CategoryPanel 右侧显示 card count、子分类列表、快速索引
- 所有新增元素遵守工业控制台设计规范

### Must NOT Have (Guardrails)
- **不改 Go 后端**：不改 API 结构，不改 Category model
- **不改 Rust 层**：不动 rust-workspace/ 下任何文件
- **不改功能逻辑**：卡片 CRUD、保存、删除等行为不变
- **不加 color 字段**：仪表盘的"主题色"暂用前端模拟，不改后端
- **AI Slop 防御**：不加多余注释、不创建文档、不做无关重构

---

## Verification Strategy

### Test Decision
- **Automated tests**: None (纯 UI 重构 + 功能增强)
- **Verification**: vue-tsc 类型检查 + grep 审计 + 手动功能验证

### QA Policy
每个 task 完成后执行 vue-tsc 验证。

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Store 改造 — 2 parallel tasks):
├── Task 1: layout store 快捷键改绑 + useCardListStore 过滤逻辑 [quick]
└── Task 2: useCardListStore 新增 selectedCategoryId + 过滤 computed [quick]

Wave 2 (组件改造 — 3 parallel tasks after Wave 1):
├── Task 3: App.vue 接入 isRightPanelOpen + TheForge 添加图谱按钮 [visual-engineering]
├── Task 4: LeftSidebar 添加 Category Ribbon 过滤标签栏 [visual-engineering]
└── Task 5: CategoryPanel 右侧重设计为分类仪表盘 [visual-engineering]

Wave FINAL (Verification):
└── Task F1: vue-tsc + 功能验证 [quick]
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| 1 | — | 3 | 1 |
| 2 | — | 4 | 1 |
| 3 | 1 | F1 | 2 |
| 4 | 2 | F1 | 2 |
| 5 | — | F1 | 2 |
| F1 | ALL | user | FINAL |

---

## TODOs

- [ ] 1. layout store 快捷键改绑 (Ctrl+G → Ctrl+\) + RightAstrolabe toggle 清理

  **What to do**:
  - `composables/useGlobalShortcuts.ts`：将 Ctrl+G 改为 Ctrl+\（反斜杠，`e.key === '\\'`）
  - `stores/layout.ts`：将 `isRightPanelOpen` 默认值从 `false` 保持（已正确）。确保 `toggleRightPanel` 不再与 left drawer 互斥（当前逻辑中 toggleRightPanel 会 closeLeftDrawer — 但现在左侧是常驻的，不会是 drawer 模式，所以互斥逻辑无影响，保留即可）

  **Must NOT do**:
  - 不改变 layout store 的整体架构
  - 不删除任何现有函数

  **Recommended Agent Profile**: `quick` + `coding-standards`

  **Parallelization**: Wave 1, parallel with Task 2

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: 快捷键绑定已更新
    Tool: Bash (grep)
    Steps:
      1. grep "e.key === '\\\\\\" frontend-workspace/apps/admin-tauri/src/composables/useGlobalShortcuts.ts
    Expected Result: 匹配到 `'e.key === '\\\\'`，之前 Ctrl+G 的匹配应不再存在
    Failure Indicators: 仍然有 Ctrl+G 的匹配
    Evidence: .sisyphus/evidence/task-1-shortcut-grep.txt

  Scenario: vue-tsc 通过
    Tool: Bash
    Steps:
      1. cd frontend-workspace && npx vue-tsc --noEmit --project apps/admin-tauri/tsconfig.json
    Expected Result: 零错误
    Failure Indicators: 任何类型错误
    Evidence: .sisyphus/evidence/task-1-tsc.txt
  ```

  **What to do**:
  - 在 `stores/useCardListStore.ts` 中添加：
    ```typescript
    const selectedCategoryId = ref<number | null>(null); // null = show all
    ```
  - 修改 `filteredOrphans` computed：在 searchQuery 过滤之后，追加 category_id 过滤
    ```typescript
    const filteredOrphans = computed(() => {
      let result = orphanCards.value;
      if (searchQuery.value) {
        const q = searchQuery.value.toLowerCase();
        result = result.filter((c) => c.title.toLowerCase().includes(q));
      }
      if (selectedCategoryId.value !== null) {
        result = result.filter((c) => c.category_id === selectedCategoryId.value);
      }
      return result;
    });
    ```
  - 同样修改 `filteredRecent`
  - 将 `selectedCategoryId` 加入 return 导出
  - 在 knowledge.ts 主 store 中，从 cardListStore 解构导出 `selectedCategoryId`

  **Must NOT do**:
  - 不改 orphanCards/recentCards 的加载逻辑
  - 不改 loadOrphans/loadRecent 函数
  - 不加后端 API 调用

  **Recommended Agent Profile**: `quick` + `coding-standards`

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: selectedCategoryId 导出正确
    Tool: Bash (grep)
    Steps:
      1. grep "selectedCategoryId" frontend-workspace/apps/admin-tauri/src/stores/useCardListStore.ts
      2. grep "selectedCategoryId" frontend-workspace/apps/admin-tauri/src/stores/knowledge.ts
    Expected Result: useCardListStore.ts 中有 ref 定义, knowledge.ts 中有导出

  Scenario: 过滤逻辑包含 category_id 条件
    Tool: Bash (grep)
    Steps:
      1. grep "selectedCategoryId" frontend-workspace/apps/admin-tauri/src/stores/useCardListStore.ts
    Expected Result: filteredOrphans/filteredRecent computed 中有 category_id 过滤

  Scenario: vue-tsc 通过
    Tool: Bash
    Steps:
      1. cd frontend-workspace && npx vue-tsc --noEmit --project apps/admin-tauri/tsconfig.json
    Expected Result: 零错误
  ```

  **Parallelization**: Wave 1, parallel with Task 1

- [ ] 3. App.vue 接入 isRightPanelOpen + TheForge 添加图谱切换按钮

  **What to do**:
  
  **App.vue**:
  - 导入 `useLayoutStore`，解构 `isRightPanelOpen`
  - RightAstrolabe 用 `<Transition>` 包裹，通过 `v-show="isRightPanelOpen"` 控制显示
  - 保持 RightAstrolabe 的 `w-[380px]` 和 `shrink-0`（折叠时宽度归零）
  
  **实现方案** — 使用 CSS transition 控制宽度动画：
  ```html
  <aside v-show="isRightPanelOpen" class="w-[380px] bg-ms-panel flex flex-col border-l border-ms-border shrink-0 transition-all duration-200">
    <RightAstrolabe />
  </aside>
  ```
  注意：v-show 只是 display:none，不会触发宽度动画。更好的方案是：
  ```html
  <div :class="isRightPanelOpen ? 'w-[380px]' : 'w-0'" class="shrink-0 transition-[width] duration-200 overflow-hidden border-l border-ms-border">
    <RightAstrolabe />
  </div>
  ```
  这样折叠/展开有平滑的宽度过渡动画。

  **TheForge.vue**:
  - 在头部 toolbar 区域（约 line 261 附近，save 按钮旁）添加一个图谱切换按钮
  - 按钮使用雷达/网络图标，点击调用 `layoutStore.toggleRightPanel()`
  - 当 `isRightPanelOpen` 为 true 时按钮高亮（text-neon）
  - 样式：与现有 save 按钮风格一致（rounded-sm, 工业控制台风格）

  **Must NOT do**:
  - 不改 RightAstrolabe.vue 内部逻辑
  - 不改 TheForge 的编辑/预览功能
  - 不删除 layout store 的互斥逻辑（保留以备将来使用）

  **Recommended Agent Profile**: `visual-engineering` + `frontend-design`, `coding-standards`

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: App.vue 使用 isRightPanelOpen 控制 Astrolabe
    Tool: Bash (grep)
    Steps:
      1. grep "isRightPanelOpen" frontend-workspace/apps/admin-tauri/src/App.vue
    Expected Result: App.vue 中存在 isRightPanelOpen 引用，RightAstrolabe 被 v-show 或条件 class 包裹
    Failure Indicators: isRightPanelOpen 未在 App.vue 中使用
    Evidence: .sisyphus/evidence/task-3-toggle-grep.txt

  Scenario: TheForge 有图谱切换按钮
    Tool: Bash (grep)
    Steps:
      1. grep "toggleRightPanel\|isRightPanelOpen" frontend-workspace/apps/admin-tauri/src/components/TheForge.vue
    Expected Result: TheForge.vue 中存在对 layout store 的 toggle 引用
    Failure Indicators: TheForge.vue 中没有图谱相关按钮
    Evidence: .sisyphus/evidence/task-3-forge-btn.txt

  Scenario: vue-tsc 通过
    Tool: Bash
    Steps:
      1. cd frontend-workspace && npx vue-tsc --noEmit --project apps/admin-tauri/tsconfig.json
    Expected Result: 零错误
    Evidence: .sisyphus/evidence/task-3-tsc.txt
  ```

  **Parallelization**: Wave 2, after Task 1

- [ ] 4. LeftSidebar 添加 Category Ribbon 过滤标签栏

  **What to do**:
  - 在搜索框（约 line 103）和 View Tabs（约 line 111）之间插入 Category Ribbon
  - 使用 `storeToRefs(store)` 获取 `categories` 和 `selectedCategoryId`
  - 样式规范：
    - 横向排列，可滚动（overflow-x-auto）
    - 每个标签：纯文字 `text-slate-500 font-mono text-[10px] tracking-wider uppercase`
    - 选中态：`text-neon border-b-2 border-neon`
    - 未选中态：`text-slate-600 border-b-2 border-transparent hover:text-slate-400`
    - 第一个标签："ALL"（selectedCategoryId = null）
    - 后续标签：从 categories 渲染，点击设置 selectedCategoryId
  - 代码结构：
    ```html
    <div class="flex px-3 py-1.5 gap-3 overflow-x-auto border-b border-ms-border/50">
      <button @click="selectedCategoryId = null"
        class="shrink-0 font-mono text-[10px] tracking-wider uppercase pb-0.5 border-b-2 transition-colors"
        :class="selectedCategoryId === null ? 'text-neon border-b-neon' : 'text-slate-600 border-b-transparent hover:text-slate-400'">
        ALL
      </button>
      <button v-for="cat in categories" :key="cat.id" @click="selectedCategoryId = cat.id"
        class="shrink-0 font-mono text-[10px] tracking-wider uppercase pb-0.5 border-b-2 transition-colors"
        :class="selectedCategoryId === cat.id ? 'text-neon border-b-neon' : 'text-slate-600 border-b-transparent hover:text-slate-400'">
        {{ cat.name }}
      </button>
    </div>
    ```
  - 注意：`selectedCategoryId` 来自 `storeToRefs(store)` — 需要确认 knowledge store 是否正确导出了它（Task 2 负责这个）

  **Must NOT do**:
  - 不改卡片列表的渲染逻辑
  - 不改 View Tabs 的功能
  - 不加背景色块或圆角标签（保持线框风格）

  **Recommended Agent Profile**: `visual-engineering` + `frontend-design`, `coding-standards`

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Category Ribbon 存在于 LeftSidebar
    Tool: Bash (grep)
    Steps:
      1. grep "selectedCategoryId\|Category.*Ribbon\|categories.*v-for" frontend-workspace/apps/admin-tauri/src/components/LeftSidebar.vue
    Expected Result: LeftSidebar.vue 中存在 selectedCategoryId 绑定和 categories v-for 渲染
    Failure Indicators: 没有 category 过滤相关代码
    Evidence: .sisyphus/evidence/task-4-ribbon-grep.txt

  Scenario: ALL 标签和 category 标签共存
    Tool: Bash (grep)
    Steps:
      1. grep "ALL\|selectedCategoryId === null" frontend-workspace/apps/admin-tauri/src/components/LeftSidebar.vue
    Expected Result: 存在 "ALL" 标签（重置为 null）和 category 绑定
    Failure Indicators: 缺少 ALL 重置选项
    Evidence: .sisyphus/evidence/task-4-all-tag.txt

  Scenario: 无新增 rounded-lg/xl
    Tool: Bash (grep)
    Steps:
      1. grep "rounded-lg\|rounded-xl\|rounded-2xl" frontend-workspace/apps/admin-tauri/src/components/LeftSidebar.vue
    Expected Result: 0 匹配
    Evidence: .sisyphus/evidence/task-4-no-soft-corners.txt

  Scenario: vue-tsc 通过
    Tool: Bash
    Steps:
      1. cd frontend-workspace && npx vue-tsc --noEmit --project apps/admin-tauri/tsconfig.json
    Expected Result: 零错误
    Evidence: .sisyphus/evidence/task-4-tsc.txt
  ```

  **Parallelization**: Wave 2, after Task 2

- [ ] 5. CategoryPanel 右侧重设计为"分类仪表盘"

  **What to do**:
  重设计 CategoryPanel.vue 的右侧面板（当前 lines 223-291），从简单的编辑框改为三个区块的仪表盘：

  **区块 A — 数据读出器 (Data Readout)** (顶部):
  - 横向排列 2-3 个统计卡片
  - 统计卡片样式：`bg-ms-deep border border-ms-border p-3`，无圆角
  - 数据项：
    - CARDS: 该分类下的卡片数量（从 recentCards + orphanCards 中计算 `cards.filter(c => c.category_id === selectedId).length`）
    - CHILDREN: 子分类数量（从 categories 中计算 `categories.filter(c => c.parent_id === selectedId).length`）
  - 数字用大号等宽：`text-2xl font-mono text-neon`
  - 标签用极小号：`text-[10px] text-slate-600 uppercase tracking-widest`

  **区块 B — 快速索引 (Quick Index)** (中部):
  - 列出该分类下所有卡片（紧凑格式）
  - 每行：卡片标题 + 更新时间
  - 样式：与 LeftSidebar 的卡片列表风格一致
  - 每行右侧有"解绑"按钮（小号，hover 时显示），点击将卡片的 category_id 设为 null
  - 解绑需要调用 `knowledgeStore.saveCard()` 的变体或直接 PUT cards/{id}（body 中不包含 category_id）
  
  **注意**：解绑功能需要新增一个 store action `unlinkCardFromCategory(cardId)` — 在 knowledge store 中添加：
  ```typescript
  async function unlinkCardFromCategory(cardId: string) {
    await invoke("api_request", {
      method: "PUT",
      endpoint: `/cards/${cardId}`,
      body: { category_id: null },
    });
    await refreshWorkspace();
  }
  ```

  **区块 C — 编辑 + 危险区 (Edit + Danger Zone)** (底部):
  - 分类名称输入框（保留原有）
  - 分类描述 textarea（保留原有）
  - 保存按钮（保留原有）
  - 分隔线 + 红色区域标签："DANGER ZONE"
  - 删除按钮移到这里，改为线框红色风格：`border border-red-500/30 text-red-500/70 hover:bg-red-500/10 rounded-sm`

  **Bug Fix**:
  - 修复 `handleSelect` 中 `editDescription.value = ""` → `editDescription.value = cat.description || ""`

  **数据来源**：
  - 需要导入 `useKnowledgeStore` 获取 `recentCards`, `orphanCards` 来计算统计
  - 或者通过 props 传递（但当前 CategoryPanel 不接收 props）
  - 推荐：直接在 CategoryPanel 中 `useKnowledgeStore()` 获取卡片数据

  **Must NOT do**:
  - 不改左侧分类树结构
  - 不改 CategoryTreeNode 组件
  - 不加后端 API 调用（卡片数量从前端计算）
  - 不加 color 选择器（后端不支持 color 字段）
  - 不改 CategoryPanel 的打开/关闭逻辑

  **Recommended Agent Profile**: `visual-engineering` + `frontend-design`, `coding-standards`

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: 仪表盘有三个区块
    Tool: Bash (grep)
    Steps:
      1. grep "Data Readout\|Quick Index\|Danger\|CARDS\|CHILDREN" frontend-workspace/apps/admin-tauri/src/components/CategoryPanel.vue
    Expected Result: 存在统计卡片（CARDS/CHILDREN）、快速索引列表、危险区标签
    Failure Indicators: 右侧面板仍只有 name/description 编辑框
    Evidence: .sisyphus/evidence/task-5-dashboard-sections.txt

  Scenario: handleSelect bug 已修复
    Tool: Bash (grep)
    Steps:
      1. grep -A 3 "handleSelect\|editDescription.value" frontend-workspace/apps/admin-tauri/src/components/CategoryPanel.vue
    Expected Result: editDescription.value 被赋值为 cat.description（不再是 ""）
    Failure Indicators: editDescription.value = "" 仍存在
    Evidence: .sisyphus/evidence/task-5-bugfix.txt

  Scenario: unlinkCardFromCategory 存在
    Tool: Bash (grep)
    Steps:
      1. grep "unlinkCardFromCategory" frontend-workspace/apps/admin-tauri/src/stores/knowledge.ts
    Expected Result: 函数存在且导出
    Failure Indicators: 函数未定义
    Evidence: .sisyphus/evidence/task-5-unlink.txt

  Scenario: 无新增 rounded-lg/xl/2xl
    Tool: Bash (grep)
    Steps:
      1. grep "rounded-lg\|rounded-xl\|rounded-2xl" frontend-workspace/apps/admin-tauri/src/components/CategoryPanel.vue
    Expected Result: 0 匹配
    Evidence: .sisyphus/evidence/task-5-no-soft-corners.txt

  Scenario: vue-tsc 通过
    Tool: Bash
    Steps:
      1. cd frontend-workspace && npx vue-tsc --noEmit --project apps/admin-tauri/tsconfig.json
    Expected Result: 零错误
    Evidence: .sisyphus/evidence/task-5-tsc.txt
  ```

  **Parallelization**: Wave 2, no dependency on Tasks 1-2

---

## Final Verification Wave

- [ ] F1. **Type Check + Build Verification** — `quick`
  - Run `vue-tsc --noEmit` to verify zero type errors
  - Verify no import errors in all modified files
  - Grep audit: no new `rounded-lg/xl/2xl` introduced
  - Output: `vue-tsc [PASS/FAIL] | Grep [CLEAN/N issues] | VERDICT`

---

## Commit Strategy

- **Wave 1 (Store)**: `feat(layout): add category filter state and astrolabe toggle shortcut`
- **Wave 2 (Components)**: `feat(ui): collapsible astrolabe + category ribbon + dashboard panel`
- Each commit: verify vue-tsc passes before committing

---

## Success Criteria

### Verification Commands
```bash
# Type check
cd frontend-workspace && npx vue-tsc --noEmit --project apps/admin-tauri/tsconfig.json

# No new soft corners
grep -rn "rounded-lg\|rounded-xl\|rounded-2xl" frontend-workspace/apps/admin-tauri/src --include="*.vue"
```

### Final Checklist
- [ ] RightAstrolabe 默认隐藏，Ctrl+\ 可切换
- [ ] TheForge 有图谱切换按钮
- [ ] Category Ribbon 在搜索框下方，点击可过滤卡片
- [ ] CategoryPanel 右侧显示卡片统计和快速索引
- [ ] 所有新增 UI 保持工业控制台美学
