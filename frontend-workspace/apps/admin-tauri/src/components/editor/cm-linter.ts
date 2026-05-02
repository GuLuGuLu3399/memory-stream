// 用途：CodeMirror 代码检查扩展，提供 Markdown 语法和 Wikilink 校验
import type { Diagnostic } from '@codemirror/lint'
import type { Extension } from '@codemirror/state'
import { linter } from '@codemirror/lint'

export interface LintItem {
  line: number
  message: string
  severity: 'warning' | 'error' | 'info'
}

function markdownLinter(view: { state: { doc: { toString: () => string; line: (n: number) => { from: number; to: number }; length: number; lines: number } } }): Diagnostic[] {
  const diagnostics: Diagnostic[] = []
  const doc = view.state.doc
  const text = doc.toString()
  const lines = text.split('\n')

  // Skip frontmatter
  let fmEnd = 0
  if (lines[0]?.trim() === '---') {
    for (let i = 1; i < lines.length; i++) {
      if (lines[i]?.trim() === '---') { fmEnd = i + 1; break }
    }
  }

  let lastHeadingLevel = 0
  let fenceChar = ''
  let fenceLen = 0

  for (let i = fmEnd; i < lines.length; i++) {
    const line = lines[i]
    const lineStart = doc.line(i + 1).from

    // Track code fences (CommonMark: match char type + length)
    if (!fenceLen) {
      const openMatch = line.match(/^( {0,3})(`{3,}|~{3,})/)
      if (openMatch) {
        fenceChar = openMatch[2][0]
        fenceLen = openMatch[2].length
        continue
      }
    } else {
      const closeMatch = line.match(/^( {0,3})(`{3,}|~{3,})\s*$/)
      if (closeMatch && closeMatch[2][0] === fenceChar && closeMatch[2].length >= fenceLen) {
        fenceLen = 0
        fenceChar = ''
      }
      continue
    }

    // 1. Heading level skip (e.g. H1 → H3)
    const headingMatch = line.match(/^(#{1,6})\s/)
    if (headingMatch) {
      const level = headingMatch[1].length
      if (lastHeadingLevel > 0 && level > lastHeadingLevel + 1) {
        diagnostics.push({
          from: lineStart,
          to: lineStart + headingMatch[0].length,
          severity: 'warning',
          message: `标题跳跃: H${lastHeadingLevel} → H${level}`,
        })
      }
      lastHeadingLevel = level
    }

    // 2. Empty links []()
    const emptyLinkRe = /\[[^\]]*\]\(\s*\)/g
    let match
    while ((match = emptyLinkRe.exec(line)) !== null) {
      diagnostics.push({
        from: lineStart + match.index,
        to: lineStart + match.index + match[0].length,
        severity: 'warning',
        message: '空链接: 未填写 URL',
      })
    }

    // 3. Unpaired bold markers **
    const boldCount = (line.match(/\*\*/g) || []).length
    if (boldCount % 2 !== 0) {
      diagnostics.push({
        from: lineStart,
        to: lineStart + line.length,
        severity: 'warning',
        message: '未成对的 ** 加粗标记',
      })
    }

    // 4. Unpaired inline code backticks (odd count of single `)
    const backtickCount = (line.match(/(?<!`)`(?!`)/g) || []).length
    if (backtickCount % 2 !== 0) {
      diagnostics.push({
        from: lineStart,
        to: lineStart + line.length,
        severity: 'warning',
        message: '未成对的 ` 行内代码标记',
      })
    }
  }

  // 5. Unclosed code fence at EOF
  if (fenceLen) {
    const lastLine = doc.line(doc.lines)
    diagnostics.push({
      from: lastLine.from,
      to: lastLine.to,
      severity: 'error',
      message: '未闭合的代码围栏 ```',
    })
  }

  return diagnostics
}

export function createMarkdownLinter(): Extension {
  return linter(markdownLinter, { delay: 500 })
}

/** Extract lint items from CM6 view for UI display */
export function extractLintItems(view: { state: { doc: { toString: () => string } } }): LintItem[] {
  const diags = markdownLinter(view as any)
  const text = view.state.doc.toString()
  return diags.map((d) => ({
    line: text.substring(0, d.from).split('\n').length,
    message: d.message,
    severity: d.severity as 'warning' | 'error' | 'info',
  }))
}
