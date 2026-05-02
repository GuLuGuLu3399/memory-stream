// 用途：CodeMirror 格式化扩展，通过 Prettier 格式化 Markdown 内容
import * as prettier from 'prettier'
import * as parserMarkdown from 'prettier/plugins/markdown'
import type { EditorView } from '@codemirror/view'

export async function formatWithPrettier(view: EditorView): Promise<boolean> {
  const doc = view.state.doc.toString()
  try {
    const formatted = await prettier.format(doc, {
      parser: 'markdown',
      plugins: [parserMarkdown],
      proseWrap: 'preserve',
      printWidth: 80,
    })
    if (formatted === doc) return false
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: formatted },
    })
    return true
  } catch {
    return false
  }
}
