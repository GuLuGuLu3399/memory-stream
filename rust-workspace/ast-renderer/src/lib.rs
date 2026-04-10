//! `MemoryStream` AST → HTML 渲染器。
//!
//! 将 `AstNode` 树递归渲染为安全的 HTML 字符串。
//! 内置 XSS 防护：所有文本经过 HTML 转义，URL 经过白名单过滤。

use ast_core::{error::MSResult, AlignType, AstNode};
use regex::Regex;
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::LazyLock;

/// Wikilink 正则：匹配 `[[Card Name]]` 格式。
static WIKILINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[([^\]]+)\]\]").expect("wikilink regex should compile"));

/// 将文本中的 wikilink `[[Card Name]]` 转换为带样式的 anchor 标签。
///
/// # 参数
/// - `text`: 原始文本
/// - `card_name_to_id`: 可选的卡片名称到 UUID 的映射
///
/// # 渲染规则
/// - 当提供 `HashMap` 且包含卡片名称时：`<a href="/cards/{uuid}" class="reference-link">Card Name</a>`
/// - 当未提供 `HashMap` 或名称未找到时：`<a class="reference-link" data-card-name="Card Name">Card Name</a>`
fn render_wikilinks(text: &str, card_name_to_id: Option<&HashMap<String, String>>) -> String {
    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;

    for caps in WIKILINK_REGEX.captures_iter(text) {
        let full_match = caps.get(0).unwrap();

        if full_match.start() > last_end {
            result.push_str(&escape_html(&text[last_end..full_match.start()]));
        }

        let card_name = caps.get(1).map_or("", |m| m.as_str());
        let escaped_name = escape_html(card_name);

        // 尝试从映射中获取 UUID
        if let Some(uuid) = card_name_to_id.and_then(|map| map.get(card_name)) {
            // 有 UUID：渲染为带 href 的链接
            write!(
                result,
                r#"<a href="/cards/{}" class="reference-link">{}</a>"#,
                escape_html(uuid),
                escaped_name
            ).unwrap();
        } else {
            // 无 UUID：渲染为 data-card-name 属性
            write!(
                result,
                r#"<a class="reference-link" data-card-name="{escaped_name}">{escaped_name}</a>"#
            ).unwrap();
        }

        last_end = full_match.end();
    }

    if last_end < text.len() {
        result.push_str(&escape_html(&text[last_end..]));
    }

    result
}

