# Rust Workspace — Memory Stream 核心引擎

> 4 个完全独立的 crate，极致扁平化依赖图，零反向依赖。
> Markdown 解析 / 图谱算法 / SQLite+FTS5 索引 / 文件与媒体 IO。

## 架构总览

```
rust-workspace/
├── ms-ast/      # Markdown 解析 / AST / TOC / ParsedDocument 单次解析
├── ms-graph/    # 知识图谱算法（petgraph, rayon）+ IPC 友好错误类型
├── ms-meta/     # SQLite 元数据索引 + FTS5 全文搜索（schema.sql 编译期嵌入）
└── ms-io/       # 原子写入 / Vault 扫描 / WebP 压缩 / ZIP 导出 / S3（opt-in）
```

## 依赖关系（极致扁平化）

```
[Tauri App] ──→ ms-ast       （独立）
            ──→ ms-graph      （独立，graph-only 不拉 ms-io）
            ──→ ms-meta       （独立）
            ──→ ms-io         （独立）
```

所有 crate 之间 **零互相依赖**。`ms-graph` 的 `io-conversions` feature 仅用于桥接 `ms-io` 错误类型，纯 `graph` feature 完全独立。

## 快速开始

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

验证独立性：
```bash
cargo tree -p ms-graph --no-default-features --features graph
# 只包含 petgraph + rayon，不含 ms-io
```

## 各 Crate 说明

### `ms-ast` — Markdown 解析引擎

接收原始 Markdown 文本，返回结构化 AST + 目录树。核心 API `parse_document()` 单次解析提取 AST JSON、TOC、外链、摘要。

```rust
use ms_ast::parse_document;

let doc = parse_document(&content)?;
// doc.ast_json, doc.toc, doc.outgoing_links, doc.excerpt
```

**Feature flags：**

| Feature  | 默认开启 | 说明                        |
| -------- | -------- | --------------------------- |
| `parser` | 是       | pulldown-cmark Markdown 解析 |
| `toc`    | 是       | 目录树提取                   |

**关键依赖：** pulldown-cmark, regex

---

### `ms-graph` — 知识图谱运算引擎

纯内存图算法：BFS、拓扑排序、确定性布局。IPC 友好的 `GraphError` 可直接序列化给前端。

```rust
use ms_graph::{KnowledgeGraph, compute_deterministic_layout, GraphResult};
let mut kg = KnowledgeGraph::new();
kg.add_node("uuid-1", "title");
let sorted = kg.topological_layers();

// 确定性布局：连通分量切片 → Sugiyama → Shelf Bin-Pack → Orphan 网格
let positions = compute_deterministic_layout(&kg)?;
```

**Feature flags：**

| Feature           | 默认开启 | 说明                                  |
| ----------------- | -------- | ------------------------------------- |
| `graph`           | 是       | petgraph + rayon 图算法               |
| `io-conversions`  | 是       | `From<ms_io::IoError> for GraphError` |

**关键依赖：** petgraph, rayon

---

### `ms-meta` — SQLite 元数据索引 + FTS5

本地 Vault 元数据持久化 + 内置全文搜索。SQL schema 通过 `include_str!` 编译期嵌入，零运行时开销。`SyncStatus` 强类型 enum 杜绝魔术字符串。

```rust
use ms_meta::{MetaDb, CardIndex, FtsHit, SyncStatus};
let db = MetaDb::open(&vault_path)?;

// 卡片索引（sync_status 为强类型 enum）
db.upsert_card(&CardIndex {
    uuid, file_path, file_hash, version,
    sync_status: SyncStatus::Synced,
    last_synced_hash: None,
})?;

// 全文搜索（事务保护的 upsert，PRAGMA busy_timeout=5000）
db.upsert_fts("uuid-1", "Rust 入门", "Rust 所有权机制...", "programming")?;
let hits: Vec<FtsHit> = db.search_fts("ownership", 10)?;
```

**目录结构：**

```
ms-meta/src/
├── schema.sql    ← 纯 SQL，编辑器语法高亮，include_str! 编译期嵌入
├── db.rs         ← MetaDb 连接 + 初始化 + 遗留迁移
├── repo.rs       ← card / relation / asset CRUD
├── fts.rs        ← FTS5 全文搜索
├── types.rs      ← CardIndex, SyncStatus, FtsHit 等数据结构
└── error.rs      ← MetaDbError
```

**关键依赖：** rusqlite (bundled), sha2

---

### `ms-io` — 文件与媒体控制器

原子文件写入（纳秒随机后缀）、Vault 目录扫描、WebP 压缩、ZIP 导出为默认能力，S3 存储为 opt-in。

```rust
use ms_io::fs::write_atomic;
use ms_io::scanner::scan_markdown_files;

write_atomic(&path, &content)?;
let md_files = scan_markdown_files(&vault_root)?;
```

**Feature flags：**

| Feature    | 默认开启 | 说明               |
| ---------- | -------- | ------------------ |
| `export`   | 是       | ZIP 知识库导出     |
| `media`    | 是       | WebP 图片压缩      |
| `storage`  | 否       | S3 对象存储（opt-in） |

**关键依赖：** walkdir, image, webp, zip, rust-s3, tokio

## 错误处理规范

所有 crate 使用 `thiserror` 定义错误类型，生产代码禁止 `unwrap()` / `expect()`：

```rust
// ms-ast:    MSResult<T>
// ms-io:     IoResult<T>
// ms-graph:  GraphResult<T>   (统一 IPC 错误契约，可序列化给前端)
// ms-meta:   MetaResult<T>
```

## 测试覆盖

| Crate     | 测试数 | 状态 |
| --------- | ------ | ---- |
| ms-ast    | 55     | 通过 |
| ms-graph  | 23     | 通过 |
| ms-io     | 17     | 通过 |
| ms-meta   | 4      | 通过 |

**99 tests, 0 clippy warnings, 0 unsafe, 0 production unwrap().**

## 版本策略

所有 crate 共享 `Cargo.workspace` 依赖版本管理，统一 `Cargo.lock` 确保版本一致性。
Release profile 启用 LTO + strip + panic=abort，产出最小二进制体积。
