// 用途：Toast 通知组合式函数，管理通知队列的创建和自动销毁
import { ref } from 'vue'

export type ToastType = 'success' | 'warning' | 'error'

export interface Toast {
  id: number
  message: string
  type: ToastType
  duration: number
}

let nextId = 0

const toasts = ref<Toast[]>([])

export function useToast() {
  function show(message: string, type: ToastType = 'success', duration = 3000) {
    const id = nextId++
    toasts.value.push({ id, message, type, duration })

    if (duration > 0) {
      setTimeout(() => dismiss(id), duration)
    }
  }

  function dismiss(id: number) {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }

  function success(message: string) { show(message, 'success') }
  function warning(message: string) { show(message, 'warning') }
  function error(message: string) { show(message, 'error', 5000) }

  return { toasts, show, dismiss, success, warning, error }
}
