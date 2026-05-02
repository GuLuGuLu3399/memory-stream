use crate::{AstNode, visitor::{collect_plain_text, generate_slug}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TocNode {
    pub level: u8,
    pub text: String,
    pub slug: String,
    pub children: Vec<TocNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TocFlatItem {
    pub level: u8,
    pub text: String,
    pub slug: String,
}

#[must_use]
pub fn extract_toc(node: &AstNode) -> Vec<TocNode> {
    let headings = collect_headings(node);
    build_tree(&headings)
}

/// 从 JSON 字符串提取 TOC。
///
/// # Errors
/// 返回错误如果 JSON 解析失败或 TOC 提取失败。
pub fn extract_toc_from_json(json: &str) -> Result<Vec<TocNode>, crate::error::MSError> {
    let node: crate::AstNodeOwned = serde_json::from_str(json)
        .map_err(|e| crate::error::MSError::ParseError(format!("AST JSON 反序列化失败: {e}")))?;
    Ok(extract_toc(&node))
}

#[must_use]
pub fn extract_toc_flat(node: &AstNode) -> Vec<TocFlatItem> {
    collect_headings(node)
        .into_iter()
        .map(|(level, text)| TocFlatItem {
            slug: generate_slug(&text),
            level,
            text,
        })
        .collect()
}

fn collect_headings(node: &AstNode) -> Vec<(u8, String)> {
    let mut result = Vec::new();
    walk_for_headings(node, &mut result);
    result
}

fn walk_for_headings(node: &AstNode, out: &mut Vec<(u8, String)>) {
    match node {
        AstNode::Root { children } | AstNode::Blockquote { children } => {
            for child in children {
                walk_for_headings(child, out);
            }
        }
        AstNode::Heading { level, children, .. } => {
            let text = collect_plain_text(children);
            out.push((*level, text));
        }
        _ => {}
    }
}

pub(crate) fn build_tree(headings: &[(u8, String)]) -> Vec<TocNode> {
    let mut root: Vec<TocNode> = Vec::new();
    let mut stack: Vec<TocNode> = Vec::new();

    for (level, text) in headings {
        let node = TocNode {
            level: *level,
            slug: generate_slug(text),
            text: text.clone(),
            children: Vec::new(),
        };

        while let Some(parent) = stack.last() {
            if parent.level < *level {
                break;
            }
            let Some(popped) = stack.pop() else {
                break;
            };
            if let Some(grandparent) = stack.last_mut() {
                grandparent.children.push(popped);
            } else {
                root.push(popped);
            }
        }

        stack.push(node);
    }

    while let Some(popped) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            parent.children.push(popped);
        } else {
            root.push(popped);
        }
    }

    root
}

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

    #[test]
    fn test_single_heading() {
        let ast = root(vec![heading(1, "Hello")]);
        let toc = extract_toc(&ast);
        assert_eq!(toc.len(), 1);
        assert_eq!(toc[0].level, 1);
        assert_eq!(toc[0].text, "Hello");
        assert_eq!(toc[0].slug, "hello");
    }

    #[test]
    fn test_nested_headings() {
        let ast = root(vec![
            heading(1, "A"),
            heading(2, "A.1"),
            heading(2, "A.2"),
            heading(3, "A.2.1"),
            heading(1, "B"),
        ]);
        let toc = extract_toc(&ast);
        assert_eq!(toc.len(), 2);
        assert_eq!(toc[0].text, "A");
        assert_eq!(toc[0].children.len(), 2);
        assert_eq!(toc[0].children[0].text, "A.1");
        assert_eq!(toc[0].children[1].text, "A.2");
        assert_eq!(toc[0].children[1].children.len(), 1);
        assert_eq!(toc[0].children[1].children[0].text, "A.2.1");
        assert_eq!(toc[1].text, "B");
    }

    #[test]
    fn test_no_headings() {
        let ast = root(vec![AstNode::Paragraph {
            children: vec![AstNode::Text {
                value: Cow::Borrowed("just text"),
            }],
        }]);
        let toc = extract_toc(&ast);
        assert!(toc.is_empty());
    }

    #[test]
    fn test_empty_root() {
        let ast = root(vec![]);
        let toc = extract_toc(&ast);
        assert!(toc.is_empty());
    }

    #[test]
    fn test_flat_extraction() {
        let ast = root(vec![
            heading(1, "One"),
            heading(2, "Two"),
            heading(3, "Three"),
        ]);
        let flat = extract_toc_flat(&ast);
        assert_eq!(flat.len(), 3);
        assert_eq!(flat[0].level, 1);
        assert_eq!(flat[1].level, 2);
        assert_eq!(flat[2].level, 3);
    }

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

    #[test]
    fn test_json_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let ast = root(vec![heading(1, "Title"), heading(2, "Sub")]);
        let json = serde_json::to_string(&ast)?;
        let toc = extract_toc_from_json(&json)?;
        assert_eq!(toc.len(), 1);
        assert_eq!(toc[0].text, "Title");
        assert_eq!(toc[0].children.len(), 1);
        assert_eq!(toc[0].children[0].text, "Sub");
        Ok(())
    }

    #[test]
    fn test_json_invalid() {
        let result = extract_toc_from_json("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_heading_with_formatted_children() {
        let ast = root(vec![AstNode::Heading {
            level: 2,
            status: None,
            children: vec![
                AstNode::Text {
                    value: Cow::Borrowed("Hello "),
                },
                AstNode::Strong {
                    children: vec![AstNode::Text {
                        value: Cow::Borrowed("World"),
                    }],
                },
            ],
        }]);
        let toc = extract_toc(&ast);
        assert_eq!(toc[0].text, "Hello World");
    }

    #[test]
    fn test_flat_heading_inside_blockquote() {
        let ast = root(vec![
            heading(1, "Outside"),
            AstNode::Blockquote {
                children: vec![heading(3, "Inside Quote")],
            },
        ]);
        let toc = extract_toc(&ast);
        assert_eq!(toc.len(), 1);
        assert_eq!(toc[0].text, "Outside");
    }

    #[test]
    fn test_with_real_parser() -> Result<(), Box<dyn std::error::Error>> {
        let md = r"# Main

## Section A

### Sub A1

## Section B
";
        let ast = crate::parser::parse_markdown(md)?;
        let toc = extract_toc(&ast);
        assert_eq!(toc.len(), 1);
        assert_eq!(toc[0].text, "Main");
        assert_eq!(toc[0].children.len(), 2);
        assert_eq!(toc[0].children[0].text, "Section A");
        assert_eq!(toc[0].children[0].children.len(), 1);
        assert_eq!(toc[0].children[0].children[0].text, "Sub A1");
        assert_eq!(toc[0].children[1].text, "Section B");
        Ok(())
    }

    #[test]
    fn test_tree_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let toc = vec![TocNode {
            level: 1,
            text: "A".into(),
            slug: "a".into(),
            children: vec![TocNode {
                level: 2,
                text: "A1".into(),
                slug: "a1".into(),
                children: vec![],
            }],
        }];
        let json = serde_json::to_string(&toc)?;
        assert!(json.contains("\"level\":1"));
        assert!(json.contains("\"children\":[{"));
        let back: Vec<TocNode> = serde_json::from_str(&json)?;
        assert_eq!(toc, back);
        Ok(())
    }
}
