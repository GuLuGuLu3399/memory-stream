<script setup lang="ts">
/**
 * DetailDrawer — 经文卷轴（血肉神殿）
 *
 * 右侧滑出，占 45% 宽度。
 * 玄色基底 + WASM Markdown 渲染。
 * 监听 selectedId 变化，按需加载卡片详情。
 *
 * Features:
 * - FloatingPanel wrapper for consistent modal/drawer behavior
 * - Gold-leaf decorative rule at content top
 * - Paper-grain texture overlay on content area
 * - SkeletonBlock for loading states
 * - Swipe gesture support (useSwipeClose)
 * - Collapsible backlinks panel with copper-green styling
 * - Bottom bar with creation date + gold-accented ID
 */

import { ref, watch, computed } from "vue";
import { storeToRefs } from "pinia";
import FloatingPanel from "@memory-stream/ui-shared/components/FloatingPanel.vue";
import SkeletonBlock from "@memory-stream/ui-shared/components/SkeletonBlock.vue";
import MarkdownViewer from "@memory-stream/ui-shared/components/MarkdownViewer.vue";
import { useGraphStore } from "../store/useGraphStore";
import { useCards } from "../composables/useCards";
import type { CardDetail } from "../composables/useCards";
import { useBreakpoints } from "../composables/useBreakpoints";
import { useSwipeClose } from "../composables/useSwipeClose";
import { ArrowLeft, Link2, ChevronDown, ChevronUp, Calendar, Hash } from "lucide-vue-next";
import { api } from "../api";

interface BacklinkItem {
    source_id: string;
    source_title: string;
    relation_type: string;
    context_snippet?: string;
}

const store = useGraphStore();
const { selectedId } = storeToRefs(store);
const { loadDetail } = useCards();
const { isMobile } = useBreakpoints();

const detail = ref<CardDetail | null>(null);
const loading = ref(false);
const createdAt = ref("");

const backlinks = ref<BacklinkItem[]>([]);
const backlinksLoading = ref(false);
const backlinksOpen = ref(false);

const { offsetX: swipeOffset } = useSwipeClose({
    onClose: () => store.selectNode(null),
});

// Compute panel width based on breakpoint
const panelWidth = computed(() => isMobile.value ? "100%" : "45%");

watch(selectedId, async (newId) => {
    if (!newId) {
        detail.value = null;
        backlinks.value = [];
        createdAt.value = "";
        return;
    }

    loading.value = true;
    detail.value = null;
    backlinks.value = [];
    createdAt.value = "";

    try {
        const result = await loadDetail(newId);
        detail.value = result;

        // Fetch backlinks
        backlinksLoading.value = true;
        try {
            const res = await api.getBacklinks(newId);
            backlinks.value = res.backlinks || [];
        } catch {
            backlinks.value = [];
        } finally {
            backlinksLoading.value = false;
        }

        // Fetch card details to get created_at
        try {
            const cardData = await api.getCard(newId);
            createdAt.value = cardData.created_at || "";
        } catch {
            createdAt.value = "";
        }
    } finally {
        loading.value = false;
    }
});

function close() {
    store.selectNode(null);
}

function onBackdropClick() {
    close();
}

function onProseMouseOver(e: MouseEvent) {
    const target = (e.target as HTMLElement).closest("a.wikilink, a[data-card-id]");
    if (target) {
        const cardId = (target as HTMLAnchorElement).dataset.cardId ||
                      (target as HTMLAnchorElement).getAttribute("href")?.replace("/card/", "");
        if (cardId) store.highlightNode(cardId);
    }
}

function onProseMouseOut(e: MouseEvent) {
    const target = (e.target as HTMLElement).closest("a.wikilink, a[data-card-id]");
    if (target) store.highlightNode(null);
}

function navigateToBacklink(cardId: string) {
    store.selectNode(cardId);
}

function getBadgeClass(relationType: string): string {
    if (relationType === "sequence") {
        return "text-xuepo bg-xuepo/10 border-xuepo/30";
    }
    return "text-ms-smoke bg-ms-smoke/10 border-ms-smoke/30";
}
</script>

