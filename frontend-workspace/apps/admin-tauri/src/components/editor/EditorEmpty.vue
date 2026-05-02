// 用途：编辑器空状态视图，显示快捷键提示和品牌水印
<script setup lang="ts">
import { useTreeStore } from '@/stores/tree'

const emit = defineEmits<{ openPalette: [] }>()
const treeStore = useTreeStore()
</script>

<template>
    <div class="editor-empty">
        <div class="zero-state-grid" />
        <div class="zero-state-content">
            <div class="zero-watermark">
                <div class="watermark-frame">
                    <div class="watermark-lines">
                        <div class="watermark-line" />
                        <div class="watermark-line" />
                        <div class="watermark-line" />
                    </div>
                    <span class="watermark-text">MS</span>
                </div>
            </div>
            <div class="zero-shortcuts">
                <button class="keycap-group" @click="treeStore.setActive(null)">
                    <kbd class="keycap">Ctrl</kbd>
                    <kbd class="keycap">N</kbd>
                    <span class="keycap-label">新建卡片</span>
                </button>
                <button class="keycap-group" @click="emit('openPalette')">
                    <kbd class="keycap">Ctrl</kbd>
                    <kbd class="keycap">P</kbd>
                    <span class="keycap-label">打开卡片</span>
                </button>
                <button class="keycap-group" disabled>
                    <kbd class="keycap">Ctrl</kbd>
                    <kbd class="keycap">G</kbd>
                    <span class="keycap-label">局部图谱</span>
                </button>
            </div>
        </div>
    </div>
</template>

<style scoped>
.editor-empty {
    display: flex;
    flex: 1;
    align-items: center;
    justify-content: center;
    background: var(--ms-deep);
    position: relative;
    overflow: hidden;
}

.zero-state-grid {
    position: absolute;
    inset: 0;
    background:
        repeating-linear-gradient(0deg, var(--ms-border) 0 1px, transparent 1px 40px),
        repeating-linear-gradient(90deg, var(--ms-border) 0 1px, transparent 1px 40px);
    opacity: 0.08;
    pointer-events: none;
}

.zero-state-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 40px;
    z-index: 1;
}

.zero-watermark {
    opacity: 0.04;
}

.watermark-frame {
    width: 80px;
    height: 80px;
    border-radius: 8px;
    border: 1px solid var(--ms-smoke);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    box-shadow:
        inset 0 1px 2px rgba(0, 0, 0, 0.3),
        inset 0 -1px 1px rgba(255, 255, 255, 0.02);
}

.watermark-lines {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.watermark-line {
    width: 32px;
    height: 2px;
    background: var(--ms-smoke);
}

.watermark-text {
    font-family: var(--font-sans);
    font-size: 14px;
    font-weight: 600;
    color: var(--ms-smoke);
    letter-spacing: 0.1em;
}

.zero-shortcuts {
    display: flex;
    gap: 24px;
}

.keycap-group {
    display: flex;
    align-items: center;
    gap: 6px;
    border: none;
    background: transparent;
    cursor: pointer;
    padding: 0;
    transition: transform 150ms var(--ease-snap);
}

.keycap-group:hover:not(:disabled) {
    transform: translateY(-1px);
}

.keycap-group:disabled {
    cursor: default;
    opacity: 0.5;
}

.keycap {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 2px 8px;
    background: var(--ms-surface);
    border-radius: 4px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--ms-smoke);
    box-shadow:
        0 2px 0 var(--ms-border),
        0 1px 3px rgba(0, 0, 0, 0.3);
    transition: box-shadow 150ms var(--ease-snap),
        transform 150ms var(--ease-snap);
}

.keycap-group:hover:not(:disabled) .keycap {
    box-shadow:
        0 1px 0 var(--ms-border),
        0 0 2px rgba(0, 0, 0, 0.2);
    transform: translateY(1px);
}

.keycap-label {
    font-family: var(--font-sans);
    font-size: 11px;
    color: var(--text-muted);
    margin-left: 4px;
}
</style>
