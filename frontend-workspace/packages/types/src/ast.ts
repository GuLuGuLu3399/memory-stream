// ────────────────────────────────────────────────────────────────
// AstNode — mirrors ms-ast/src/node.rs
// serde: #[serde(tag = "type")] → TS discriminated union
// ────────────────────────────────────────────────────────────────

export interface AstRoot {
  type: "Root";
  children: AstNode[];
}

export interface AstHeading {
  type: "Heading";
  level: number;
  children: AstNode[];
  status?: TaskStatus;
}

export interface AstParagraph {
  type: "Paragraph";
  children: AstNode[];
}

export interface AstText {
  type: "Text";
  value: string;
}

export interface AstWikilink {
  type: "Wikilink";
  target: string;
  alias: string | null;
}

export interface AstInlineCode {
  type: "InlineCode";
  value: string;
}

export interface AstCodeBlock {
  type: "CodeBlock";
  language: string | null;
  value: string;
}

export interface AstLink {
  type: "Link";
  url: string;
  children: AstNode[];
}

export interface AstImage {
  type: "Image";
  url: string;
  alt: string;
}

export interface AstMath {
  type: "Math";
  value: string;
  inline: boolean;
}

export interface AstBlockquote {
  type: "Blockquote";
  children: AstNode[];
}

export interface AstList {
  type: "List";
  ordered: boolean;
  start: number | null;
  children: AstNode[];
}

export interface AstListItem {
  type: "ListItem";
  children: AstNode[];
}

export interface AstThematicBreak {
  type: "ThematicBreak";
}

export interface AstStrong {
  type: "Strong";
  children: AstNode[];
}

export interface AstEmphasis {
  type: "Emphasis";
  children: AstNode[];
}

export interface AstTable {
  type: "Table";
  alignments: Array<AlignType | null>;
  children: AstNode[];
}

export interface AstTableHead {
  type: "TableHead";
  children: AstNode[];
}

export interface AstTableRow {
  type: "TableRow";
  children: AstNode[];
}

export interface AstTableCell {
  type: "TableCell";
  children: AstNode[];
}

export interface AstStrikethrough {
  type: "Strikethrough";
  children: AstNode[];
}

export interface AstTaskListMarker {
  type: "TaskListMarker";
  checked: boolean;
}

export interface AstFootnoteDefinition {
  type: "FootnoteDefinition";
  name: string;
  children: AstNode[];
}

export interface AstFootnoteReference {
  type: "FootnoteReference";
  name: string;
}

export interface AstDefinitionList {
  type: "DefinitionList";
  children: AstNode[];
}

export interface AstDefinitionListTitle {
  type: "DefinitionListTitle";
  children: AstNode[];
}

export interface AstDefinitionListDefinition {
  type: "DefinitionListDefinition";
  children: AstNode[];
}

export interface AstCloze {
  type: "Cloze";
  children: AstNode[];
}

export interface AstConceptRef {
  type: "ConceptRef";
  term: string;
}

export interface AstAdmonition {
  type: "Admonition";
  kind: AdmonitionKind;
  children: AstNode[];
}

export type AstNode =
  | AstRoot
  | AstHeading
  | AstParagraph
  | AstText
  | AstWikilink
  | AstInlineCode
  | AstCodeBlock
  | AstLink
  | AstImage
  | AstMath
  | AstBlockquote
  | AstList
  | AstListItem
  | AstThematicBreak
  | AstStrong
  | AstEmphasis
  | AstTable
  | AstTableHead
  | AstTableRow
  | AstTableCell
  | AstStrikethrough
  | AstTaskListMarker
  | AstFootnoteDefinition
  | AstFootnoteReference
  | AstDefinitionList
  | AstDefinitionListTitle
  | AstDefinitionListDefinition
  | AstCloze
  | AstConceptRef
  | AstAdmonition;

// ────────────────────────────────────────────────────────────────
// AlignType — mirrors ms-ast/src/node.rs
// serde: unit variants → plain strings
// ────────────────────────────────────────────────────────────────

export type AlignType = "Left" | "Center" | "Right" | "None";

// ────────────────────────────────────────────────────────────────
// Enum types — mirrors ms-ast/src/node.rs (PascalCase serde defaults)
// ────────────────────────────────────────────────────────────────

export type AdmonitionKind = "Warning" | "Tip" | "Question";

export type TaskStatus = "Done" | "Undone" | "Unclear";

// ────────────────────────────────────────────────────────────────
// TOC — mirrors ms-ast/src/toc.rs
// ────────────────────────────────────────────────────────────────

export interface TocNode {
  level: number;
  text: string;
  slug: string;
  children: TocNode[];
}

export interface TocFlatItem {
  level: number;
  text: string;
  slug: string;
}

// ────────────────────────────────────────────────────────────────
// DocAnalysis — document telemetry contract from Rust
// ────────────────────────────────────────────────────────────────

export interface DocumentStats {
  lines: number
  chars: number
  words: number
  read_time: number
}

export interface TocItem {
  level: number
  text: string
  id: string
}

export interface DocAnalysis {
  stats: DocumentStats
  toc: TocItem[]
  excerpt: string
  outbound_links: number
}

// ────────────────────────────────────────────────────────────────
// ParsedDocument — mirrors admin-tauri Rust parsed document contract
// ────────────────────────────────────────────────────────────────

import type { CardMeta } from "./meta";

export interface ParsedDocument {
  meta: CardMeta | null;
  analysis: DocAnalysis;
  ast: AstNode;
  content: string;
}
