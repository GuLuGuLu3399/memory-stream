/**
 * Promise-based Confirm Dialog composable
 *
 * 替代浏览器原生 confirm()，提供与深炭灰主题一致的模态对话框。
 *
 * 用法：
 *   const { confirm } = useConfirmDialog();
 *   const ok = await confirm('确定删除？', { danger: true });
 */
import { ref, readonly } from "vue";

export interface ConfirmOptions {
  title?: string;
  confirmText?: string;
  cancelText?: string;
  danger?: boolean;
}

interface DialogState {
  visible: boolean;
  message: string;
  title: string;
  confirmText: string;
  cancelText: string;
  danger: boolean;
  resolve: ((value: boolean) => void) | null;
}

const state = ref<DialogState>({
  visible: false,
  message: "",
  title: "确认操作",
  confirmText: "确认",
  cancelText: "取消",
  danger: false,
  resolve: null,
});

export function useConfirmDialog() {
  function confirm(
    message: string,
    options?: ConfirmOptions,
  ): Promise<boolean> {
    return new Promise((resolve) => {
      state.value = {
        visible: true,
        message,
        title: options?.title ?? "确认操作",
        confirmText: options?.confirmText ?? "确认",
        cancelText: options?.cancelText ?? "取消",
        danger: options?.danger ?? false,
        resolve,
      };
    });
  }

  function handleConfirm() {
    state.value.resolve?.(true);
    state.value.visible = false;
  }

  function handleCancel() {
    state.value.resolve?.(false);
    state.value.visible = false;
  }

  return {
    dialogState: readonly(state),
    confirm,
    handleConfirm,
    handleCancel,
  };
}
