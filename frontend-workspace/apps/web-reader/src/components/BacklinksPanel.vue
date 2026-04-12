<script setup lang="ts">
/**
 * BacklinksPanel — 引渡经幡（血肉神殿）
 *
 * 显示指向当前卡片的所有反向引用。
 *
 * Features:
 * - Copper-green (ms-patina) title with decorative underline
 * - altar-glow-sm hover effect on backlink cards
 * - Inline BacklinkCard component with:
 *   - Left arrow icon (xuepo on hover)
 *   - Title in ms-bone-dim → xuepo on hover
 *   - Relation badge with dynamic styling
 *   - Context snippet in italic ms-smoke
 * - Smooth expand/collapse with ms-slide-down transition
 */

import { ref, watch, computed } from "vue";
import { ArrowLeft, Link2, ChevronDown, ChevronUp } from "lucide-vue-next";
import { api } from "../api";
import type { InferredBacklinkItem } from "../api/schemas";
import { useGraphStore } from "../store/useGraphStore";

const props = defineProps<{
    cardId: string;
    isOpen?: boolean;
    backlinks?: BacklinkItem[];
    loading?: boolean;
}>();

const emit = defineEmits<{
    (e: "toggle"): void;
    (e: "navigate", cardId: string): void;
}>();

const store = useGraphStore();

interface BacklinkItem extends InferredBacklinkItem {
    context_snippet?: string;
}

// Internal state for standalone mode, external props for controlled mode
const internalBacklinks = ref<BacklinkItem[]>([]);
const internalLoading = ref(false);
const internalIsOpen = ref(false);

const backlinks = props.backlinks !== undefined ? computed(() => props.backlinks ?? []) : internalBacklinks;
const loading = props.loading !== undefined ? computed(() => props.loading) : internalLoading;
const isOpen = props.isOpen !== undefined ? computed(() => props.isOpen) : internalIsOpen;

async function fetchBacklinks() {
    if (!props.cardId) {
        internalBacklinks.value = [];
        return;
    }

    internalLoading.value = true;
    try {
        const res = await api.getBacklinks(props.cardId);
        internalBacklinks.value = res.backlinks || [];
    } catch {
        internalBacklinks.value = [];
    } finally {
        internalLoading.value = false;
    }
}

// Only fetch if in standalone mode (no external backlinks prop)
if (props.backlinks === undefined) {
    watch(() => props.cardId, fetchBacklinks, { immediate: true });
}

function navigateToCard(cardId: string) {
    emit("navigate", cardId);
    store.selectNode(cardId);
}

function toggleOpen() {
    if (props.isOpen !== undefined) {
        emit("toggle");
    } else {
        internalIsOpen.value = !internalIsOpen.value;
    }
}

function getBadgeClass(relationType: string): string {
    if (relationType === "sequence") {
        return "text-xuepo bg-xuepo/10 border-xuepo/30";
    }
    return "text-ms-smoke bg-ms-smoke/10 border-ms-smoke/30";
}
</script>

<template>
    <div v-if="backlinks.length > 0 || loading" class="backlinks-panel">
        <!-- Header with Copper-green Title -->
        <button
            class="backlinks-panel__header"
            @click="toggleOpen">
            <div class="flex items-center gap-2">
                <Link2 :size="12" class="backlinks-panel__icon" />
                <span class="backlinks-panel__title">被引用</span>
                <span v-if="loading"
                    class="backlinks-panel__spinner" />
                <span v-else class="backlinks-panel__count">({{ backlinks.length }})</span>
            </div>
            <ChevronDown
                v-if="!isOpen"
                :size="14"
                class="backlinks-panel__chevron" />
            <ChevronUp
                v-else
                :size="14"
                class="backlinks-panel__chevron backlinks-panel__chevron--open" />
        </button>

        <!-- Decorative Copper Line -->
        <div class="backlinks-panel__line" />

        <!-- Backlinks List with Slide Down Transition -->
        <Transition name="ms-slide-down">
            <div v-if="isOpen" class="backlinks-panel__list">
                <button
                    v-for="bl in backlinks"
                    :key="bl.source_id"
                    class="backlink-card"
                    @click="navigateToCard(bl.source_id)">
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
</template>

<style scoped>
.backlinks-panel {
    margin-top: 32px;
    padding-top: 20px;
    border-top: 1px solid theme('colors.ms-copper');
}

/* Header */
.backlinks-panel__header {
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

.backlinks-panel__header:hover .backlinks-panel__title {
    color: theme('colors.xuepo.DEFAULT');
}

.backlinks-panel__header:hover .backlinks-panel__icon {
    color: theme('colors.xuepo.DEFAULT');
}

.backlinks-panel__icon {
    color: theme('colors.ms-patina');
    transition: color 200ms ease;
}

.backlinks-panel__title {
    font-size: 12px;
    font-weight: 500;
    color: theme('colors.ms-patina');
    transition: color 200ms ease;
    letter-spacing: 0.04em;
}

.backlinks-panel__count {
    font-size: 11px;
    color: theme('colors.ms-ash');
    font-family: "JetBrains Mono", monospace;
}

.backlinks-panel__spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 1.5px solid theme('colors.ms-ash');
    border-top-color: theme('colors.xuepo.DEFAULT');
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}

.backlinks-panel__chevron {
    color: theme('colors.ms-patina');
    transition: transform 200ms ease, color 200ms ease;
}

.backlinks-panel__chevron--open {
    transform: rotate(180deg);
}

.backlinks-panel__header:hover .backlinks-panel__chevron {
    color: theme('colors.xuepo.DEFAULT');
}

/* Decorative Copper Line */
.backlinks-panel__line {
    height: 1px;
    background: linear-gradient(
        90deg,
        transparent 0%,
        theme('colors.ms-patina') 15%,
        theme('colors.ms-patina') 85%,
        transparent 100%
    );
    margin-top: 8px;
    opacity: 0.6;
}

/* List */
.backlinks-panel__list {
    margin-top: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
}

/* Backlink Card */
.backlink-card {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    width: 100%;
    padding: 12px 14px;
    text-align: left;
    background: theme('colors.ms-xiang');
    border: 1px solid theme('colors.ms-copper');
    transition: all 200ms ease;
    cursor: pointer;
}

.backlink-card:hover {
    border-color: theme('colors.xuepo.DEFAULT');
    box-shadow: 0 0 4px rgba(166, 38, 38, 0.15);
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
    letter-spacing: 0.04em;
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
</style>
