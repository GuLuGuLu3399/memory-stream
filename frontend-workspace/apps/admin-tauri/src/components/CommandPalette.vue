<script setup lang="ts">
/**
 * CommandPalette — 机械祭坛命令台
 *
 * 机械祭坛主题设计:
 * - 黄铜铆钉装饰（四个角）
 * - 选中项霓虹辉 + 黄铜边框组合
 * - 命令模式动画黄铜齿轮指示器
 * - 空开显示最近卡片
 */
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import { useKnowledgeStore } from "../stores/knowledge";
import { useLayoutStore } from "../stores/layout";
import { storeToRefs } from "pinia";
import { Settings, FileText, RotateCw, Search } from "lucide-vue-next";
import { useKeyboardListNavigation } from "@memory-stream/ui-shared";

const store = useKnowledgeStore();
const layoutStore = useLayoutStore();
const { recentCards, orphanCards } = storeToRefs(store);

const isOpen = ref(false);
const query = ref("");
const inputRef = ref<HTMLInputElement | null>(null);

const isCommandMode = computed(() => query.value.startsWith(">"));

const commandItems = computed(() => {
    if (!isCommandMode.value) return [];
    return [
        {
            id: "merge",
            label: "merge",
            description: "概念坍缩引擎 — 打开合并控制台",
            danger: true,
        },
    ];
});

const filteredCommands = computed(() => {
    if (!isCommandMode.value) return [];
    const q = query.value.slice(1).toLowerCase().trim();
    if (!q) return commandItems.value;
    return commandItems.value.filter((c) => c.label.toLowerCase().includes(q));
});

const allCards = computed(() => {
    const seen = new Set<string>();
    const result = [...recentCards.value, ...orphanCards.value];
    const unique = result.filter((c) => {
        if (seen.has(c.id)) return false;
        seen.add(c.id);
        return true;
    });
    if (!query.value || isCommandMode.value) return unique.slice(0, 15);
    const q = query.value.toLowerCase();
    return unique.filter((c) => c.title.toLowerCase().includes(q)).slice(0, 15);
});

const displayItems = computed(() => {
    if (isCommandMode.value) {
        return filteredCommands.value;
    }
    return allCards.value;
});

const itemCount = computed(() => displayItems.value.length);

// Keyboard navigation using shared composable
const { selectedIndex, handleKeydown: handleNavKeydown, reset: resetSelection, setIndex } = useKeyboardListNavigation(
    itemCount,
    () => handleSelect(),
    { wrap: true, initialIndex: 0 }
);

function open() {
    isOpen.value = true;
    query.value = "";
    resetSelection();
    nextTick(() => inputRef.value?.focus());
}

function close() {
    isOpen.value = false;
    query.value = "";
}

function selectCard(cardId: string) {
    store.loadAndActivateCard(cardId);
    close();
}

function selectCommand(commandId: string) {
    if (commandId === "merge") {
        layoutStore.openMergeConsole();
        close();
    }
}

function handleSelect() {
    if (isCommandMode.value) {
        const cmd = filteredCommands.value[selectedIndex.value];
        if (cmd) selectCommand(cmd.id);
    } else {
        const card = allCards.value[selectedIndex.value];
        if (card) selectCard(card.id);
    }
}

function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        if (isOpen.value) {
            close();
        } else {
            open();
        }
        return;
    }

    if (!isOpen.value) return;

    if (e.key === "Escape") {
        close();
        return;
    }

    // Delegate arrow/enter navigation to useKeyboardListNavigation
    const handled = handleNavKeydown(e);
    if (handled) return;
}

watch(query, () => {
    resetSelection();
});