/// 将 AST 节点递归渲染为 HTML 字符串。
///
/// # 安全性
/// - 所有文本内容经过 HTML 实体转义（`<` → `<` 等）
/// - URL 白名单过滤（仅允许 http/https/mailto/相对路径，阻止 <javascript:/data>:）
/// - Mermaid 代码块使用专用 `<pre class="mermaid">` 标签
///
/// # 参数
/// - `node`: AST 节点引用
///
/// # 返回
/// 渲染后的 HTML 字符串
///
/// # Errors
/// 返回错误如果渲染过程中发生失败（如格式化错误）。
///
/// # Panics
/// CC-理由: 核心渲染逻辑，拆分会降低可读性
#[allow(clippy::too_many_lines)]
pub fn render_to_html(node: &AstNode) -> MSResult<String> {
    let html = match node {
        AstNode::Root { children }
        | AstNode::TableHead { children }
        | AstNode::TableRow { children }
        | AstNode::TableCell { children } => render_children(children)?,

        AstNode::Heading { level, children } => {
            let inner_html = render_children(children)?;
            format!("<h{level}>{inner_html}</h{level}>")
        }

        AstNode::Paragraph { children } => {
            format!("<p>{}</p>", render_children(children)?)
        }

        AstNode::Text { value } => render_wikilinks(value, None),

        AstNode::Strong { children } | AstNode::Emphasis { children } => {
            let tag = if matches!(node, AstNode::Strong { .. }) { "strong" } else { "em" };
            format!("<{tag}>{}</{tag}>", render_children(children)?)
        }

        AstNode::List {
            ordered,
            start,
            children,
        } => {
            let tag = if *ordered { "ol" } else { "ul" };
            let start_attr = match start {
                Some(s) if *ordered && *s != 1 => format!(" start=\"{s}\""),
                _ => String::new(),
            };
            format!(
                "<{}{}>\n{}\n</{}>",
                tag,
                start_attr,
                render_children(children)?,
                tag
            )
        }
        AstNode::ListItem { children } => format!("  <li>{}</li>", render_children(children)?),

        AstNode::CodeBlock { language, value } => match language {
            Some(lang) if lang == "mermaid" => {
                format!("<pre class=\"mermaid\">\n{}\n</pre>", escape_html(value))
            }
            Some(lang) => {
                format!(
                    "<pre><code class=\"language-{}\">\n{}\n</code></pre>",
                    escape_html(lang),
                    escape_html(value)
                )
            }
            None => {
                format!("<pre><code>\n{}\n</code></pre>", escape_html(value))
            }
        },

        AstNode::Blockquote { children } => {
            format!("<blockquote>{}</blockquote>", render_children(children)?)
        }
        AstNode::ThematicBreak => "<hr />".to_string(),

        AstNode::Link { url, children } => {
            format!(
                "<a href=\"{}\">{}</a>",
                sanitize_url(url),
                render_children(children)?
            )
        }
        AstNode::Image { url, alt } => {
            format!(
                "<img src=\"{}\" alt=\"{}\" loading=\"lazy\" />",
                sanitize_url(url),
                escape_html(alt)
            )
        }
        AstNode::Math { value, inline } => {
            if *inline {
                format!(
                    "<span class=\"math-inline\">\\({}\\)</span>",
                    escape_html(value)
                )
            } else {
                format!(
                    "<div class=\"math-block\">\\[{}\\]</div>",
                    escape_html(value)
                )
            }
        }
        AstNode::Table {
            alignments,
            children,
        } => {
            let mut html = String::from("<table><thead>");
            let mut first = true;
            for child in children {
                match child {
                    AstNode::TableHead {
                        children: head_children,
                    } => {
                        html.push_str("<tr>");
                        for (i, cell) in head_children.iter().enumerate() {
                            let align = alignments
                                .get(i)
                                .and_then(|a| a.as_ref())
                                .map_or("", |a| match a {
                                    AlignType::Left => " style=\"text-align:left\"",
                                    AlignType::Center => " style=\"text-align:center\"",
                                    AlignType::Right => " style=\"text-align:right\"",
                                    AlignType::None => "",
                                });
                            write!(html, "<th{align}>").unwrap();
                            if let AstNode::TableCell {
                                children: cell_children,
                            } = cell
                            {
                                html.push_str(&render_children(cell_children)?);
                            }
                            html.push_str("</th>");
                        }
                        html.push_str("</tr>");
                    }
                    AstNode::TableRow {
                        children: row_children,
                    } => {
                        if first {
                            html.push_str("</thead><tbody>");
                            first = false;
                        }
                        html.push_str("<tr>");
                        for (i, cell) in row_children.iter().enumerate() {
                            let align = alignments
                                .get(i)
                                .and_then(|a| a.as_ref())
                                .map_or("", |a| match a {
                                    AlignType::Left => " style=\"text-align:left\"",
                                    AlignType::Center => " style=\"text-align:center\"",
                                    AlignType::Right => " style=\"text-align:right\"",
                                    AlignType::None => "",
                                });
                            write!(html, "<td{align}>").unwrap();
                            if let AstNode::TableCell {
                                children: cell_children,
                            } = cell
                            {
                                html.push_str(&render_children(cell_children)?);
                            }
                            html.push_str("</td>");
                        }
                        html.push_str("</tr>");
                    }
                    other => {
                        html.push_str(&render_to_html(other)?);
                    }
                }
            }
            if first {
                html.push_str("</thead>");
            }
            html.push_str("</tbody></table>");
            html
        }
        AstNode::Strikethrough { children } => {
            format!("<del>{}</del>", render_children(children)?)
        }
        AstNode::TaskListMarker { checked } => {
            format!(
                "<input type=\"checkbox\" {} disabled />",
                if *checked { "checked" } else { "" }
            )
        }
    };

    Ok(html)
}

