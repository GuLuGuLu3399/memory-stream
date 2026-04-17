declare module 'katex/dist/contrib/auto-render.mjs' {
  import katex from 'katex';
  export default renderMathInElement;
  function renderMathInElement(
    elem: HTMLElement,
    options?: {
      delimiters?: Array<{ left: string; right: string; display: boolean }>;
      ignoredTags?: string[];
      throwOnError?: boolean;
    }
  ): void;
}
