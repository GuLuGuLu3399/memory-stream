use crate::{error::{MSError, MSResult}, AlignType, AstNode, AdmonitionKind, TaskStatus};
use pulldown_cmark::{CowStr, Event, Options, Parser as CmarkParser, Tag};
use regex::Regex;
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::OnceLock;

// ────────────────────────────────────────────────────────────────
// ParseOptions — 可配置的解析选项
// ────────────────────────────────────────────────────────────────

/// Markdown 解析选项，控制启用哪些 GFM / CommonMark 扩展语法。
///
/// 使用 [`ParseOptions::default()`] 获取推荐的全部启用配置。
#[derive(Debug, Clone)]
pub struct ParseOptions {
    /// GFM 表格（`| col1 | col2 |`）
    pub gfm_tables: bool,
    /// 任务列表（`- [x] done`）
    pub tasklists: bool,
    /// 删除线（`~~text~~`）
    pub strikethrough: bool,
    /// 智能标点（引号、破折号等）
    pub smart_punctuation: bool,
    /// 数学公式（`$inline$` 和 `$$display$$`）
    pub math: bool,
    /// 脚注（`[^label]: content`）
    pub footnotes: bool,
    /// 定义列表
    pub definition_list: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            gfm_tables: true,
            tasklists: true,
            strikethrough: true,
            smart_punctuation: true,
            math: true,
            footnotes: true,
            definition_list: true,
        }
    }
}

impl ParseOptions {
    /// 转换为 pulldown-cmark 的位标志。
    fn to_cmark_options(&self) -> Options {
        let mut flags = Options::empty();
        if self.gfm_tables {
            flags.insert(Options::ENABLE_TABLES);
        }
        if self.tasklists {
            flags.insert(Options::ENABLE_TASKLISTS);
        }
        if self.strikethrough {
            flags.insert(Options::ENABLE_STRIKETHROUGH);
        }
        if self.smart_punctuation {
            flags.insert(Options::ENABLE_SMART_PUNCTUATION);
        }
        if self.math {
            flags.insert(Options::ENABLE_MATH);
        }
        if self.footnotes {
            flags.insert(Options::ENABLE_FOOTNOTES);
        }
        if self.definition_list {
            flags.insert(Options::ENABLE_DEFINITION_LIST);
        }
        flags
    }
}

// ────────────────────────────────────────────────────────────────
// 辅助函数
// ────────────────────────────────────────────────────────────────

/// 将 pulldown-cmark 的 `CowStr` 转换为标准库的 `Cow<str>`。
fn cowstr_to_cow(s: CowStr) -> Cow<str> {
    match s {
        CowStr::Borrowed(s) => Cow::Borrowed(s),
        _ => Cow::Owned(s.into_string()),
    }
}

/// 获取 link 正则实例（延迟初始化，仅编译一次）。
fn link_regex() -> MSResult<&'static Regex> {
    static WIKILINK_REGEX: OnceLock<Result<Regex, regex::Error>> = OnceLock::new();
    WIKILINK_REGEX
        .get_or_init(|| Regex::new(r"\[\[(.+?)(?:\|(.+?))?\]\]"))
        .as_ref()
        .map_err(|e| MSError::ParseError(format!("regex 编译失败: {e}")))
}

/// 获取 cloze 正则实例（延迟初始化，仅编译一次）。
fn cloze_regex() -> MSResult<&'static Regex> {
    static RE: OnceLock<Result<Regex, regex::Error>> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\?\?(.+?)\?\?"))
        .as_ref()
        .map_err(|e| MSError::ParseError(format!("regex 编译失败: {e}")))
}

/// 获取 concept_ref 正则实例（延迟初始化，仅编译一次）。
fn concept_ref_regex() -> MSResult<&'static Regex> {
    static RE: OnceLock<Result<Regex, regex::Error>> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"@([\p{L}\p{N}_-]+)"))
        .as_ref()
        .map_err(|e| MSError::ParseError(format!("regex 编译失败: {e}")))
}

