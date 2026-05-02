# Memory Stream — 个人知识图谱系统

> Markdown 卡片 × 有向图边关系 → 知识网络。桌面端高效管理，Rust 引擎驱动，Go 后端同步。
>
> **版本**: [3.5.0](./VERSION)

---

## 系统架构

```
                    ┌─────────────────────────────────────────┐
                    │            Memory Stream                 │
                    │                                          │
  ┌──────────────┐  │  ┌──────────────┐   ┌────────────────┐  │
  │  Web         │  │  │  Tauri       │   │  Go Server     │  │
  │  Reader      │◀─┼─▶│  Admin       │──▶│  REST API      │  │
  │  (Vue 3)     │  │  │  (Desktop)   │   │  Gin + GORM    │  │
  └──────────────┘  │  └──────┬───────┘   └───────┬────────┘  │
                    │         │                    │           │
                    │  ┌──────┴───────┐   ┌───────┴────────┐  │
                    │  │ Rust Engine  │   │  PostgreSQL    │  │
                    │  │ 4 crates     │   │  (云端镜像)     │  │
                    │  └──────────────┘   └────────────────┘  │
                    └─────────────────────────────────────────┘
```

---

## 四大模块

| 模块              | 目录                                  | 说明                  | 文档                                                        |
| ----------------- | ------------------------------------- | --------------------- | ----------------------------------------------------------- |
| Web Reader        | `frontend-workspace/apps/reader-web`  | 沉浸式图谱阅读器（重构中） | —                                                           |
| Tauri Admin       | `frontend-workspace/apps/admin-tauri` | 桌面管理端            | [README →](frontend-workspace/apps/admin-tauri/README.md)   |
| Go Server         | `go-server/`                          | REST API + JWT Auth   | [API →](go-server/API.md)                                   |
| Rust Workspace    | `rust-workspace/`                     | 核心引擎（4 crates）  | [README →](rust-workspace/README.md)                        |

---

## 安全审计 (v3.5.0)

v3.5.0 通过全面安全审计，修复 4 Critical / 10 High / 8 Medium / 4 Low 问题：

| 等级    | 数量 | 关键修复                                                     |
| ------- | ---- | ------------------------------------------------------------ |
| Critical | 4   | DOMPurify XSS 防御、路径遍历防御、同步 TOCTOU 冲突检测、Windows 保留名拦截 |
| High    | 10   | SQLite WAL + busy_timeout、FTS 事务合并、原子文件创建、rAF 内存泄漏修复、HMR 安全、Watcher 事件一致性 |
| Medium  | 8    | Sidebar 排序优化、slugify Unicode 清洗、API URL 校验、FTS 查询截断 |
| Low     | 4    | 按钮无障碍标签（aria-label）                                 |

---

## 快速启动

```bash
# 开发模式（Tauri + Go 并行启动）
make dev

# 或单独启动
make dev-tauri    # Tauri 桌面端（需要 Rust 工具链）
make dev-go       # Go 后端（需要 PostgreSQL）
```

---

## 技术栈

| 层级            | 技术                                                           |
| --------------- | -------------------------------------------------------------- |
| 前端（Desktop） | Tauri v2 · Vue 3 · TypeScript · Pinia · shadcn-vue · Vue Flow |
| 前端（Web）     | Vue 3 · TypeScript（重构中）                                   |
| 后端            | Go · Gin · GORM · PostgreSQL · JWT · WebSocket                 |
| 核心引擎        | Rust（ms-ast · ms-graph · ms-meta · ms-io）                    |
| 存储            | PostgreSQL（云端镜像）+ Vault 文件系统（本地源）+ S3（图片）   |

---

## 核心能力

### Local-First 架构

- **Vault 文件系统** — Markdown 文件即卡片，文件夹路径即分类，本地文件系统为唯一真相源
- **Manifest 增量同步** — UUID+版本号 manifest 差分计算，仅推送变更卡片
- **Tombstone 防复活** — 删除操作写入 pending_delete 状态，同步时云端确认后清除
- **.bunker 系统目录** — SQLite 索引 + JSONL 同步日志，与用户数据隔离

### 知识图谱

- **连通分量切片** — Union-Find 孤岛检测 → 独立 Sugiyama 布局 → Shelf Bin-Packing 排列
- **Orphan 网格** — 无连接节点 10 列矩阵排列，与子图区域分离
- **刚体 / 战术解锁** — 默认锁定节点位置，按需解锁拖拽，重锁自动归位
- **局部雷达** — 点击节点 N 度邻居子图提取，独立于全域星图
- **UUID-first 匹配** — 云端同步优先 UUID 匹配，避免标题变更导致重复文件

