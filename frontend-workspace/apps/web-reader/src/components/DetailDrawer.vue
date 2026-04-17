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

import { computed } from "vue";
import { storeToRefs } from "pinia";
import FloatingPanel from "@memory-stream/ui-shared/components/FloatingPanel.vue";
import { useGraphStore } from "../store/useGraphStore";
import { useDetailDrawer } from "../composables/useDetailDrawer";
import { useBreakpoints } from "../composables/useBreakpoints";
import { useSwipeClose } from "../composables/useSwipeClose";
import ZenEdgeHandle from "./detail/ZenEdgeHandle.vue";
import DetailDrawerContent from "./detail/DetailDrawerContent.vue";
import DetailDrawerFooter from "./detail/DetailDrawerFooter.vue";
import ZenSealButton from "./detail/ZenSealButton.vue";
import { resolveWikilinkTarget } from "../composables/useWikilinkNavigation";

const store = useGraphStore();
const { selectedId, zenMode } = storeToRefs(store);
const { isMobile } = useBreakpoints();

// Detail drawer logic
const drawer = useDetailDrawer(() => selectedId.value);

useSwipeClose({
  onClose: () => store.selectNode(null),
});

// Compute panel width based on breakpoint
const panelWidth = computed(() => isMobile.value ? "100%" : "45%");

// Handlers
function close() {
  store.selectNode(null);
}

function onBackdropClick() {
  close();
}

function onProseMouseOver(e: MouseEvent) {
  drawer.onProseMouseOver(e, (cardId) => store.highlightNode(cardId));
}

function onProseMouseOut(e: MouseEvent) {
  drawer.onProseMouseOut(e, () => store.highlightNode(null));
}

function onProseClick(e: MouseEvent) {
  void drawer.onProseClick(e, (cardId) => store.selectNode(cardId));
}

async function onWikilinkClick(targetId: string) {
  const resolvedId = await resolveWikilinkTarget(targetId);
  if (!resolvedId) return;
  store.selectNode(resolvedId);
}

function toggleBacklinks() {
  drawer.backlinksOpen.value = !drawer.backlinksOpen.value;
}

function navigateToBacklink(cardId: string) {
  drawer.navigateToBacklink(cardId, (id) => store.selectNode(id));
}

function toggleZenMode() {
  store.toggleZenMode();
}
</script>

<template>
  <FloatingPanel position="right" :open="!!selectedId" :width="panelWidth" @close="onBackdropClick">

    <!-- Header — 标题 + 禅模式入口 -->
    <template #header>
      <div class="flex items-center gap-2 w-full min-w-0">
        <h2 class="detail-drawer__title" @dblclick="!isMobile && toggleZenMode()">
          {{ drawer.detail.value?.title || "加载中..." }}
        </h2>
        <!-- 桌面端：双击标题提示 -->
        <span v-if="!isMobile && drawer.detail.value" class="detail-drawer__hint">
          双击进入禅
        </span>
        <!-- 移动端：朱印禅章 -->
        <ZenSealButton v-if="isMobile && drawer.detail.value" :is-active="zenMode" @click="toggleZenMode" />
      </div>
    </template>

    <!-- 左侧隐形机械把手 — 禅模式触发 (fixed, 不随内容滚动) -->
    <ZenEdgeHandle v-if="drawer.detail.value" :is-active="zenMode" :is-mobile="isMobile" @click="toggleZenMode" />

    <!-- Content Area -->
    <DetailDrawerContent :detail="drawer.detail.value" :loading="drawer.loading.value"
      :backlinks="drawer.backlinks.value" :backlinks-loading="drawer.backlinksLoading.value"
      :backlinks-open="drawer.backlinksOpen.value" @toggle-backlinks="toggleBacklinks"
      @navigate-to-backlink="navigateToBacklink" @prose-mouse-over="onProseMouseOver" @prose-mouse-out="onProseMouseOut"
      @prose-click="onProseClick" @wikilink-click="onWikilinkClick" />

    <!-- Footer — minimal seal -->
    <template #footer>
      <DetailDrawerFooter :detail="drawer.detail.value" :created-at="drawer.createdAt.value" />
    </template>
  </FloatingPanel>
</template>

<style scoped>
.detail-drawer__title {
  font-size: 14px;
  font-weight: bold;
  color: theme('colors.ms-ivory');
  truncate: true;
  flex: 1;
  min-width: 0;
  user-select: none;
  cursor: default;
}

.detail-drawer__hint {
  font-size: 11px;
  font-family: ui-monospace, monospace;
  color: rgba(148, 163, 184, 0.4);
  user-select: none;
  display: none;
}

@media (min-width: 1024px) {
  .detail-drawer__hint {
    display: inline;
  }
}
</style>