// ────────────────────────────────────────────────────────────────
// link 提取
// ────────────────────────────────────────────────────────────────

/// 从 Markdown 文本中提取 link（`[[link]]`）。
///
/// 使用非贪婪匹配（`.+?`），支持链接内容包含括号等特殊字符，
/// 例如 `[[矩阵(Matrix)特征值]]` → `"矩阵(Matrix)特征值"`。
///
/// # Errors
/// 正则表达式编译失败时返回 `MSError::ParseError`（理论上不会发生）。
pub fn extract_links(md: &str) -> MSResult<Vec<String>> {
    let re = link_regex()?;

    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for cap in re.captures_iter(md) {
        let name = cap.get(1).map_or("", |m| m.as_str()).trim().to_string();
        if name.is_empty() {
            continue;
        }
        if seen.insert(name.clone()) {
            result.push(name);
        }
    }

    Ok(result)
}

// ────────────────────────────────────────────────────────────────
// Markdown → AST 解析 — 公共 API
// ────────────────────────────────────────────────────────────────

/// 使用默认选项将 Markdown 文本解析为 AST。
///
/// 等价于 `parse_markdown_with(md_text, &ParseOptions::default())`。
///
/// # Errors
/// 当 Markdown 文本格式无法解析时返回 `MSError::ParseError`。
///
/// # 示例
/// ```ignore
/// let ast = parse_markdown("## Hello")?;
/// ```
pub fn parse_markdown(md_text: &str) -> MSResult<AstNode<'_>> {
    parse_markdown_with(md_text, &ParseOptions::default())
}

/// 使用自定义选项将 Markdown 文本解析为 AST。
///
/// # 参数
/// - `md_text`: 原始 Markdown 文本
/// - `opts`: 解析选项，控制启用哪些扩展语法
///
/// # 返回
/// 解析成功返回 `AstNode::Root`，失败返回 `MSError::ParseError`。
///
/// # Errors
/// 当文档树未正确闭合或内部状态不一致时返回 `MSError::ParseError`。
pub fn parse_markdown_with<'a>(md_text: &'a str, opts: &ParseOptions) -> MSResult<AstNode<'a>> {
    let options = opts.to_cmark_options();
    let parser = CmarkParser::new_ext(md_text, options);
    let mut stack: Vec<AstNode<'a>> = vec![AstNode::Root { children: vec![] }];
    let mut pending_text = String::new();

    for event in parser {
        match event {
            Event::Text(text) => {
                if should_buffer_text(&stack) {
                    pending_text.push_str(&text);
                } else {
                    flush_pending_text(&mut pending_text, &mut stack)?;
                    handle_text(text, &mut stack)?;
                }
            }
            Event::Start(tag) => {
                flush_pending_text(&mut pending_text, &mut stack)?;
                handle_start_tag(tag, &mut stack)?;
            }
            Event::End(_) => {
                flush_pending_text(&mut pending_text, &mut stack)?;
                handle_end_tag(&mut stack)?;
            }
            Event::Code(code) => {
                flush_pending_text(&mut pending_text, &mut stack)?;
                handle_inline_code(code, &mut stack)?;
            }
            Event::Rule => {
                flush_pending_text(&mut pending_text, &mut stack)?;
                push_leaf(&mut stack, AstNode::ThematicBreak)?;
            }
            Event::InlineMath(text) => {
                flush_pending_text(&mut pending_text, &mut stack)?;
                handle_math(text, true, &mut stack)?;
            }
            Event::DisplayMath(text) => {
                flush_pending_text(&mut pending_text, &mut stack)?;
                handle_math(text, false, &mut stack)?;
            }
            Event::TaskListMarker(checked) => {
                flush_pending_text(&mut pending_text, &mut stack)?;
                push_leaf(&mut stack, AstNode::TaskListMarker { checked })?;
            }
            Event::FootnoteReference(name) => {
                flush_pending_text(&mut pending_text, &mut stack)?;
                push_leaf(
                    &mut stack,
                    AstNode::FootnoteReference {
                        name: cowstr_to_cow(name),
                    },
                )?;
            }
            _ => {}
        }
    }

    finalize_stack(&mut stack)
}

