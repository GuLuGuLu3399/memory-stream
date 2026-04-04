# 🌐 Web Reader — Memory Stream 阅读前端

> 沉浸式知识图谱阅读器。多连通分量星图 + 右侧滑出抽屉 + 禅模式 + WASM Markdown 渲染。
>
> **路线图**：[CHECKLIST.md](../../../CHECKLIST.md) · **当前版本**：V3.4

## 架构总览

```
web-reader/src/
├── views/
│   ├── GraphView.vue       # 星图主视图（Vue Flow + Dagre + 多分量布局）
│   └── ListView.vue        # 列表视图（时间线排序）
├── components/
│   ├── DetailDrawer.vue    # 右侧阅读抽屉（Markdown 渲染 + Backlinks）
│   ├── LeftDock.vue        # 左侧导航中枢（视图切换 + 聚光灯深度 + 搜索）
│   ├── ZenReader.vue       # 全屏禅模式阅读（TOC 侧边栏）
│   ├── CommandPalette.vue  # Cmd+K 全局搜索
│   ├── FloatingCompass.vue # 浮动指南针（MiniMap + 快捷操作）
│   ├── EntranceAnimation.vue # 入场动画（粒子脉冲 + Logo 扩散）
│   └── ui/
│       ├── CardNode.vue    # 图谱卡片节点（热度光晕 + 孤岛标记）
│       └── SkeletonLine.vue # 骨架屏加载态
├── composables/
│   ├── useGraph.ts         # 图谱数据加载（nodes + edges）
│   ├── useGraphSync.ts     # WebSocket 实时增量同步（Auth-on-Connect + Ping/Pong RTT）
│   ├── useCards.ts         # 卡片 CRUD + 详情加载
│   ├── useActiveHeading.ts # TOC 活跃标题追踪
│   ├── useBreakpoints.ts   # 响应式断点（移动端检测）
│   ├── useSwipeClose.ts    # 移动端右滑关闭抽屉
│   └── useKeyboardNav.ts   # 全局键盘导航（Cmd+K, Esc）
├── store/
│   └── useGraphStore.ts    # Pinia 全局状态（选中节点、聚光灯、视图模式）
├── api/
│   └── index.ts            # Axios HTTP 客户端 + API 封装
└── utils/
    └── graphLayout.ts      # Dagre 布局 + potpack 多分量打包 + 聚光灯算法
```

## 快速开始

```bash
# 在 frontend-workspace 根目录
pnpm install
pnpm --filter web-reader dev

# 开发服务器默认 http://localhost:5173
```

## 核心功能

### WebSocket 实时同步（useGraphSync）

- **Auth-on-Connect**：连接后 3s 内必须发送 AUTH，否则断开
- **Ping/Pong RTT**：每 15s 心跳，计算延迟（LeftDock 底部显示）
- **增量更新**：监听 CARD_CREATED / CARD_UPDATED / CARD_DELETED / EDGE_CREATED / EDGE_DELETED
- **全局状态**：模块级 `wsConnected` / `wsAuthenticated` / `wsLatency` 供任意组件读取
- **自动重连**：指数退避（3s → 30s），认证失败自动重试

### 星图视图（GraphView）

- **多连通分量布局**：graphology 切割孤岛 → Dagre 独立布局 → potpack 矩阵打包
- **聚光灯模式**：点击节点 → N 度邻居高亮，其余暗化
- **一键归位**：底部浮动按钮重新布局 + fitView

### 阅读抽屉（DetailDrawer）

- 右侧 45% 宽度滑出，毛玻璃背景
- WASM Markdown 渲染（wikilink、代码高亮）
- **Backlinks**：显示哪些卡片引用了当前卡片，点击可跳转
- 移动端右滑关闭

### 禅模式（ZenReader）

- 全屏沉浸式阅读
- WASM TOC 侧边栏
- 从 DetailDrawer 的专注按钮进入

### 入场动画（EntranceAnimation）

- 首次加载播放 2.4s 序列动画
- 粒子场 + Logo 光晕 + 脉冲环扩散
- 自动消失，不阻塞交互

## 设计系统

### 色板

| Token       | 值        | 用途     |
| ----------- | --------- | -------- |
| `ms-deep`   | `#0d0d0d` | 主背景   |
| `ms-panel`  | `#1a1a1a` | 面板背景 |
| `ms-border` | `#333`    | 边框     |
| `neon`      | `#00e5ff` | 主强调色 |
| `ms-danger` | `#ff4757` | 危险操作 |

### 动效法则

- **250ms 法则**：所有交互过渡 250ms cubic-bezier(0.2, 0, 0, 1)
- **300ms 抽屉**：滑入滑出 300ms Expo-Out
- **800ms fitView**：图谱归位动画

## 技术栈

- **Vue 3** + **TypeScript**
- **Pinia** — 状态管理
- **Vue Flow** — 图谱渲染引擎
- **Tailwind CSS** — 原子化样式
- **Axios** — HTTP 客户端
- **Lucide Vue** — 图标库
- **Dagre** — 有向图布局
- **graphology** — 图算法（连通分量切割）
- **potpack** — 矩形打包算法
