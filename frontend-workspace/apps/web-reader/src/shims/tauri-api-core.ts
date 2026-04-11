// Shim for @tauri-apps/api/core — web-reader does not run in Tauri.
// MarkdownEditor in ui-shared conditionally imports this; the runtime guard
// (`'__TAURI__' in window`) prevents actual invocation in non-Tauri contexts.
export const invoke = async () => {
  throw new Error("@tauri-apps/api/core is not available outside Tauri");
};
