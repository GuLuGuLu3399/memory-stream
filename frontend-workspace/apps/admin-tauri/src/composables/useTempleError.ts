/**
 * useTempleError — 结构化错误处理
 *
 * 解析 Tauri invoke 返回的错误，支持两种格式：
 * 1. 旧格式: plain string (子模块 command 仍返回 String)
 * 2. 新格式: TempleError JSON { code, message, details }
 */

export interface TempleError {
  code: string
  message: string
  details?: string
}

/** 错误码 → 用户友好的展示配置 */
const ERROR_UI: Record<string, { icon: string; title: string }> = {
  FILE_NOT_FOUND:      { icon: '👻', title: '文件已遁入虚空' },
  MARKDOWN_PARSE_FAILED:{ icon: '⚠️', title: 'Markdown 解析失败' },
  AST_RENDER_FAILED:   { icon: '⚠️', title: '渲染失败' },
  NETWORK_UNREACHABLE: { icon: '📡', title: '网络连接中断' },
  API_ERROR:           { icon: '🔌', title: '服务器错误' },
  AUTH_EXPIRED:        { icon: '🔐', title: '凭证已过期' },
  WS_CONNECTION_FAILED:{ icon: '🔗', title: '实时连接断开' },
  CACHE_NOT_INITIALIZED:{ icon: '💾', title: '缓存未就绪' },
  GRAPH_NODE_NOT_FOUND:{ icon: '🔍', title: '卡片未找到' },
  S3_UPLOAD_FAILED:    { icon: '☁️', title: '存储上传失败' },
  STORAGE_CONFIG_ERROR:{ icon: '⚙️', title: '存储配置错误' },
  IMAGE_DECODE_FAILED: { icon: '🖼️', title: '图片解码失败' },
  EXPORT_ZIP_FAILED:   { icon: '📦', title: '导出失败' },
  DRAFT_SQL_FAILED:    { icon: '📝', title: '草稿保存失败' },
}

/**
 * 尝试将 Tauri invoke 错误解析为 TempleError
 * 兼容 string 和 object 两种格式
 */
export function parseTempleError(e: unknown): TempleError {
  if (typeof e === 'string') {
    // 尝试解析 JSON string (Tauri 可能传序列化后的 JSON 字符串)
    try {
      const parsed = JSON.parse(e)
      if (parsed && parsed.code && parsed.message) {
        return parsed as TempleError
      }
    } catch {
      // 不是 JSON，是普通字符串
    }
    return { code: 'UNKNOWN', message: e }
  }

  if (e && typeof e === 'object' && 'code' in e && 'message' in e) {
    return e as TempleError
  }

  return { code: 'UNKNOWN', message: String(e) }
}

/** 获取错误的 UI 展示配置 */
export function getErrorDisplay(error: TempleError) {
  return ERROR_UI[error.code] ?? { icon: '❌', title: '操作失败' }
}

/** 获取错误的用户友好消息（用于 toast） */
export function formatErrorMessage(e: unknown): string {
  const err = parseTempleError(e)
  const display = getErrorDisplay(err)
  return `${display.title}: ${err.message}`
}
