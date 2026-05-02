use crate::error::MSError;
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

/// 警示块类型。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AdmonitionKind {
    Warning,
    Tip,
    Question,
}

/// 标题进度标记。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Done,
    Undone,
    Unclear,
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
        status: Option<TaskStatus>,
    },
    /// 段落节点
    Paragraph { children: Vec<AstNode<'a>> },
    /// 纯文本叶子节点
    Text { value: Cow<'a, str> },
    /// Wikilink 节点（`[[target]]` 或 `[[target|alias]]`）
    Wikilink {
        target: Cow<'a, str>,
        alias: Option<Cow<'a, str>>,
    },
    /// 行内代码（`code`）
    InlineCode { value: Cow<'a, str> },
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
    /// 脚注定义（`[^label]: 内容`）
    FootnoteDefinition {
        name: Cow<'a, str>,
        children: Vec<AstNode<'a>>,
    },
    /// 脚注引用（`[^label]`）
    FootnoteReference { name: Cow<'a, str> },
    /// 定义列表（`Term: Definition`）
    DefinitionList { children: Vec<AstNode<'a>> },
    /// 定义列表术语
    DefinitionListTitle { children: Vec<AstNode<'a>> },
    /// 定义列表定义
    DefinitionListDefinition { children: Vec<AstNode<'a>> },
    /// 记忆遮罩（`??text??`）
    Cloze { children: Vec<AstNode<'a>> },
    /// 概念引用（`@术语`）
    ConceptRef { term: Cow<'a, str> },
    /// 警示块（`> [!warning]` / `> [!tip]` / `> [!question]`）
    Admonition {
        kind: AdmonitionKind,
        children: Vec<AstNode<'a>>,
    },
    // 预留：Superscript / Subscript — 等 pulldown-cmark 支持后添加
}

impl<'a> AstNode<'a> {
    /// 向当前节点追加子节点。
    ///
    /// 仅对容器类型节点有效（Root、Heading、Paragraph 等），
    /// 对叶子节点（Text、ThematicBreak、Image）返回 `InvalidOperation` 错误。
    ///
    /// # Errors
    /// 当当前节点是叶子节点时返回 `MSError::InvalidOperation`。
    pub fn push_child(&mut self, child: AstNode<'a>) -> Result<(), MSError> {
        match self {
            AstNode::Root { children }
            | AstNode::Heading { children, .. }
            | AstNode::Paragraph { children }
            | AstNode::Blockquote { children }
            | AstNode::List { children, .. }
            | AstNode::ListItem { children }
            | AstNode::Strong { children }
            | AstNode::Emphasis { children }
            | AstNode::Link { children, .. }
            | AstNode::Table { children, .. }
            | AstNode::TableHead { children }
            | AstNode::TableRow { children }
            | AstNode::TableCell { children }
            | AstNode::Strikethrough { children }
            | AstNode::FootnoteDefinition { children, .. }
            | AstNode::DefinitionList { children }
            | AstNode::DefinitionListTitle { children }
            | AstNode::DefinitionListDefinition { children }
            | AstNode::Cloze { children }
            | AstNode::Admonition { children, .. } => {
                children.push(child);
                Ok(())
            }
            _ => Err(MSError::InvalidOperation(format!(
                "Cannot push child to leaf node {:?}",
                self.variant_name()
            ))),
        }
    }

    /// 返回节点变体名称（用于错误信息）。
    fn variant_name(&self) -> &'static str {
        match self {
            AstNode::Root { .. } => "Root",
            AstNode::Heading { .. } => "Heading",
            AstNode::Paragraph { .. } => "Paragraph",
            AstNode::Text { .. } => "Text",
            AstNode::Wikilink { .. } => "Wikilink",
            AstNode::InlineCode { .. } => "InlineCode",
            AstNode::CodeBlock { .. } => "CodeBlock",
            AstNode::Link { .. } => "Link",
            AstNode::Image { .. } => "Image",
            AstNode::Math { .. } => "Math",
            AstNode::Blockquote { .. } => "Blockquote",
            AstNode::List { .. } => "List",
            AstNode::ListItem { .. } => "ListItem",
            AstNode::ThematicBreak => "ThematicBreak",
            AstNode::Strong { .. } => "Strong",
            AstNode::Emphasis { .. } => "Emphasis",
            AstNode::Table { .. } => "Table",
            AstNode::TableHead { .. } => "TableHead",
            AstNode::TableRow { .. } => "TableRow",
            AstNode::TableCell { .. } => "TableCell",
            AstNode::Strikethrough { .. } => "Strikethrough",
            AstNode::TaskListMarker { .. } => "TaskListMarker",
            AstNode::FootnoteDefinition { .. } => "FootnoteDefinition",
            AstNode::FootnoteReference { .. } => "FootnoteReference",
            AstNode::DefinitionList { .. } => "DefinitionList",
            AstNode::DefinitionListTitle { .. } => "DefinitionListTitle",
            AstNode::DefinitionListDefinition { .. } => "DefinitionListDefinition",
            AstNode::Cloze { .. } => "Cloze",
            AstNode::ConceptRef { .. } => "ConceptRef",
            AstNode::Admonition { .. } => "Admonition",
        }
    }
}
