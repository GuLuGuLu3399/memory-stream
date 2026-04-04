# 🖥️ Admin Tauri — Memory Stream 桌面管理端

> 基于 Tauri v2 的跨平台桌面应用，提供知识卡片的编辑、图谱管理、文件监控、离线缓存等完整管理功能。
>
> **路线图**：[CHECKLIST.md](../../../CHECKLIST.md) · **当前版本**：V3.4

## 架构总览

```
admin-tauri/
├── src/                        # Vue 3 前端
│   ├── components/
│   │   ├── TheForge.vue        # Markdown 编辑器（代码镜像 + 实时预览）
│   │   ├── LeftSidebar.vue     # 左侧卡片库（孤岛雷达/分类/全部 + 搜索）
│   │   ├── RightAstrolabe.vue  # 右侧图谱面板（Vue Flow 迷你星图）
│   │   ├── TitleBar.vue        # 自定义标题栏（无框窗口）
│   │   ├── GlobalToolbar.vue   # 全局工具栏
│   │   ├── CommandPalette.vue  # 命令面板
│   │   └── ConfirmDialog.vue   # 确认弹窗
│   ├── stores/
│   │   ├── knowledge.ts        # 卡片知识库状态（CRUD + 搜索 + 分类）
│   │   └── layout.ts           # UI 布局状态（侧边栏开关）
│   └── composables/
│       └── useConfirmDialog.ts # 异步确认弹窗
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── lib.rs              # 主入口：命令注册 + 状态管理 + 系统托盘
│   │   ├── api.rs              # HTTP 客户端网关（代理前端请求）
│   │   ├── auth.rs             # JWT 认证状态管理
│   │   ├── cache.rs            # SQLite 本地缓存（布局 + 边数据）
│   │   ├── draft.rs            # 本地草稿管理（ms-local-draft）
│   │   ├── export.rs           # 知识库导出（ms-kb-exporter → ZIP）
│   │   ├── image.rs            # 图片压缩 + S3 上传管道（WebP → S3）
│   │   ├── toc.rs              # TOC 目录树提取（ms-toc-extractor）
│   │   ├── watcher.rs          # 文件系统监控（Markdown Vault 目录）
│   │   └── ws_client.rs        # WebSocket 客户端（实时边操作）
│   └── Cargo.toml
```

## 快速开始

```bash
# 在 frontend-workspace 根目录
pnpm install

# 开发模式（需要 Rust 工具链）
pnpm --filter admin-tauri tauri dev

# 构建生产版本
pnpm --filter admin-tauri tauri build
```

## Tauri IPC 命令一览

| 模块     | 命令                                                            | 说明                      |
| -------- | --------------------------------------------------------------- | ------------------------- |
| Auth     | `login` / `genesis` / `set_auth_token` / `get_auth_status`      | JWT 认证流程              |
| Markdown | `process_markdown`                                              | 本地 AST 解析 + HTML 渲染 |
| Image    | `compress_image_to_webp`                                        | WebP 压缩                 |
| Image    | `compress_and_upload_image`                                     | 压缩 + S3 上传一站式管道  |
| Cache    | `get_cached_layouts` / `sync_from_server`                       | 离线缓存管理              |
| Cards    | `create_card_with_relation` / `delete_card` / `get_card_detail` | 卡片 CRUD                 |
| File     | `poll_file_changes` / `start_watcher`                           | Markdown Vault 文件监控   |
| Edges    | `create_edge_cmd` / `delete_edge_cmd`                           | WebSocket 图谱边操作      |
| Draft    | `save_draft` / `load_draft` / `list_drafts` / `delete_draft`    | 本地草稿                  |
| Export   | `export_knowledge_base`                                         | 全量 ZIP 导出             |
| TOC      | `extract_toc`                                                   | 目录树提取                |

## 系统集成

### 系统托盘

- 关闭窗口 → 隐藏到托盘（不退出）
- 托盘右键菜单：显示 / 退出
- 左键点击托盘图标 → 唤起窗口

### 全局快捷键

- `Alt+Space` — 唤起/隐藏窗口

### Deep Link

- 协议：`memory-stream://card/{id}`
- 用于从 Web Reader 跳转到桌面端编辑

### 自动更新

- 启动 5s 后静默检查更新
- 发现新版本 → 通知前端
- 使用 `tauri-plugin-updater`

## Rust 依赖关系

```
admin-tauri
├── md-parser          # Markdown 解析
├── ast-renderer       # HTML 渲染
├── ms-storage         # S3 上传（图片管道）
├── ms-local-draft     # 草稿管理
├── ms-toc-extractor   # TOC 提取
├── ms-kb-exporter     # 知识库导出
├── image              # WebP 编解码
├── rusqlite           # SQLite 缓存
├── reqwest            # HTTP 客户端
└── tokio-tungstenite  # WebSocket
```

## 环境变量（图片上传管道）

| 变量                 | 必填 | 说明                   |
| -------------------- | ---- | ---------------------- |
| `S3_ENDPOINT`        | ✅   | S3 兼容端点            |
| `S3_REGION`          | ❌   | 区域（默认 us-east-1） |
| `S3_BUCKET`          | ✅   | 存储桶名称             |
| `S3_ACCESS_KEY`      | ✅   | Access Key             |
| `S3_SECRET_KEY`      | ✅   | Secret Key             |
| `S3_PUBLIC_URL_BASE` | ❌   | CDN 公开 URL 前缀      |

## 技术栈

### 前端

- **Vue 3** + **TypeScript**
- **Pinia** — 状态管理
- **Tailwind CSS** — 样式
- **Vue Flow** — 迷你图谱
- **Lucide Vue** — 图标

### Rust 后端

- **Tauri v2** — 桌面框架
- **SQLite** — 本地缓存
- **image** — WebP 编解码
- **reqwest** — HTTP 客户端
- **tokio** — 异步运行时
