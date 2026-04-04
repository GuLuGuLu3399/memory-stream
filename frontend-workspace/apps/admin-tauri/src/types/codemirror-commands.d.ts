declare module '@codemirror/commands' {
  // Minimal ambient declarations to satisfy TS in environments
  // where the package typings are not available.
  export const defaultKeymap: any;
  export const history: any;
  export const historyKeymap: any;
}
