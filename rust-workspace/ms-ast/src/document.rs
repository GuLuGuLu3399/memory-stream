//! 一次解析，多维榨取 — 单次 AST 遍历提取全部文档元数据。
//!
//! 对外暴露 [`parse_document`] / [`parse_document_with`]，
//! 输入 Markdown 文本，输出 [`ParsedDocument`] 大礼包：
//! AST JSON + TOC 树 + 双链列表 + 纯文本摘要。

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::error::MSResult;
use crate::parser::{parse_markdown_with, ParseOptions};
use crate::toc::{build_tree, TocNode};
use crate::visitor::{collect_plain_text, AstVisitor, walk};
use crate::AstNode;

// ────────────────────────────────────────────────────────────────
// ParsedDocument — 一次解析的全部产物
// ────────────────────────────────────────────────────────────────

/// 文档解构结果 — 单次 AST 遍历提取的完整元数据。
///
/// 调用方按需消费各字段：
/// - `ast_json` → 前端 ASTRenderer 渲染
/// - `toc` → 侧边栏大纲导航
/// - `outgoing_links` → ms-graph 建图 / ms-meta 关系索引
/// - `excerpt` → 卡片列表预览 / FTS 全文搜索
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDocument {
    /// JSON 序列化的 AST
    pub ast_json: String,
    /// 目录树（层级嵌套结构）
    pub toc: Vec<TocNode>,
    /// 页面内所有双链（去重，保留首次出现顺序）
    pub outgoing_links: Vec<String>,
    /// 纯文本摘要（前 N 字符，剥离全部 Markdown 语法）
    pub excerpt: String,
    /// YAML frontmatter 原始文本（不含 `---` 分隔符），无 frontmatter 时为空字符串
    pub frontmatter: String,
}

// ────────────────────────────────────────────────────────────────
// Frontmatter 剥离
// ────────────────────────────────────────────────────────────────

/// 从 Markdown 文本中检测并剥离 YAML frontmatter。
///
/// 返回 `(正文部分, frontmatter原始文本)`。
/// frontmatter 不含 `---` 分隔符；无 frontmatter 时返回空字符串。
fn strip_frontmatter(md: &str) -> (&str, String) {
    let after_open = md
        .strip_prefix("---\n")
        .or_else(|| md.strip_prefix("---\r\n"));
    let Some(after_open) = after_open else {
        return (md, String::new());
    };

    // 查找闭合的 `---`（必须独占一行）
    for pos in match_indices_newline_aware(after_open) {
        let remaining = &after_open[pos..];
        if remaining.starts_with("\n---\n")
            || remaining.starts_with("\n---\r\n")
            || remaining.starts_with("\r\n---\n")
            || remaining.starts_with("\r\n---\r\n")
        {
            let yaml = &after_open[..pos];
            // 跳过换行 + "---" + 换行
            let sep_end = remaining.find('\n').map_or(remaining.len(), |i| i + 1);
            let body = &remaining[sep_end..];
            return (body.trim_start(), yaml.to_string());
        }
    }

    // 未找到闭合 `---`，安全回退
    (md, String::new())
}

/// 在文本中查找所有独占一行的 `---` 的起始偏移（返回 `---` 前换行符的偏移）。
fn match_indices_newline_aware(s: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            let start = i + 1;
            if s[start..].starts_with("---\n") || s[start..].starts_with("---\r\n") {
                positions.push(i);
            }
        } else if bytes[i] == b'\r' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
            let start = i + 2;
            if s[start..].starts_with("---\n") || s[start..].starts_with("---\r\n") {
                positions.push(i);
            }
        }
        i += 1;
    }
    positions
}

// ────────────────────────────────────────────────────────────────
// DocumentExtractor — 单次遍历收集器
// ────────────────────────────────────────────────────────────────

/// 基于 Visitor 模式的单次遍历提取器。
///
/// 同时收集：目录标题 + 双链 + 纯文本摘要。
/// - 标题文本进入 TOC，不混入摘要
/// - 代码块内容跳过（不在摘要和双链中出现）
/// - 双链去重，保留首次出现顺序
struct DocumentExtractor {
    headings: Vec<(u8, String)>,
    links: Vec<String>,
    seen_links: HashSet<String>,
    excerpt: String,
    excerpt_chars: usize,
    excerpt_limit: usize,
}

impl DocumentExtractor {
    fn new(excerpt_limit: usize) -> Self {
        Self {
            headings: Vec::new(),
            links: Vec::new(),
            seen_links: HashSet::new(),
            excerpt: String::with_capacity(excerpt_limit * 3),
            excerpt_chars: 0,
            excerpt_limit,
        }
    }
}