onMounted(() => {
    window.addEventListener("keydown", handleKeydown);
});
onUnmounted(() => {
    window.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
    <Teleport to="body">
        <Transition name="ms-scale">
            <div v-if="isOpen" class="fixed inset-x-0 bottom-0 top-titlebar z-modal flex items-start justify-center pt-[20vh]"
                @click.self="close">
                <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" @click="close"></div>

                <div role="combobox" :aria-expanded="isOpen" aria-haspopup="listbox"
                    class="relative w-full max-w-lg bg-ms-panel overflow-hidden"
                    style="box-shadow: inset 0 1px 0 0 rgba(255,255,255,0.04), 4px 4px 0 0 rgba(0,0,0,0.6), 0 0 40px rgba(0,229,255,0.05);">
                    <!-- Brass rivet decorations at corners -->
                    <div class="absolute top-0 left-0 w-3 h-3 border-l-2 border-t-2 border-brass/60 rounded-tl-sm" />
                    <div class="absolute top-0 right-0 w-3 h-3 border-r-2 border-t-2 border-brass/60 rounded-tr-sm" />
                    <div class="absolute bottom-0 left-0 w-3 h-3 border-l-2 border-b-2 border-brass/60 rounded-bl-sm" />
                    <div class="absolute bottom-0 right-0 w-3 h-3 border-r-2 border-b-2 border-brass/60 rounded-br-sm" />

                    <!-- Header -->
                    <div class="flex items-center px-4 border-b border-ms-border bg-ms-deep">
                        <Search v-if="!isCommandMode" :size="16" class="text-slate-500 shrink-0" />
                        <RotateCw v-else :size="16" class="text-brass shrink-0 animate-spin-slow" />
                        <input ref="inputRef" v-model="query" :placeholder="isCommandMode ? 'command...' : '搜索卡片...'"
                            role="searchbox"
                            class="flex-1 px-3 py-3 text-sm outline-none bg-transparent text-slate-200 placeholder-slate-600 font-mono" />
                        <kbd class="text-2xs bg-ms-surface text-slate-500 px-1.5 py-0.5 border border-ms-border shadow-[1px_1px_0_0_rgba(0,0,0,0.4)]">ESC</kbd>
                    </div>

                    <!-- Command Mode -->
                    <div v-if="isCommandMode" class="max-h-72 overflow-y-auto" role="listbox">
                        <button v-for="(cmd, idx) in filteredCommands" :key="cmd.id" @click="selectCommand(cmd.id)"
                            @mouseenter="setIndex(idx)"
                            role="option" :id="`palette-option-${cmd.id}`"
                            :aria-selected="idx === selectedIndex"
                            class="w-full text-left px-4 py-2.5 text-sm flex items-center gap-3 transition relative"
                            :class="idx === selectedIndex
                                ? cmd.danger ? 'bg-brass/10 text-brass border-l-2 border-brass' : 'bg-neon/10 text-neon border-l-2 border-neon'
                                : 'text-slate-400 hover:bg-ms-surface border-l-2 border-transparent'">
                            <Settings :size="14" class="shrink-0"
                                :class="idx === selectedIndex
                                    ? (cmd.danger ? 'text-brass' : 'text-neon')
                                    : 'text-slate-600'" />
                            <span class="font-mono">{{ cmd.label }}</span>
                            <span class="text-2xs text-slate-600 ml-auto">{{ cmd.description }}</span>
                            <!-- Neon glow for selected -->
                            <div v-if="idx === selectedIndex && !cmd.danger"
                                class="absolute inset-0 pointer-events-none"
                                style="box-shadow: inset 0 1px 0 0 rgba(255,255,255,0.04), inset 0 -1px 0 0 rgba(0,0,0,0.1);" />
                        </button>
                        <div v-if="filteredCommands.length === 0" class="px-4 py-6 text-center text-sm text-slate-500 font-mono">
                            No command found
                        </div>
                    </div>

                    <!-- Card Search Mode / Recent Cards -->
                    <div v-else class="max-h-72 overflow-y-auto" role="listbox">
                        <button v-for="(card, idx) in allCards" :key="card.id" @click="selectCard(card.id)"
                            @mouseenter="setIndex(idx)"
                            role="option" :id="`palette-option-${card.id}`"
                            :aria-selected="idx === selectedIndex"
                            class="w-full text-left px-4 py-2.5 text-sm flex items-center gap-3 transition relative border-l-2"
                            :class="idx === selectedIndex
                                ? 'bg-neon/10 text-neon border-neon'
                                : 'text-slate-400 hover:bg-ms-surface border-transparent'">
                            <FileText :size="16" class="shrink-0"
                                :class="idx === selectedIndex ? 'text-neon' : 'text-slate-600'" />
                            <span class="truncate font-mono text-xs">{{ card.title || "无标题" }}</span>
                            <!-- Neon glow for selected -->
                            <div v-if="idx === selectedIndex"
                                class="absolute inset-0 pointer-events-none"
                                style="box-shadow: inset 0 1px 0 0 rgba(255,255,255,0.04), inset 0 -1px 0 0 rgba(0,0,0,0.1);" />
                        </button>
                        <div v-if="allCards.length === 0" class="px-4 py-6 text-center text-sm text-slate-500 font-mono">
                            未找到匹配的卡片
                        </div>
                    </div>

                    <!-- Footer -->
                    <div class="px-4 py-2 border-t border-ms-border text-2xs text-slate-600 flex gap-3 font-mono bg-ms-deep/50">
                        <span>↑↓ 导航</span>
                        <span>↵ 确认</span>
                        <span>Esc 关闭</span>
                        <span class="ml-auto text-brass/60">&gt; 命令模式</span>
                    </div>
                </div>
            </div>
        </Transition>
    </Teleport>
</template>

<style scoped>
/* All transition styles removed - using shared transitions.css */
</style>