<template>
    <FloatingPanel
        position="right"
        :open="!!selectedId"
        :width="panelWidth"
        @close="onBackdropClick">

        <!-- Header — 双击标题区进入禅模式 -->
        <template #header>
            <h2 class="text-sm font-bold text-ms-ivory truncate pr-4 select-none"
                @dblclick="store.toggleZenMode()">
                {{ detail?.title || "加载中..." }}
            </h2>
        </template>

        <!-- 左侧隐形机械把手 — 禅模式触发 (fixed, 不随内容滚动) -->
        <button
            v-if="detail"
            class="zen-edge-handle"
            :class="{ 'zen-edge-handle--active': store.zenMode }"
            :title="store.zenMode ? '退出禅模式' : '进入禅模式'"
            @click.stop="store.toggleZenMode()"
        />

        <!-- Content Area with Texture Overlay -->
        <div
            class="detail-drawer__content prose-container scrollbar-thin relative"
            @mouseover="onProseMouseOver"
            @mouseout="onProseMouseOut">

            <!-- Paper-grain texture overlay -->
            <div class="detail-drawer__texture" />

            <!-- Loading State -->
            <div v-if="loading" class="detail-drawer__loading">
                <div class="space-y-6">
                    <SkeletonBlock variant="text" :lines="3" />
                    <div class="pt-4 border-t border-ms-copper/30">
                        <SkeletonBlock variant="text" :lines="4" />
                    </div>
                    <div class="pt-4 border-t border-ms-copper/30">
                        <SkeletonBlock variant="text" :lines="3" />
                    </div>
                    <div class="pt-4 border-t border-ms-copper/30">
                        <SkeletonBlock variant="rect" height="120px" />
                    </div>
                </div>
            </div>

            <!-- Content -->
            <Transition name="ms-morph" mode="out-in">
                <div v-if="detail" :key="detail.id" class="detail-drawer__markdown relative z-10">
                    <!-- Gold-leaf decorative rule -->
                    <div class="detail-drawer__gold-rule" />

                    <MarkdownViewer :html-content="detail.html" />

                    <!-- Backlinks Section -->
                    <div v-if="backlinks.length > 0 || backlinksLoading" class="detail-drawer__backlinks">
                        <button
                            class="detail-drawer__backlinks-header"
                            @click="backlinksOpen = !backlinksOpen">
                            <div class="flex items-center gap-2">
                                <Link2 :size="12" class="text-ms-patina" />
                                <span class="font-medium text-ms-patina">被引用</span>
                                <span v-if="backlinksLoading"
                                    class="inline-block w-3 h-3 border border-ms-ash border-t-xuepo rounded-full animate-spin" />
                                <span v-else class="text-ms-ash">({{ backlinks.length }})</span>
                            </div>
                            <ChevronDown
                                v-if="!backlinksOpen"
                                :size="14"
                                class="text-ms-patina transition-transform duration-200" />
                            <ChevronUp
                                v-else
                                :size="14"
                                class="text-ms-patina transition-transform duration-200" />
                        </button>

                        <Transition name="ms-slide-down">
                            <div v-if="backlinksOpen" class="detail-drawer__backlinks-list">
                                <button
                                    v-for="bl in backlinks"
                                    :key="bl.source_id"
                                    class="backlink-card"
                                    @click="navigateToBacklink(bl.source_id)">
                                    <ArrowLeft :size="10" class="backlink-card__arrow" />
                                    <div class="backlink-card__content">
                                        <div class="backlink-card__title-row">
                                            <span class="backlink-card__title">{{ bl.source_title }}</span>
                                            <span
                                                class="backlink-card__badge"
                                                :class="getBadgeClass(bl.relation_type)">
                                                {{ bl.relation_type }}
                                            </span>
                                        </div>
                                        <p v-if="bl.context_snippet" class="backlink-card__snippet">
                                            {{ bl.context_snippet }}
                                        </p>
                                    </div>
                                </button>
                            </div>
                        </Transition>
                    </div>
                </div>

                <!-- Error State -->
                <div v-else class="detail-drawer__error">
                    <span class="text-ms-smoke text-sm">无法加载卡片内容</span>
                </div>
            </Transition>
        </div>

        <!-- Footer -->
        <template v-if="detail" #footer>
            <div class="detail-drawer__footer">
                <div class="flex items-center gap-4">
                    <div class="flex items-center gap-1.5 text-ms-gold font-mono text-2xs">
                        <Hash :size="10" />
                        <span>{{ detail.id.slice(0, 8) }}</span>
                    </div>
                    <div v-if="createdAt" class="flex items-center gap-1.5 text-ms-ash font-mono text-2xs">
                        <Calendar :size="10" />
                        <span>{{ new Date(createdAt).toLocaleDateString("zh-CN") }}</span>
                    </div>
                </div>
                <div class="text-ms-ash font-mono text-2xs">
                    {{ detail.updatedAt }}
                </div>
            </div>
        </template>
    </FloatingPanel>
