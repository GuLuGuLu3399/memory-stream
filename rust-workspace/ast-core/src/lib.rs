//! MemoryStream AST 核心类型定义。
//!
//! 定义了 Markdown 文档的抽象语法树节点类型，支持零拷贝解析（`Cow<'a, str>`）
//! 和 JSON 序列化/反序列化（`serde` tag = "type" 判别式）。

pub mod error;

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// 拥有所有权的 AST 节点类型（等价于 `AstNode<'static>`）。
/// 用于反序列化和长期存储场景。
pub type AstNodeOwned = AstNode<'static>;

/// 表格列对齐类型。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlignType {
    Left,
    Center,
    Right,
    None,
}

/// Markdown AST 节点枚举。
///
/// 使用 `#[serde(tag = "type")]` 进行 JSON 序列化，输出格式如：
/// `{"type": "Heading", "level": 2, "children": [...]}`
///
/// 所有文本字段使用 `Cow<'a, str>` 实现零拷贝解析。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum AstNode<'a> {
    /// 文档根节点
    Root { children: Vec<AstNode<'a>> },
    /// 标题节点（h1-h6）
    Heading {
        level: u8,
        children: Vec<AstNode<'a>>,
    },
    /// 段落节点
    Paragraph { children: Vec<AstNode<'a>> },
    /// 纯文本叶子节点
    Text { value: Cow<'a, str> },
    /// 代码块（支持语言标注）
    CodeBlock {
        language: Option<Cow<'a, str>>,
        value: Cow<'a, str>,
    },
    /// 超链接
    Link {
        url: Cow<'a, str>,
        children: Vec<AstNode<'a>>,
    },
    /// 图片（含 alt 文本）
    Image {
        url: Cow<'a, str>,
        alt: Cow<'a, str>,
    },
    /// 数学公式（`inline=true` 为行内公式，`false` 为块级公式）
    Math { value: Cow<'a, str>, inline: bool },
    /// 引用块
    Blockquote { children: Vec<AstNode<'a>> },
    /// 列表（有序/无序）
    List {
        ordered: bool,
        start: Option<u64>,
        children: Vec<AstNode<'a>>,
    },
    /// 列表项
    ListItem { children: Vec<AstNode<'a>> },
    /// 水平分割线
    ThematicBreak,
    /// 加粗文本
    Strong { children: Vec<AstNode<'a>> },
    /// 斜体文本
    Emphasis { children: Vec<AstNode<'a>> },
    /// 表格（GFM）
    Table {
        alignments: Vec<Option<AlignType>>,
        children: Vec<AstNode<'a>>,
    },
    /// 表头行
    TableHead { children: Vec<AstNode<'a>> },
    /// 表格数据行
    TableRow { children: Vec<AstNode<'a>> },
    /// 表格单元格
    TableCell { children: Vec<AstNode<'a>> },
    /// 删除线（GFM）
    Strikethrough { children: Vec<AstNode<'a>> },
    /// 任务列表标记
    TaskListMarker { checked: bool },
}

impl<'a> AstNode<'a> {
    /// 向当前节点追加子节点。
    ///
    /// 仅对容器类型节点有效（Root、Heading、Paragraph 等），
    /// 对叶子节点（Text、ThematicBreak、Image）静默忽略。
    pub fn push_child(&mut self, child: AstNode<'a>) {
        match self {
            AstNode::Root { children } => children.push(child),
            AstNode::Heading { children, .. } => children.push(child),
            AstNode::Paragraph { children } => children.push(child),
            AstNode::Blockquote { children } => children.push(child),
            AstNode::List { children, .. } => children.push(child),
            AstNode::ListItem { children } => children.push(child),
            AstNode::Strong { children } => children.push(child),
            AstNode::Emphasis { children } => children.push(child),
            AstNode::Link { children, .. } => children.push(child),
            AstNode::Table { children, .. } => children.push(child),
            AstNode::TableHead { children } => children.push(child),
            AstNode::TableRow { children } => children.push(child),
            AstNode::TableCell { children } => children.push(child),
            AstNode::Strikethrough { children } => children.push(child),
            _ => {}
        }
    }
}
