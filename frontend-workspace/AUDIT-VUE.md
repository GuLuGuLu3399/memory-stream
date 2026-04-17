# Vue Frontend Audit Report — Blood Temple Vue

> Audit Date: 2026-04-15
> Scope: `frontend-workspace/apps/admin-tauri/` + `frontend-workspace/apps/web-reader/`

---

## Executive Summary

| War Zone | P0 | P1 | P2 | CLEAN |
|---|---|---|---|---|
| **1. 主线程 & Dagre** | 2 | 0 | 1 | 5 |
| **2. 响应式震荡** | 1 | 3 | 1 | 3 |
| **3. 内存泄漏 & 生命周期** | 1 | 1 | 1 | 10 |
| **4. 渲染管线 & 虚拟化** | 0 | 2 | 2 | 5 |
| **合计** | **4** | **6** | **5** | **23** |

**关键洞察**: admin-tauri 的 `RightAstrolabe.vue` 集中了 4 个战区中 3 个的 P0 问题——它是前端性能瓶颈的震中。web-reader 的工程实践明显更成熟（Worker 分治、TanStack Virtual、RAII 式清理），admin-tauri 应系统性对齐。

---

## War Zone 1: 主线程阻塞 & Dagre

### P0-1: RightAstrolabe — Dagre 在 computed 中同步执行

**File**: `admin-tauri/src/components/RightAstrolabe.vue`

`flowNodes` 是一个 `computed` 属性，直接调用 `dagre` 布局算法。每次 `localNodes`、`localEdges`、`activeCard`、`recentCards`、`orphanCards`、`categories` 中任意一个响应式依赖变化时，整个 Dagre 布局都会重新计算——**同步阻塞主线程**。

```typescript
import dagre from "dagre"   // ← 顶层静态导入，首屏即加载 ~150KB
```

`flowNodes` computed 内部：
1. 遍历所有 nodes 设置 dagre 节点
2. 遍历所有 edges 设置 dagre 边
3. 调用 `dagre.layout(g)` — **O(V+E) 同步计算**
4. 映射回 Vue Flow 节点数组

一次 `activeCard` 切换、一个 category 的 hover、一条新边创建，都会触发完整 Dagre relayout。节点数超过 100 时帧率断崖下跌。

**根因**: admin-tauri 没有复用 web-reader 的 Worker 分治方案。

**修复方向**:
- 将 `flowNodes` 从 `computed` 拆成 `shallowRef` + `watchEffect`（或 `watch`）
- Dagre 计算移入 Web Worker（复用 `layout.worker.ts` 模式）
- 添加 debounce（~150ms），避免快速连续触发

### P0-2: RightAstrolabe — loadSummonPool() 挂载时瀑布式拉取 4000 张卡片

**File**: `admin-tauri/src/components/RightAstrolabe.vue`

```typescript
const loadSummonPool = async () => {
  let page = 1
  while (true) {
    const res = await cardApi.list({ page, pageSize: 200 })
    summonPool.push(...res.data)
    if (res.data.length < 200 || page >= 20) break
    page++
  }
}
```

`onMounted` 调用 → 最多 20 次串行 HTTP 请求，全部返回后才渲染。4000 张卡片原始 JSON 解析 + 响应式代理化 = 可观 GC 压力和主线程冻结。

**修复方向**:
- 按需分页（lazy load），或设定合理上限（如 500）
- 后端提供聚合接口（只需 id + title），避免传输完整 raw_md/ast_data

### P2-1: ZenReader — loadDetail() 无请求取消，stale response 风险

**File**: `web-reader/src/components/ZenReader.vue`

滚动用 `requestAnimationFrame` 节流，这是正确的。但 `loadDetail()` 无 `AbortController` 或请求序列号，用户快速切换卡片时旧请求可能覆盖新状态。

**影响**: P2 — 快速导航时可能闪烁或短暂内容错位。

### 正面发现

| 组件 | 做法 | 评价 |
|---|---|---|
| `GraphView.vue` | 自适应阈值 120 节点，低于 sync，高于走 Worker | EXEMPLARY |
| `graphLayout.ts` | `computePositions()` 异步，动态 import dagre/graphology/potpack | EXEMPLARY |
| `layout.worker.ts` | 独立 Worker，postMessage 通信 | CLEAN |
| `useGraph.ts` | `shallowRef` 管理 nodes/edges | CORRECT |
| `StatsWidget.vue` | 纯 SVG computed sparkline，零副作用 | CLEAN |

---

## War Zone 2: 响应式震荡（Reactivity Thrashing）

### P0-1: knowledge.ts — checkDirty() 每次按键 JSON.stringify 整张卡片

**File**: `admin-tauri/src/stores/knowledge.ts:124-132`

```typescript
function checkDirty() {
  if (!activeCard.value) return;
  const current = JSON.stringify({
    title: activeCard.value.title,
    content: activeCard.value.content,   // ← 可能几 KB 的 raw_md
    category_id: activeCard.value.category_id,
  });
  isDirty.value = current !== savedSnapshot;
}
```

