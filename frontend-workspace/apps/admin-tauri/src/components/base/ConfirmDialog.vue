<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import { useConfirmDialog } from "../composables/useConfirmDialog";

const { dialogState, handleConfirm, handleCancel } = useConfirmDialog();
const rootEl = ref<HTMLElement | null>(null);
const previouslyFocused = ref<HTMLElement | null>(null);

// 焦点管理：打开时聚焦，关闭时还原
watch(() => dialogState.value.visible, async (visible) => {
    if (visible) {
        previouslyFocused.value = document.activeElement as HTMLElement;
        await nextTick();
        const confirmBtn = rootEl.value?.querySelector<HTMLButtonElement>('[data-action="confirm"]');
        confirmBtn?.focus();
    } else {
        previouslyFocused.value?.focus();
        previouslyFocused.value = null;
    }
});

function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") handleCancel();
    if (e.key === "Enter") handleConfirm();

    // 焦点陷阱：Tab 循环在 dialog 内
    if (e.key === "Tab") {
        const focusable = rootEl.value?.querySelectorAll<HTMLElement>(
            'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
        );
        if (!focusable || focusable.length === 0) return;
        const first = focusable[0];
        const last = focusable[focusable.length - 1];
        if (e.shiftKey && document.activeElement === first) {
            e.preventDefault();
            last.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
            e.preventDefault();
            first.focus();
        }
    }
}
</script>

<template>
    <Teleport to="body">
        <Transition name="ms-scale">
            <div v-if="dialogState.visible" ref="rootEl" role="dialog" aria-modal="true" aria-labelledby="dialog-title"
                aria-describedby="dialog-message" class="fixed inset-x-0 bottom-0 top-titlebar z-modal flex items-center justify-center"
                @keydown="onKeydown" tabindex="-1">
                <!-- Backdrop -->
                <div class="absolute inset-0 bg-gradient-to-b from-black/80 via-black/70 to-black/80 backdrop-blur-md"
                    aria-hidden="true" @click="handleCancel" />

                <!-- Dialog Card -->
                <div class="relative z-10 w-[360px] bg-ms-carbon border border-brass/40 rounded-sm dialog-card"
                    style="box-shadow: inset 0 1px 0 0 rgba(255,255,255,0.04), 4px 4px 0 0 rgba(0,0,0,0.6);">
                    <!-- Brass Corner Accents -->
                    <span class="corner-accent corner-tl" />
                    <span class="corner-accent corner-tr" />
                    <span class="corner-accent corner-bl" />
                    <span class="corner-accent corner-br" />

                    <div class="p-6">
                        <!-- Title -->
                        <h3 id="dialog-title" class="text-sm font-mono text-neon uppercase tracking-wider mb-3">
                            {{ dialogState.title }}
                        </h3>

                        <!-- Message -->
                        <p id="dialog-message" class="text-slate-300 text-sm leading-relaxed mb-6">
                            {{ dialogState.message }}
                        </p>

                        <!-- Actions -->
                        <div class="flex justify-end gap-2">
                            <button @click="handleCancel"
                                class="px-4 py-2 text-xs rounded-sm bg-ms-surface text-slate-500 hover:text-white hover:bg-ms-border transition-all border border-ms-border shadow-[1px_1px_0_0_rgba(0,0,0,0.4)] hover:shadow-[2px_2px_0_0_rgba(0,0,0,0.4)] hover:translate-x-[-1px] hover:translate-y-[-1px] active:translate-x-[1px] active:translate-y-[1px] active:shadow-none">
                                {{ dialogState.cancelText }}
                            </button>
                            <button data-action="confirm" @click="handleConfirm"
                                class="px-4 py-2 text-xs rounded-sm font-medium transition-all border relative overflow-hidden shadow-[2px_2px_0_0_rgba(0,0,0,0.5)] hover:shadow-[3px_3px_0_0_rgba(0,0,0,0.5)] hover:translate-x-[-1px] hover:translate-y-[-1px] active:translate-x-[1px] active:translate-y-[1px] active:shadow-none" :class="dialogState.danger
                                    ? 'bg-red-500/20 text-red-400 border-red-500/30 hover:bg-red-500/30 danger-pulse'
                                    : 'bg-neon/20 text-neon border-neon/30 hover:bg-neon/30'
                                    ">
                                {{ dialogState.confirmText }}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </Transition>
    </Teleport>
</template>

<style scoped>
.dialog-card {
    position: relative;
}

/* 黄铜角饰 */
.corner-accent {
    position: absolute;
    width: 10px;
    height: 10px;
    border-color: theme('colors.brass.DEFAULT');
    border-style: solid;
    pointer-events: none;
}

.corner-tl {
    top: -1px;
    left: -1px;
    border-width: 2px 0 0 2px;
}

.corner-tr {
    top: -1px;
    right: -1px;
    border-width: 2px 2px 0 0;
}

.corner-bl {
    bottom: -1px;
    left: -1px;
    border-width: 0 0 2px 2px;
}

.corner-br {
    bottom: -1px;
    right: -1px;
    border-width: 0 2px 2px 0;
}

/* 危险模式红脉 — hard entity shadow */
@keyframes dangerPulse {
    0%, 100% {
        box-shadow: 2px 2px 0 0 rgba(0,0,0,0.5), inset 0 1px 0 0 rgba(255,255,255,0.04);
    }
    50% {
        box-shadow: 2px 2px 0 0 rgba(0,0,0,0.5), 0 0 8px 2px rgba(239, 68, 68, 0.3);
    }
}

.danger-pulse {
    animation: dangerPulse 2s ease-in-out infinite;
}

/* 过渡动画 */
.ms-scale-enter-active,
.ms-scale-leave-active {
    transition: all 200ms cubic-bezier(0.16, 1, 0.3, 1);
}

.ms-scale-enter-from,
.ms-scale-leave-to {
    opacity: 0;
    transform: scale(0.95);
}

.ms-scale-enter-to,
.ms-scale-leave-from {
    opacity: 1;
    transform: scale(1);
}
</style>