### Markdown 引擎

- **Rust 原生解析** — pulldown-cmark → AST，支持 wikilink / 代码高亮 / 数学公式 / Mermaid 图表
- **桌面端渲染** — Shiki 语法高亮 + KaTeX 数学 + Mermaid 流程图（带缩放灯箱）
- **TOC 提取** — 目录树自动生成，支持禅模式侧边栏导航

### 桌面端专属

- **文件监控** — Markdown Vault 目录变更自动同步，过滤 .bunker/.git/.obsidian
- **本地优先写入** — frontmatter 自愈合 + 元数据持久化，断网可持续编辑
- **图片管道** — WebP 压缩 + S3 上传 + 编辑器拖拽/粘贴
- **全文搜索** — SQLite FTS5 本地索引，实时搜索卡片内容
- **知识库导出** — 全量 Markdown + 元数据 → ZIP

### 安全加固 (v3.5.0)

- **XSS 防御** — DOMPurify 消毒 Mermaid SVG / KaTeX HTML / Shiki 回退输出
- **路径遍历防御** — `category_path()` 绝对路径规范化 + 前缀校验，拒绝 `..` 穿越
- **原子文件创建** — `OpenOptions::create_new(true)` 消除 TOCTOU 竞态
- **Windows 保留名拦截** — `sanitize_file_stem()` 阻断 CON/PRN/NUL 等保留文件名
- **SQLite 并发安全** — WAL 模式 + 5s busy_timeout + synchronous NORMAL
- **同步冲突检测** — 远端覆盖前重校验本地文件 hash，TOCTOU 时自动升级为冲突
- **输入边界** — 50MB 图片上传限制、500 字符 FTS 查询截断、版本号溢出保护、API URL scheme 校验

---

## Rust Workspace（4 Crates）

| Crate        | 职责                            | 关键依赖                     | 测试 |
| ------------ | ------------------------------- | ---------------------------- | ---- |
| **ms-ast**   | Markdown 解析 / AST / TOC / Visitor | pulldown-cmark, serde   | 55   |
| **ms-graph** | 知识图谱算法 + 布局引擎         | petgraph, rayon              | 23   |
| **ms-meta**  | SQLite 元数据索引 + FTS5 搜索（WAL）| rusqlite, sha2           | 4    |
| **ms-io**    | 图片压缩 / ZIP 导出 / S3(opt-in)| image, webp, zip, rust-s3    | 17   |

99 tests, 0 clippy warnings, 0 production `unwrap()`。详见 [rust-workspace/README →](rust-workspace/README.md)。

---

## 设计系统

双端独立主题，共享 z-index / 字体 / 间距体系：

| 端          | 主题     | 基底色                                     | 强调色                     | 辅助色                                      |
| ----------- | -------- | ------------------------------------------ | -------------------------- | ------------------------------------------- |
| Web Reader  | 血肉神殿 | `ms-xuan` `#0a0806` → `ms-xiang` `#1c1814` | `xuepo` `#a62626` (血珀红) | `ms-gold` `#c9a84c` · `ms-patina` `#4a7c6f` |
| Admin Tauri | 机械祭坛 | `ms-deep` `#0d0d0d` → `ms-panel` `#1a1a1a` | `neon` `#00e5ff` (霓虹青)  | `brass` `#b8860b`                           |

**动效法则：** 所有交互过渡 ≤ 250ms · 抽屉滑入 300ms Expo-Out · 图谱归位 800ms

---

## 构建 & 验证

```bash
make build    # Rust + Go + Frontend
make test     # 全量测试
make lint     # cargo clippy + go vet + vue-tsc
```

---

## 开发规范

- **Git**：Conventional Commits（`feat:` / `fix:` / `docs:`）
- **Rust**：零 `unwrap()`，`thiserror` 错误传播，`cargo clippy -- -D warnings`
- **Go**：`ctx context.Context` 首参，`fmt.Errorf("...: %w", err)` 错误包装
- **Vue**：`<script setup>` exclusively，Pinia 严格类型，`onUnmounted` 清理副作用
- **样式**：Tailwind CSS v4 原子化，Mechanical Altar 设计 Token 统一

## License

Private — All rights reserved
