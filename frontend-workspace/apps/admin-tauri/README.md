# Memory Stream — Tauri Admin

> 桌面端知识图谱管理器。Tauri v2 + Vue 3 + Rust 原生引擎，本地优先架构。

## 功能概览

### 知识图谱
- **全域星图 (Starmap)** — 所有卡片节点 + 边关系的宏观视图，连通分量自动切片 + Shelf Bin-Packing 排列
- **局部雷达 (Local Radar)** — 选中卡片的 N 度邻居子图，独立于全域视图
- **刚体 / 战术解锁** — 默认锁定节点位置（算法统治），按需解锁拖拽，重锁自动归位
- **Trunk / Link 双边类型** — Trunk（主干强连接）+ Link（普通引用），支持方向反转 / 断开

### Markdown 编辑器
- **CodeMirror 6** — 实时编辑 + 语法高亮 + Vim 模式支持
- **AST 实时预览** — Rust 解析 → 结构化 AST → Vue 组件递归渲染
- **Shiki 代码高亮** — github-dark 主题，带一键复制按钮（ease-snap 机械动画）
- **KaTeX 数学公式** — 行内 `$...$` / 块级 `$$...$$`，暗色主题适配
- **Mermaid 图表** — 流程图 / 序列图等，支持缩放灯箱（Teleport overlay + dot-grid 背景）

### Vault 文件管理
- **扁平分类** — 文件夹 = 分类，二级结构（无嵌套），文件夹置顶排序
- **拖拽归类** — 卡片拖拽到分类文件夹即完成移动
- **文件监控** — Vault 目录变更自动刷新侧边栏和图谱，过滤 .bunker/.git/.obsidian
- **元数据自愈合** — frontmatter 损坏时自动重建，保持索引一致性

### 搜索 & 同步
- **FTS5 全文搜索** — 本地 SQLite 索引，实时搜索卡片标题和内容
- **Manifest 增量同步** — UUID+版本号差分，仅推送变更卡片到 Go Server
- **Tombstone 防复活** — 删除 → pending_delete → 云端确认后清除
- **S3 图片上传** — WebP 压缩 + S3 存储 + 公开 URL 回写

---

## 技术架构

```
admin-tauri/
├── src/                          # Vue 3 前端
│   ├── components/
│   │   ├── editor/               # CodeMirror 编辑器 + AST 预览
│   │   ├── graph/                # 知识图谱（Starmap / Local Radar / HUD）
│   │   ├── vault/                # 侧边栏（文件树 / 分类 / 拖拽）
│   │   ├── base/                 # Dialog / Confirm / Rename 等基础组件
│   │   └── system/               # AppLayout / SettingsPanel / SyncConflictModal
│   ├── composables/              # Vue composables (useToast, useEditor...)
│   ├── services/                 # 业务逻辑封装 (card, search, sync, assets)
│   ├── stores/                   # Pinia 状态管理
│   ├── bridge/                   # Tauri invoke 类型封装
│   └── assets/                   # Mechanical Altar 主题 CSS + 设计 Token
│
└── src-tauri/                    # Rust 后端
    └── src/
        ├── lib.rs                # Tauri commands 注册入口
        ├── commands/
        │   ├── vault.rs          # 卡片 CRUD / 分类 / Vault 扫描
        │   ├── sync.rs           # 增量同步（push/pull/tombstone）
        │   └── search.rs         # FTS5 全文搜索
        ├── db.rs                 # SQLite (ms-meta) 操作封装
        ├── models.rs             # 数据结构 + frontmatter 解析
        ├── state.rs              # 全局状态 (config, meta_db, watcher)
        ├── watcher.rs            # 文件系统监控 (notify crate)
        ├── events.rs             # Tauri 事件定义
        └── protocol.rs           # 自定义协议处理
```

### Tauri Commands 分组

| 分类       | 命令                                                                                       |
| ---------- | ------------------------------------------------------------------------------------------ |
| 配置       | `get_config` `set_config` `set_storage_config` `clear_storage_config`                     |
| Markdown   | `parse_markdown` `parse_live_markdown` `save_document_io` `check_title_exists`             |
| Vault      | `read_card_file` `create_card` `delete_card` `rename_card` `move_card` `scan_vault_tree`   |
| 分类       | `create_category`                                                                          |
| 搜索       | `search_fts` `materialize_ghost`                                                           |
| 图谱       | `get_full_graph` `get_graph_neighborhood` `create_trunk` `delete_trunk` `create_link`      |
| 图谱布局   | `reverse_trunk` `save_node_positions` `compute_layout` `compute_neighborhood_layout`       |
| 同步       | `sync_pull` `sync_push` `sync_delete_tombstones` `sync_now`                                |
| 资产       | `upload_image`                                                                             |

---

## 设计系统 — Mechanical Altar

深色赛博工业主题，所有 UI 遵循统一 Token：

| Token          | 值                         | 用途                 |
| -------------- | -------------------------- | -------------------- |
| `--ms-void`    | `#050505`                  | 最深背景（灯箱/弹窗）|
| `--ms-deep`    | `#0d0d0d`                  | 主背景               |
| `--ms-carbon`  | `#141414`                  | 卡片/代码块背景      |
| `--ms-panel`   | `#1a1a1a`                  | 面板背景             |
| `--ms-surface` | `#222222`                  | 悬浮面               |
| `--ms-smoke`   | `#5a4f3e`                  | 暗示色（图标默认态） |
| `--neon`       | `#00e5ff`                  | 主强调（霓虹青）     |
| `--brass`      | `#b8860b`                  | 辅助强调（黄铜）     |

**动效曲线：** `--ease-hydraulic` (150-250ms) · `--ease-snap` (100ms 机械回弹) · `--ease-emerge` (入场景)

**字体：** Space Grotesk (UI) · JetBrains Mono (代码) · Noto Serif SC (中文衬线)

---

## 关键依赖

### 前端
| 库                | 用途                       |
| ----------------- | -------------------------- |
| Vue 3 + Pinia     | 响应式 UI + 状态管理       |
| CodeMirror 6      | Markdown 编辑器            |
| Vue Flow          | 知识图谱可视化             |
| shadcn-vue        | UI 组件库 (Dialog/Toast等) |
| lucide-vue-next   | 图标                       |
| Fuse.js           | 模糊搜索                   |
| Tailwind CSS v4   | 原子化样式                 |

### Rust 后端
| Crate      | 用途                       |
| ---------- | -------------------------- |
| Tauri 2    | 桌面应用框架               |
| ms-ast     | Markdown → AST 解析        |
| ms-graph   | 图谱算法 + 布局引擎        |
| ms-meta    | SQLite 索引 + FTS5 搜索    |
| ms-io      | 原子写入 / WebP / ZIP / S3 |
| gray_matter | YAML frontmatter 解析      |
| notify     | 文件系统监控               |
| rusqlite   | SQLite 绑定                |

---

## 开发

```bash
# 安装依赖
cd frontend-workspace && pnpm install

# 开发模式
cd apps/admin-tauri && pnpm tauri dev

# 构建
cd apps/admin-tauri && pnpm tauri build

# 类型检查
cd apps/admin-tauri && npx vue-tsc --noEmit
```

### IDE 推荐
- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
