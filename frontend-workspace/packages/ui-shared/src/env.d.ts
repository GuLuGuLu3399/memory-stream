/// <reference types="vue" />
declare module "katex/dist/katex.min.css";

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<object, object, unknown>;
  export default component;
}
