<script setup lang="ts">
/**
 * SearchBar — 铜镜搜索（血肉神殿）
 *
 * 血肉神殿主题设计:
 * - 铜丝花边边框
 * - 搜索结果左侧边框标识关系类型（雪珀=序列, ms-copper=引用）
 * - 选中项烛光辉
 * - 骨白文字, ms-xiang 结果卡片
 */
import { ref, computed, watch, nextTick } from "vue";
import { storeToRefs } from "pinia";
import { Search, FileText, X, Terminal } from "lucide-vue-next";
import { useGraphStore } from "../store/useGraphStore";
import { useKeyboardListNavigation } from "@memory-stream/ui-shared";
import { api } from "../api";

const store = useGraphStore();
const { commandPaletteOpen } = storeToRefs(store);

const query = ref("");
const results = ref<Array<{
    id: string;
    title: string;
    excerpt: string;
    rank: number;
    relationType?: "sequence" | "reference";
}>>([]);
const loading = ref(false);
const inputRef = ref<HTMLInputElement>();

let debounceTimer: ReturnType<typeof setTimeout> | null = null;
let searchGeneration = 0;

// Command mode detection
const isCommandMode = computed(() => query.value.startsWith(">"));

// Result count for navigation
const resultCount = computed(() => results.value.length);

// Keyboard navigation using shared composable
const { selectedIndex, handleKeydown: handleNavKeydown, reset: resetSelection } = useKeyboardListNavigation(
    resultCount,
    (index) => {
        const item = results.value[index];
        if (item) selectCard(item.id);
    },
    { wrap: true, initialIndex: 0 }
);

watch(commandPaletteOpen, async (open) => {
    if (open) {
        query.value = "";
        results.value = [];
        resetSelection();
        await nextTick();
        inputRef.value?.focus();
    }
});

watch(query, (q) => {
    if (debounceTimer) clearTimeout(debounceTimer);

    if (!q.trim() || isCommandMode.value) {
        results.value = [];
        return;
    }

    const gen = ++searchGeneration;
    debounceTimer = setTimeout(async () => {
        loading.value = true;
        try {
            const response = await api.searchCards(q, 8);
            if (gen !== searchGeneration) return;
            results.value = response.results.map(r => ({
                ...r,
                relationType: r.relationType,
            }));
            resetSelection();
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
    // Delegate arrow/enter navigation to useKeyboardListNavigation
    const handled = handleNavKeydown(e);
    if (handled) return;
}
</script>

<template>
    <Teleport to="body">
        <Transition name="ms-scale">
            <div v-if="commandPaletteOpen" class="fixed inset-0 z-modal flex items-start justify-center pt-[20vh]"
                @click="close" @keydown="onKeydown">
                <div class="absolute inset-0 bg-ms-xuan/70 backdrop-blur-sm" />

                <div class="relative w-full max-w-lg bg-ms-xiang overflow-hidden search-panel"
                    @click.stop>
                    <!-- Copper filigree border -->
                    <div class="absolute inset-0 border-[3px] border-ms-copper/40 pointer-events-none" />
                    <div class="absolute inset-0 border border-ms-copper/20 pointer-events-none" />

                    <!-- Header -->
                    <div class="flex items-center gap-3 px-5 py-4 border-b border-ms-copper/30 bg-ms-xiang">
                        <Search :size="16" class="text-ms-ash shrink-0" />
                        <input
                            ref="inputRef"
                            v-model="query"
                            type="text"
                            :placeholder="isCommandMode ? '输入命令...' : '搜索经文...'"
                            class="flex-1 bg-transparent text-sm text-ms-bone placeholder-ms-ash focus:outline-none font-serif border-0"
                        />
                        <Terminal v-if="isCommandMode" :size="14" class="text-ms-gold shrink-0" />
                        <div v-else-if="loading" class="text-xuepo text-xs animate-pulse">
                            搜索中...
                        </div>
                        <kbd v-else class="text-2xs text-ms-ash bg-ms-mo px-1.5 py-0.5 border border-ms-copper/30 shadow-[1px_1px_0_0_rgba(0,0,0,0.4)] font-mono">
                            ESC
                        </kbd>
                    </div>

                    <!-- Results -->
                    <div v-if="results.length > 0" class="max-h-dropdown overflow-y-auto py-2">
                        <button
                            v-for="(item, idx) in results"
                            :key="item.id"
                            class="w-full flex items-center gap-3 px-5 py-3 text-left transition-all duration-150 group relative"
                            :class="idx === selectedIndex
                                ? 'bg-ms-copper/20 search-result--selected'
                                : 'hover:bg-ms-mo/50'"
                            @click="selectCard(item.id)"
                            @mouseenter="selectedIndex = idx"
                        >
                            <!-- Relation type border -->
                            <div class="absolute left-0 top-2 bottom-2 w-0.5 rounded-r"
                                :class="idx === selectedIndex
                                    ? (item.relationType === 'sequence' ? 'bg-xuepo' : 'bg-ms-copper')
                                    : (item.relationType === 'sequence' ? 'bg-xuepo/40' : 'bg-ms-copper/40')" />
                            <div class="absolute left-0 top-0 bottom-0 w-1"
                                :class="idx === selectedIndex
                                    ? (item.relationType === 'sequence' ? 'bg-xuepo' : 'bg-ms-copper')
                                    : 'bg-transparent'" />

                            <FileText
                                :size="14"
                                class="shrink-0 transition-colors ml-1"
                                :class="idx === selectedIndex ? 'text-ms-gold' : 'text-ms-ash group-hover:text-ms-bone-dim'"
                            />
                            <div class="min-w-0">
                                <div
                                    class="text-sm truncate font-serif"
                                    :class="idx === selectedIndex ? 'text-ms-bone' : 'text-ms-bone-dim'"
                                >
                                    {{ item.title }}
                                </div>
                                <div v-if="item.excerpt" class="text-1.5xs text-ms-smoke truncate mt-0.5">
                                    {{ item.excerpt.slice(0, 80) }}
                                </div>
                            </div>
                        </button>
                    </div>

                    <!-- Empty states -->
                    <div v-else-if="query && !loading && !isCommandMode" class="py-8 text-center">
                        <X :size="20" class="text-ms-ash mx-auto mb-2" />
                        <p class="text-xs text-ms-smoke">没有找到匹配的卡片</p>
                    </div>

                    <div v-else class="py-6 text-center">
                        <p class="text-xs text-ms-ash font-serif">输入关键词搜索知识库</p>
                        <p v-if="!isCommandMode" class="text-2xs text-ms-ash/60 font-mono mt-1">输入 &gt; 进入命令模式</p>
                    </div>

                    <!-- Footer -->
                    <div class="px-5 py-2 border-t border-ms-copper/30 text-2xs text-ms-ash flex gap-3 font-mono bg-ms-xiang/80">
                        <span>↑↓ 导航</span>
                        <span>↵ 打开</span>
                        <span>Esc 关闭</span>
                        <span class="ml-auto text-ms-gold/60">&gt; 命令</span>
                    </div>
                </div>
            </div>
        </Transition>
    </Teleport>
</template>

<style scoped>
.search-panel {
    box-shadow: 4px 4px 0 0 rgba(0, 0, 0, 0.6), 0 0 60px rgba(166, 38, 38, 0.06);
}

.search-result--selected {
    box-shadow: inset 0 1px 0 0 rgba(255, 255, 255, 0.04), inset 0 -1px 0 0 rgba(0, 0, 0, 0.1);
}
</style>
