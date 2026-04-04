/**
 * 🌟 核心引擎契约 (Interface)
 * 所有解析引擎（Tauri IPC、WASM、纯 JS）都必须实现这个接口
 */
export interface RenderResult {
  html: string;
  ast_json: string;
}

/**
 * 解析引擎函数签名
 */
export type ParseEngine = (markdown: string) => Promise<RenderResult>;

/**
 * 保存事件回调参数
 */
export interface SavePayload {
  rawMd: string;
  astJson: string;
}
