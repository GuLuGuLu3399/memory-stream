use ast_core::{
    error::{MSError, MSResult},
    AlignType, AstNode,
};
use pulldown_cmark::{CowStr, Event, Options, Parser as CmarkParser, Tag};
use regex::Regex;
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::OnceLock;

/// 将 pulldown-cmark 的 `CowStr` 转换为标准库的 `Cow<str>`。
/// 避免 `Into` trait 冲突的辅助函数。
fn cowstr_to_cow(s: CowStr) -> Cow<str> {
    match s {
        CowStr::Borrowed(s) => Cow::Borrowed(s),
        _ => Cow::Owned(s.into_string()),
    }
}

/// 从 Markdown 文本中提取 wikilink。
///
/// # Panics
/// 不会 panic（正则表达式编译在编译时已验证）。
pub fn extract_wikilinks(md: &str) -> Vec<String> {
    static WIKILINK_REGEX: OnceLock<Regex> = OnceLock::new();
    let re = WIKILINK_REGEX.get_or_init(|| Regex::new(r"\[\[([^\]]+)\]\]").unwrap());

    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for cap in re.captures_iter(md) {
        let name = cap[1].to_string();
        if seen.insert(name.clone()) {
            result.push(name);
        }
    }

    result
}

/// 将 Markdown 文本解析为 AST（抽象语法树）。
///
/// 基于 pulldown-cmark 解析器，支持以下扩展语法：
/// - GFM 表格
/// - 任务列表（`- [x]`）
/// - 删除线（`~~text~~`）
/// - 智能标点
/// - 数学公式（`$inline$` 和 `$$display$$`）
///
/// # 参数
/// - `md_text`: 原始 Markdown 文本
///
/// # 返回
/// 解析成功返回 `AstNode::Root`，失败返回 `MSError::ParseError`。
///
/// # Errors
/// 当 Markdown 文本格式无法解析时返回 `MSError::ParseError`。
///
/// # Panics
/// 当解析器内部状态不一致时可能 panic（理论上不应发生）。
///
/// # 示例
/// ```ignore
/// let ast = parse_markdown("## Hello")?;
/// ```
// CC-理由: 核心解析逻辑，拆分会降低可读性
#[allow(clippy::too_many_lines)]
pub fn parse_markdown(md_text: &str) -> MSResult<AstNode<'_>> {
    // 🌟 开启扩展解析选项：表格、任务列表、删除线、数学公式
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_SMART_PUNCTUATION);
    opts.insert(Options::ENABLE_MATH); // 保护公式不被误解析为斜体

    let parser = CmarkParser::new_ext(md_text, opts);
    let mut stack: Vec<AstNode<'_>> = vec![AstNode::Root { children: vec![] }];

    for event in parser {
        match event {
            Event::Start(tag) => {
                let node = match tag {
                    Tag::Paragraph => AstNode::Paragraph { children: vec![] },
                    Tag::Heading { level, .. } => AstNode::Heading {
                        level: level as u8,
                        children: vec![],
                    },
                    Tag::Strong => AstNode::Strong { children: vec![] },
                    Tag::Emphasis => AstNode::Emphasis { children: vec![] },
                    Tag::BlockQuote(_) => AstNode::Blockquote { children: vec![] },
                    Tag::CodeBlock(kind) => {
                        let lang = match kind {
                            pulldown_cmark::CodeBlockKind::Fenced(lang) if !lang.is_empty() => {
                                Some(cowstr_to_cow(lang))
                            }
                            _ => None,
                        };
                        AstNode::CodeBlock {
                            language: lang,
                            value: Cow::default(),
                        }
                    }
                    Tag::List(start) => AstNode::List {
                        ordered: start.is_some(),
                        start,
                        children: vec![],
                    },
                    Tag::Item => AstNode::ListItem { children: vec![] },
                    Tag::Link { dest_url, .. } => AstNode::Link {
                        url: cowstr_to_cow(dest_url),
                        children: vec![],
                    },
                    Tag::Image { dest_url, .. } => AstNode::Image {
                        url: cowstr_to_cow(dest_url),
                        alt: Cow::default(),
                    },
                    Tag::Table(alignments) => {
                        let aligns: Vec<Option<AlignType>> = alignments
                            .iter()
                            .map(|a| match a {
                                pulldown_cmark::Alignment::Left => Some(AlignType::Left),
                                pulldown_cmark::Alignment::Center => Some(AlignType::Center),
                                pulldown_cmark::Alignment::Right => Some(AlignType::Right),
                                pulldown_cmark::Alignment::None => Some(AlignType::None),
                            })
                            .collect();
                        AstNode::Table {
                            alignments: aligns,
                            children: vec![],
                        }
                    }
                    Tag::TableHead => AstNode::TableHead { children: vec![] },
                    Tag::TableRow => AstNode::TableRow { children: vec![] },
                    Tag::TableCell => AstNode::TableCell { children: vec![] },
                    Tag::Strikethrough => AstNode::Strikethrough { children: vec![] },
                    _ => continue,
                };
                stack.push(node);
            }

            Event::Text(text) => {
                if let Some(top_node) = stack.last_mut() {
                    match top_node {
                        AstNode::CodeBlock { value, .. } => {
                            value.to_mut().push_str(&text);
                        }
                        AstNode::Image { alt, .. } => {
                            alt.to_mut().push_str(&text);
                        }
                        _ => top_node.push_child(AstNode::Text {
                            value: cowstr_to_cow(text),
                        }),
                    }
                }
            }

            Event::Code(code_text) => {
                if let Some(top_node) = stack.last_mut() {
                    top_node.push_child(AstNode::Text {
                        value: Cow::Owned(format!("`{code_text}`")),
                    });
                }
            }

            Event::End(_tag) => {
                if stack.len() > 1 {
                    let finished_node = stack.pop().unwrap();
                    if let Some(parent_node) = stack.last_mut() {
                        parent_node.push_child(finished_node);
                    }
                }
            }

            Event::Rule => {
                if let Some(top_node) = stack.last_mut() {
                    top_node.push_child(AstNode::ThematicBreak);
                }
            }

            // 🌟 核心修复：独立拦截数学公式事件！(不是 Tag，是独立的 Event)
            Event::InlineMath(text) => {
                if let Some(top_node) = stack.last_mut() {
                    top_node.push_child(AstNode::Math {
                        value: cowstr_to_cow(text),
                        inline: true,
                    });
                }
            }
            Event::DisplayMath(text) => {
                if let Some(top_node) = stack.last_mut() {
                    top_node.push_child(AstNode::Math {
                        value: cowstr_to_cow(text),
                        inline: false,
                    });
                }
            }
            Event::TaskListMarker(checked) => {
                if let Some(top_node) = stack.last_mut() {
                    top_node.push_child(AstNode::TaskListMarker { checked });
                }
            }

            _ => {}
        }
    }

    if stack.len() == 1 {
        Ok(stack.pop().unwrap())
    } else {
        Err(MSError::ParseError("文档树未正确闭合".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast_core::AstNode;

    fn get_first_child<'a>(ast: &'a AstNode<'a>) -> &'a AstNode<'a> {
        match ast {
            AstNode::Root { children } => children.first().expect("AST 树为空"),
            _ => panic!("解析出的根节点不是 Root"),
        }
    }

    #[test]
    fn test_parse_heading() {
        let md = "### 这是一个三级标题";
        let ast = parse_markdown(md).unwrap();
        let first_node = get_first_child(&ast);

        match first_node {
            AstNode::Heading { level, children } => {
                assert_eq!(*level, 3);
                assert_eq!(children.len(), 1);
                if let AstNode::Text { value } = &children[0] {
                    assert_eq!(value, "这是一个三级标题");
                } else {
                    panic!("标题子节点不是 Text");
                }
            }
            _ => panic!("未能正确解析出 Heading 节点"),
        }
    }

    #[test]
    fn test_parse_inline_formatting() {
        let md = "正常**加粗***斜体*";
        let ast = parse_markdown(md).unwrap();
        let first_node = get_first_child(&ast);

        if let AstNode::Paragraph { children } = first_node {
            assert_eq!(children.len(), 3, "应该被解析为 3 个独立的行内节点");

            assert!(matches!(children[0], AstNode::Text { .. }));
            assert!(matches!(children[1], AstNode::Strong { .. }));
            assert!(matches!(children[2], AstNode::Emphasis { .. }));
        } else {
            panic!("未能正确解析出 Paragraph 节点");
        }
    }

    #[test]
    fn test_parse_code_block() {
        let md = "```rust\nfn main() {}\n```";
        let ast = parse_markdown(md).unwrap();
        let first_node = get_first_child(&ast);

        if let AstNode::CodeBlock { language, value } = first_node {
            assert_eq!(language.as_deref(), Some("rust"));
            assert_eq!(value.trim(), "fn main() {}");
        } else {
            panic!("未能正确解析出 CodeBlock 节点");
        }
    }

    #[test]
    fn test_parse_complex_list() {
        let md = "- 核心**引擎**";
        let ast = parse_markdown(md).unwrap();
        let first_node = get_first_child(&ast);

        if let AstNode::List {
            ordered, children, ..
        } = first_node
        {
            assert!(!(*ordered));
            assert_eq!(children.len(), 1);

            if let AstNode::ListItem {
                children: item_children,
            } = &children[0]
            {
                assert!(!item_children.is_empty());
            } else {
                panic!("List 子节点不是 ListItem");
            }
        } else {
            panic!("未能正确解析出 List 节点");
        }
    }

    #[test]
    fn test_parse_link() {
        let md = "[点击这里](https://example.com)";
        let ast = parse_markdown(md).unwrap();
        let first_node = get_first_child(&ast);

        if let AstNode::Paragraph { children } = first_node {
            assert_eq!(children.len(), 1);
            if let AstNode::Link { url, children } = &children[0] {
                assert_eq!(url, "https://example.com");
                assert_eq!(children.len(), 1);
                if let AstNode::Text { value } = &children[0] {
                    assert_eq!(value, "点击这里");
                } else {
                    panic!("Link 子节点不是 Text");
                }
            } else {
                panic!("未能正确解析出 Link 节点");
            }
        } else {
            panic!("未能正确解析出 Paragraph 节点");
        }
    }

    #[test]
    fn test_parse_image() {
        let md = "![这是一张图片](https://example.com/img.png)";
        let ast = parse_markdown(md).unwrap();
        let first_node = get_first_child(&ast);

        if let AstNode::Paragraph { children } = first_node {
            assert_eq!(children.len(), 1);
            if let AstNode::Image { url, alt } = &children[0] {
                assert_eq!(url, "https://example.com/img.png");
                assert_eq!(alt, "这是一张图片");
            } else {
                panic!("未能正确解析出 Image 节点");
            }
        } else {
            panic!("未能正确解析出 Paragraph 节点");
        }
    }
}