impl<'a> AstVisitor<'a> for DocumentExtractor {
    /// 收集标题到 TOC，不递归子节点（标题文本不混入摘要）
    fn visit_heading(&mut self, level: u8, children: &[AstNode<'a>]) {
        let text = collect_plain_text(children);
        self.headings.push((level, text));
    }

    /// 收集双链目标（去重）
    fn visit_wikilink(&mut self, target: &str, _alias: Option<&str>) {
        if self.seen_links.insert(target.to_string()) {
            self.links.push(target.to_string());
        }
    }

    /// 追加纯文本到摘要（按字符数截断）
    fn visit_text(&mut self, value: &str) {
        if self.excerpt_chars >= self.excerpt_limit {
            return;
        }
        let remaining = self.excerpt_limit - self.excerpt_chars;
        for ch in value.chars().take(remaining) {
            self.excerpt.push(ch);
            self.excerpt_chars += 1;
        }
    }

    /// 跳过代码块 — 不混入摘要，也不提取其中的双链
    fn visit_code_block(&mut self, _language: Option<&str>, _value: &str) {}
}

// ────────────────────────────────────────────────────────────────
// 公共 API
// ────────────────────────────────────────────────────────────────

const DEFAULT_EXCERPT_LIMIT: usize = 200;

/// 一次调用完成 Markdown 解析 + 全维度提取（默认 200 字摘要）。
///
/// # Errors
/// Markdown 解析失败时返回 `MSError::ParseError`。
pub fn parse_document(md: &str) -> MSResult<ParsedDocument> {
    parse_document_with(md, &ParseOptions::default(), DEFAULT_EXCERPT_LIMIT)
}

/// 自定义选项的文档解析。
///
/// # 参数
/// - `md`: 原始 Markdown 文本
/// - `opts`: 解析选项（控制 GFM 表格/脚注等扩展）
/// - `excerpt_limit`: 摘要最大字符数
///
/// # Errors
/// Markdown 解析或 JSON 序列化失败时返回错误。
pub fn parse_document_with(md: &str, opts: &ParseOptions, excerpt_limit: usize) -> MSResult<ParsedDocument> {
    let (body, frontmatter) = strip_frontmatter(md);

    let ast = parse_markdown_with(body, opts)?;
    let ast_json = serde_json::to_string(&ast)?;

    // 单次遍历：TOC + 双链 + 摘要
    let mut extractor = DocumentExtractor::new(excerpt_limit);
    walk(&mut extractor, &ast);
    let toc = build_tree(&extractor.headings);

    Ok(ParsedDocument {
        ast_json,
        toc,
        outgoing_links: extractor.links,
        excerpt: extractor.excerpt,
        frontmatter,
    })
}

// ────────────────────────────────────────────────────────────────
// 测试
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_document_basic() -> MSResult<()> {
        let md = "# Rust 学习\n\n这是一篇关于 **Rust** 的笔记。\n\n## 所有权\n\nRust 的所有权机制是核心特性。";
        let doc = parse_document(md)?;

        // TOC
        assert_eq!(doc.toc.len(), 1);
        assert_eq!(doc.toc[0].text, "Rust 学习");
        assert_eq!(doc.toc[0].children.len(), 1);
        assert_eq!(doc.toc[0].children[0].text, "所有权");

        // 摘要（跳过标题，只取段落文本）
        assert!(doc.excerpt.contains("这是一篇关于"));
        assert!(doc.excerpt.contains("Rust"));

        // 双链：无
        assert!(doc.outgoing_links.is_empty());

        // AST 非空
        assert!(!doc.ast_json.is_empty());
        Ok(())
    }

    #[test]
    fn test_parse_document_wikilinks() -> MSResult<()> {
        let md = "参见 [[Rust 入门]] 和 [[Go 并发]]，也看 [[Rust 入门]]（重复）。";
        let doc = parse_document(md)?;

        // 去重，保留首次出现顺序
        assert_eq!(doc.outgoing_links, vec!["Rust 入门", "Go 并发"]);
        Ok(())
    }

    #[test]
    fn test_parse_document_wikilink_not_in_code() -> MSResult<()> {
        let md = "```js\nconst s = \"[[FakeLink]]\";\n```\n\n真实链接是 [[RealLink]]。\n";
        let doc = parse_document(md)?;

        // 代码块中的 [[FakeLink]] 不应被提取
        assert_eq!(doc.outgoing_links, vec!["RealLink"]);
        // 摘要也不含代码块内容
        assert!(!doc.excerpt.contains("FakeLink"));
        Ok(())
    }

