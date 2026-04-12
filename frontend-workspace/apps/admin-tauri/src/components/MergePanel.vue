<script setup lang="ts">
import { watch, computed } from "vue";
import { useLayoutStore } from "../stores/layout";
import { useMergePanel } from "../composables/useMergePanel";
import { useLongPress } from "../composables/useLongPress";
import MergeVictimsColumn from "./merge/MergeVictimsColumn.vue";
import MergeSurvivorColumn from "./merge/MergeSurvivorColumn.vue";
import MergeBlastRadius from "./merge/MergeBlastRadius.vue";
import MergeActionBar from "./merge/MergeActionBar.vue";

// Props & Emits
interface CardData {
  id: string;
  title: string;
}

const props = defineProps<{
  cards: CardData[];
}>();

const emit = defineEmits<{
  (e: "merge-completed", payload: { survivorId: string; victimIds: string[] }): void;
}>();

const layoutStore = useLayoutStore();

// Composables
const merge = useMergePanel(props.cards);

// Long-press for merge execution
const longPress = useLongPress(
  async () => {
    const result = await merge.executeMerge();
    if (result?.success && result.survivorId && result.victimIds) {
      emit("merge-completed", {
        survivorId: result.survivorId,
        victimIds: result.victimIds,
      });
      if (merge.fileWriteErrors.value.length === 0) {
        merge.scheduleAutoClose(() => layoutStore.closeMergeConsole(), 1500);
      }
    }
  },
  () => merge.canShowBlastRadius.value
);

// Watch for survivor selection changes
watch(() => merge.selectedSurvivor.value, (newSurvivor) => {
  // Auto-remove from victims if selected as survivor
  if (newSurvivor && merge.selectedVictims.value.includes(newSurvivor)) {
    merge.selectedVictims.value = merge.selectedVictims.value.filter(id => id !== newSurvivor);
  }
});

// Close panel handler
function handleClose() {
  layoutStore.closeMergeConsole();
}

// Handlers for victim column
function handleVictimToggle(cardId: string) {
  merge.handleVictimToggle(cardId);
}

function handleVictimSelectAll() {
  merge.selectAllVictims();
}

function handleVictimClearAll() {
  merge.clearAllVictims();
}

function handleVictimSearchUpdate(value: string) {
  merge.victimSearchQuery.value = value;
}

// Handlers for survivor column
function handleSurvivorSelect(cardId: string) {
  merge.handleSurvivorSelect(cardId);
}

function handleSurvivorSearchUpdate(value: string) {
  merge.survivorSearchQuery.value = value;
}

// Available cards for victims (excludes survivor)
const availableVictimCards = computed(() => {
  return merge.selectedSurvivor.value
    ? props.cards.filter((c) => c.id !== merge.selectedSurvivor.value)
    : props.cards;
});
</script>

<template>
  <div class="fixed inset-x-0 bottom-0 top-titlebar z-overlay bg-ms-deep flex flex-col">
    <!-- Header Bar -->
    <div class="merge-panel__header">
      <div class="flex items-center gap-3">
        <span class="text-neon text-sm select-none">◆</span>
        <span class="merge-panel__header-title">
          MERGE STAGING AREA
        </span>
        <span class="merge-panel__header-subtitle">概念坍缩引擎</span>
      </div>
      <button
        @click="handleClose"
        class="merge-panel__close"
        title="关闭 (ESC)"
      >
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>

    <!-- Three-Column Layout -->
    <div class="flex-1 flex min-h-0">
      <!-- Column 1: SACRIFICES (Victims) -->
      <MergeVictimsColumn
        :cards="availableVictimCards"
        :selected-ids="merge.selectedVictims.value"
        :search-query="merge.victimSearchQuery.value"
        @toggle="handleVictimToggle"
        @selectAll="handleVictimSelectAll"
        @clearAll="handleVictimClearAll"
        @update:searchQuery="handleVictimSearchUpdate"
      />

      <!-- Column 2: BLAST RADIUS (Impact Preview) -->
      <MergeBlastRadius
        :can-show="merge.canShowBlastRadius.value"
        :is-loading="merge.isLoadingImpact.value"
        :error="merge.impactError.value"
        :impact-result="merge.impactResult.value"
      />

      <!-- Column 3: SURVIVOR -->
      <MergeSurvivorColumn
        :cards="props.cards"
        :selected-id="merge.selectedSurvivor.value"
        :search-query="merge.survivorSearchQuery.value"
        @select="handleSurvivorSelect"
        @update:searchQuery="handleSurvivorSearchUpdate"
      />
    </div>

    <!-- Bottom Action Bar -->
    <MergeActionBar
      :can-show="merge.canShowBlastRadius.value"
      :is-executing="longPress.isExecuting.value"
      :is-holding="longPress.isHolding.value"
      :hold-progress="longPress.holdProgress.value"
      :progress-offset="longPress.progressOffset.value"
      :warning-message="merge.warningMessage.value"
      :general-error="merge.generalError.value"
      @pointer-down="longPress.handlePointerDown"
      @pointer-up="longPress.handlePointerUp"
      @pointer-leave="longPress.handlePointerLeave"
    />

    <!-- Success Toast -->
    <Transition name="ms-slide-up">
      <div
        v-if="merge.showSuccessToast.value"
        class="fixed bottom-20 right-4 z-toast bg-brass/10 border border-brass/30 px-4 py-2 font-mono text-xs text-brass"
      >
        合并完成 — 所有文件写入成功
      </div>
    </Transition>

    <!-- Warning Toast (file write failures) -->
    <Transition name="ms-slide-up">
      <div
        v-if="merge.showWarningToast.value"
        class="fixed bottom-20 right-4 z-toast bg-amber-500/10 border border-amber-500/30 px-4 py-3 font-mono text-xs text-amber-400 max-w-md"
      >
        <div class="mb-2">数据库合并成功，但文件本地覆写受阻</div>
        <div v-for="w in merge.fileWriteErrors.value" :key="w.file" class="flex items-center gap-2 mt-1">
          <span class="text-slate-500 truncate max-w-48">{{ w.file }}</span>
          <button
            @click="merge.retryFileWrite(w.file)"
            class="text-amber-400 underline hover:text-amber-300 shrink-0"
          >
            重试
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.merge-panel__header {
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  border-bottom: 1px solid theme('colors.ms-border');
  background: theme('colors.ms-carbon');
  flex-shrink: 0;
}

.merge-panel__header-title {
  font-weight: bold;
  color: theme('colors.ms-slate.300');
  font-size: 12px;
  font-family: ui-monospace, monospace;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.merge-panel__header-subtitle {
  font-size: 10px;
  color: theme('colors.ms-slate.600');
  font-family: ui-monospace, monospace;
}

.merge-panel__close {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: theme('colors.ms-slate.500');
  transition: color 150ms ease, background-color 150ms ease;
}

.merge-panel__close:hover {
  color: theme('colors.ms-slate.300');
  background: theme('colors.ms-panel');
}
</style>
