# 🌊 Memory Stream — 个人知识图谱系统

> Markdown 卡片 × 有向图边关系 → 知识网络。Web 端沉浸式阅读，桌面端高效管理，Rust 引擎驱动。

---

## 系统架构

```
                    ┌─────────────────────────────────────────┐
                    │            Memory Stream                 │
                    │                                          │
  ┌──────────────┐  │  ┌──────────────┐   ┌────────────────┐  │
  │  🌐 Web      │  │  │  🖥️ Tauri    │   │  🐹 Go Server  │  │
  │  Reader      │◀─┼─▶│  Admin       │──▶│  REST + WS     │  │
  │  (Vue 3)     │  │  │  (Desktop)   │   │  Gin + GORM    │  │
  └──────────────┘  │  └──────┬───────┘   └───────┬────────┘  │
                    │         │                    │           │
                    │  ┌──────┴───────┐   ┌───────┴────────┐  │
                    │  │ 🦀 Rust      │   │  PostgreSQL    │  │
                    │  │ 10 crates    │   │  (主数据库)     │  │
                    │  └──────────────┘   └────────────────┘  │
                    └─────────────────────────────────────────┘
```

---

## 四大模块

| 模块              | 目录                                  | 说明                  | 文档                                                        |
| ----------------- | ------------------------------------- | --------------------- | ----------------------------------------------------------- |
| 🌐 Web Reader     | `frontend-workspace/apps/web-reader`  | 沉浸式图谱阅读器      | [README →](frontend-workspace/apps/web-reader/README.md)    |
| 🖥️ Tauri Admin    | `frontend-workspace/apps/admin-tauri` | 桌面管理端            | [README →](frontend-workspace/apps/admin-tauri/README.md)   |
| 🐹 Go Server      | `go-server/`                          | REST API + WebSocket  | [README →](go-server/README.md) · [API →](go-server/API.md) |
| 🦀 Rust Workspace | `rust-workspace/`                     | 核心引擎（10 crates） | [README →](rust-workspace/README.md)                        |

---

## 快速启动

```bash
# 1. 启动 Go 后端（需要 PostgreSQL）
cd go-server && cp .env.example .env && go run cmd/api/main.go

# 2. 启动 Web Reader
cd frontend-workspace && pnpm install && pnpm --filter web-reader dev

# 3. 启动 Tauri Admin（需要 Rust 工具链）
cd frontend-workspace && pnpm --filter admin-tauri tauri dev
```

---

## 技术栈

| 层级            | 技术                                                 |
| --------------- | ---------------------------------------------------- |
| 前端（Web）     | Vue 3 · TypeScript · Vue Flow · Tailwind CSS · Pinia |
| 前端（Desktop） | Tauri v2 · Vue 3 · Lucide Icons                      |
| 后端            | Go · Gin · GORM · PostgreSQL · WebSocket             |
| 核心引擎        | Rust（md-parser · ast-renderer · ms-storage · WASM） |
| 存储            | PostgreSQL + SQLite（本地缓存）+ S3（图片）          |

---

## 核心能力

### 📊 知识图谱

- **多连通分量星图** — graphology 孤岛切割 → Dagre 独立布局 → potpack 矩阵打包
- **聚光灯模式** — 点击节点 N 度邻居高亮，其余 blur + grayscale
- **增量同步** — WebSocket 实时推送节点增删，无需手动刷新

### 📝 Markdown 引擎

- **Rust 原生解析** — pulldown-cmark → AST → HTML，支持 wikilink / 代码高亮
- **WASM 浏览器端** — md-parser 编译为 WASM，Web 端零延迟渲染
- **TOC 提取** — 目录树自动生成，支持禅模式侧边栏导航

### 🔐 安全 & 实时

- **Auth-on-Connect** — WebSocket 建立后 3s 内 AUTH 握手，超时断开
- **RTT 心跳** — 每 15s Ping/Pong，客户端计算延迟
- **自动重连** — 指数退避（3s → 30s），认证失败自动重试

### 🖥️ 桌面端专属

- **文件监控** — Markdown Vault 目录变更自动同步
- **本地草稿** — SQLite 离线编辑，上线后一键推送
- **图片管道** — WebP 压缩 + S3 上传 + 编辑器拖拽/粘贴
- **知识库导出** — 全量 Markdown + 元数据 → ZIP

---

## 设计系统

| Token       | 值        | 用途     |
| ----------- | --------- | -------- |
| `ms-deep`   | `#0d0d0d` | 主背景   |
| `ms-panel`  | `#1a1a1a` | 面板背景 |
| `ms-border` | `#333`    | 边框     |
| `neon`      | `#00e5ff` | 主强调色 |

**动效法则：** 所有交互过渡 ≤ 250ms · 抽屉滑入 300ms Expo-Out · 图谱归位 800ms

---

## 开发规范

- **Git**：Conventional Commits（`feat:` / `fix:` / `docs:`）
- **样式**：Tailwind CSS 原子化，设计 Token 统一
- **API**：RESTful + JWT Bearer Token
- **路线图**：见 [CHECKLIST.md](./CHECKLIST.md)

## License

Private — All rights reserved