    #[test]
    fn test_parse_document_excerpt_limit() -> MSResult<()> {
        let md = "这是一段很长的文本，用于测试摘要截断功能是否正常工作。";
        let doc = parse_document_with(md, &ParseOptions::default(), 10)?;

        assert!(doc.excerpt.chars().count() <= 10);
        assert!(doc.excerpt.starts_with("这是一段很长的文"));
        Ok(())
    }

    #[test]
    fn test_parse_document_excerpt_skips_headings() -> MSResult<()> {
        let md = "# 大标题\n\n正文内容。\n";
        let doc = parse_document(md)?;

        // 摘要不包含标题文本
        assert!(!doc.excerpt.contains("大标题"));
        assert!(doc.excerpt.contains("正文内容"));
        Ok(())
    }

    #[test]
    fn test_parse_document_empty() -> MSResult<()> {
        let doc = parse_document("")?;
        assert!(doc.toc.is_empty());
        assert!(doc.outgoing_links.is_empty());
        assert!(doc.excerpt.is_empty());
        Ok(())
    }

    #[test]
    fn test_parse_document_roundtrip_json() -> MSResult<()> {
        let md = "# Hello\n\nWorld [[Link1]] [[Link2]]\n";
        let doc = parse_document(md)?;
        let json = serde_json::to_string(&doc)?;
        let back: ParsedDocument = serde_json::from_str(&json)?;
        assert_eq!(back.toc.len(), doc.toc.len());
        assert_eq!(back.outgoing_links, doc.outgoing_links);
        assert_eq!(back.excerpt, doc.excerpt);
        Ok(())
    }

    // ── Frontmatter 测试 ──

    #[test]
    fn test_frontmatter_stripped_from_ast() -> MSResult<()> {
        let md = "---\nuuid: abc-123\ntitle: 测试卡片\ncategory: 默认\n---\n\n## 正文标题\n\n这是正文内容。";
        let doc = parse_document(md)?;

        // AST 中不应包含 frontmatter 文本
        assert!(!doc.ast_json.contains("uuid"));
        assert!(!doc.ast_json.contains("abc-123"));
        assert!(!doc.ast_json.contains("category"));

        // 正文标题正确解析
        assert_eq!(doc.toc.len(), 1);
        assert_eq!(doc.toc[0].text, "正文标题");

        // 摘要是正文，不是 frontmatter
        assert!(!doc.excerpt.contains("uuid"));
        assert!(doc.excerpt.contains("正文内容"));
        Ok(())
    }

    #[test]
    fn test_frontmatter_raw_captured() -> MSResult<()> {
        let md = "---\nuuid: abc-123\ntitle: 测试\n---\n\n正文";
        let doc = parse_document(md)?;

        assert!(doc.frontmatter.contains("uuid: abc-123"));
        assert!(doc.frontmatter.contains("title: 测试"));
        // 不含分隔符
        assert!(!doc.frontmatter.contains("---"));
        Ok(())
    }

    #[test]
    fn test_no_frontmatter() -> MSResult<()> {
        let md = "# 没有前置信息\n\n纯正文。";
        let doc = parse_document(md)?;

        assert!(doc.frontmatter.is_empty());
        assert_eq!(doc.toc.len(), 1);
        assert_eq!(doc.toc[0].text, "没有前置信息");
        Ok(())
    }

    #[test]
    fn test_malformed_frontmatter_fallback() -> MSResult<()> {
        // 只有开头 `---`，没有闭合，应安全回退为原文解析
        let md = "---\nuuid: abc\n## 标题\n正文";
        let doc = parse_document(md)?;

        assert!(doc.frontmatter.is_empty());
        // 回退为原文解析，所以内容都还在
        assert!(!doc.ast_json.is_empty());
        Ok(())
    }

    #[test]
    fn test_frontmatter_crlf() -> MSResult<()> {
        let md = "---\r\nuuid: abc\r\ntitle: CRLF\r\n---\r\n\r\n## 正文\r\n\r\n内容。";
        let doc = parse_document(md)?;

        assert!(doc.frontmatter.contains("uuid: abc"));
        assert!(doc.frontmatter.contains("title: CRLF"));
        assert!(!doc.ast_json.contains("uuid"));
        assert_eq!(doc.toc.len(), 1);
        assert_eq!(doc.toc[0].text, "正文");
        Ok(())
    }
}