/// HTML 实体转义 — 防止 XSS 注入。
/// 转义 `&`、`<`、`>`、`"`、`'` 五个特殊字符。
fn escape_html(s: &str) -> String {
    let extra = s.chars().fold(0usize, |acc, c| match c {
        '&' | '\'' => acc + 4,
        '<' | '>' => acc + 3,
        '"' => acc + 5,
        _ => acc,
    });
    let mut buf = String::with_capacity(s.len() + extra);
    for c in s.chars() {
        match c {
            '&' => buf.push_str("&amp;"),
            '<' => buf.push_str("&lt;"),
            '>' => buf.push_str("&gt;"),
            '"' => buf.push_str("&quot;"),
            '\'' => buf.push_str("&#39;"),
            _ => buf.push(c),
        }
    }
    buf
}

/// URL 白名单过滤 — 阻止 <javascript:/data>: 等危险协议。
/// 允许的协议：http、https、mailto、相对路径（/、#）。
fn sanitize_url(url: &str) -> String {
    let trimmed = url.trim_start();
    let lower = trimmed.to_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with('/')
        || lower.starts_with('#')
        || trimmed.is_empty()
    {
        escape_html(url)
    } else {
        "#".to_string()
    }
}

/// 批量渲染子节点列表，拼接为单个 HTML 字符串。
fn render_children(children: &[AstNode]) -> MSResult<String> {
    let mut html = String::with_capacity(children.len() * 64);
    for child in children {
        html.push_str(&render_to_html(child)?);
    }
    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast_core::AstNode;
    use std::borrow::Cow;

    #[test]
    fn test_escape_html_all_special_chars() {
        assert_eq!(escape_html("&<>\"'"), "&amp;&lt;&gt;&quot;&#39;");
        assert_eq!(escape_html("hello"), "hello");
        assert_eq!(escape_html(""), "");
    }

    #[test]
    fn test_render_heading_and_paragraph() {
        let ast = AstNode::Root {
            children: vec![
                AstNode::Heading {
                    level: 2,
                    children: vec![AstNode::Text {
                        value: Cow::Borrowed("渲染测试"),
                    }],
                },
                AstNode::Paragraph {
                    children: vec![AstNode::Text {
                        value: Cow::Borrowed("纯文本"),
                    }],
                },
            ],
        };

        let html = render_to_html(&ast).unwrap();
        assert_eq!(html, "<h2>渲染测试</h2><p>纯文本</p>");
    }

    #[test]
    fn test_render_nested_styles() {
        let ast = AstNode::Paragraph {
            children: vec![
                AstNode::Text {
                    value: Cow::Borrowed("你好"),
                },
                AstNode::Strong {
                    children: vec![AstNode::Emphasis {
                        children: vec![AstNode::Text {
                            value: Cow::Borrowed("世界"),
                        }],
                    }],
                },
            ],
        };

        let html = render_to_html(&ast).unwrap();
        assert_eq!(html, "<p>你好<strong><em>世界</em></strong></p>");
    }

    #[test]
    fn test_render_text_xss_prevention() {
        let ast = AstNode::Paragraph {
            children: vec![AstNode::Text {
                value: Cow::Borrowed("<script>alert('xss')</script>"),
            }],
        };

        let html = render_to_html(&ast).unwrap();
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_render_mermaid_xss_prevention() {
        let ast = AstNode::CodeBlock {
            language: Some(Cow::Borrowed("mermaid")),
            value: Cow::Borrowed("</pre><script>alert('mermaid-xss')</script><pre>"),
        };

        let html = render_to_html(&ast).unwrap();
        assert!(html.starts_with("<pre class=\"mermaid\">"));
        assert!(html.contains("&lt;/pre&gt;"));
        assert!(!html.contains("</pre><script>"));
    }

    #[test]
    fn test_render_mermaid_whitespace_preserved() {
        let ast = AstNode::CodeBlock {
            language: Some(Cow::Borrowed("mermaid")),
            value: Cow::Borrowed("graph TD\n    A --> B"),
        };

        let html = render_to_html(&ast).unwrap();
        assert!(html.starts_with("<pre class=\"mermaid\">"));
        assert!(html.contains("graph TD\n    A --&gt; B"));
        assert!(html.ends_with("</pre>"));
    }

    #[test]
    fn test_render_code_block_with_language() {
        let ast = AstNode::CodeBlock {
            language: Some(Cow::Borrowed("rust")),
            value: Cow::Borrowed("fn main() {}"),
        };

        let html = render_to_html(&ast).unwrap();
        assert!(html.contains("class=\"language-rust\""));
        assert!(html.contains("fn main() {}"));
    }

    #[test]
    fn test_render_code_block_language_escaped() {
        let ast = AstNode::CodeBlock {
            language: Some(Cow::Borrowed(
                "rust\"><script>alert(1)</script><span class=\"rust",
            )),
            value: Cow::Borrowed("code"),
        };

        let html = render_to_html(&ast).unwrap();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_sanitize_url_blocks_javascript_scheme() {
        let ast = AstNode::Link {
            url: Cow::Borrowed("javascript:alert('xss')"),
            children: vec![AstNode::Text {
                value: Cow::Borrowed("点击"),
            }],
        };

        let html = render_to_html(&ast).unwrap();
        assert_eq!(html, "<a href=\"#\">点击</a>");
    }

    #[test]
    fn test_sanitize_url_blocks_javascript_mixed_case() {
        let ast = AstNode::Link {
            url: Cow::Borrowed("JaVaScRiPt:alert(1)"),
            children: vec![AstNode::Text {
                value: Cow::Borrowed("link"),
            }],
        };

        let html = render_to_html(&ast).unwrap();
        assert_eq!(html, "<a href=\"#\">link</a>");
    }

    #[test]
    fn test_sanitize_url_blocks_data_scheme() {
        let ast = AstNode::Link {
            url: Cow::Borrowed("data:text/html,<script>alert(1)</script>"),
            children: vec![AstNode::Text {
                value: Cow::Borrowed("link"),
            }],
        };

        let html = render_to_html(&ast).unwrap();
        assert_eq!(html, "<a href=\"#\">link</a>");
    }

    #[test]
    fn test_sanitize_url_blocks_data_scheme_image() {
        let ast = AstNode::Image {
            url: Cow::Borrowed("data:image/svg+xml,<svg onload=alert(1)>"),
            alt: Cow::Borrowed("img"),
        };

        let html = render_to_html(&ast).unwrap();
        assert_eq!(html, "<img src=\"#\" alt=\"img\" loading=\"lazy\" />");
    }

    #[test]
    fn test_sanitize_url_allows_https() {
        let ast = AstNode::Link {
            url: Cow::Borrowed("https://example.com/path?q=1&b=2"),
            children: vec![AstNode::Text {
                value: Cow::Borrowed("安全链接"),
            }],
        };

        let html = render_to_html(&ast).unwrap();
        assert!(html.starts_with("<a href=\"https://example.com/path?q=1&amp;b=2\">"));
        assert!(html.ends_with("安全链接</a>"));
    }

    #[test]
    fn test_sanitize_url_allows_http() {
        let ast = AstNode::Link {
            url: Cow::Borrowed("http://example.com"),
            children: vec![AstNode::Text {
                value: Cow::Borrowed("link"),
            }],
        };

        let html = render_to_html(&ast).unwrap();
        assert!(html.contains("href=\"http://example.com\""));
    }

    #[test]
    fn test_sanitize_url_allows_mailto() {
        let ast = AstNode::Link {
            url: Cow::Borrowed("mailto:user@example.com"),
            children: vec![AstNode::Text {
                value: Cow::Borrowed("email"),
            }],
        };

        let html = render_to_html(&ast).unwrap();
        assert!(html.contains("href=\"mailto:user@example.com\""));
    }

    #[test]
    fn test_sanitize_url_allows_relative_paths() {
        let cases = vec!["/about", "#section", "/api/v1/cards"];
        for url in cases {
            let ast = AstNode::Link {
                url: Cow::Borrowed(url),
                children: vec![AstNode::Text {
                    value: Cow::Borrowed("link"),
                }],
            };
            let html = render_to_html(&ast).unwrap();
            assert!(
                html.contains(&format!("href=\"{url}\"")),
                "relative URL '{url}' should pass through: {html}"
            );
        }
    }

    #[test]
    fn test_sanitize_url_blocks_leading_whitespace_javascript() {
        let ast = AstNode::Link {
            url: Cow::Borrowed("   javascript:alert(1)"),
            children: vec![AstNode::Text {
                value: Cow::Borrowed("link"),
            }],
        };

        let html = render_to_html(&ast).unwrap();
        assert_eq!(html, "<a href=\"#\">link</a>");
    }

    #[test]
    fn test_render_thematic_break() {
        let ast = AstNode::ThematicBreak;
        assert_eq!(render_to_html(&ast).unwrap(), "<hr />");
    }

    #[test]
    fn test_render_blockquote() {
        let ast = AstNode::Blockquote {
            children: vec![AstNode::Paragraph {
                children: vec![AstNode::Text {
                    value: Cow::Borrowed("引用"),
                }],
            }],
        };
        assert_eq!(
            render_to_html(&ast).unwrap(),
            "<blockquote><p>引用</p></blockquote>"
        );
    }

    #[test]
    fn test_render_ordered_list_with_start() {
        let ast = AstNode::List {
            ordered: true,
            start: Some(3),
            children: vec![AstNode::ListItem {
                children: vec![AstNode::Text {
                    value: Cow::Borrowed("第三项"),
                }],
            }],
        };

        let html = render_to_html(&ast).unwrap();
        assert!(html.contains(" start=\"3\""));
        assert!(html.contains("<ol"));
        assert!(html.contains("<li>第三项</li>"));
    }

    #[test]
    fn test_render_unordered_list() {
        let ast = AstNode::List {
            ordered: false,
            start: None,
            children: vec![AstNode::ListItem {
                children: vec![AstNode::Text {
                    value: Cow::Borrowed("项"),
                }],
            }],
        };

        let html = render_to_html(&ast).unwrap();
        assert!(html.starts_with("<ul>"));
        assert!(html.ends_with("</ul>"));
        assert!(!html.contains("start="));
    }

    #[test]
    fn test_render_inline_math() {
        let ast = AstNode::Math {
            value: Cow::Borrowed("E=mc^2"),
            inline: true,
        };
        let html = render_to_html(&ast).unwrap();
        assert_eq!(html, "<span class=\"math-inline\">\\(E=mc^2\\)</span>");
    }

    #[test]
    fn test_render_block_math() {
        let ast = AstNode::Math {
            value: Cow::Borrowed("E=mc^2"),
            inline: false,
        };
        let html = render_to_html(&ast).unwrap();
        assert_eq!(html, "<div class=\"math-block\">\\[E=mc^2\\]</div>");
    }

    #[test]
    fn test_render_image() {
        let ast = AstNode::Image {
            url: Cow::Borrowed("https://example.com/img.png"),
            alt: Cow::Borrowed("图片\"描述"),
        };
        let html = render_to_html(&ast).unwrap();
        assert!(html.contains("src=\"https://example.com/img.png\""));
        assert!(html.contains("alt=\"图片&quot;描述\""));
        assert!(html.contains("loading=\"lazy\""));
    }

    #[test]
    fn test_render_empty_url_link() {
        let ast = AstNode::Link {
            url: Cow::Borrowed(""),
            children: vec![AstNode::Text {
                value: Cow::Borrowed("空链接"),
            }],
        };
        let html = render_to_html(&ast).unwrap();
        assert_eq!(html, "<a href=\"\">空链接</a>");
    }

    #[test]
    fn test_wikilink_basic() {
        let ast = AstNode::Paragraph {
            children: vec![AstNode::Text {
                value: Cow::Borrowed("Check [[Card Name]] for details"),
            }],
        };
        let html = render_to_html(&ast).unwrap();
        assert!(
            html.contains(r#"<a class="reference-link" data-card-name="Card Name">Card Name</a>"#)
        );
        assert!(html.contains("Check "));
        assert!(html.contains(" for details"));
    }

    #[test]
    fn test_wikilink_multiple() {
        let ast = AstNode::Paragraph {
            children: vec![AstNode::Text {
                value: Cow::Borrowed("See [[A]] and [[B]]"),
            }],
        };
        let html = render_to_html(&ast).unwrap();
        assert!(html.contains(r#"<a class="reference-link" data-card-name="A">A</a>"#));
        assert!(html.contains(r#"<a class="reference-link" data-card-name="B">B</a>"#));
    }

    #[test]
    fn test_wikilink_special_chars() {
        let ast = AstNode::Paragraph {
            children: vec![AstNode::Text {
                value: Cow::Borrowed("See [[C++ Notes]]"),
            }],
        };
        let html = render_to_html(&ast).unwrap();
        assert!(html.contains(r#"data-card-name="C++ Notes""#));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_wikilink_no_match() {
        let ast = AstNode::Paragraph {
            children: vec![AstNode::Text {
                value: Cow::Borrowed("No links here"),
            }],
        };
        let html = render_to_html(&ast).unwrap();
        assert_eq!(html, "<p>No links here</p>");
        assert!(!html.contains("reference-link"));
    }

    #[test]
    fn test_wikilink_with_html_chars() {
        let ast = AstNode::Paragraph {
            children: vec![AstNode::Text {
                value: Cow::Borrowed("[[Card<script>]] text"),
            }],
        };
        let html = render_to_html(&ast).unwrap();
        assert!(html.contains(r#"data-card-name="Card&lt;script&gt;""#));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_wikilink_with_uuid_resolution() {
        let mut card_map: HashMap<String, String> = HashMap::new();
        card_map.insert("Card Name".to_string(), "uuid-123-abc".to_string());

        let text = "Check [[Card Name]] for details";
        let html = render_wikilinks(text, Some(&card_map));

        assert!(html.contains(r#"href="/cards/uuid-123-abc""#));
        assert!(html.contains(r#"class="reference-link""#));
        assert!(html.contains(">Card Name</a>"));
        assert!(!html.contains("data-card-name"));
    }

    #[test]
    fn test_wikilink_without_uuid_map() {
        let text = "Check [[Card Name]] for details";
        let html = render_wikilinks(text, None);

        assert!(html.contains(r#"data-card-name="Card Name""#));
        assert!(html.contains(r#"class="reference-link""#));
        assert!(!html.contains("href="));
    }

    #[test]
    fn test_wikilink_uuid_map_name_not_found() {
        let mut card_map: HashMap<String, String> = HashMap::new();
        card_map.insert("Other Card".to_string(), "uuid-other".to_string());

        let text = "Check [[Card Name]] for details";
        let html = render_wikilinks(text, Some(&card_map));

        assert!(html.contains(r#"data-card-name="Card Name""#));
        assert!(html.contains(r#"class="reference-link""#));
        assert!(!html.contains("href="));
    }

    #[test]
    fn test_wikilink_mixed_resolution() {
        let mut card_map: HashMap<String, String> = HashMap::new();
        card_map.insert("Known Card".to_string(), "uuid-known".to_string());

        let text = "See [[Known Card]] and [[Unknown Card]]";
        let html = render_wikilinks(text, Some(&card_map));

        assert!(html.contains(r#"href="/cards/uuid-known""#));
        assert!(html.contains(r#"data-card-name="Unknown Card""#));
    }

    #[test]
    fn test_wikilink_uuid_escaped() {
        let mut card_map: HashMap<String, String> = HashMap::new();
        card_map.insert("Card".to_string(), "uuid<\"evil>".to_string());

        let text = "[[Card]]";
        let html = render_wikilinks(text, Some(&card_map));

        assert!(html.contains(r#"href="/cards/uuid&lt;&quot;evil&gt;""#));
        assert!(!html.contains("<\"evil>"));
    }
}
