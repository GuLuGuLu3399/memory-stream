//! AST 核心数据结构 — 定义 Markdown 抽象语法树的节点类型和访问者模式。
//!
//! 本模块提供了 Memory Stream 的核心抽象语法树（AST）定义，包括：
//! - `AstNode`：AST 节点枚举，涵盖各种 Markdown 元素（标题、段落、代码块等）
//! - `visitor`：访问者模式实现，用于遍历和转换 AST
//! - `AdmonitionKind`、`AlignType`、`TaskStatus`：各种辅助类型
//!
//! # 功能特性
//!
//! - **条件编译**：
//!   - `parser` 特性：启用 Markdown 解析功能（`parse_markdown`、`extract_links` 等）
//!   - `toc` 特性：启用目录提取功能（`extract_toc`、`extract_toc_flat` 等）
//! - **WASM 兼容**：支持编译到 WebAssembly，用于浏览器环境
//! - **零拷贝设计**：尽可能使用引用避免不必要的克隆

pub mod error;
mod node;
pub mod visitor;
pub use node::{AdmonitionKind, AlignType, AstNode, AstNodeOwned, TaskStatus};

#[cfg(feature = "parser")]
pub mod parser;
#[cfg(feature = "parser")]
pub use parser::{extract_links, parse_markdown, parse_markdown_with, ParseOptions};

#[cfg(feature = "parser")]
pub mod document;
#[cfg(feature = "parser")]
pub use document::{parse_document, parse_document_with, ParsedDocument};

#[cfg(feature = "toc")]
pub mod toc;
#[cfg(feature = "toc")]
pub use toc::{extract_toc, extract_toc_flat, extract_toc_from_json, TocFlatItem, TocNode};
