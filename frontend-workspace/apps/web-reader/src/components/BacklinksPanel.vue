<script setup lang="ts">
/**
 * 📎 BacklinksPanel — 反向引用面板
 *
 * 显示指向当前卡片的所有反向引用（Backlinks）。
 * 只读展示，无编辑/删除功能。
 *
 * 设计：
 * - relation_type 徽章：sequence=青色，reference=灰色
 * - source_title 可点击跳转
 * - context_snippet 灰色斜体截断显示
 */

import { ref, watch } from "vue";
import { ArrowLeft } from "lucide-vue-next";
import { api } from "../api";
import { useGraphStore } from "../store/useGraphStore";

// ── Props ──
const props = defineProps<{
    cardId: string;
}>();

// ── Store ──
const store = useGraphStore();

// ── State ──
interface BacklinkItem {
    source_id: string;
    source_title: string;
    relation_type: string;
    context_snippet?: string;
}

const backlinks = ref<BacklinkItem[]>([]);
const loading = ref(false);
const isOpen = ref(false);

// ── Fetch backlinks ──
async function fetchBacklinks() {
    if (!props.cardId) {
        backlinks.value = [];
        return;
    }

    loading.value = true;
    try {
        const res = await api.getBacklinks(props.cardId);
        backlinks.value = res.backlinks || [];
    } catch {
        backlinks.value = [];
    } finally {
        loading.value = false;
    }
}

// ── Watch cardId changes ──
watch(() => props.cardId, fetchBacklinks, { immediate: true });

// ── Navigate to source card ──
function navigateToCard(cardId: string) {
    store.selectNode(cardId);
}

// ── Badge color based on relation type ──
function getBadgeClass(relationType: string): string {
    if (relationType === "sequence") {
        return "text-cyan-400 bg-cyan-500/10 border-cyan-500/30";
    }
    // reference or other types
    return "text-gray-400 bg-gray-500/10 border-gray-500/30";
}
</script>

<template>
    <div v-if="backlinks.length > 0 || loading" class="mt-8 pt-5 border-t border-ms-border/50">
        <!-- Header -->
        <button
            class="flex items-center gap-2 text-xs text-gray-400 hover:text-neon transition-colors duration-200 mb-3"
            @click="isOpen = !isOpen">
            <ArrowLeft :size="12" />
            <span class="font-medium">被引用</span>
            <span v-if="loading"
                class="inline-block w-3 h-3 border border-gray-500 border-t-neon rounded-full animate-spin" />
            <span v-else class="text-gray-600">({{ backlinks.length }})</span>
            <svg class="w-3 h-3 transition-transform duration-200" :class="{ 'rotate-180': isOpen }" viewBox="0 0 12 12"
                fill="none">
                <path d="M3 4.5L6 7.5L9 4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"
                    stroke-linejoin="round" />
            </svg>
        </button>

        <!-- Backlinks List -->
        <Transition name="ms-slide-up">
            <div v-if="isOpen" class="space-y-2">
                <button v-for="bl in backlinks" :key="bl.source_id"
                    class="w-full flex flex-col items-start gap-1.5 px-3 py-2.5 rounded-md text-left transition-all duration-150 group"
                    :class="[
                        'hover:bg-neon/5'
                    ]"
                    @click="navigateToCard(bl.source_id)">
                    <!-- Title row with badge -->
                    <div class="flex items-center gap-2 w-full">
                        <ArrowLeft :size="10"
                            class="flex-shrink-0 text-gray-600 group-hover:text-neon transition-colors" />
                        <span class="text-xs text-gray-300 group-hover:text-neon transition-colors truncate flex-1">
                            {{ bl.source_title }}
                        </span>
                        <span class="flex-shrink-0 px-1.5 py-0.5 text-[10px] font-mono border rounded-sm"
                            :class="getBadgeClass(bl.relation_type)">
                            {{ bl.relation_type }}
                        </span>
                    </div>

                    <!-- Context snippet -->
                    <div v-if="bl.context_snippet" class="pl-5 w-full">
                        <p class="text-[11px] text-gray-500 italic truncate leading-relaxed">
                            {{ bl.context_snippet }}
                        </p>
                    </div>
                </button>
            </div>
 </Transition>
    </div>
</template>
