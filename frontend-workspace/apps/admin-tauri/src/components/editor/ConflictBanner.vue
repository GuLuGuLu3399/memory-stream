// 用途：同步冲突横幅，提供保留本地或载入云端两种解决方案
<script setup lang="ts">
defineProps<{ disabled?: boolean }>()
defineEmits<{ keepLocal: []; keepRemote: [] }>()
</script>

<template>
    <div class="conflict-banner">
        <span class="conflict-text">数据冲突：云端版本与本地编辑发生碰撞</span>
        <button class="conflict-btn keep-local-btn" :disabled="disabled" @click="$emit('keepLocal')">覆盖云端 (Keep Local)</button>
        <button class="conflict-btn keep-remote-btn" :disabled="disabled" @click="$emit('keepRemote')">载入云端 (Keep Remote)</button>
    </div>
</template>

<style scoped>
.conflict-banner {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 16px;
    background: oklch(0.18 0.06 20);
    border-bottom: 1px solid var(--destructive);
    font-size: 0.85em;
}

.conflict-text {
    color: oklch(0.7 0.12 20);
    flex: 1;
    font-weight: 500;
}

.conflict-btn {
    padding: 4px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.82em;
    font-weight: 500;
    transition: background var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      opacity var(--duration-fast) var(--ease-out);
}

.conflict-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
}

.keep-local-btn {
    border: 1px solid var(--destructive);
    background: var(--destructive);
    color: var(--ms-void);
}

.keep-local-btn:hover:not(:disabled) {
    background: oklch(0.58 0.18 20);
    border-color: oklch(0.58 0.18 20);
}

.keep-remote-btn {
    border: 1px solid var(--ms-border-light);
    background: transparent;
    color: var(--text-secondary);
}

.keep-remote-btn:hover:not(:disabled) {
    border-color: var(--neon);
    color: var(--neon);
}
</style>
