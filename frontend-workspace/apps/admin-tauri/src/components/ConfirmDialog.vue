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
    <Transition name="ms-scale">
        <div v-if="dialogState.visible" ref="rootEl" role="dialog" aria-modal="true" aria-labelledby="dialog-title"
            aria-describedby="dialog-message" class="fixed inset-x-0 bottom-0 top-titlebar z-modal flex items-center justify-center"
            @keydown="onKeydown" tabindex="-1">
            <!-- Backdrop -->
            <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" aria-hidden="true" @click="handleCancel" />

            <!-- Dialog Card -->
            <div class="relative z-10 w-[360px] bg-ms-carbon border border-ms-border rounded-sm shadow-2xl p-6">
                <!-- Title -->
                <h3 id="dialog-title" class="text-sm font-mono text-slate-400 uppercase tracking-wider mb-3">
                    {{ dialogState.title }}
                </h3>

                <!-- Message -->
                <p id="dialog-message" class="text-slate-200 text-sm leading-relaxed mb-6">
                    {{ dialogState.message }}
                </p>

                <!-- Actions -->
                <div class="flex justify-end gap-2">
                    <button @click="handleCancel"
                        class="px-4 py-2 text-xs rounded-sm bg-ms-surface text-slate-400 hover:text-slate-200 hover:bg-ms-border transition-all border border-ms-border">
                        {{ dialogState.cancelText }}
                    </button>
                    <button data-action="confirm" @click="handleConfirm"
                        class="px-4 py-2 text-xs rounded-sm font-medium transition-all" :class="dialogState.danger
                            ? 'bg-red-500/20 text-red-400 border border-red-500/30 hover:bg-red-500/30'
                            : 'bg-neon-600 text-ms-deep hover:bg-neon shadow-sm'
                            ">
                        {{ dialogState.confirmText }}
                    </button>
                </div>
            </div>
        </div>
    </Transition>
</template>