</template>

<style scoped>
.detail-drawer__content {
    position: relative;
    padding: 20px 24px;
    overflow-y: auto;
    flex: 1;
}

/* Paper-grain texture overlay */
.detail-drawer__texture {
    position: absolute;
    inset: 0;
    pointer-events: none;
    opacity: 0.03;
    z-index: 1;
    background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)'/%3E%3C/svg%3E");
}

/* ── 左侧隐形机械把手 — 有机-机械缝合线 ── */
.zen-edge-handle {
    position: fixed;
    right: 45%;
    top: 50%;
    transform: translateY(-50%) translateX(-50%);
    z-index: 52;
    width: 10px;
    height: 96px;
    border-radius: 5px;
    background: linear-gradient(
        180deg,
        rgba(90, 79, 62, 0.15) 0%,
        rgba(138, 126, 110, 0.3) 30%,
        rgba(138, 126, 110, 0.35) 50%,
        rgba(138, 126, 110, 0.3) 70%,
        rgba(90, 79, 62, 0.15) 100%
    );
    border: 1px solid rgba(58, 50, 40, 0.4);
    border-left: none;
    border-radius: 0 5px 5px 0;
    padding: 0;
    cursor: w-resize;
    transition: all 350ms cubic-bezier(0.16, 1, 0.3, 1);
    box-shadow:
        inset 1px 0 0 0 rgba(138, 126, 110, 0.08),
        1px 0 4px rgba(0, 0, 0, 0.3);
    animation: zen-handle-breathe 4s ease-in-out infinite;
}

/* 中心机械凹槽 */
.zen-edge-handle::before {
    content: '';
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: rgba(138, 126, 110, 0.4);
    box-shadow:
        0 -10px 0 rgba(138, 126, 110, 0.2),
        0 10px 0 rgba(138, 126, 110, 0.2),
        0 -20px 0 rgba(138, 126, 110, 0.1),
        0 20px 0 rgba(138, 126, 110, 0.1);
    transition: all 350ms cubic-bezier(0.16, 1, 0.3, 1);
}

/* 呼吸脉动 */
@keyframes zen-handle-breathe {
    0%, 100% { opacity: 0.5; }
    50% { opacity: 0.65; }
}

.zen-edge-handle:hover {
    opacity: 1;
    width: 12px;
    height: 108px;
    background: linear-gradient(
        180deg,
        rgba(90, 79, 62, 0.3) 0%,
        rgba(200, 191, 168, 0.6) 30%,
        rgba(200, 191, 168, 0.7) 50%,
        rgba(200, 191, 168, 0.6) 70%,
        rgba(90, 79, 62, 0.3) 100%
    );
    border-color: rgba(200, 191, 168, 0.3);
    transform: translateY(-50%) translateX(-60%);
    box-shadow:
        inset 1px 0 0 0 rgba(200, 191, 168, 0.15),
        -3px 0 12px rgba(200, 191, 168, 0.12),
        1px 0 8px rgba(0, 0, 0, 0.4);
    animation: none;
}

.zen-edge-handle:hover::before {
    background: rgba(232, 223, 208, 0.7);
    box-shadow:
        0 -10px 0 rgba(232, 223, 208, 0.4),
        0 10px 0 rgba(232, 223, 208, 0.4),
        0 -20px 0 rgba(232, 223, 208, 0.2),
        0 20px 0 rgba(232, 223, 208, 0.2);
}

