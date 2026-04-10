<script setup lang="ts">
/**
 * 🌟 ListView — 极简脊柱流 (Minimalist Spine)
 *
 * 三列网格：铭文日期 | 微光脊柱 | 极简卡片
 * - 单行描述 + 大间距：呼吸感的纵向节奏
 * - 隐藏式时间：仅 hover 节点时显现
 * - 节点分级：Genesis 双环 / 普通 2px 微点
 */
import { ref, onMounted, computed, watchEffect } from "vue";
import { storeToRefs } from "pinia";
import { useVirtualizer } from "@tanstack/vue-virtual";
import { Inbox } from "lucide-vue-next";
import { useCards } from "../composables/useCards";
import { useGraphStore } from "../store/useGraphStore";
import SkeletonLine from "../components/ui/SkeletonLine.vue";
import StatsWidget from "../components/StatsWidget.vue";

const store = useGraphStore();
const { sortBy } = storeToRefs(store);
const { cardIndex, loadIndex, loading } = useCards();

const searchQuery = ref("");

interface CardData {
    id: string;
    title: string;
    excerpt: string;
    hot_score: number;
    updated_at: string;
    relation: "sequence" | "reference";
}
interface CardRow {
    type: "card";
    data: CardData;
    isFirstInDay: boolean;
    dateLabel: string;
}

const filteredCards = computed(() => {
    let cards = cardIndex.value;
    if (searchQuery.value) {
        const q = searchQuery.value.toLowerCase();
        cards = cards.filter(
            (card) =>
                card.title.toLowerCase().includes(q) ||
                (card.excerpt && card.excerpt.toLowerCase().includes(q)),
        );
    }
    if (sortBy.value === "hot") {
        cards = [...cards].sort((a, b) => (b.hot_score || 0) - (a.hot_score || 0));
    }
    return cards;
});

const flatItems = computed<CardRow[]>(() => {
    const items: CardRow[] = [];
    let lastDate = "";

    for (const card of filteredCards.value) {
        let isFirstInDay = false;
        let dateLabel = "";

        if (sortBy.value === "updated") {
            const dateStr = card.updated_at?.slice(0, 10) || "unknown";
            if (dateStr !== lastDate) {
                isFirstInDay = true;
                dateLabel = formatDateLabel(dateStr);
                lastDate = dateStr;
            }
        }

        items.push({ type: "card", data: card, isFirstInDay, dateLabel });
    }
    return items;
});

function formatDateLabel(dateStr: string): string {
    if (dateStr === "unknown") return "未知";
    const d = new Date(dateStr + "T00:00:00");
    const month = d.getMonth() + 1;
    const day = d.getDate();
    return `${month}.${day}`;
}

function selectCard(id: string) {
    store.selectNode(id);
}

function formatTime(iso: string): string {
    if (!iso) return "";
    const d = new Date(iso);
    const hour = String(d.getHours()).padStart(2, "0");
    const min = String(d.getMinutes()).padStart(2, "0");
    return `${hour}:${min}`;
}

onMounted(() => {
    loadIndex();
});

