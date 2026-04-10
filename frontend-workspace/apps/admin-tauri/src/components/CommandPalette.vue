<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import { useKnowledgeStore } from "../stores/knowledge";
import { useLayoutStore } from "../stores/layout";
import { storeToRefs } from "pinia";

const store = useKnowledgeStore();
const layoutStore = useLayoutStore();
const { recentCards, orphanCards } = storeToRefs(store);

const isOpen = ref(false);
const query = ref("");
const selectedIndex = ref(0);
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

function open() {
    isOpen.value = true;
    query.value = "";
    selectedIndex.value = 0;
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

    const maxIndex = displayItems.value.length - 1;

    if (e.key === "ArrowDown") {
        e.preventDefault();
        selectedIndex.value = Math.min(selectedIndex.value + 1, maxIndex);
        return;
    }

    if (e.key === "ArrowUp") {
        e.preventDefault();
        selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
        return;
    }

    if (e.key === "Enter" && displayItems.value[selectedIndex.value]) {
        e.preventDefault();
        handleSelect();
        return;
    }
}

watch(query, () => {
    selectedIndex.value = 0;
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
                <div class="absolute inset-0 bg-black/40 backdrop-blur-sm" @click="close"></div>

                <div role="combobox" :aria-expanded="isOpen" aria-haspopup="listbox"
                    class="relative w-full max-w-lg bg-ms-carbon shadow-2xl border border-ms-border overflow-hidden">
                    <div class="flex items-center px-4 border-b border-ms-border">
                        <svg v-if="!isCommandMode" class="w-4 h-4 text-slate-500 shrink-0" fill="none" viewBox="0 0 24 24"
                            stroke="currentColor" aria-hidden="true">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                        </svg>
                        <span v-else class="text-amber-400 mr-2 font-mono text-sm select-none">&gt;</span>
                        <input ref="inputRef" v-model="query" :placeholder="isCommandMode ? 'command...' : '搜索卡片...'"
                            role="searchbox"
                            :aria-activedescendant="isOpen && displayItems.length > 0 ? `palette-option-${displayItems[selectedIndex]?.id ?? selectedIndex}` : undefined"
                            class="flex-1 px-3 py-3 text-sm outline-none bg-transparent text-slate-200 placeholder-slate-600 font-mono" />
                        <kbd
                            class="text-2xs bg-ms-surface text-slate-500 px-1.5 py-0.5 rounded border border-ms-border">ESC</kbd>
                    </div>

                    <!-- Command Mode -->
                    <div v-if="isCommandMode" class="max-h-72 overflow-y-auto" role="listbox">
                        <button v-for="(cmd, idx) in filteredCommands" :key="cmd.id" @click="selectCommand(cmd.id)"
                            @mouseenter="selectedIndex = idx"
                            role="option" :id="`palette-option-${cmd.id}`"
                            :aria-selected="idx === selectedIndex"
                            class="w-full text-left px-4 py-2.5 text-sm flex items-center gap-3 transition"
                            :class="idx === selectedIndex
                                ? cmd.danger ? 'bg-amber-500/10 text-amber-400' : 'bg-neon/10 text-neon'
                                : 'text-slate-400 hover:bg-ms-surface'">
                            <span class="font-mono text-xs">&gt;</span>
                            <span class="font-mono">{{ cmd.label }}</span>
                            <span class="text-2xs text-slate-600 ml-auto">{{ cmd.description }}</span>
                        </button>
                        <div v-if="filteredCommands.length === 0" class="px-4 py-6 text-center text-sm text-slate-500 font-mono">
                            No command found
                        </div>
                    </div>

                    <!-- Card Search Mode -->
                    <div v-else class="max-h-72 overflow-y-auto" role="listbox">
                        <button v-for="(card, idx) in allCards" :key="card.id" @click="selectCard(card.id)"
                            @mouseenter="selectedIndex = idx"
                            role="option" :id="`palette-option-${card.id}`"
                            :aria-selected="idx === selectedIndex"
                            class="w-full text-left px-4 py-2.5 text-sm flex items-center gap-3 transition" :class="idx === selectedIndex
                                ? 'bg-neon/10 text-neon'
                                : 'text-slate-400 hover:bg-ms-surface'">
                            <svg class="w-4 h-4 shrink-0"
                                :class="idx === selectedIndex ? 'text-neon' : 'text-slate-600'" fill="none"
                                viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                    d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                            </svg>
                            <span class="truncate font-mono text-xs">{{ card.title || "无标题" }}</span>
                        </button>
                        <div v-if="allCards.length === 0" class="px-4 py-6 text-center text-sm text-slate-500 font-mono">
                            未找到匹配的卡片
                        </div>
                    </div>

                    <div class="px-4 py-2 border-t border-ms-border text-2xs text-slate-600 flex gap-3 font-mono">
                        <span>↑↓ 导航</span>
                        <span>↵ 确认</span>
                        <span>Esc 关闭</span>
                        <span class="ml-auto text-amber-500/60">&gt; 命令模式</span>
                    </div>
                </div>
            </div>
        </Transition>
    </Teleport>
</template>

<style scoped>
/* All transition styles removed - using shared transitions.css */
</style>