.zen-edge-handle:active {
    width: 14px;
    height: 104px;
    background: linear-gradient(
        180deg,
        rgba(166, 38, 38, 0.4) 0%,
        rgba(166, 38, 38, 0.85) 40%,
        rgba(166, 38, 38, 0.9) 50%,
        rgba(166, 38, 38, 0.85) 60%,
        rgba(166, 38, 38, 0.4) 100%
    );
    border-color: rgba(166, 38, 38, 0.5);
    transform: translateY(-50%) translateX(-70%);
    box-shadow:
        -6px 0 20px rgba(166, 38, 38, 0.35),
        -2px 0 8px rgba(166, 38, 38, 0.5);
    transition-duration: 80ms;
}

.zen-edge-handle:active::before {
    background: rgba(255, 255, 255, 0.6);
    box-shadow: 0 0 6px rgba(255, 255, 255, 0.3);
}

/* 禅模式激活 — 血珀缝合线脉动 */
.zen-edge-handle--active {
    background: linear-gradient(
        180deg,
        rgba(166, 38, 38, 0.15) 0%,
        rgba(166, 38, 38, 0.45) 30%,
        rgba(166, 38, 38, 0.5) 50%,
        rgba(166, 38, 38, 0.45) 70%,
        rgba(166, 38, 38, 0.15) 100%
    );
    border-color: rgba(166, 38, 38, 0.35);
    box-shadow:
        -2px 0 8px rgba(166, 38, 38, 0.2),
        1px 0 4px rgba(0, 0, 0, 0.3);
    animation: zen-handle-pulse 3s ease-in-out infinite;
}

.zen-edge-handle--active::before {
    background: rgba(166, 38, 38, 0.6);
    box-shadow:
        0 -10px 0 rgba(166, 38, 38, 0.3),
        0 10px 0 rgba(166, 38, 38, 0.3),
        0 -20px 0 rgba(166, 38, 38, 0.15),
        0 20px 0 rgba(166, 38, 38, 0.15);
}

@keyframes zen-handle-pulse {
    0%, 100% {
        opacity: 0.55;
        box-shadow: -2px 0 8px rgba(166, 38, 38, 0.15), 1px 0 4px rgba(0, 0, 0, 0.3);
    }
    50% {
        opacity: 0.8;
        box-shadow: -3px 0 14px rgba(166, 38, 38, 0.3), 1px 0 4px rgba(0, 0, 0, 0.3);
    }
}

.zen-edge-handle--active:hover {
    opacity: 1;
    background: linear-gradient(
        180deg,
        rgba(194, 54, 22, 0.4) 0%,
        rgba(194, 54, 22, 0.8) 30%,
        rgba(194, 54, 22, 0.9) 50%,
        rgba(194, 54, 22, 0.8) 70%,
        rgba(194, 54, 22, 0.4) 100%
    );
    border-color: rgba(194, 54, 22, 0.5);
    animation: none;
}

.zen-edge-handle--active:hover::before {
    background: rgba(224, 112, 112, 0.8);
    box-shadow:
        0 -10px 0 rgba(224, 112, 112, 0.5),
        0 10px 0 rgba(224, 112, 112, 0.5);
}

.zen-edge-handle--active:active {
    background: linear-gradient(
        180deg,
        rgba(201, 168, 76, 0.4) 0%,
        rgba(201, 168, 76, 0.9) 40%,
        rgba(201, 168, 76, 1) 50%,
        rgba(201, 168, 76, 0.9) 60%,
        rgba(201, 168, 76, 0.4) 100%
    );
    border-color: rgba(201, 168, 76, 0.5);
    box-shadow: -6px 0 24px rgba(201, 168, 76, 0.45);
}

.zen-edge-handle--active:active::before {
    background: rgba(255, 255, 255, 0.7);
    box-shadow: 0 0 8px rgba(201, 168, 76, 0.5);
}

.detail-drawer__loading {
    padding: 20px 0;
}

.detail-drawer__markdown {
    max-width: 72ch;
    margin: 0 auto;
}

