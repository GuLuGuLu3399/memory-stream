/**
 * @module useToast
 *
 * Toast 通知系统 — 独立的 Pinia Store
 */

import { defineStore } from "pinia";
import { ref } from "vue";

/** Toast 通知消息 */
export interface ToastMessage {
  id: number;
  text: string;
  type: "success" | "error" | "info";
}

export const useToast = defineStore("toast", () => {
  // ---- 响应式状态 ----
  const toasts = ref<ToastMessage[]>([]);
  let toastCounter = 0;

  // ---- Toast 通知 ----

  /** 添加一条 Toast 通知，3 秒后自动消失 */
  function addToast(text: string, type: "success" | "error" | "info" = "info") {
    const id = ++toastCounter;
    toasts.value.push({ id, text, type });
    setTimeout(() => {
      toasts.value = toasts.value.filter((t) => t.id !== id);
    }, 3000);
  }

  return {
    toasts,
    addToast,
  };
});
