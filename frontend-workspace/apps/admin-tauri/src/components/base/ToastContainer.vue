// 全局通知容器，固定在右下角，通过 useToast 管理通知列表

<script setup lang="ts">
import { useToast } from '@/composables/core/useToast'
import ToastItem from './ToastItem.vue'

const { toasts, dismiss } = useToast()
</script>

<template>
  <Teleport to="body">
    <div class="toast-container">
      <ToastItem v-for="toast in toasts" :key="toast.id" :toast="toast" @dismiss="dismiss" />
    </div>
  </Teleport>
</template>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 16px;
  right: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  z-index: var(--z-modal);
  pointer-events: none;
}

.toast-container>* {
  pointer-events: auto;
}
</style>