/* Gold-leaf decorative rule */
.detail-drawer__gold-rule {
    width: 100%;
    height: 2px;
    margin: 0 auto 24px;
    background: linear-gradient(
        90deg,
        transparent 0%,
        theme('colors.ms-gold') 20%,
        theme('colors.ms-gold-dim') 50%,
        theme('colors.ms-gold') 80%,
        transparent 100%
    );
    position: relative;
}

.detail-drawer__gold-rule::before {
    content: "";
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    width: 6px;
    height: 6px;
    background: theme('colors.ms-gold');
    transform: translate(-50%, -50%) rotate(45deg);
    box-shadow: 0 0 4px theme('colors.ms-gold');
}

.detail-drawer__error {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    min-height: 200px;
}

/* Backlinks Section */
.detail-drawer__backlinks {
    margin-top: 48px;
    padding-top: 20px;
    border-top: 1px solid theme('colors.ms-patina');
}

.detail-drawer__backlinks-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 8px 0;
    background: transparent;
    border: none;
    cursor: pointer;
    transition: all 200ms ease;
}

.detail-drawer__backlinks-header:hover {
    opacity: 0.8;
}

.detail-drawer__backlinks-list {
    margin-top: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
}

/* Backlink Card — hard entity shadow */
.backlink-card {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    width: 100%;
    padding: 12px 14px;
    text-align: left;
    background: #12100c;
    border: 1px solid theme('colors.ms-copper');
    box-shadow: inset 0 1px 0 0 rgba(255, 255, 255, 0.03), 2px 2px 0 0 rgba(0, 0, 0, 0.5);
    transition: transform 0.15s ease, box-shadow 0.15s ease, border-color 0.2s ease;
    cursor: pointer;
}

.backlink-card:hover {
    border-color: theme('colors.xuepo.DEFAULT');
    transform: translate(-1px, -1px);
    box-shadow: inset 0 1px 0 0 rgba(255, 255, 255, 0.05), 3px 3px 0 0 rgba(0, 0, 0, 0.5);
}

.backlink-card:active {
    transform: translate(1px, 1px);
    box-shadow: inset 0 1px 0 0 rgba(255, 255, 255, 0.02), 0px 0px 0 0 rgba(0, 0, 0, 0.5);
}

.backlink-card__arrow {
    flex-shrink: 0;
    color: theme('colors.ms-ash');
    transition: color 200ms ease;
    margin-top: 2px;
}

.backlink-card:hover .backlink-card__arrow {
    color: theme('colors.xuepo.DEFAULT');
}

.backlink-card__content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.backlink-card__title-row {
    display: flex;
    align-items: center;
    gap: 8px;
}

.backlink-card__title {
    font-size: 12px;
    color: theme('colors.ms-bone-dim');
    transition: color 200ms ease;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
}

.backlink-card:hover .backlink-card__title {
    color: theme('colors.xuepo.DEFAULT');
}

.backlink-card__badge {
    flex-shrink: 0;
    padding: 2px 6px;
    font-size: 9px;
    font-family: "JetBrains Mono", monospace;
    border: 1px solid;
    border-radius: 2px;
    white-space: nowrap;
}

.backlink-card__snippet {
    font-size: 11px;
    color: theme('colors.ms-smoke');
    font-style: italic;
    line-height: 1.5;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin: 0;
    padding-left: 20px;
}

/* Footer */
.detail-drawer__footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    background: theme('colors.ms-mo');
    border-top: 1px solid theme('colors.ms-copper');
}

/* Prose container overrides */
.prose-container :deep(.prose) {
    color: theme('colors.ms-bone-dim');
}

.prose-container :deep(.prose h1) {
    color: theme('colors.ms-ivory');
}

.prose-container :deep(.prose h2) {
    color: theme('colors.ms-bone');
    border-bottom-color: theme('colors.ms-copper');
}

.prose-container :deep(.prose a) {
    color: theme('colors.xuepo.DEFAULT');
}

.prose-container :deep(.prose code) {
    background: rgba(166, 38, 38, 0.12);
    color: theme('colors.xuepo.600');
}
</style>