fn should_buffer_text(stack: &[AstNode<'_>]) -> bool {
    !matches!(
        stack.last(),
        Some(AstNode::CodeBlock { .. }) | Some(AstNode::Image { .. })
    )
}

fn flush_pending_text<'a>(pending_text: &mut String, stack: &mut Vec<AstNode<'a>>) -> MSResult<()> {
    if pending_text.is_empty() {
        return Ok(());
    }
    if let Some(top) = stack.last_mut() {
        let text = std::mem::take(pending_text);
        split_text_into_nodes(CowStr::from(text), top)?;
    }
    Ok(())
}

// ────────────────────────────────────────────────────────────────
// 内部事件处理器
// ────────────────────────────────────────────────────────────────

/// 处理 `Event::Start`：创建对应节点并入栈。
fn handle_start_tag<'a>(tag: Tag<'a>, stack: &mut Vec<AstNode<'a>>) -> MSResult<()> {
    let node = match tag {
        Tag::Paragraph => AstNode::Paragraph { children: vec![] },
        Tag::Heading { level, .. } => AstNode::Heading {
            level: level as u8,
            children: vec![],
            status: None,
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
        Tag::FootnoteDefinition(name) => AstNode::FootnoteDefinition {
            name: cowstr_to_cow(name),
            children: vec![],
        },
        Tag::DefinitionList => AstNode::DefinitionList { children: vec![] },
        Tag::DefinitionListTitle => AstNode::DefinitionListTitle { children: vec![] },
        Tag::DefinitionListDefinition => AstNode::DefinitionListDefinition { children: vec![] },
        _ => return Ok(()),
    };
    stack.push(node);
    Ok(())
}

/// 处理 `Event::End`：出栈当前节点，追加到父节点。
///
/// 包含后处理：Heading 状态提取、Blockquote → Admonition 转换。
fn handle_end_tag(stack: &mut Vec<AstNode<'_>>) -> MSResult<()> {
    if stack.len() > 1 {
        let mut finished = stack
            .pop()
            .ok_or_else(|| MSError::ParseError("解析栈意外为空".to_string()))?;

        extract_heading_status(&mut finished);
        finished = maybe_convert_admonition(finished);

        if let Some(parent) = stack.last_mut() {
            parent.push_child(finished)?;
        }
    }
    Ok(())
}

fn try_strip_heading_prefix(s: &str) -> Option<(TaskStatus, &str)> {
    [("[x]", TaskStatus::Done), ("[ ]", TaskStatus::Undone), ("[~]", TaskStatus::Unclear)]
        .into_iter()
        .find_map(|(prefix, status)| {
            s.strip_prefix(prefix).map(|rest| (status, rest.trim_start()))
        })
}

fn extract_heading_status(node: &mut AstNode<'_>) {
    if let AstNode::Heading { children, status, .. } = node {
        if status.is_some() {
            return;
        }
        if let Some(AstNode::Text { value }) = children.first_mut() {
            if let Some((marker, rest)) = try_strip_heading_prefix(value.as_ref()) {
                *status = Some(marker);
                *value = Cow::Owned(rest.to_string());
            }
        }
        if matches!(children.first(), Some(AstNode::Text { value }) if value.is_empty()) {
            children.remove(0);
        }
    }
}

fn try_parse_admonition_prefix(s: &str) -> Option<(AdmonitionKind, &str)> {
    [
        ("[!warning]", AdmonitionKind::Warning),
        ("[!tip]", AdmonitionKind::Tip),
        ("[!question]", AdmonitionKind::Question),
    ]
    .into_iter()
    .find_map(|(prefix, kind)| {
        s.strip_prefix(prefix).map(|rest| (kind, rest.trim_start()))
    })
}

