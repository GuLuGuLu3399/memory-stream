# Admin-Tauri 前端重构 — UX 评估与代码审计报告

## UX 痛点

### P0: 操作无反馈（影响信任）
- **30+ 处 async 操作缺少 loading 态**：Sidebar 卡片创建/删除/移动、EditorView 保存/冲突解决、GraphPanel 边操作等
- 用户点击按钮后无视觉变化，不知道操作是否在进行
- **建议**：为所有 mutate 操作添加 `isBusy` ref + 按钮禁用态

### P1: 视觉层级模糊
- **ConflictBanner 硬编码 11 种颜色**：完全脱离设计系统，红/青/灰混合混乱
- **GraphPanel/StarmapView 硬编码 `#3A3A3A`、`#B8860B`**：应使用 `--text-muted`、`--brass`
- **FrontmatterModal `!important` + 硬编码 `#2ea043`**：验证成功色应使用 `--success` token
- **建议**：一次性扫除所有硬编码色值，替换为设计令牌

### P2: 过渡与动效缺失
- Overlay 面板（CommandPalette、SettingsPanel、FrontmatterModal）无入场/退场动画
- GraphPanel/StarmapView 面板内容瞬间出现，无 fade-in
- **建议**：统一使用 `<Transition>` + `--duration-normal` + `--ease-out`

### P3: 组件过大影响可维护性
| 组件 | 行数 | 风险 |
|------|------|------|
| EditorView.vue | 1,006 | 编辑器+标题+TOC+反向链接混杂 |
| Sidebar.vue | 878 | 树+CRUD+拖放+搜索+上下文菜单 |
| SettingsPanel.vue | 631 | 配置表单+存储管理+主题选择 |
| GraphPanel.vue | 578 | 图谱+菜单+搜索弹窗 |
| StarmapView.vue | 489 | 全局图谱+控制+菜单 |

## 代码质量问题汇总

| 类别 | 数量 | 严重度 |
|------|------|--------|
| 硬编码颜色 | 16 处 | 中 |
| `any` 类型 | 2 处 | 低 |
| `!important` | 5 处 | 中 |
| 缺失 loading 态 | 30+ 处 | 高 |
| 内联 style | 9 处 | 低 |
| 超大组件 (>300行) | 5 个 | 高 |

## 重构优先级

### Phase 1: 设计令牌统一（快速修复，影响全局）
- 扫除所有硬编码颜色 → CSS 变量
- 移除 `!important`
- 预计改动：~50 行

### Phase 2: 操作反馈补全
- Sidebar CRUD 操作 → loading + toast
- EditorView 保存/冲突 → loading 态
- GraphPanel 边操作 → 已有部分 toast，补全 loading
- 预计改动：~120 行

### Phase 3: 过渡动效统一
- Overlay 面板入场/退场动画
- 面板内容 fade-in
- 预计改动：~60 行

### Phase 4: 组件拆分（可选，风险较高）
- EditorView → TitleBar + TocPanel + BacklinkSection
- Sidebar → TreeView + CardContextMenu + CategoryContextMenu
- 此阶段影响面大，建议单独 PR

## 不改动
- 业务逻辑和 API 调用
- Pinia store 结构
- Tauri bridge 层
- Tailwind 配置和主题变量
