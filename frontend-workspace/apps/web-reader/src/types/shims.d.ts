// ── 模块类型声明补丁 ──

// KaTeX auto-render ESM 模块
declare module "katex/dist/contrib/auto-render.mjs" {
  import type { KatexOptions } from "katex";

  interface AutoRenderOptions extends KatexOptions {
    delimiters?: Array<{
      left: string;
      right: string;
      display: boolean;
    }>;
    ignoredTags?: string[];
    throwOnError?: boolean;
  }

  function renderMathInElement(
    element: HTMLElement,
    options?: AutoRenderOptions,
  ): void;

  export default renderMathInElement;
}
