//! AST 访问者 trait 与共享遍历工具。
//!
//! 提供标准的 AST 遍历机制和跨 crate 复用的工具函数：
//! - [`AstVisitor`] — 可扩展的访问者 trait，每个变体一个钩子方法
//! - [`walk`] / [`walk_children`] — 默认递归遍历驱动器
//! - [`collect_plain_text`] — 从子节点提取纯文本
//! - [`generate_slug`] / [`is_cjk`] — URL 安全 slug 生成

use crate::AdmonitionKind;
use crate::AlignType;
use crate::AstNode;

// ────────────────────────────────────────────────────────────────
// AstVisitor trait
// ────────────────────────────────────────────────────────────────

/// AST 访问者 trait — 支持自定义遍历逻辑。
///
/// 每个方法对应一个 `AstNode` 变体，默认实现会递归遍历子节点。
/// 实现者只需覆盖关心的节点类型。
///
/// # 生命期
/// `'a` 是 AST 内部文本数据（`Cow<'a, str>`）的生命期，
/// 与访问者自身的生命期无关。
///
/// # 示例
///
/// ```ignore
/// use ast_core::visitor::{AstVisitor, walk};
/// use ast_core::AstNode;
///
/// /// 收集文档中所有标题的级别和文本。
/// struct HeadingCollector {
///     headings: Vec<(u8, String)>,
/// }
///
/// impl<'a> AstVisitor<'a> for HeadingCollector {
///     fn visit_heading(&mut self, level: u8, children: &[AstNode<'a>]) {
///         let text = ast_core::visitor::collect_plain_text(children);
///         self.headings.push((level, text));
///         // 不再递归子节点 — 标题内部文本已被提取
///     }
/// }
///
/// let mut collector = HeadingCollector { headings: vec![] };
/// walk(&mut collector, &ast);
/// ```
pub trait AstVisitor<'a> {
    // ── 容器节点（默认递归遍历子节点） ──

    fn visit_root(&mut self, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_heading(&mut self, _level: u8, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_paragraph(&mut self, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_blockquote(&mut self, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_list(&mut self, _ordered: bool, _start: Option<u64>, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_list_item(&mut self, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_strong(&mut self, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_emphasis(&mut self, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_link(&mut self, _url: &str, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_table(&mut self, _alignments: &[Option<AlignType>], children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_table_head(&mut self, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_table_row(&mut self, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_table_cell(&mut self, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_strikethrough(&mut self, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_footnote_definition(&mut self, _name: &str, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_definition_list(&mut self, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_definition_list_title(&mut self, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_definition_list_definition(&mut self, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_cloze(&mut self, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    fn visit_concept_ref(&mut self, _term: &str) {}

    fn visit_admonition(&mut self, _kind: &AdmonitionKind, children: &[AstNode<'a>]) {
        for child in children {
            self.visit(child);
        }
    }

    // ── 叶子节点（默认无操作） ──

    fn visit_text(&mut self, _value: &str) {}

    fn visit_wikilink(&mut self, _target: &str, _alias: Option<&str>) {}

    fn visit_inline_code(&mut self, _value: &str) {}

    fn visit_code_block(&mut self, _language: Option<&str>, _value: &str) {}

    fn visit_image(&mut self, _url: &str, _alt: &str) {}

    fn visit_math(&mut self, _value: &str, _inline: bool) {}

    fn visit_thematic_break(&mut self) {}

    fn visit_task_list_marker(&mut self, _checked: bool) {}

    fn visit_footnote_link(&mut self, _name: &str) {}

    // ── 入口 ──

    /// 入口方法：将节点分发到对应的 `visit_*` 方法。
    ///
    /// 默认实现根据节点类型分发。覆盖此方法可拦截所有节点。
    fn visit(&mut self, node: &AstNode<'a>) {
        match node {
            AstNode::Root { children } => self.visit_root(children),
            AstNode::Heading { level, children, .. } => self.visit_heading(*level, children),
            AstNode::Paragraph { children } => self.visit_paragraph(children),
            AstNode::Text { value } => self.visit_text(value),
            AstNode::Wikilink { target, alias } => {
                self.visit_wikilink(target, alias.as_deref());
            }
            AstNode::InlineCode { value } => self.visit_inline_code(value),
            AstNode::CodeBlock { language, value } => {
                self.visit_code_block(language.as_deref(), value);
            }
            AstNode::Link { url, children } => self.visit_link(url, children),
            AstNode::Image { url, alt } => self.visit_image(url, alt),
            AstNode::Math { value, inline } => self.visit_math(value, *inline),
            AstNode::Blockquote { children } => self.visit_blockquote(children),
            AstNode::List {
                ordered,
                start,
                children,
            } => self.visit_list(*ordered, *start, children),
            AstNode::ListItem { children } => self.visit_list_item(children),
            AstNode::ThematicBreak => self.visit_thematic_break(),
            AstNode::Strong { children } => self.visit_strong(children),
            AstNode::Emphasis { children } => self.visit_emphasis(children),
            AstNode::Table {
                alignments,
                children,
            } => self.visit_table(alignments, children),
            AstNode::TableHead { children } => self.visit_table_head(children),
            AstNode::TableRow { children } => self.visit_table_row(children),
            AstNode::TableCell { children } => self.visit_table_cell(children),
            AstNode::Strikethrough { children } => self.visit_strikethrough(children),
            AstNode::TaskListMarker { checked } => self.visit_task_list_marker(*checked),
            AstNode::FootnoteDefinition { name, children } => {
                self.visit_footnote_definition(name, children);
            }
            AstNode::FootnoteReference { name } => self.visit_footnote_link(name),
            AstNode::DefinitionList { children } => self.visit_definition_list(children),
            AstNode::DefinitionListTitle { children } => self.visit_definition_list_title(children),
            AstNode::DefinitionListDefinition { children } => {
                self.visit_definition_list_definition(children);
            }
            AstNode::Cloze { children } => self.visit_cloze(children),
            AstNode::ConceptRef { term } => self.visit_concept_ref(term),
            AstNode::Admonition { kind, children } => self.visit_admonition(kind, children),
        }
    }
}

// ────────────────────────────────────────────────────────────────
// 遍历驱动器
// ────────────────────────────────────────────────────────────────

/// 使用访问者遍历 AST 节点。
///
/// 等价于 `visitor.visit(node)`，语义上更明确。
pub fn walk<'a>(visitor: &mut dyn AstVisitor<'a>, node: &AstNode<'a>) {
    visitor.visit(node);
}

/// 使用访问者遍历子节点列表。
pub fn walk_children<'a>(visitor: &mut dyn AstVisitor<'a>, children: &[AstNode<'a>]) {
    for child in children {
        visitor.visit(child);
    }
}

// ────────────────────────────────────────────────────────────────
// 共享工具函数
// ────────────────────────────────────────────────────────────────

/// 从 AST 子节点中提取纯文本。
///
/// 递归遍历 `Text`、`Strong`、`Emphasis`、`Link`、`Paragraph` 等容器节点，
/// 忽略所有格式化语义，仅拼接文本内容。
///
/// # 示例
/// ```ignore
/// // 从标题子节点提取纯文本
/// if let AstNode::Heading { children, .. } = node {
///     let text = collect_plain_text(children);
/// }
/// ```
pub fn collect_plain_text(children: &[AstNode<'_>]) -> String {
    let mut out = String::new();
    for child in children {
        append_text(child, &mut out);
    }
    out
}

/// 递归追加节点的纯文本到输出缓冲区。
fn append_text(node: &AstNode<'_>, out: &mut String) {
    match node {
        AstNode::Text { value } | AstNode::CodeBlock { value, .. } => out.push_str(value),
        AstNode::Wikilink { target, alias } => {
            out.push_str(alias.as_deref().unwrap_or(target));
        }
        AstNode::Strong { children }
        | AstNode::Emphasis { children }
        | AstNode::Link { children, .. }
        | AstNode::Paragraph { children } => {
            for child in children {
                append_text(child, out);
            }
        }
        _ => {}
    }
}

/// 从文本生成 URL 安全的 slug。
///
/// - 保留字母数字和 CJK 字符
/// - 空格 / 短横线 / 下划线转为 `-`
/// - 其他字符移除
/// - 空结果回退为 `"heading"`
pub fn generate_slug(text: &str) -> String {
    let lower = text.to_lowercase();
    let mut slug = String::with_capacity(lower.len());
    for ch in lower.chars() {
        match ch {
            'a'..='z' | '0'..='9' => slug.push(ch),
            _ if is_cjk(ch) => slug.push(ch),
            ' ' | '-' | '_' if !slug.ends_with('-') && !slug.is_empty() => {
                slug.push('-');
            }
            _ => {}
        }
    }
    let trimmed = slug.trim_end_matches('-');
    if trimmed.is_empty() {
        "heading".to_string()
    } else {
        trimmed.to_string()
    }
}

/// 检查字符是否属于 CJK 统一表意文字区域。
pub fn is_cjk(ch: char) -> bool {
    matches!(
        ch,
        '\u{4E00}'..='\u{9FFF}'
            | '\u{3400}'..='\u{4DBF}'
            | '\u{3000}'..='\u{303F}'
            | '\u{F900}'..='\u{FAFF}'
    )
}

// ────────────────────────────────────────────────────────────────
// 测试
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    fn heading(level: u8, text: &str) -> AstNode<'static> {
        AstNode::Heading {
            level,
            status: None,
            children: vec![AstNode::Text {
                value: Cow::Owned(text.to_string()),
            }],
        }
    }

    fn root(children: Vec<AstNode<'static>>) -> AstNode<'static> {
        AstNode::Root { children }
    }

    // ── Visitor 测试 ──

    /// 统计所有文本节点数量。
    struct TextNodeCounter {
        count: usize,
    }

    impl<'a> AstVisitor<'a> for TextNodeCounter {
        fn visit_text(&mut self, _value: &str) {
            self.count += 1;
        }
    }

    #[test]
    fn test_visitor_counts_text_nodes() {
        let ast = root(vec![
            AstNode::Paragraph {
                children: vec![
                    AstNode::Text {
                        value: Cow::Borrowed("hello"),
                    },
                    AstNode::Strong {
                        children: vec![AstNode::Text {
                            value: Cow::Borrowed("world"),
                        }],
                    },
                ],
            },
            AstNode::Paragraph {
                children: vec![AstNode::Text {
                    value: Cow::Borrowed("end"),
                }],
            },
        ]);

        let mut counter = TextNodeCounter { count: 0 };
        walk(&mut counter, &ast);
        assert_eq!(counter.count, 3);
    }

    /// 只收集标题，跳过其他节点。
    struct HeadingCollector {
        headings: Vec<(u8, String)>,
    }

    impl<'a> AstVisitor<'a> for HeadingCollector {
        fn visit_heading(&mut self, level: u8, children: &[AstNode<'a>]) {
            let text = collect_plain_text(children);
            self.headings.push((level, text));
            // 不递归子节点 — 已经提取了文本
        }
    }

    #[test]
    fn test_visitor_collects_headings() {
        let ast = root(vec![
            heading(1, "Title"),
            AstNode::Paragraph {
                children: vec![AstNode::Text {
                    value: Cow::Borrowed("text"),
                }],
            },
            heading(2, "Subtitle"),
        ]);

        let mut collector = HeadingCollector { headings: vec![] };
        walk(&mut collector, &ast);
        assert_eq!(
            collector.headings,
            vec![(1, "Title".to_string()), (2, "Subtitle".to_string()),]
        );
    }

    #[test]
    fn test_visitor_skips_nested_when_overridden() {
        let ast = root(vec![heading(1, "Title")]);

        let mut counter = TextNodeCounter { count: 0 };
        walk(&mut counter, &ast);
        assert_eq!(counter.count, 1);
    }

    /// 收集所有代码块语言。
    struct CodeLangCollector {
        langs: Vec<Option<String>>,
    }

    impl<'a> AstVisitor<'a> for CodeLangCollector {
        fn visit_code_block(&mut self, language: Option<&str>, _value: &str) {
            self.langs.push(language.map(str::to_string));
        }
    }

    #[test]
    fn test_visitor_collects_code_languages() {
        let ast = root(vec![
            AstNode::CodeBlock {
                language: Some(Cow::Borrowed("rust")),
                value: Cow::Borrowed("fn main() {}"),
            },
            AstNode::CodeBlock {
                language: None,
                value: Cow::Borrowed("plain"),
            },
        ]);

        let mut collector = CodeLangCollector { langs: vec![] };
        walk(&mut collector, &ast);
        assert_eq!(collector.langs.len(), 2);
        assert_eq!(collector.langs[0], Some("rust".to_string()));
        assert_eq!(collector.langs[1], None);
    }

    #[test]
    fn test_visitor_walks_deep_nesting() {
        let ast = root(vec![AstNode::Blockquote {
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
        }]);

        let mut counter = TextNodeCounter { count: 0 };
        walk(&mut counter, &ast);
        assert_eq!(counter.count, 1);
    }

    // ── collect_plain_text 测试 ──

    #[test]
    fn test_collect_plain_text_simple() {
        let children = vec![AstNode::Text {
            value: Cow::Borrowed("hello"),
        }];
        assert_eq!(collect_plain_text(&children), "hello");
    }

    #[test]
    fn test_collect_plain_text_nested() {
        let children = vec![
            AstNode::Text {
                value: Cow::Borrowed("Hello "),
            },
            AstNode::Strong {
                children: vec![AstNode::Text {
                    value: Cow::Borrowed("World"),
                }],
            },
        ];
        assert_eq!(collect_plain_text(&children), "Hello World");
    }

    #[test]
    fn test_collect_plain_text_ignores_formatting() {
        let children = vec![
            AstNode::Text {
                value: Cow::Borrowed("A"),
            },
            AstNode::InlineCode {
                value: Cow::Borrowed("B"),
            },
            AstNode::ThematicBreak,
        ];
        assert_eq!(collect_plain_text(&children), "A");
    }

    // ── generate_slug 测试 ──

    #[test]
    fn test_slug_english() {
        assert_eq!(generate_slug("Hello World"), "hello-world");
    }

    #[test]
    fn test_slug_chinese() {
        assert_eq!(generate_slug("核心技术"), "核心技术");
    }

    #[test]
    fn test_slug_mixed() {
        assert_eq!(generate_slug("Rust 学习笔记"), "rust-学习笔记");
    }

    #[test]
    fn test_slug_special_chars() {
        assert_eq!(generate_slug("Hello! @World# 2024"), "hello-world-2024");
    }

    #[test]
    fn test_slug_empty() {
        assert_eq!(generate_slug("!!!"), "heading");
    }

    // ── is_cjk 测试 ──

    #[test]
    fn test_is_cjk_basic() {
        assert!(is_cjk('中'));
        assert!(is_cjk('文'));
        assert!(!is_cjk('a'));
        assert!(!is_cjk('1'));
        assert!(!is_cjk(' '));
    }
}
