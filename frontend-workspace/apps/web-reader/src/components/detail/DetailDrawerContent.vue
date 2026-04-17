<script setup lang="ts">
import { computed } from "vue";
import SkeletonBlock from "@memory-stream/ui-shared/components/SkeletonBlock.vue";
import MarkdownViewer from "@memory-stream/ui-shared/components/MarkdownViewer.vue";
import BacklinksPanel from "../BacklinksPanel.vue";
import type { CardDetail } from "../../composables/useCards";
import type { BacklinkItem } from "../../composables/useDetailDrawer";

const props = defineProps<{
  detail: CardDetail | null;
  loading: boolean;
  backlinks: BacklinkItem[];
  backlinksLoading: boolean;
  backlinksOpen: boolean;
}>();

const emit = defineEmits<{
  (e: "toggle-backlinks"): void;
  (e: "navigate-to-backlink", cardId: string): void;
  (e: "prose-mouse-over", event: MouseEvent): void;
  (e: "prose-mouse-out", event: MouseEvent): void;
  (e: "prose-click", event: MouseEvent): void;
  (e: "wikilink-click", targetId: string): void;
}>();

const hasBacklinks = computed(() => props.backlinks.length > 0 || props.backlinksLoading);
</script>

<template>
  <div class="detail-drawer__content prose-container scrollbar-thin relative"
    @mouseover="emit('prose-mouse-over', $event)" @mouseout="emit('prose-mouse-out', $event)"
    @click="emit('prose-click', $event)">
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

        <MarkdownViewer :html-content="detail.html" @wikilink-click="emit('wikilink-click', $event)" />

        <!-- Backlinks Section -->
        <BacklinksPanel v-if="hasBacklinks" :card-id="detail.id" :is-open="backlinksOpen" :backlinks="backlinks"
          :loading="backlinksLoading" @toggle="emit('toggle-backlinks')"
          @navigate="emit('navigate-to-backlink', $event)" />
      </div>

      <!-- Error State -->
      <div v-else class="detail-drawer__error">
        <span class="text-ms-smoke text-sm">无法加载卡片内容</span>
      </div>
    </Transition>
  </div>
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
  background: linear-gradient(90deg,
      transparent 0%,
      theme('colors.ms-gold') 20%,
      theme('colors.ms-gold-dim') 50%,
      theme('colors.ms-gold') 80%,
      transparent 100%);
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