fn maybe_convert_admonition(node: AstNode<'_>) -> AstNode<'_> {
    let AstNode::Blockquote { mut children } = node else { return node };

    let Some(AstNode::Paragraph { children: para_children }) = children.first() else {
        return AstNode::Blockquote { children };
    };
    let Some(AstNode::Text { value }) = para_children.first() else {
        return AstNode::Blockquote { children };
    };
    let Some((kind, rest)) = try_parse_admonition_prefix(value.as_ref()) else {
        return AstNode::Blockquote { children };
    };
    let rest_owned = rest.to_string();

    if let Some(AstNode::Paragraph { children: para_children }) = children.first_mut() {
        if let Some(AstNode::Text { value }) = para_children.first_mut() {
            *value = Cow::Owned(rest_owned);
        }
        if matches!(para_children.first(), Some(AstNode::Text { value }) if value.is_empty()) {
            para_children.remove(0);
        }
    }

    AstNode::Admonition { kind, children }
}

/// 处理 `Event::Text`：追加文本到栈顶节点。
fn handle_text<'a>(text: CowStr<'a>, stack: &mut Vec<AstNode<'a>>) -> MSResult<()> {
    if let Some(top) = stack.last_mut() {
        match top {
            AstNode::CodeBlock { value, .. } => {
                value.to_mut().push_str(&text);
            }
            AstNode::Image { alt, .. } => {
                alt.to_mut().push_str(&text);
            }
            _ => {
                split_text_into_nodes(text, top)?;
            }
        }
    }
    Ok(())
}

/// 将文本裂解为 Text / Wikilink / Cloze / ConceptRef 节点并挂载到父节点。
///
/// 三遍正则管线：先拆 wikilink `[[...]]`，再拆 cloze `??...??`，最后拆 concept_ref `@term`。
fn split_text_into_nodes<'a>(text: CowStr<'a>, top: &mut AstNode<'a>) -> MSResult<()> {
    let owned = text.into_string();
    if owned.is_empty() {
        return Ok(());
    }

    // Pass 1: wikilink [[...]]
    let mut pass1 = Vec::new();
    split_by_wikilink(&owned, &mut pass1)?;

    // Pass 2: cloze ??...??
    let mut pass2 = Vec::new();
    for frag in pass1 {
        match frag {
            AstNode::Text { value } => split_by_cloze(&value, &mut pass2)?,
            other => pass2.push(other),
        }
    }

    // Pass 3: concept_ref @term
    let mut pass3 = Vec::new();
    for frag in pass2 {
        match frag {
            AstNode::Text { value } => split_by_concept_ref(&value, &mut pass3)?,
            other => pass3.push(other),
        }
    }

    for node in pass3 {
        top.push_child(node)?;
    }
    Ok(())
}

/// 按 wikilink `[[target|alias]]` 拆分文本。
fn split_by_wikilink<'a>(s: &str, out: &mut Vec<AstNode<'a>>) -> MSResult<()> {
    let re = link_regex()?;
    let mut last_end = 0;
    let mut has_match = false;

    for caps in re.captures_iter(s) {
        has_match = true;
        let Some(full) = caps.get(0) else { continue };

        if full.start() > last_end {
            out.push(AstNode::Text {
                value: Cow::Owned(s[last_end..full.start()].to_string()),
            });
        }

        let target = caps.get(1).map_or("", |m| m.as_str()).trim();
        if !target.is_empty() {
            let alias = caps
                .get(2)
                .map(|m| m.as_str().trim())
                .filter(|a| !a.is_empty())
                .map(|a| Cow::Owned(a.to_string()));
            out.push(AstNode::Wikilink {
                target: Cow::Owned(target.to_string()),
                alias,
            });
        } else {
            out.push(AstNode::Text {
                value: Cow::Owned(full.as_str().to_string()),
            });
        }

        last_end = full.end();
    }

    if has_match && last_end < s.len() {
        out.push(AstNode::Text {
            value: Cow::Owned(s[last_end..].to_string()),
        });
    }
    if !has_match && !s.is_empty() {
        out.push(AstNode::Text {
            value: Cow::Owned(s.to_string()),
        });
    }
    Ok(())
}

