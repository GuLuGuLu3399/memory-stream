declare module "katex/dist/contrib/auto-render.mjs" {
  interface RenderMathInElementOptions {
    delimiters?: Array<{ left: string; right: string; display: boolean }>;
    throwOnError?: boolean;
    [key: string]: any;
  }

  function renderMathInElement(
    element: HTMLElement,
    options?: RenderMathInElementOptions,
  ): void;
  export default renderMathInElement;
}