const todayCount = computed(() => {
    const today = new Date();
    const todayStr = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, "0")}-${String(today.getDate()).padStart(2, "0")}`;
    return filteredCards.value.filter((c) => c.updated_at?.startsWith(todayStr)).length;
});

const avgHot = computed(() => {
    if (filteredCards.value.length === 0) return "0.0";
    const sum = filteredCards.value.reduce((acc, c) => acc + (c.hot_score || 0), 0);
    return (sum / filteredCards.value.length).toFixed(1);
});

const sortLabel = computed(() => (sortBy.value === "hot" ? "热度排序" : "时间排序"));

// ── 虚拟滚动（大间距：32px gap） ──
const listRef = ref<HTMLElement>();
const itemCount = computed(() => flatItems.value.length);
const virtualizer = useVirtualizer({
    count: itemCount.value,
    getScrollElement: () => listRef.value ?? null,
    estimateSize: () => 100,
    overscan: 10,
    gap: 32,
});

watchEffect(() => {
    virtualizer.value.options.count = itemCount.value;
});
</script>

<template>
    <div class="list-view bg-ms-deep h-full flex flex-col pt-8 pb-0">
        <!-- 加载态 -->
        <div v-if="loading" class="flex-1 px-8 py-4 space-y-3 max-w-3xl mx-auto w-full">
            <div v-for="i in 5" :key="i" class="flex items-center gap-4 p-6 rounded-sm border border-ms-border">
                <SkeletonLine width="4px" height="60px" />
                <div class="flex-1 space-y-3">
                    <SkeletonLine width="60%" height="16px" />
                    <SkeletonLine width="90%" height="12px" />
                    <SkeletonLine width="40%" height="10px" />
                </div>
            </div>
        </div>

        <!-- 卡片列表（极简脊柱） -->
        <div v-else-if="filteredCards.length > 0" ref="listRef" class="flex-1 min-h-0 overflow-y-auto pb-6 relative">

            <!-- 全局脊柱线 - 神圣光束 -->
            <div class="absolute left-24 top-0 bottom-0 w-24 z-0 pointer-events-none"
                style="background: radial-gradient(ellipse at center, rgba(0,229,255,0.06) 0%, transparent 70%);" />

            <div class="max-w-4xl mx-auto">
                <div :style="{ height: `${virtualizer.getTotalSize()}px`, position: 'relative' }">
                    <div v-for="row in virtualizer.getVirtualItems()" :key="row.index" :style="{
                        position: 'absolute',
                        top: 0,
                        transform: `translateY(${row.start}px)`,
                        width: '100%',
                    }" class="grid grid-cols-spine items-start group">

                        <!-- ━━━ 第1列：铭文日期 ━━━ -->
                        <div class="relative flex items-start justify-end pr-3 pt-2">
                            <div v-if="(flatItems[row.index] as CardRow)?.isFirstInDay"
                                class="sticky top-4 z-20 text-slate-600/40 font-mono text-2xs tracking-widest whitespace-nowrap select-none"
                                style="writing-mode: vertical-rl; transform: rotate(180deg);">
                                {{ (flatItems[row.index] as CardRow).dateLabel }}
                            </div>
                        </div>

                        <!-- ━━━ 第2列：脊柱节点 ━━━ -->
                        <div class="group/node relative flex items-start justify-center pt-5 z-10">

                            <!-- 普通节点：星尘微点 -->
                            <div v-if="!(flatItems[row.index] as CardRow)?.isFirstInDay"
                                class="w-1 h-1 rounded-full transition-all duration-700 cursor-pointer"
                                :class="store.selectedId === (flatItems[row.index] as CardRow)?.data?.id
                                    ? 'bg-white/80 shadow-white-glow'
                                    : 'bg-white/30 group-hover/node:bg-white/60 group-hover/node:shadow-white-glow-sm'" />

                            <!-- Genesis Node：能量流动双环 -->
                            <div v-else
                                class="genesis-node w-4 h-4 rounded-full transition-all duration-300 cursor-pointer relative"
                                :class="store.selectedId === (flatItems[row.index] as CardRow)?.data?.id
                                    ? 'genesis-selected shadow-white-glow-lg'
                                    : 'group-hover/node:shadow-neon-glow-soft'">
                                <!-- 外层能量环（conic-gradient 旋转） -->
                                <div class="genesis-ring absolute inset-0 rounded-full" />
                                <!-- 选中态：额外旋转外环 -->
                                <div v-if="store.selectedId === (flatItems[row.index] as CardRow)?.data?.id"
                                    class="genesis-spin-ring absolute -inset-[3px] rounded-full border border-white/30" />
                                <!-- 内核 -->
                                <div class="absolute inset-[3px] rounded-full transition-all duration-300" :class="store.selectedId === (flatItems[row.index] as CardRow)?.data?.id
                                    ? 'bg-white/60'
                                    : 'bg-white/20 group-hover/node:bg-white/60'" />
                            </div>

                            <!-- 时间浮窗（hover 节点 / 行时显现） -->
                            <div v-if="(flatItems[row.index] as CardRow)?.data"
                                class="absolute left-1/2 -translate-x-1/2 -top-2 opacity-0 group-hover/node:opacity-100 group-hover:opacity-80 transition-opacity duration-200 pointer-events-none z-30">
                                <span
                                    class="text-2xs font-mono text-white/70 bg-ms-deep/90 backdrop-blur-sm px-1.5 py-0.5 border border-white/15 whitespace-nowrap">
                                    {{ formatTime((flatItems[row.index] as CardRow).data.updated_at) }}
                                </span>
                            </div>
                        </div>

                        <!-- ━━━ 第3列：极简卡片 ━━━ -->
                        <div v-if="(flatItems[row.index] as CardRow)?.data" class=" relative cursor-pointer pl-6 pr-12 py-8"
                            @click="selectCard((flatItems[row.index] as CardRow).data.id)">

                            <!-- 连接线 — 主干: 实线霓虹 / 参考: 虚线灰色 -->
                            <div class="absolute left-0 top-4.5 w-3 h-0.5 z-10 transition-opacity duration-300 overflow-hidden"
                                :class="[
                                    (flatItems[row.index] as CardRow).data.relation === 'sequence'
                                        ? (store.selectedId === (flatItems[row.index] as CardRow).data.id
                                            ? 'opacity-80'
                                            : 'opacity-0 group-hover:opacity-60')
                                        : (store.selectedId === (flatItems[row.index] as CardRow).data.id
                                            ? 'opacity-50'
                                            : 'opacity-0 group-hover:opacity-30'),
                                ]" :style="(flatItems[row.index] as CardRow).data.relation !== 'sequence'
                                    ? { borderStyle: 'dashed', borderWidth: '1px 0 0 0', borderColor: 'rgba(100,116,139,0.4)' }
                                    : {}">
                                <div v-if="(flatItems[row.index] as CardRow).data.relation === 'sequence'"
                                    class="w-full h-full bg-gradient-to-r from-neon/60 via-neon/20 to-transparent" />
                                <div v-else
                                    class="w-full h-full bg-gradient-to-r from-slate-500/40 via-slate-500/15 to-transparent" />
                                <div v-if="(flatItems[row.index] as CardRow).data.relation === 'sequence'"
                                    class="sweep-light absolute inset-0 bg-gradient-to-r from-transparent via-white/40 to-transparent" />
                            </div>

                            <!-- 卡片本体 -->
                            <div class="peer group relative max-w-2xl ml-2
                                       transition-all duration-700" :class="[
                                        store.selectedId === (flatItems[row.index] as CardRow).data.id
                                            ? 'bg-ms-surface/60 border-neon/20 shadow-card-active'
                                            : 'bg-transparent hover:bg-ms-void/30'
                                    ]">

                                <!-- Top edge glow — soft temple hover indicator -->
                                <div class="absolute top-0 left-[10%] right-[10%] h-px bg-gradient-to-r from-transparent via-neon/40 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-1000 pointer-events-none"
                                    style="filter: blur(1px);"></div>

                                <!-- 激活态左侧霓虹边条 -->
                                <div v-if="store.selectedId === (flatItems[row.index] as CardRow).data.id"
                                    class="absolute left-0 top-0 bottom-0 w-0.5 bg-neon/60 shadow-neon-glow-xs" />

                                <!-- 标题行 -->
                                <div class="flex items-center gap-3 mb-2">
                                    <h3 class="text-lg font-serif font-semibold tracking-spine transition-colors duration-700 truncate flex-1"
                                        :class="store.selectedId === (flatItems[row.index] as CardRow).data.id
                                            ? 'text-slate-100'
                                            : 'text-slate-300 group-hover:text-slate-100'">
                                        {{ (flatItems[row.index] as CardRow).data.title || '无标题' }}
                                    </h3>

                                    <!-- 关系类型标识 — 终端标识符风格 -->
                                    <span
                                        class="shrink-0 text-2xs font-mono px-1.5 py-0.5 border select-none uppercase tracking-widest"
                                        :class="(flatItems[row.index] as CardRow).data.relation === 'sequence'
                                            ? 'text-slate-400 border-slate-700/50 bg-slate-800/30'
                                            : 'text-slate-500 border-slate-700/50 bg-slate-800/30'">
                                        {{ (flatItems[row.index] as CardRow).data.relation === 'sequence' ? 'SEQ' :
                                            'REF' }}
                                    </span>

                                    <!-- 热度数据读数 — 终端仪表板风格（仅有值时显示） -->
                                    <div v-if="(flatItems[row.index] as CardRow).data.hot_score > 0"
                                        class="shrink-0 text-2xs font-mono tracking-widest text-slate-600 uppercase border-b border-slate-800 pb-1">
                                        <span class="mr-1.5">Vol</span>
                                        <span class="text-neon">{{ Math.round((flatItems[row.index] as CardRow).data.hot_score) }}</span>
                                    </div>
                                </div>

                                <!-- 描述（单行截断） -->
                                <p class="text-slate-500/70 text-sm truncate leading-loose tracking-wide group-hover:text-slate-400 transition-colors duration-700">
                                    {{ (flatItems[row.index] as CardRow).data.excerpt || '暂无摘要' }}
                                </p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- 空态 -->
        <div v-else class="flex-1 flex items-center justify-center">
            <div class="text-center py-16 px-6">
                <!-- 脉冲空态装饰 -->
                <div class="relative mx-auto mb-6 w-20 h-20">
                    <div class="absolute inset-0 rounded-full border border-neon/15 animate-ping opacity-30" />
                    <div class="absolute inset-2 rounded-full border border-neon/20 animate-pulse opacity-40" />
                    <Inbox :size="32" class="absolute inset-0 m-auto text-gray-600" />
                </div>
                <h3 class="text-gray-300 text-base font-semibold mb-2">
                    {{ searchQuery ? "没有找到匹配的卡片" : "记忆流是空的" }}
                </h3>
                <p v-if="!searchQuery" class="text-gray-500 text-sm mb-1">
                    快去桌面端写几张卡片吧
                </p>
                <p v-if="!searchQuery" class="text-gray-600 text-xs font-mono">
                    创建卡片后图谱会自动生成
                </p>
            </div>
        </div>

        <!-- 右下角悬浮数据面板 -->
        <StatsWidget v-if="filteredCards.length > 0" :total-nodes="filteredCards.length" :today-count="todayCount"
            :avg-hot="avgHot" :sort-label="sortLabel"
            :sparkline-data="filteredCards.slice(0, 20).map(c => c.hot_score || 0)" />
    </div>
</template>

<style scoped>
/* ── Temple card — sacred monolith aesthetic ── */
.mechanic-card {
    transition: background-color 0.7s cubic-bezier(0.25, 1, 0.5, 1);
}

.mechanic-card:hover {
    background-color: transparent;
}

/* ── Genesis Node 能量流动 ── */
@keyframes genesis-energy-flow {
    0% {
        background: conic-gradient(from 0deg,
                transparent 0%,
                rgba(255, 255, 255, 0.09) 20%,
                rgba(255, 255, 255, 0.3) 50%,
                rgba(255, 255, 255, 0.09) 80%,
                transparent 100%);
        filter: drop-shadow(0 0 3px rgba(255, 255, 255, 0.18));
    }

    50% {
        filter: drop-shadow(0 0 8px rgba(255, 255, 255, 0.36));
    }

    100% {
        background: conic-gradient(from 360deg,
                transparent 0%,
                rgba(255, 255, 255, 0.09) 20%,
                rgba(255, 255, 255, 0.3) 50%,
                rgba(255, 255, 255, 0.09) 80%,
                transparent 100%);
        filter: drop-shadow(0 0 3px rgba(255, 255, 255, 0.18));
    }
}

@keyframes genesis-spin {
    from {
        transform: rotate(0deg);
    }

    to {
        transform: rotate(360deg);
    }
}

@keyframes genesis-pulse-glow {

    0%,
    100% {
        opacity: 0.4;
    }

    50% {
        opacity: 0.8;
    }
}

@keyframes sweep-light {
    from {
        transform: translateX(-200%);
    }

    to {
        transform: translateX(200%);
    }
}

.genesis-node {
    animation: genesis-energy-flow 3s linear infinite;
}

.genesis-node .genesis-ring {
    background: conic-gradient(from 0deg,
            transparent 0%,
            rgba(255, 255, 255, 0.09) 20%,
            rgba(255, 255, 255, 0.3) 50%,
            rgba(255, 255, 255, 0.09) 80%,
            transparent 100%);
    animation: genesis-spin 3s linear infinite;
}

.genesis-node.genesis-selected .genesis-ring {
    animation-duration: 1.5s;
}

.genesis-node .genesis-spin-ring {
    animation: genesis-spin 2s linear infinite, genesis-pulse-glow 1.5s ease-in-out infinite;
}

/* hover 加速能量流动 */
.group:hover .genesis-node {
    animation-duration: 1.5s;
}

.group:hover .genesis-node .genesis-ring {
    animation-duration: 1s;
}

/* 扫光动画 - hover/选中时触发 */
.group:hover .sweep-light,
.genesis-selected~.sweep-light {
    animation: sweep-light 1.2s ease-in-out infinite;
}
</style>