/// 按 cloze `??content??` 拆分文本。
fn split_by_cloze<'a>(s: &str, out: &mut Vec<AstNode<'a>>) -> MSResult<()> {
    let re = cloze_regex()?;
    let mut last_end = 0;
    let mut has_match = false;

    for caps in re.captures_iter(s) {
        has_match = true;
        let Some(full) = caps.get(0) else { continue };

        if full.start() > last_end {
            out.push(AstNode::Text {
                value: Cow::Owned(s[last_end..full.start()].to_string()),
            });
        }

        let content = caps.get(1).map_or("", |m| m.as_str());
        out.push(AstNode::Cloze {
            children: vec![AstNode::Text {
                value: Cow::Owned(content.to_string()),
            }],
        });

        last_end = full.end();
    }

    if has_match && last_end < s.len() {
        out.push(AstNode::Text {
            value: Cow::Owned(s[last_end..].to_string()),
        });
    }
    if !has_match && !s.is_empty() {
        out.push(AstNode::Text {
            value: Cow::Owned(s.to_string()),
        });
    }
    Ok(())
}

/// 按 concept_ref `@term` 拆分文本。
/// 跳过前导为字母/数字/下划线/斜杠的 @（排除邮箱地址和 URL）。
fn split_by_concept_ref<'a>(s: &str, out: &mut Vec<AstNode<'a>>) -> MSResult<()> {
    let re = concept_ref_regex()?;
    let mut last_end = 0;
    let mut has_match = false;

    for caps in re.captures_iter(s) {
        let Some(full) = caps.get(0) else { continue };

        if full.start() > 0 {
            let prev = s.as_bytes()[full.start() - 1];
            if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'/' {
                continue;
            }
        }

        has_match = true;

        if full.start() > last_end {
            out.push(AstNode::Text {
                value: Cow::Owned(s[last_end..full.start()].to_string()),
            });
        }

        let term = caps.get(1).map_or("", |m| m.as_str());
        out.push(AstNode::ConceptRef {
            term: Cow::Owned(term.to_string()),
        });

        last_end = full.end();
    }

    if has_match && last_end < s.len() {
        out.push(AstNode::Text {
            value: Cow::Owned(s[last_end..].to_string()),
        });
    }
    if !has_match && !s.is_empty() {
        out.push(AstNode::Text {
            value: Cow::Owned(s.to_string()),
        });
    }
    Ok(())
}

/// 处理 `Event::Code`：追加行内代码到栈顶节点。
fn handle_inline_code<'a>(code: CowStr<'a>, stack: &mut Vec<AstNode<'a>>) -> MSResult<()> {
    if let Some(top) = stack.last_mut() {
        top.push_child(AstNode::InlineCode {
            value: cowstr_to_cow(code),
        })?;
    }
    Ok(())
}

/// 处理 `Event::InlineMath` / `Event::DisplayMath`：追加数学公式节点。
fn handle_math<'a>(text: CowStr<'a>, inline: bool, stack: &mut Vec<AstNode<'a>>) -> MSResult<()> {
    if let Some(top) = stack.last_mut() {
        top.push_child(AstNode::Math {
            value: cowstr_to_cow(text),
            inline,
        })?;
    }
    Ok(())
}

/// 向栈顶节点追加一个叶子节点（`ThematicBreak`、`TaskListMarker` 等）。
fn push_leaf<'a>(stack: &mut Vec<AstNode<'a>>, node: AstNode<'a>) -> MSResult<()> {
    if let Some(top) = stack.last_mut() {
        top.push_child(node)?;
    }
    Ok(())
}

