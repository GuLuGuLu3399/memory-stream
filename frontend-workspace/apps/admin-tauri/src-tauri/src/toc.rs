//! # 目录树提取命令
//!
//! 对接 `ms-toc-extractor` Crate，从 AST JSON 中提取层级目录结构。
//! 纯 CPU 计算，速度极快（微秒级）。

use ms_toc_extractor::TocNode;
use serde::Serialize;

/// 目录树节点（直接复用 crate 的结构，增加 Serialize）
///
/// 前端可用于渲染侧边栏目录导航。
#[derive(Serialize)]
pub struct TocNodeDto {
    pub level: u8,
    pub text: String,
    pub slug: String,
    pub children: Vec<TocNodeDto>,
}

impl From<TocNode> for TocNodeDto {
    fn from(n: TocNode) -> Self {
        Self {
            level: n.level,
            text: n.text,
            slug: n.slug,
            children: n.children.into_iter().map(TocNodeDto::from).collect(),
        }
    }
}

/// 从 AST JSON 字符串中提取目录树
///
/// 接收 `process_markdown` 返回的 `ast_json` 字符串，
/// 解析后提取所有标题节点，构建层级目录树。
///
/// # 参数
/// - `ast_json`: `process_markdown` 返回的 AST JSON 字符串
///
/// # 返回
/// 层级目录树数组（顶层为 h1 节点）
#[tauri::command]
pub fn extract_toc(ast_json: String) -> Result<Vec<TocNodeDto>, String> {
    let toc = ms_toc_extractor::extract_toc_from_json(&ast_json)
        .map_err(|e| format!("目录提取失败: {:?}", e))?;
    Ok(toc.into_iter().map(TocNodeDto::from).collect())
}
