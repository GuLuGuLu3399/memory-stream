# 🦀 Rust Workspace — Memory Stream 核心引擎

> Memory Stream 系统的 Rust 基础层，提供 Markdown 解析、AST 渲染、图片压缩、S3 存储、本地草稿、TOC 提取、知识库导出等核心能力。
>
> **路线图**：[CHECKLIST.md](../CHECKLIST.md) · **当前版本**：V3.4

## 架构总览

```
rust-workspace/
├── ast-core/        # AST 共享类型定义（所有 AST 相关 crate 的基础依赖）
├── ast-gen/         # Markdown → AST 解析器（基于 pulldown-cmark）
├── ast-renderer/    # AST → HTML 渲染器（支持 wikilink、代码高亮、数学公式）
├── md-parser/       # 高层封装：一行调用完成 parse → AST → HTML
├── image-compressor/# WebP 图片压缩（lossless，内存中操作）
├── ms-storage/      # S3 兼容对象存储抽象层（MinIO/Aliyun OSS/AWS S3）
├── ms-toc-extractor/# Markdown TOC 目录树提取器
├── ms-kb-exporter/  # 知识库全量导出（Markdown + 元数据 → ZIP）
├── ms-local-draft/  # 本地 SQLite 草稿管理（离线编辑支持）
└── wasm-engine/     # WASM 编译目标（将 md-parser 编译为浏览器可用模块）
```

## 依赖关系图

```
md-parser ──→ ast-gen ──→ ast-core
    └──────→ ast-renderer ──→ ast-core

wasm-engine ──→ md-parser
admin-tauri ──→ md-parser, ast-renderer, ms-storage, ms-toc-extractor, ms-kb-exporter, ms-local-draft
```

## 快速开始

```bash
# 构建 workspace
cargo build --workspace

# 运行测试
cargo test --workspace

# 构建 WASM（需要 wasm-pack）
cd wasm-engine && wasm-pack build --target web
```

## 各 Crate 说明

### `md-parser`

核心入口。接收原始 Markdown 文本，返回结构化 AST + HTML。

```rust
let ast = parse_markdown(&content)?;
let html = render_to_html(&ast)?;
```

### `ast-gen` / `ast-core` / `ast-renderer`

三层分离设计：

- **ast-core**：纯类型定义，无任何逻辑依赖
- **ast-gen**：Markdown 文本 → AST（pulldown-cmark 扩展）
- **ast-renderer**：AST → HTML（支持自定义渲染规则）

### `ms-storage`

S3 兼容对象存储的 trait 抽象 + 实现。

```rust
let backend = S3Backend::new(&config)?;
let url = backend.upload("key", &data, "image/webp").await?;
```

### `image-compressor`

WebP 无损压缩，用于 Tauri 端图片优化。

### `ms-local-draft`

SQLite 草稿存储，支持 CRUD + 自动时间戳。

### `ms-toc-extractor`

从 Markdown AST 提取 TOC（目录树）结构。

### `ms-kb-exporter`

知识库全量导出，生成 ZIP 压缩包。

### `wasm-engine`

将 md-parser 编译为 WASM，供 Web 端直接在浏览器中解析 Markdown。

## 版本策略

所有 crate 共享 `Cargo.workspace` 依赖版本管理，统一 `Cargo.lock` 确保版本一致性。