`checkDirty()` 被外部在每次用户输入时调用。每次调用完整序列化 card content（可能数 KB 的 Markdown 原文）为 JSON 字符串，再与 snapshot 做字符串比较。

用户每按一个键 → `JSON.stringify(几KB)` × 2 → 字符串全量比较。快速输入场景（5 次/秒），每秒产生 ~50KB 临时字符串，加重 GC 压力。

**修复方向**:
- 字段级比较：`a.title !== b.title || a.content !== b.content || a.category_id !== b.category_id`
- 或对 content 做 lazy hash（仅 content 变化时才重新 hash）

### P1-1: knowledge.ts — updateLayouts() 触发三连响应式雪崩

**File**: `admin-tauri/src/stores/knowledge.ts:487-501`

```typescript
function updateLayouts(layouts) {
  localNodes.value = localNodes.value.map(...);   // ← 触发 #1
  orphanCards.value = orphanCards.value.map(...);  // ← 触发 #2
  recentCards.value = recentCards.value.map(...);  // ← 触发 #3
}
```

三个 `shallowRef` 连续赋新值，三次 `.map()` 串行执行 O(n) × 3。

**修复方向**: 合并更新或惰性更新（仅更新可见节点）。

### P1-2: useMergePanel.ts — deep: true 监听简单引用类型

**File**: `admin-tauri/src/composables/useMergePanel.ts:94-101`

```typescript
watch([selectedSurvivor, selectedVictims], async () => {
  // ...
}, { deep: true });
```

`selectedSurvivor` 是 `ref<string>`，`selectedVictims` 是 `ref<string[]>`。赋值模式是 `filter()` 产生新数组（引用已变），`deep: true` 完全多余。

**修复方向**: 移除 `{ deep: true }`。

### P1-3: SidebarCardItem.vue — 模板内函数调用触发逐卡重算

**File**: `admin-tauri/src/components/sidebar/SidebarCardItem.vue:83`

```vue
{{ contentPreview(card.content) }}
```

`contentPreview()` 内部 5 次 `.replace()` 正则替换。模板内函数调用无 memoization，父组件 re-render 时每张卡片重新执行。100 张卡片 = 500 次正则替换。

**修复方向**: 移入 `computed` 或缓存。

### P2-1: useCardListStore.ts — filtered computed 双重 O(n) 遍历

**File**: `admin-tauri/src/stores/useCardListStore.ts:46-69`

`filteredOrphans` 和 `filteredRecent` 各做两次 `.filter()` 遍历。由于搜索有 300ms debounce 保护，实际抖动有限。

### 正面发现

| 文件 | 做法 | 评价 |
|---|---|---|
| `useCardListStore.ts` | `shallowRef` 管理 orphanCards/recentCards | CORRECT |
| `knowledge.ts` | `shallowRef` 管理 localNodes/localEdges/backlinks | CORRECT |
| `knowledge.ts` | `silentRefresh()` 500ms debounce 防 WS 刷新风暴 | GOOD |
| web-reader `useGraph.ts` | `shallowRef` 全链路 | EXEMPLARY |

---

## War Zone 3: 内存泄漏 & 生命周期

### P0-1: useGraphSync.ts — WebSocket 连接不绑定组件生命周期

**File**: `web-reader/src/composables/useGraphSync.ts:109-553`

`useGraphSync()` 创建 WebSocket、心跳（`setInterval` 15s）、认证超时（`setTimeout`）、重连定时器——但 **自身不注册 `onUnmounted` 钩子**。`disconnect()` 被返回但从未自动调用。

| 资源 | 泄漏方式 |
|---|---|
| WebSocket 连接 | TCP 连接永不关闭 |
| `pingInterval` | 15s setInterval 永久运行 |
| `pongTimeout` | setTimeout 累积 |
| `reconnectTimer` | setTimeout 累积 |
| `authTimer` | setTimeout 可能触发 |

**修复方向**: 在 `useGraphSync()` 内部添加 `onUnmounted(() => { disconnect(); });`。

### P1-1: SearchBar.vue — debounceTimer 未在 unmount 时清理

**File**: `web-reader/src/components/SearchBar.vue:64-72`

组件无 `onUnmounted` 钩子。debounce 等待期间导航离开，回调仍执行网络请求。

**修复方向**: 添加 `onUnmounted(() => { if (debounceTimer) clearTimeout(debounceTimer); })`。

### P2-1: RightAstrolabe — summonPool 8MB 常驻内存

**File**: `admin-tauri/src/components/RightAstrolabe.vue`（跨战区）

`summonPool` 可累积至 4000 张卡片（~8MB），无 LRU 逐出，无 WeakRef，组件不卸载则永不释放。

### 正面发现

