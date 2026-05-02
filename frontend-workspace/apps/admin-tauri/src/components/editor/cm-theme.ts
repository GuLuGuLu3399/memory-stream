// 用途：CodeMirror 主题配置，定义编辑器的暗色配色方案
import { EditorView } from "@codemirror/view"
import { tags as t } from "@lezer/highlight"
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language"

export const altarTheme = EditorView.theme({
  "&": {
    backgroundColor: "transparent",
    color: "var(--text-primary)",
    height: "100%",
    fontSize: "14px",
    fontFamily: "var(--font-mono)",
  },
  ".cm-content": {
    caretColor: "var(--neon, #00e5ff)",
    fontFamily: "var(--font-mono)",
  },
  "&.cm-focused": {
    outline: "none",
  },
  "&.cm-focused .cm-cursor": {
    borderLeftColor: "var(--neon, #00e5ff)",
    borderLeftWidth: "2px",
  },
  "&.cm-focused .cm-selectionBackground, ::selection": {
    backgroundColor: "oklch(0.78 0.17 200 / 0.2)",
  },
  ".cm-gutters": {
    backgroundColor: "transparent",
    color: "var(--text-muted)",
    border: "none",
    opacity: "0.5",
  },
  ".cm-activeLineGutter": {
    backgroundColor: "transparent",
    color: "var(--text-secondary)",
  },
  ".cm-activeLine": {
    backgroundColor: "oklch(1 0 0 / 0.03)",
  },
  ".cm-selectionMatch": {
    backgroundColor: "oklch(0.78 0.17 200 / 0.1)",
  },
  ".cm-matchingBracket": {
    backgroundColor: "oklch(0.78 0.17 200 / 0.25)",
    outline: "1px solid oklch(0.78 0.17 200 / 0.4)",
  },
  ".cm-lintRange-warning": {
    backgroundImage: "repeating-linear-gradient(to bottom, oklch(0.75 0.15 85 / 0.6) 0, oklch(0.75 0.15 85 / 0.6) 100%)",
    backgroundSize: "2px 2px",
    backgroundPositionY: "bottom",
    backgroundRepeat: "repeat-x",
    paddingBottom: "2px",
  },
  ".cm-lintRange-error": {
    backgroundImage: "repeating-linear-gradient(to bottom, oklch(0.65 0.2 25 / 0.6) 0, oklch(0.65 0.2 25 / 0.6) 100%)",
    backgroundSize: "2px 2px",
    backgroundPositionY: "bottom",
    backgroundRepeat: "repeat-x",
    paddingBottom: "2px",
  },
  ".cm-diagnostic": {
    fontFamily: "var(--font-mono)",
    fontSize: "11px",
  },
}, { dark: true })

export const altarHighlightStyle = syntaxHighlighting(HighlightStyle.define([
  { tag: t.heading1, color: "#f8fafc", fontWeight: "bold", fontSize: "1.4em" },
  { tag: t.heading2, color: "#f1f5f9", fontWeight: "bold", fontSize: "1.2em" },
  { tag: t.heading3, color: "#e2e8f0", fontWeight: "bold", fontSize: "1.1em" },
  { tag: t.heading, color: "#e2e8f0", fontWeight: "bold" },
  { tag: t.strong, fontWeight: "bold", color: "#f1f5f9" },
  { tag: t.emphasis, fontStyle: "italic", color: "#cbd5e1" },
  { tag: t.strikethrough, textDecoration: "line-through", color: "var(--text-muted)" },
  { tag: t.link, color: "#60a5fa", textDecoration: "underline" },
  { tag: t.url, color: "#60a5fa" },
  { tag: t.monospace, color: "var(--brass, #d4af37)", backgroundColor: "oklch(1 0 0 / 0.06)" },
  { tag: t.quote, color: "var(--text-muted)", fontStyle: "italic" },
  { tag: t.keyword, color: "var(--neon, #00e5ff)" },
  { tag: t.string, color: "#86efac" },
  { tag: t.meta, color: "var(--text-muted)" },
  { tag: t.comment, color: "var(--text-muted)", fontStyle: "italic" },
  { tag: t.processingInstruction, color: "var(--neon, #00e5ff)", opacity: "0.6" },
  { tag: t.punctuation, color: "var(--text-muted)", opacity: "0.6" },
]))
