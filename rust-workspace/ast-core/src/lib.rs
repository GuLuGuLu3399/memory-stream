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

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    // ---------------------------------------------------------------------------
    // AlignType
    // ---------------------------------------------------------------------------

    #[test]
    fn test_align_type_equality() {
        assert_eq!(AlignType::Left, AlignType::Left);
        assert_ne!(AlignType::Left, AlignType::Center);
        assert_ne!(AlignType::Center, AlignType::Right);
        assert_ne!(AlignType::Right, AlignType::None);
    }

    #[test]
    fn test_align_type_serde_roundtrip() {
        for variant in [
            AlignType::Left,
            AlignType::Center,
            AlignType::Right,
            AlignType::None,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: AlignType = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn test_align_type_clone_debug() {
        let a = AlignType::Center;
        let b = a.clone();
        assert_eq!(a, b);
        let debug = format!("{a:?}");
        assert!(debug.contains("Center"));
    }

    // ---------------------------------------------------------------------------
    // AstNode construction & field access
    // ---------------------------------------------------------------------------

    #[test]
    fn test_text_borrowed() {
        let node = AstNode::Text {
            value: Cow::Borrowed("hello"),
        };
        if let AstNode::Text { value } = &node {
            assert_eq!(value.as_ref(), "hello");
        } else {
            panic!("expected Text variant");
        }
    }

    #[test]
    fn test_text_owned() {
        let node = AstNode::Text {
            value: Cow::Owned("owned string".to_string()),
        };
        if let AstNode::Text { value } = &node {
            assert_eq!(value.as_ref(), "owned string");
        } else {
            panic!("expected Text variant");
        }
    }

    #[test]
    fn test_heading_fields() {
        let node = AstNode::Heading {
            level: 3,
            children: vec![AstNode::Text {
                value: Cow::Borrowed("Title"),
            }],
        };
        if let AstNode::Heading { level, children } = &node {
            assert_eq!(*level, 3);
            assert_eq!(children.len(), 1);
        } else {
            panic!("expected Heading variant");
        }
    }

    #[test]
    fn test_code_block_with_language() {
        let node = AstNode::CodeBlock {
            language: Some(Cow::Borrowed("rust")),
            value: Cow::Borrowed("fn main() {}"),
        };
        if let AstNode::CodeBlock { language, value } = &node {
            assert_eq!(language.as_deref(), Some("rust"));
            assert_eq!(value.as_ref(), "fn main() {}");
        } else {
            panic!("expected CodeBlock variant");
        }
    }

    #[test]
    fn test_code_block_without_language() {
        let node = AstNode::CodeBlock {
            language: None,
            value: Cow::Borrowed("plain text"),
        };
        if let AstNode::CodeBlock { language, .. } = &node {
            assert!(language.is_none());
        } else {
            panic!("expected CodeBlock variant");
        }
    }

    #[test]
    fn test_link_fields() {
        let node = AstNode::Link {
            url: Cow::Borrowed("https://example.com"),
            children: vec![AstNode::Text {
                value: Cow::Borrowed("click"),
            }],
        };
        if let AstNode::Link { url, children } = &node {
            assert_eq!(url.as_ref(), "https://example.com");
            assert_eq!(children.len(), 1);
        } else {
            panic!("expected Link variant");
        }
    }

    #[test]
    fn test_image_fields() {
        let node = AstNode::Image {
            url: Cow::Borrowed("img.png"),
            alt: Cow::Borrowed("alt text"),
        };
        if let AstNode::Image { url, alt } = &node {
            assert_eq!(url.as_ref(), "img.png");
            assert_eq!(alt.as_ref(), "alt text");
        } else {
            panic!("expected Image variant");
        }
    }

    #[test]
    fn test_math_inline_and_block() {
        let inline = AstNode::Math {
            value: Cow::Borrowed("E=mc^2"),
            inline: true,
        };
        let block = AstNode::Math {
            value: Cow::Borrowed("\\int_0^1"),
            inline: false,
        };
        if let AstNode::Math { value, inline: is_inline } = &inline {
            assert_eq!(value.as_ref(), "E=mc^2");
            assert!(is_inline);
        }
        if let AstNode::Math { inline: is_inline, .. } = &block {
            assert!(!is_inline);
        }
    }

    #[test]
    fn test_list_ordered_with_start() {
        let node = AstNode::List {
            ordered: true,
            start: Some(3),
            children: vec![AstNode::ListItem {
                children: vec![AstNode::Text {
                    value: Cow::Borrowed("item"),
                }],
            }],
        };
        if let AstNode::List {
            ordered,
            start,
            children,
        } = &node
        {
            assert!(ordered);
            assert_eq!(*start, Some(3));
            assert_eq!(children.len(), 1);
        } else {
            panic!("expected List variant");
        }
    }

    #[test]
    fn test_list_unordered() {
        let node = AstNode::List {
            ordered: false,
            start: None,
            children: vec![],
        };
        if let AstNode::List {
            ordered,
            start,
            children,
        } = &node
        {
            assert!(!ordered);
            assert!(start.is_none());
            assert!(children.is_empty());
        } else {
            panic!("expected List variant");
        }
    }

    #[test]
    fn test_thematic_break() {
        let node = AstNode::ThematicBreak;
        assert!(matches!(node, AstNode::ThematicBreak));
    }

    #[test]
    fn test_task_list_marker() {
        let checked = AstNode::TaskListMarker { checked: true };
        let unchecked = AstNode::TaskListMarker { checked: false };
        if let AstNode::TaskListMarker { checked: c } = &checked {
            assert!(c);
        }
        if let AstNode::TaskListMarker { checked: c } = &unchecked {
            assert!(!c);
        }
    }

    #[test]
    fn test_table_with_alignments() {
        let node = AstNode::Table {
            alignments: vec![Some(AlignType::Left), None, Some(AlignType::Center)],
            children: vec![AstNode::TableHead { children: vec![] }],
        };
        if let AstNode::Table {
            alignments, children, ..
        } = &node
        {
            assert_eq!(alignments.len(), 3);
            assert_eq!(alignments[0], Some(AlignType::Left));
            assert_eq!(alignments[1], None);
            assert_eq!(alignments[2], Some(AlignType::Center));
            assert_eq!(children.len(), 1);
        } else {
            panic!("expected Table variant");
        }
    }

    #[test]
    fn test_strikethrough() {
        let node = AstNode::Strikethrough {
            children: vec![AstNode::Text {
                value: Cow::Borrowed("deleted"),
            }],
        };
        if let AstNode::Strikethrough { children } = &node {
            assert_eq!(children.len(), 1);
        } else {
            panic!("expected Strikethrough variant");
        }
    }

    #[test]
    fn test_blockquote() {
        let node = AstNode::Blockquote {
            children: vec![AstNode::Paragraph {
                children: vec![AstNode::Text {
                    value: Cow::Borrowed("quote"),
                }],
            }],
        };
        if let AstNode::Blockquote { children } = &node {
            assert_eq!(children.len(), 1);
        } else {
            panic!("expected Blockquote variant");
        }
    }

    #[test]
    fn test_strong_and_emphasis() {
        let strong = AstNode::Strong {
            children: vec![AstNode::Text {
                value: Cow::Borrowed("bold"),
            }],
        };
        let em = AstNode::Emphasis {
            children: vec![AstNode::Text {
                value: Cow::Borrowed("italic"),
            }],
        };
        assert!(matches!(strong, AstNode::Strong { .. }));
        assert!(matches!(em, AstNode::Emphasis { .. }));
    }

    // ---------------------------------------------------------------------------
    // push_child
    // ---------------------------------------------------------------------------

    #[test]
    fn test_push_child_root() {
        let mut root = AstNode::Root { children: vec![] };
        root.push_child(AstNode::ThematicBreak);
        if let AstNode::Root { children } = &root {
            assert_eq!(children.len(), 1);
        }
    }

    #[test]
    fn test_push_child_heading() {
        let mut h = AstNode::Heading {
            level: 1,
            children: vec![],
        };
        h.push_child(AstNode::Text {
            value: Cow::Borrowed("hi"),
        });
        if let AstNode::Heading { children, .. } = &h {
            assert_eq!(children.len(), 1);
        }
    }

    #[test]
    fn test_push_child_paragraph() {
        let mut p = AstNode::Paragraph { children: vec![] };
        p.push_child(AstNode::Text {
            value: Cow::Borrowed("text"),
        });
        if let AstNode::Paragraph { children } = &p {
            assert_eq!(children.len(), 1);
        }
    }

    #[test]
    fn test_push_child_blockquote() {
        let mut bq = AstNode::Blockquote { children: vec![] };
        bq.push_child(AstNode::Paragraph { children: vec![] });
        if let AstNode::Blockquote { children } = &bq {
            assert_eq!(children.len(), 1);
        }
    }

    #[test]
    fn test_push_child_list() {
        let mut list = AstNode::List {
            ordered: false,
            start: None,
            children: vec![],
        };
        list.push_child(AstNode::ListItem { children: vec![] });
        if let AstNode::List { children, .. } = &list {
            assert_eq!(children.len(), 1);
        }
    }

    #[test]
    fn test_push_child_list_item() {
        let mut li = AstNode::ListItem { children: vec![] };
        li.push_child(AstNode::Text {
            value: Cow::Borrowed("x"),
        });
        if let AstNode::ListItem { children } = &li {
            assert_eq!(children.len(), 1);
        }
    }

    #[test]
    fn test_push_child_strong() {
        let mut s = AstNode::Strong { children: vec![] };
        s.push_child(AstNode::Text {
            value: Cow::Borrowed("bold"),
        });
        if let AstNode::Strong { children } = &s {
            assert_eq!(children.len(), 1);
        }
    }

    #[test]
    fn test_push_child_emphasis() {
        let mut e = AstNode::Emphasis { children: vec![] };
        e.push_child(AstNode::Text {
            value: Cow::Borrowed("em"),
        });
        if let AstNode::Emphasis { children } = &e {
            assert_eq!(children.len(), 1);
        }
    }

    #[test]
    fn test_push_child_link() {
        let mut link = AstNode::Link {
            url: Cow::Borrowed("https://example.com"),
            children: vec![],
        };
        link.push_child(AstNode::Text {
            value: Cow::Borrowed("label"),
        });
        if let AstNode::Link { children, .. } = &link {
            assert_eq!(children.len(), 1);
        }
    }

    #[test]
    fn test_push_child_table_variants() {
        let mut table = AstNode::Table {
            alignments: vec![],
            children: vec![],
        };
        table.push_child(AstNode::TableHead { children: vec![] });
        if let AstNode::Table { children, .. } = &table {
            assert_eq!(children.len(), 1);
        }

        let mut head = AstNode::TableHead { children: vec![] };
        head.push_child(AstNode::TableCell { children: vec![] });
        if let AstNode::TableHead { children } = &head {
            assert_eq!(children.len(), 1);
        }

        let mut row = AstNode::TableRow { children: vec![] };
        row.push_child(AstNode::TableCell { children: vec![] });
        if let AstNode::TableRow { children } = &row {
            assert_eq!(children.len(), 1);
        }

        let mut cell = AstNode::TableCell { children: vec![] };
        cell.push_child(AstNode::Text {
            value: Cow::Borrowed("data"),
        });
        if let AstNode::TableCell { children } = &cell {
            assert_eq!(children.len(), 1);
        }
    }

    #[test]
    fn test_push_child_strikethrough() {
        let mut s = AstNode::Strikethrough { children: vec![] };
        s.push_child(AstNode::Text {
            value: Cow::Borrowed("struck"),
        });
        if let AstNode::Strikethrough { children } = &s {
            assert_eq!(children.len(), 1);
        }
    }

    #[test]
    fn test_push_child_ignored_on_leaf_nodes() {
        // Text is a leaf — push_child should silently do nothing.
        let mut text = AstNode::Text {
            value: Cow::Borrowed("leaf"),
        };
        text.push_child(AstNode::ThematicBreak);
        assert!(matches!(text, AstNode::Text { .. }));

        // ThematicBreak is a leaf.
        let mut tb = AstNode::ThematicBreak;
        tb.push_child(AstNode::Text {
            value: Cow::Borrowed("x"),
        });
        assert!(matches!(tb, AstNode::ThematicBreak));

        // Image is a leaf.
        let mut img = AstNode::Image {
            url: Cow::Borrowed("a.png"),
            alt: Cow::Borrowed("a"),
        };
        img.push_child(AstNode::Text {
            value: Cow::Borrowed("x"),
        });
        if let AstNode::Image { url, alt } = &img {
            assert_eq!(url.as_ref(), "a.png");
            assert_eq!(alt.as_ref(), "a");
        }

        // Math is a leaf.
        let mut math = AstNode::Math {
            value: Cow::Borrowed("x"),
            inline: true,
        };
        math.push_child(AstNode::ThematicBreak);
        if let AstNode::Math { value, inline } = &math {
            assert_eq!(value.as_ref(), "x");
            assert!(inline);
        }

        // TaskListMarker is a leaf.
        let mut tlm = AstNode::TaskListMarker { checked: false };
        tlm.push_child(AstNode::ThematicBreak);
        if let AstNode::TaskListMarker { checked } = &tlm {
            assert!(!checked);
        }

        // CodeBlock is a leaf.
        let mut cb = AstNode::CodeBlock {
            language: None,
            value: Cow::Borrowed("code"),
        };
        cb.push_child(AstNode::ThematicBreak);
        if let AstNode::CodeBlock { language, value } = &cb {
            assert!(language.is_none());
            assert_eq!(value.as_ref(), "code");
        }
    }

    // ---------------------------------------------------------------------------
    // Serde round-trip (tag = "type")
    // ---------------------------------------------------------------------------

    #[test]
    fn test_serde_root() {
        let node = AstNode::Root { children: vec![] };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"Root""#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_heading() {
        let node = AstNode::Heading {
            level: 2,
            children: vec![AstNode::Text {
                value: Cow::Borrowed("Hello"),
            }],
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"Heading""#));
        assert!(json.contains(r#""level":2"#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_paragraph_with_text() {
        let node = AstNode::Paragraph {
            children: vec![AstNode::Text {
                value: Cow::Borrowed("world"),
            }],
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"Paragraph""#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_text() {
        let node = AstNode::Text {
            value: Cow::Borrowed("plain"),
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"Text""#));
        assert!(json.contains(r#""value":"plain""#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_code_block_with_lang() {
        let node = AstNode::CodeBlock {
            language: Some(Cow::Borrowed("rust")),
            value: Cow::Borrowed("let x = 1;"),
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"CodeBlock""#));
        assert!(json.contains(r#""language":"rust""#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_code_block_without_lang() {
        let node = AstNode::CodeBlock {
            language: None,
            value: Cow::Borrowed("plain"),
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"CodeBlock""#));
        // language should be null when None
        assert!(json.contains(r#""language":null"#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_link() {
        let node = AstNode::Link {
            url: Cow::Borrowed("https://example.com"),
            children: vec![AstNode::Text {
                value: Cow::Borrowed("link text"),
            }],
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"Link""#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_image() {
        let node = AstNode::Image {
            url: Cow::Borrowed("photo.jpg"),
            alt: Cow::Borrowed("A photo"),
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"Image""#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_math_inline() {
        let node = AstNode::Math {
            value: Cow::Borrowed("x^2"),
            inline: true,
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"Math""#));
        assert!(json.contains(r#""inline":true"#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_math_block() {
        let node = AstNode::Math {
            value: Cow::Borrowed("\\sum"),
            inline: false,
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""inline":false"#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_blockquote() {
        let node = AstNode::Blockquote {
            children: vec![AstNode::Paragraph {
                children: vec![AstNode::Text {
                    value: Cow::Borrowed("quoted"),
                }],
            }],
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"Blockquote""#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_list_ordered() {
        let node = AstNode::List {
            ordered: true,
            start: Some(5),
            children: vec![AstNode::ListItem {
                children: vec![AstNode::Text {
                    value: Cow::Borrowed("item"),
                }],
            }],
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"List""#));
        assert!(json.contains(r#""ordered":true"#));
        assert!(json.contains(r#""start":5"#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_list_unordered_no_start() {
        let node = AstNode::List {
            ordered: false,
            start: None,
            children: vec![],
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""ordered":false"#));
        assert!(json.contains(r#""start":null"#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_list_item() {
        let node = AstNode::ListItem {
            children: vec![AstNode::Text {
                value: Cow::Borrowed("entry"),
            }],
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"ListItem""#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_thematic_break() {
        let node = AstNode::ThematicBreak;
        let json = serde_json::to_string(&node).unwrap();
        assert_eq!(json, r#"{"type":"ThematicBreak"}"#);
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_strong() {
        let node = AstNode::Strong {
            children: vec![AstNode::Text {
                value: Cow::Borrowed("bold"),
            }],
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"Strong""#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_emphasis() {
        let node = AstNode::Emphasis {
            children: vec![AstNode::Text {
                value: Cow::Borrowed("italic"),
            }],
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"Emphasis""#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_table() {
        let node = AstNode::Table {
            alignments: vec![Some(AlignType::Left), None],
            children: vec![
                AstNode::TableHead {
                    children: vec![AstNode::TableCell {
                        children: vec![AstNode::Text {
                            value: Cow::Borrowed("H1"),
                        }],
                    }],
                },
                AstNode::TableRow {
                    children: vec![AstNode::TableCell {
                        children: vec![AstNode::Text {
                            value: Cow::Borrowed("D1"),
                        }],
                    }],
                },
            ],
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"Table""#));
        assert!(json.contains(r#""type":"TableHead""#));
        assert!(json.contains(r#""type":"TableRow""#));
        assert!(json.contains(r#""type":"TableCell""#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_strikethrough() {
        let node = AstNode::Strikethrough {
            children: vec![AstNode::Text {
                value: Cow::Borrowed("deleted"),
            }],
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"Strikethrough""#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_task_list_marker() {
        let node = AstNode::TaskListMarker { checked: true };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"TaskListMarker""#));
        assert!(json.contains(r#""checked":true"#));
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn test_serde_deeply_nested_tree() {
        // Build: Root > Blockquote > List > ListItem > Paragraph > Strong > Text
        let tree = AstNode::Root {
            children: vec![AstNode::Blockquote {
                children: vec![AstNode::List {
                    ordered: false,
                    start: None,
                    children: vec![AstNode::ListItem {
                        children: vec![AstNode::Paragraph {
                            children: vec![AstNode::Strong {
                                children: vec![AstNode::Text {
                                    value: Cow::Borrowed("deep"),
                                }],
                            }],
                        }],
                    }],
                }],
            }],
        };
        let json = serde_json::to_string(&tree).unwrap();
        let back: AstNodeOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(tree, back);
    }

    #[test]
    fn test_serde_deserialize_invalid_type() {
        let bad_json = r#"{"type":"NonExistent","value":"x"}"#;
        let result: Result<AstNodeOwned, _> = serde_json::from_str(bad_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serde_deserialize_missing_tag() {
        let bad_json = r#"{"level":1,"children":[]}"#;
        let result: Result<AstNodeOwned, _> = serde_json::from_str(bad_json);
        assert!(result.is_err());
    }

    // ---------------------------------------------------------------------------
    // Clone & Debug
    // ---------------------------------------------------------------------------

    #[test]
    fn test_clone_preserves_data() {
        let original = AstNode::CodeBlock {
            language: Some(Cow::Owned("ts".to_string())),
            value: Cow::Owned("console.log(1)".to_string()),
        };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_debug_output_contains_variant() {
        let node = AstNode::Heading {
            level: 1,
            children: vec![],
        };
        let debug = format!("{node:?}");
        assert!(debug.contains("Heading"));
    }

    // ---------------------------------------------------------------------------
    // AstNodeOwned type alias
    // ---------------------------------------------------------------------------

    #[test]
    fn test_owned_type_alias_deserialize() {
        let json = r#"{"type":"Text","value":"hello"}"#;
        let owned: AstNodeOwned = serde_json::from_str(json).unwrap();
        // AstNodeOwned is AstNode<'static>, so we can move it freely.
        if let AstNode::Text { value } = owned {
            assert_eq!(value.as_ref(), "hello");
        } else {
            panic!("expected Text");
        }
    }

    #[test]
    fn test_owned_from_owned_cow() {
        let node: AstNodeOwned = AstNode::Text {
            value: Cow::Owned("owned".to_string()),
        };
        if let AstNode::Text { value } = &node {
            assert_eq!(value.as_ref(), "owned");
        }
    }

    // ---------------------------------------------------------------------------
    // PartialEq
    // ---------------------------------------------------------------------------

    #[test]
    fn test_partial_eq_different_variants() {
        let a = AstNode::Text {
            value: Cow::Borrowed("x"),
        };
        let b = AstNode::ThematicBreak;
        assert_ne!(a, b);
    }

    #[test]
    fn test_partial_eq_same_variant_different_fields() {
        let a = AstNode::Heading {
            level: 1,
            children: vec![],
        };
        let b = AstNode::Heading {
            level: 2,
            children: vec![],
        };
        assert_ne!(a, b);
    }
}
