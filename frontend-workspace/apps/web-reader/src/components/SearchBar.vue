<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import { storeToRefs } from "pinia";
import { Search, FileText, X } from "lucide-vue-next";
import { useGraphStore } from "../store/useGraphStore";
import { api } from "../api";

const store = useGraphStore();
const { commandPaletteOpen } = storeToRefs(store);

const query = ref("");
const results = ref<Array<{ id: string; title: string; excerpt: string; rank: number }>>([]);
const loading = ref(false);
const selectedIndex = ref(0);
const inputRef = ref<HTMLInputElement>();

let debounceTimer: ReturnType<typeof setTimeout> | null = null;
let searchGeneration = 0;

watch(commandPaletteOpen, async (open) => {
    if (open) {
        query.value = "";
        results.value = [];
        selectedIndex.value = 0;
        await nextTick();
        inputRef.value?.focus();
    }
});

watch(query, (q) => {
    if (debounceTimer) clearTimeout(debounceTimer);

    if (!q.trim()) {
        results.value = [];
        return;
    }

    const gen = ++searchGeneration;
    debounceTimer = setTimeout(async () => {
        loading.value = true;
        try {
            const response = await api.searchCards(q, 8);
            if (gen !== searchGeneration) return; // 丢弃过期结果
            results.value = response.results;
            selectedIndex.value = 0;
        } catch (error) {
            if (gen !== searchGeneration) return;
            console.error("Search failed:", error);
            results.value = [];
        } finally {
            if (gen === searchGeneration) loading.value = false;
        }
    }, 300);
});

function selectCard(id: string) {
    store.selectNode(id);
    commandPaletteOpen.value = false;
}

function close() {
    commandPaletteOpen.value = false;
}

function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
        close();
        return;
    }
    if (e.key === "ArrowDown") {
        e.preventDefault();
        selectedIndex.value = Math.min(selectedIndex.value + 1, results.value.length - 1);
        return;
    }
    if (e.key === "ArrowUp") {
        e.preventDefault();
        selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
        return;
    }
    if (e.key === "Enter" && results.value[selectedIndex.value]) {
        e.preventDefault();
        selectCard(results.value[selectedIndex.value].id);
        return;
    }
}
</script>

<template>
    <Teleport to="body">
        <Transition name="ms-scale">
            <div v-if="commandPaletteOpen" class="fixed inset-0 z-modal flex items-start justify-center pt-[20vh]"
                @click="close" @keydown="onKeydown">
                <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" />

                <div class="relative w-full max-w-lg bg-ms-panel/95 backdrop-blur-xl border border-ms-border shadow-2xl shadow-black/50 overflow-hidden"
                    @click.stop>
                    <div class="flex items-center gap-3 px-5 py-4 border-b border-ms-border">
                        <Search :size="16" class="text-gray-500 shrink-0" />
                        <input
                            ref="inputRef"
                            v-model="query"
                            type="text"
                            placeholder="搜索卡片..."
                            class="flex-1 bg-transparent text-sm text-gray-200 placeholder-gray-600 focus:outline-none font-mono border-0 rounded-none focus:ring-1 focus:ring-neon/50"
                        />
                        <div v-if="loading" class="text-neon text-xs animate-pulse">
                            搜索中...
                        </div>
                        <kbd v-else class="text-2xs text-gray-600 bg-ms-carbon px-1.5 py-0.5 rounded border border-ms-border font-mono">
                            ESC
                        </kbd>
                    </div>

                    <div v-if="results.length > 0" class="max-h-[300px] overflow-y-auto py-2">
                        <button
                            v-for="(item, idx) in results"
                            :key="item.id"
                            class="w-full flex items-center gap-3 px-5 py-3 text-left transition-color group"
                            :class="idx === selectedIndex
                                ? 'bg-neon/10 text-neon'
                                : 'hover:bg-neon/5'"
                            @click="selectCard(item.id)"
                            @mouseenter="selectedIndex = idx"
                        >
                            <FileText
                                :size="14"
                                class="shrink-0 transition-colors"
                                :class="idx === selectedIndex ? 'text-neon' : 'text-gray-600 group-hover:text-neon'"
                            />
                            <div class="min-w-0">
                                <div
                                    class="text-sm truncate"
                                    :class="idx === selectedIndex ? 'text-neon' : 'text-gray-200'"
                                >
                                    {{ item.title }}
                                </div>
                                <div v-if="item.excerpt" class="text-1.5xs text-gray-600 truncate mt-0.5">
                                    {{ item.excerpt.slice(0, 100) }}
                                </div>
                            </div>
                        </button>
                    </div>

                    <div v-else-if="query && !loading" class="py-8 text-center">
                        <X :size="20" class="text-gray-600 mx-auto mb-2" />
                        <p class="text-xs text-gray-600">没有找到匹配的卡片</p>
                    </div>

                    <div v-else class="py-6 text-center">
                        <p class="text-xs text-gray-600 font-mono">输入关键词搜索知识库</p>
                    </div>

                    <div class="px-5 py-2 border-t border-ms-border text-2xs text-gray-600 flex gap-3 font-mono">
                        <span>↑↓ 导航</span>
                        <span>↵ 打开</span>
                        <span>Esc 关闭</span>
                    </div>
                </div>
            </div>
        </Transition>
    </Teleport>
</template>

<style scoped>
/* All transition styles removed - using shared transitions.css */
</style>