/// 从栈中弹出最终的 AST 根节点。
fn finalize_stack<'a>(stack: &mut Vec<AstNode<'a>>) -> MSResult<AstNode<'a>> {
    if stack.len() == 1 {
        stack
            .pop()
            .ok_or_else(|| MSError::ParseError("解析栈意外为空".to_string()))
    } else {
        Err(MSError::ParseError("文档树未正确闭合".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AstNode;

    fn get_first_child<'a>(ast: &'a AstNode<'a>) -> &'a AstNode<'a> {
        match ast {
            AstNode::Root { children } => children.first().expect("AST 树为空"),
            _ => panic!("解析出的根节点不是 Root"),
        }
    }

    #[test]
    fn test_parse_heading() -> Result<(), Box<dyn std::error::Error>> {
        let md = "### 这是一个三级标题";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        match first_node {
            AstNode::Heading { level, children, .. } => {
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
        Ok(())
    }

    #[test]
    fn test_parse_inline_formatting() -> Result<(), Box<dyn std::error::Error>> {
        let md = "正常**加粗***斜体*";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        if let AstNode::Paragraph { children } = first_node {
            assert_eq!(children.len(), 3, "应该被解析为 3 个独立的行内节点");

            assert!(matches!(children[0], AstNode::Text { .. }));
            assert!(matches!(children[1], AstNode::Strong { .. }));
            assert!(matches!(children[2], AstNode::Emphasis { .. }));
        } else {
            panic!("未能正确解析出 Paragraph 节点");
        }
        Ok(())
    }

    #[test]
    fn test_parse_code_block() -> Result<(), Box<dyn std::error::Error>> {
        let md = "```rust\nfn main() {}\n```";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        if let AstNode::CodeBlock { language, value } = first_node {
            assert_eq!(language.as_deref(), Some("rust"));
            assert_eq!(value.trim(), "fn main() {}");
        } else {
            panic!("未能正确解析出 CodeBlock 节点");
        }
        Ok(())
    }

    #[test]
    fn test_parse_complex_list() -> Result<(), Box<dyn std::error::Error>> {
        let md = "- 核心**引擎**";
        let ast = parse_markdown(md)?;
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
        Ok(())
    }

    #[test]
    fn test_parse_link() -> Result<(), Box<dyn std::error::Error>> {
        let md = "[点击这里](https://example.com)";
        let ast = parse_markdown(md)?;
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
        Ok(())
    }

    #[test]
    fn test_parse_image() -> Result<(), Box<dyn std::error::Error>> {
        let md = "![这是一张图片](https://example.com/img.png)";
        let ast = parse_markdown(md)?;
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
        Ok(())
    }

    // ── ParseOptions 测试 ──

    #[test]
    fn test_parse_options_default_all_enabled() {
        let opts = ParseOptions::default();
        assert!(opts.gfm_tables);
        assert!(opts.tasklists);
        assert!(opts.strikethrough);
        assert!(opts.smart_punctuation);
        assert!(opts.math);
        assert!(opts.footnotes);
        assert!(opts.definition_list);
    }

    #[test]
    fn test_parse_with_minimal_options() -> Result<(), Box<dyn std::error::Error>> {
        let opts = ParseOptions {
            gfm_tables: false,
            tasklists: false,
            strikethrough: false,
            smart_punctuation: false,
            math: false,
            footnotes: false,
            definition_list: false,
        };
        // 基础 Markdown 应始终可用，无论扩展开关如何
        let md = "# Hello\n\nParagraph text";
        let ast = parse_markdown_with(md, &opts)?;
        match ast {
            AstNode::Root { children } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("根节点不是 Root"),
        }
        Ok(())
    }

    #[test]
    fn test_parse_with_default_matches_parse_markdown() -> Result<(), Box<dyn std::error::Error>> {
        let md = "## 标题\n\n- 列表项\n\n```\ncode\n```";
        let ast_default = parse_markdown(md)?;
        let ast_explicit = parse_markdown_with(md, &ParseOptions::default())?;
        assert_eq!(ast_default, ast_explicit);
        Ok(())
    }

    #[test]
    fn test_parse_link_as_structured_node() -> Result<(), Box<dyn std::error::Error>> {
        let md = "Hello [[World]]!";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        if let AstNode::Paragraph { children } = first_node {
            assert_eq!(children.len(), 3);

            assert!(matches!(
                &children[0],
                AstNode::Text { value } if value.as_ref() == "Hello "
            ));
            assert!(matches!(
                &children[1],
                AstNode::Wikilink { target, alias } if target.as_ref() == "World" && alias.is_none()
            ));
            assert!(matches!(
                &children[2],
                AstNode::Text { value } if value.as_ref() == "!"
            ));
        } else {
            panic!("未能正确解析出 Paragraph 节点");
        }

        Ok(())
    }

    #[test]
    fn test_parse_link_not_expanded_in_code_block() -> Result<(), Box<dyn std::error::Error>> {
        let md = "```js\nconst s = \"[[World]]\";\n```";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        if let AstNode::CodeBlock { value, .. } = first_node {
            assert!(value.contains("[[World]]"));
        } else {
            panic!("未能正确解析出 CodeBlock 节点");
        }

        Ok(())
    }

    // ── 408 扩展语法测试 ──

    #[test]
    fn test_parse_cloze() -> Result<(), Box<dyn std::error::Error>> {
        let md = "操作系统 ??进程?? 和 ??线程??";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        if let AstNode::Paragraph { children } = first_node {
            assert_eq!(children.len(), 4);
            assert!(matches!(&children[0], AstNode::Text { value } if value.as_ref() == "操作系统 "));
            assert!(matches!(&children[2], AstNode::Text { value } if value.as_ref() == " 和 "));

            if let AstNode::Cloze { children: inner } = &children[1] {
                assert_eq!(inner.len(), 1);
                assert!(matches!(&inner[0], AstNode::Text { value } if value.as_ref() == "进程"));
            } else {
                panic!("children[1] 不是 Cloze");
            }
            if let AstNode::Cloze { children: inner } = &children[3] {
                assert!(matches!(&inner[0], AstNode::Text { value } if value.as_ref() == "线程"));
            } else {
                panic!("children[3] 不是 Cloze");
            }
        } else {
            panic!("未能正确解析出 Paragraph 节点");
        }
        Ok(())
    }

    #[test]
    fn test_parse_concept_ref() -> Result<(), Box<dyn std::error::Error>> {
        let md = "复习 @TCP三次握手 和 @进程间通信";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        if let AstNode::Paragraph { children } = first_node {
            assert_eq!(children.len(), 4);
            assert!(matches!(&children[0], AstNode::Text { value } if value.as_ref() == "复习 "));
            assert!(matches!(&children[1], AstNode::ConceptRef { term } if term.as_ref() == "TCP三次握手"));
            assert!(matches!(&children[2], AstNode::Text { value } if value.as_ref() == " 和 "));
            assert!(matches!(&children[3], AstNode::ConceptRef { term } if term.as_ref() == "进程间通信"));
        } else {
            panic!("未能正确解析出 Paragraph 节点");
        }
        Ok(())
    }

    #[test]
    fn test_concept_ref_skips_email() -> Result<(), Box<dyn std::error::Error>> {
        let md = "联系 user@example.com 和 @HTTP协议";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        if let AstNode::Paragraph { children } = first_node {
            assert_eq!(children.len(), 2);
            assert!(matches!(&children[0], AstNode::Text { value } if value.as_ref() == "联系 user@example.com 和 "));
            assert!(matches!(&children[1], AstNode::ConceptRef { term } if term.as_ref() == "HTTP协议"));
        } else {
            panic!("未能正确解析出 Paragraph 节点");
        }
        Ok(())
    }

    #[test]
    fn test_parse_heading_status_done() -> Result<(), Box<dyn std::error::Error>> {
        let md = "## [x] 已掌握";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        match first_node {
            AstNode::Heading { level, children, status } => {
                assert_eq!(*level, 2);
                assert_eq!(*status, Some(TaskStatus::Done));
                assert_eq!(children.len(), 1);
                assert!(matches!(&children[0], AstNode::Text { value } if value.as_ref() == "已掌握"));
            }
            _ => panic!("未能正确解析出 Heading 节点"),
        }
        Ok(())
    }

    #[test]
    fn test_parse_heading_status_undone() -> Result<(), Box<dyn std::error::Error>> {
        let md = "### [ ] 待复习";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        match first_node {
            AstNode::Heading { level, status, .. } => {
                assert_eq!(*level, 3);
                assert_eq!(*status, Some(TaskStatus::Undone));
            }
            _ => panic!("未能正确解析出 Heading 节点"),
        }
        Ok(())
    }

    #[test]
    fn test_parse_heading_status_unclear() -> Result<(), Box<dyn std::error::Error>> {
        let md = "#### [~] 不确定";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        match first_node {
            AstNode::Heading { level, status, .. } => {
                assert_eq!(*level, 4);
                assert_eq!(*status, Some(TaskStatus::Unclear));
            }
            _ => panic!("未能正确解析出 Heading 节点"),
        }
        Ok(())
    }

    #[test]
    fn test_parse_heading_no_status() -> Result<(), Box<dyn std::error::Error>> {
        let md = "## 普通标题";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        match first_node {
            AstNode::Heading { level, status, .. } => {
                assert_eq!(*level, 2);
                assert!(status.is_none());
            }
            _ => panic!("未能正确解析出 Heading 节点"),
        }
        Ok(())
    }

    #[test]
    fn test_parse_admonition_warning() -> Result<(), Box<dyn std::error::Error>> {
        let md = "> [!warning] 注意";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        match first_node {
            AstNode::Admonition { kind, children } => {
                assert_eq!(*kind, AdmonitionKind::Warning);
                if let Some(AstNode::Paragraph { children: para_children }) = children.first() {
                    if let Some(AstNode::Text { value }) = para_children.first() {
                        assert_eq!(value.as_ref(), "注意");
                    }
                } else {
                    panic!("Admonition 首子节点不是 Paragraph");
                }
            }
            _ => panic!("未能正确解析出 Admonition 节点"),
        }
        Ok(())
    }

    #[test]
    fn test_parse_admonition_tip() -> Result<(), Box<dyn std::error::Error>> {
        let md = "> [!tip] 用这个方法更快";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        match first_node {
            AstNode::Admonition { kind, .. } => {
                assert_eq!(*kind, AdmonitionKind::Tip);
            }
            _ => panic!("未能正确解析出 Admonition 节点"),
        }
        Ok(())
    }

    #[test]
    fn test_parse_admonition_question() -> Result<(), Box<dyn std::error::Error>> {
        let md = "> [!question] 为什么";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        match first_node {
            AstNode::Admonition { kind, .. } => {
                assert_eq!(*kind, AdmonitionKind::Question);
            }
            _ => panic!("未能正确解析出 Admonition 节点"),
        }
        Ok(())
    }

    #[test]
    fn test_parse_blockquote_not_admonition() -> Result<(), Box<dyn std::error::Error>> {
        let md = "> 普通引用";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        match first_node {
            AstNode::Blockquote { .. } => {}
            _ => panic!("应该解析为 Blockquote"),
        }
        Ok(())
    }

    #[test]
    fn test_parse_mixed_syntax() -> Result<(), Box<dyn std::error::Error>> {
        let md = "复习 @OS，参考 [[进程管理]]，记住 ??死锁条件??";
        let ast = parse_markdown(md)?;
        let first_node = get_first_child(&ast);

        if let AstNode::Paragraph { children } = first_node {
            // Text("复习 ") + ConceptRef("OS") + Text("，参考 ") + Wikilink("进程管理") + Text("，记住 ") + Cloze("死锁条件")
            assert!(children.len() >= 5);
            assert!(matches!(&children[0], AstNode::Text { value } if value.as_ref() == "复习 "));
            assert!(matches!(&children[1], AstNode::ConceptRef { term } if term.as_ref() == "OS"));
            assert!(matches!(&children[2], AstNode::Text { value } if value.as_ref() == "，参考 "));
            assert!(matches!(&children[3], AstNode::Wikilink { .. }));
        } else {
            panic!("未能正确解析出 Paragraph 节点");
        }
        Ok(())
    }
}
