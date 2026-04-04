//! MemoryStream WASM 渲染引擎。
//!
//! 提供 Markdown 渲染的 WebAssembly 接口，供浏览器端和 Tauri 桌面端调用。
//! 支持两种渲染路径：
//! - 完整管线（`process_markdown`）：RawMd → AST → HTML + AST JSON
//! - 快速渲染（`render_from_ast`）：AST JSON → HTML（跳过解析，零开销）

use wasm_bindgen::prelude::*;

/// WASM 渲染结果 — 包含渲染后的 HTML 和序列化的 AST JSON。
#[wasm_bindgen(getter_with_clone)]
pub struct WasmRenderResult {
    /// 渲染后的 HTML 字符串
    pub html: String,
    /// 序列化的 AST JSON（存入数据库用于快速渲染路径）
    pub ast_json: String,
}

/// 完整管线：RawMd → 解析 → AST → HTML + AST JSON。
///
/// 创建/编辑卡片时使用，输出 `ast_data` 存入数据库供后续快速渲染。
///
/// # 参数
/// - `raw_md`: 原始 Markdown 文本
///
/// # 返回
/// `WasmRenderResult` 包含 html 和 ast_json 字段
#[wasm_bindgen]
pub fn process_markdown(raw_md: &str) -> Result<WasmRenderResult, JsValue> {
    let ast = md_parser::parse_markdown(raw_md)
        .map_err(|e| JsValue::from_str(&format!("解析失败: {}", e)))?;

    let html = ast_renderer::render_to_html(&ast)
        .map_err(|e| JsValue::from_str(&format!("渲染失败: {}", e)))?;

    let ast_json = serde_json::to_string(&ast)
        .map_err(|e| JsValue::from_str(&format!("JSON序列化失败: {}", e)))?;

    Ok(WasmRenderResult { html, ast_json })
}

/// 快速渲染：AST JSON → HTML（跳过解析，直接渲染）。
///
/// 查看/阅读卡片时使用，从数据库读取 `ast_data` 直接渲染，
/// 无需重新解析 Markdown，性能接近零开销。
///
/// # 参数
/// - `ast_json`: 序列化的 AST JSON 字符串
///
/// # 返回
/// 渲染后的 HTML 字符串
#[wasm_bindgen]
pub fn render_from_ast(ast_json: &str) -> Result<String, JsValue> {
    let ast: ast_core::AstNodeOwned = serde_json::from_str(ast_json)
        .map_err(|e| JsValue::from_str(&format!("AST 反序列化失败: {}", e)))?;

    let html = ast_renderer::render_to_html(&ast)
        .map_err(|e| JsValue::from_str(&format!("渲染失败: {}", e)))?;

    Ok(html)
}

/// TOC 提取：AST JSON → 目录树 JSON。
///
/// 从序列化的 AST 中提取所有标题节点，构建树形目录结构。
/// 纯计算，无 I/O，可在浏览器端通过 WASM 零延迟调用。
///
/// # 参数
/// - `ast_json`: 序列化的 AST JSON 字符串
///
/// # 返回
/// TOC 树的 JSON 字符串
#[wasm_bindgen]
pub fn extract_toc(ast_json: &str) -> Result<String, JsValue> {
    let toc = ms_toc_extractor::extract_toc_from_json(ast_json)
        .map_err(|e| JsValue::from_str(&format!("TOC 提取失败: {}", e)))?;
    serde_json::to_string(&toc).map_err(|e| JsValue::from_str(&format!("TOC 序列化失败: {}", e)))
}