| 文件 | 做法 | 评价 |
|---|---|---|
| `admin-tauri/useWSListener.ts` | Tauri listen 批量 unlisten | EXEMPLARY |
| `admin-tauri/useMergePanel.ts` | listen/unlisten 配对 | CLEAN |
| `admin-tauri/useGlobalShortcuts.ts` | addEventListener/removeEventListener 配对 | CLEAN |
| `web-reader/useWebSocket.ts` | disconnect() 清理所有定时器 | CLEAN |
| `web-reader/ZenReader.vue` | rAF/timer 全清理 | CLEAN |
| `web-reader/GraphView.vue` | longPressTimer unmount 清理 | CLEAN |
| `web-reader/useActiveHeading.ts` | IntersectionObserver + timer 双清理 | CLEAN |
| `web-reader/useKeyboardNav.ts` | listener 配对移除 | CLEAN |
| `web-reader/useSwipeClose.ts` | touch 事件配对移除 | CLEAN |
| `web-reader/useBreakpoints.ts` | matchMedia listener 配对移除 | CLEAN |

---

## War Zone 4: 渲染管线 & 虚拟化

### P1-1: LeftSidebar.vue — 卡片列表无虚拟化，全量 DOM 渲染

**File**: `admin-tauri/src/components/LeftSidebar.vue:152-167`

```vue
<SidebarCardItem
  v-for="(card, idx) in displayedCards"
  :key="card.id"
  ...
/>
```

无虚拟化，所有卡片一次性渲染到 DOM。每个 SidebarCardItem ~20+ DOM 节点。此外 `:is-selected` prop 在 activeCard 变化时触发所有项的 prop diff。

| 卡片数 | DOM 节点估算 |
|---|---|
| 50 | ~1000 |
| 200 | ~4000 |
| 500+ | ~10000+ |

**修复方向**: 引入虚拟滚动（TanStack Virtual，与 web-reader 一致）或后端分页。

### P1-2: RightAstrolabe.vue — edgeFlow 动画在所有 sequence 边上无限循环

**File**: `admin-tauri/src/components/RightAstrolabe.vue:520-527`

```css
.astrolabe-edge--sequence {
  animation: edgeFlow 1.5s linear infinite;
}
```

每条 sequence 边运行 `stroke-dashoffset` 动画。SVG 属性动画每次变化触发 repaint。

| 边数 | 每秒 repaint 估算 (60fps × 边数) |
|---|---|
| 20 | ~1200 |
| 50 | ~3000 |
| 100 | ~6000 |

**修复方向**: 仅对 viewport 可见边启动动画（IntersectionObserver），或节点数 > 50 时禁用。

### P2-1: StatsWidget.vue — sparkline zone 未用 v-memo

**File**: `web-reader/src/components/StatsWidget.vue:183-189`

每个 zone div 的 style 对象在模板内创建新对象，hover 触发父组件更新时全部重算。数组通常 ~30 项，影响有限。

### P2-2: FlowReader.vue — 链式导航点使用 index 作 key

**File**: `web-reader/src/components/FlowReader.vue:194`

`chainIds` 是稳定 ID 数组，应使用实际 ID 作 key。数组通常 < 10 项，影响极小。

### 正面发现

| 文件 | 做法 | 评价 |
|---|---|---|
| `web-reader/ListView.vue` | **TanStack Virtual** 虚拟化长列表 | EXEMPLARY |
| `web-reader/ListView.vue` | `virtualizer.getVirtualItems()` 仅渲染可见行 | BEST PRACTICE |
| admin-tauri `v-for` | 全部使用 `:key="card.id"` 唯一键 | CORRECT |
| admin-tauri `CategoryTreeNode` | computed memoize indent style | CLEAN |
| admin-tauri `CategoryPanel` | `themeGlowStyle` computed 缓存 | CLEAN |

---

## 全部 P0 清单（按优先级排序）

| # | War Zone | File | Issue |
|---|---|---|---|
| V-P0-1 | 1 | `RightAstrolabe.vue` | Dagre 在 computed 中同步执行 |
| V-P0-2 | 1 | `RightAstrolabe.vue` | loadSummonPool 4000 卡片瀑布 |
| V-P0-3 | 2 | `stores/knowledge.ts` | checkDirty() 每次按键 JSON.stringify |
| V-P0-4 | 3 | `useGraphSync.ts` | WebSocket + 定时器不绑定生命周期 |

---

## Action Items

### P0 — 立即修复

1. **RightAstrolabe Dagre Worker 化** — 复用 web-reader 的 `layout.worker.ts` 模式，将 Dagre 移出 computed
2. **loadSummonPool 分页/裁剪** — 后端聚合接口 + 前端按需加载
3. **checkDirty 字段级比较** — 替换 JSON.stringify 为直接字段比较
4. **useGraphSync 生命周期绑定** — 内部注册 onUnmounted 自动 disconnect

### P1 — 短期治理

5. LeftSidebar 引入虚拟滚动
6. RightAstrolabe edge 动画加 visibility 守卫
7. updateLayouts 合并/惰性更新
8. useMergePanel 移除多余 deep:true
9. SidebarCardItem contentPreview 缓存
10. SearchBar 添加 unmount timer 清理
