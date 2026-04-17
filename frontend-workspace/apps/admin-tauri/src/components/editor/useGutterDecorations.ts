import {
  EditorView,
  ViewPlugin,
  ViewUpdate,
  GutterMarker,
} from "@codemirror/view";
import { RangeSetBuilder, Extension } from "@codemirror/state";

const WIKILINK_REGEX = /\[\[([^\]]+)\]\]/g;

/**
 * Brass dot marker for lines containing wikilinks
 * Shows in the gutter as a visual indicator
 */
class WikilinkMarker extends GutterMarker {
  toDOM() {
    const dot = document.createElement("span");
    dot.className = "cm-wikilink-gutter-marker";
    dot.style.cssText = `
      display: inline-block;
      width: 6px;
      height: 6px;
      background-color: #b8860b;
      border-radius: 50%;
      box-shadow: 0 0 4px rgba(184, 134, 11, 0.5);
      margin-top: 6px;
    `;
    return dot;
  }
}

/**
 * Build gutter decorations for lines containing wikilinks
 */
function buildGutterDecorations(view: EditorView) {
  const builder = new RangeSetBuilder<GutterMarker>();
  const doc = view.state.doc;

  for (let line = 1; line <= doc.lines; line++) {
    const lineObj = doc.line(line);
    const text = lineObj.text;

    WIKILINK_REGEX.lastIndex = 0;
    if (WIKILINK_REGEX.test(text)) {
      builder.add(lineObj.from, lineObj.from, new WikilinkMarker());
    }
  }

  return builder.finish();
}

/**
 * Composable for brass dot gutter decorations on wikilink lines
 *
 * Features:
 * - Shows brass dot in gutter for each line containing [[wikilinks]]
 * - Updates on document changes
 * - Rebuilds decorations only when necessary (doc changed)
 */
export function useGutterDecorations() {
  /**
   * Create the gutter decorations ViewPlugin
   */
  function createGutterDecorationsPlugin(): Extension {
    const wikilinkGutterPlugin = ViewPlugin.fromClass(
      class {
        decorations: ReturnType<typeof buildGutterDecorations>;

        constructor(view: EditorView) {
          this.decorations = buildGutterDecorations(view);
        }

        update(update: ViewUpdate) {
          if (update.docChanged) {
            this.decorations = buildGutterDecorations(update.view);
          }
        }
      },
    );

    // Return the plugin without the decorations property wrapper
    // Gutter markers are handled differently than regular decorations
    return wikilinkGutterPlugin;
  }

  return {
    createGutterDecorationsPlugin,
  };
}
