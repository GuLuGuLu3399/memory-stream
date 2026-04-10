<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { MergePreview } from "@memory-stream/types/ipc";
import { useLayoutStore } from "../stores/layout";

// ============================================================================
// Props & Emits
// ============================================================================

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

// ============================================================================
// State
// ============================================================================

const selectedSurvivor = ref<string | null>(null);
const selectedVictims = ref<string[]>([]);
const victimSearchQuery = ref("");
const survivorSearchQuery = ref("");

// Impact preview data — types from ts-rs generated IPC
const impactResult = ref<MergePreview | null>(null);
const isLoadingImpact = ref(false);
const impactError = ref<string | null>(null);

// Long-press state
const isHolding = ref(false);
const holdProgress = ref(0);
const isExecuting = ref(false);
let holdTimer: ReturnType<typeof setInterval> | null = null;
let holdStartTime = 0;
const HOLD_DURATION_MS = 3000;

// Auto-close timers
let autoCloseTimers: ReturnType<typeof setTimeout>[] = [];

// Toast states
const showSuccessToast = ref(false);
const showWarningToast = ref(false);
const fileWriteErrors = ref<Array<{ file: string; error: string }>>([]);
const generalError = ref<string | null>(null);

// ============================================================================
// Computed
// ============================================================================

/** Cards available for victim selection (excludes survivor) */
const availableVictimCards = computed(() => {
  const filtered = selectedSurvivor.value
    ? props.cards.filter((c) => c.id !== selectedSurvivor.value)
    : props.cards;

  if (!victimSearchQuery.value) return filtered;
  const q = victimSearchQuery.value.toLowerCase();
  return filtered.filter((c) => c.title.toLowerCase().includes(q));
});

/** Cards available for survivor selection */
const availableSurvivorCards = computed(() => {
  if (!survivorSearchQuery.value) return props.cards;
  const q = survivorSearchQuery.value.toLowerCase();
  return props.cards.filter((c) => c.title.toLowerCase().includes(q));
});

/** Can show blast radius (need survivor + at least 1 victim) */
const canShowBlastRadius = computed(() => {
  return selectedSurvivor.value !== null && selectedVictims.value.length > 0;
});

/** Progress bar width percentage */
const progressWidth = computed(() => {
  return `${holdProgress.value}%`;
});

/** Warning message for bottom bar */
const warningMessage = computed(() => {
  if (!selectedSurvivor.value) return "请选择一个主节点";
  if (selectedVictims.value.length === 0) return "请选择至少一个祭品节点";
  const n = selectedVictims.value.length;
  const m = impactResult.value?.files_to_modify ?? "?";
  return `将永久删除 ${n} 个节点，修改 ${m} 个文件`;
});

// ============================================================================
// Lifecycle
// ============================================================================

let unlistenFileWriteFailed: (() => void) | null = null;

onMounted(async () => {
  // Listen for file write failures from Rust backend
  unlistenFileWriteFailed = await listen<{ file: string; error: string }>(
    "merge_file_write_failed",
    (event) => {
      fileWriteErrors.value.push(event.payload);
      showWarningToast.value = true;
    }
  );
});

onUnmounted(() => {
  if (unlistenFileWriteFailed) unlistenFileWriteFailed();
  clearHoldTimer();
  autoCloseTimers.forEach(clearTimeout);
  autoCloseTimers = [];
});

// Watch for selection changes to auto-fetch impact preview
watch([selectedSurvivor, selectedVictims], async () => {
  impactResult.value = null;
  impactError.value = null;

  if (canShowBlastRadius.value && selectedSurvivor.value) {
    await fetchImpactPreview();
  }
}, { deep: true });

// ============================================================================
// Handlers
// ============================================================================

/** Handle survivor radio selection */
function handleSurvivorSelect(cardId: string) {
  selectedSurvivor.value = cardId;
  // Auto-remove from victims if selected
  if (selectedVictims.value.includes(cardId)) {
    selectedVictims.value = selectedVictims.value.filter((id) => id !== cardId);
  }
}

/** Handle victim checkbox toggle */
function handleVictimToggle(cardId: string) {
  const index = selectedVictims.value.indexOf(cardId);
  if (index > -1) {
    selectedVictims.value = selectedVictims.value.filter((id) => id !== cardId);
  } else {
    selectedVictims.value = [...selectedVictims.value, cardId];
  }
}

/** Select all visible victims */
function selectAllVictims() {
  const newVictims = new Set(selectedVictims.value);
  availableVictimCards.value.forEach((c) => newVictims.add(c.id));
  selectedVictims.value = Array.from(newVictims);
}

/** Clear all victims */
function clearAllVictims() {
  selectedVictims.value = [];
}

/** Fetch impact preview from backend */
async function fetchImpactPreview() {
  if (!selectedSurvivor.value || selectedVictims.value.length === 0) return;

  isLoadingImpact.value = true;
  impactError.value = null;

  const victimTitles = selectedVictims.value
    .map((id) => props.cards.find((c) => c.id === id)?.title)
    .filter((t): t is string => !!t);

  try {
    const result = await invoke<MergePreview>("preview_merge_impact", {
      victimTitles,
    });
    impactResult.value = result;
  } catch (e: unknown) {
    impactError.value = e instanceof Error ? e.message : String(e);
  } finally {
    isLoadingImpact.value = false;
  }
}

/** Long-press handlers */
function handlePointerDown(e: PointerEvent) {
  if (e.button !== 0) return; // Left click only
  if (!canShowBlastRadius.value || isExecuting.value) return;

  isHolding.value = true;
  holdStartTime = Date.now();
  holdProgress.value = 0;

  holdTimer = setInterval(() => {
    const elapsed = Date.now() - holdStartTime;
    holdProgress.value = Math.min(100, (elapsed / HOLD_DURATION_MS) * 100);

    if (elapsed >= HOLD_DURATION_MS) {
      clearHoldTimer();
      executeMerge();
    }
  }, 50);
}

function handlePointerUp() {
  if (!isHolding.value) return;
  clearHoldTimer();
  isHolding.value = false;
  holdProgress.value = 0;
}

function handlePointerLeave() {
  if (isHolding.value) {
    clearHoldTimer();
    isHolding.value = false;
    holdProgress.value = 0;
  }
}

function clearHoldTimer() {
  if (holdTimer) {
    clearInterval(holdTimer);
    holdTimer = null;
  }
}

/** Execute the merge flow: Go transaction -> Rust write-back */
async function executeMerge() {
  if (!selectedSurvivor.value || selectedVictims.value.length === 0) return;

  isExecuting.value = true;
  generalError.value = null;
  fileWriteErrors.value = [];
  showSuccessToast.value = false;
  showWarningToast.value = false;

  try {
    // Step 1: Go transaction - merge in database
    const result = await invoke<{ success: boolean; message?: string }>("api_request", {
      method: "POST",
      endpoint: "/cards/merge",
      body: {
        survivor_id: selectedSurvivor.value,
        victim_ids: [...selectedVictims.value],
      },
    });

    if (!result.success) {
      throw new Error(result.message || "Database merge failed");
    }

    // Step 2: Rust write-back (after Go success)
    await invoke("write_back_merged_files");

    // Emit completion event
    emit("merge-completed", {
      survivorId: selectedSurvivor.value,
      victimIds: [...selectedVictims.value],
    });

    // Show success toast (or warning if file writes failed - handled by event listener)
    if (fileWriteErrors.value.length === 0) {
      showSuccessToast.value = true;
      // Auto-close after success
      autoCloseTimers.push(setTimeout(() => {
        layoutStore.closeMergeConsole();
      }, 1500));
    } else {
      showWarningToast.value = true;
    }
  } catch (e: unknown) {
    generalError.value = e instanceof Error ? e.message : String(e);
  } finally {
    isExecuting.value = false;
    isHolding.value = false;
    holdProgress.value = 0;
  }
}

/** Retry writing a specific file that failed */
async function retryFileWrite(file: string) {
  try {
    await invoke("retry_file_write", { filePath: file });
    // Remove from errors if successful
    fileWriteErrors.value = fileWriteErrors.value.filter((w) => w.file !== file);

    // If all errors cleared, show success
    if (fileWriteErrors.value.length === 0) {
      showWarningToast.value = false;
      showSuccessToast.value = true;
      autoCloseTimers.push(setTimeout(() => {
        layoutStore.closeMergeConsole();
      }, 1500));
    }
  } catch (e: unknown) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    // Update the error in the list
    const entry = fileWriteErrors.value.find((w) => w.file === file);
    if (entry) {
      entry.error = errorMsg;
    }
  }
}
</script>

<template>
  <div class="fixed inset-x-0 bottom-0 top-[36px] z-overlay bg-ms-deep flex flex-col">
    <!-- Header Bar -->
    <div class="h-12 flex items-center justify-between px-4 border-b border-ms-border bg-ms-carbon shrink-0">
      <div class="flex items-center gap-3">
        <span class="text-neon text-sm select-none">◆</span>
        <span class="font-bold text-slate-300 text-xs font-mono tracking-wider uppercase">
          MERGE STAGING AREA
        </span>
        <span class="text-[10px] text-slate-600 font-mono">概念坍缩引擎</span>
      </div>
      <button
        @click="layoutStore.closeMergeConsole()"
        class="w-8 h-8 flex items-center justify-center text-slate-500 hover:text-slate-300 hover:bg-ms-panel transition-colors"
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
      <div class="w-80 border-r border-ms-border flex flex-col bg-ms-void">
        <!-- Section Header -->
        <div class="h-10 flex items-center justify-between px-3 border-b border-ms-border shrink-0">
          <div class="flex items-center gap-2">
            <span class="text-[10px] font-mono uppercase tracking-widest text-amber-400">
              SACRIFICES
            </span>
            <span class="text-[10px] text-slate-600 font-mono">(祭品)</span>
          </div>
          <div class="flex items-center gap-1">
            <button
              @click="selectAllVictims"
              class="text-[9px] text-slate-500 hover:text-slate-300 px-1.5 py-0.5 transition-colors"
            >
              ALL
            </button>
            <button
              @click="clearAllVictims"
              class="text-[9px] text-slate-500 hover:text-slate-300 px-1.5 py-0.5 transition-colors"
            >
              CLEAR
            </button>
          </div>
        </div>

        <!-- Search -->
        <div class="px-3 py-2 border-b border-ms-border">
          <input
            v-model="victimSearchQuery"
            type="text"
            placeholder="搜索祭品..."
            class="w-full bg-ms-deep border border-ms-border px-2 py-1.5 text-xs text-slate-300 font-mono outline-none focus:border-amber-500/50 placeholder-slate-600"
          />
        </div>

        <!-- Victim Counter -->
        <div v-if="selectedVictims.length > 0" class="px-3 py-1.5 border-b border-ms-border/50 bg-amber-500/5">
          <span class="text-[10px] text-amber-400 font-mono">
            {{ selectedVictims.length }} 个待献祭
          </span>
        </div>

        <!-- Card List -->
        <div class="flex-1 overflow-y-auto custom-scrollbar">
          <label
            v-for="card in availableVictimCards"
            :key="card.id"
            class="flex items-center gap-2 px-3 py-2 cursor-pointer transition-colors border-b border-ms-border/30"
            :class="
              selectedVictims.includes(card.id)
                ? 'bg-amber-500/10 text-amber-300'
                : 'hover:bg-ms-panel/50 text-slate-400'
            "
          >
            <input
              type="checkbox"
              :checked="selectedVictims.includes(card.id)"
              @change="handleVictimToggle(card.id)"
              class="victim-checkbox"
            />
            <span class="text-xs font-mono truncate flex-1">
              {{ card.title || "无标题" }}
            </span>
          </label>

          <div v-if="availableVictimCards.length === 0" class="px-3 py-4 text-center">
            <span class="text-[10px] text-slate-600 italic font-mono">
              无可用卡片
            </span>
          </div>
        </div>
      </div>

      <!-- Column 2: BLAST RADIUS (Impact Preview) -->
      <div class="flex-1 flex flex-col bg-ms-panel">
        <!-- Section Header -->
        <div class="h-10 flex items-center px-3 border-b border-ms-border shrink-0">
          <span class="text-[10px] font-mono uppercase tracking-widest text-cyan-400">
            BLAST RADIUS
          </span>
          <span class="text-[10px] text-slate-600 font-mono ml-2">(爆炸半径)</span>
        </div>

        <!-- Content -->
        <div class="flex-1 overflow-y-auto custom-scrollbar p-4">
          <!-- Not ready state -->
          <div v-if="!canShowBlastRadius" class="h-full flex flex-col items-center justify-center text-center">
            <div class="text-slate-600 text-xs font-mono mb-2">
              请选择一个主节点和至少一个祭品
            </div>
            <div class="text-slate-700 text-[10px] font-mono">
              以预览合并影响范围
            </div>
          </div>

          <!-- Loading state -->
          <div v-else-if="isLoadingImpact" class="h-full flex items-center justify-center">
            <div class="flex items-center gap-2 text-slate-500">
              <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
              </svg>
              <span class="text-xs font-mono">计算影响范围...</span>
            </div>
          </div>

          <!-- Error state -->
          <div v-else-if="impactError" class="h-full flex items-center justify-center">
            <div class="text-center">
              <div class="text-red-400 text-xs font-mono mb-2">影响预览失败</div>
              <div class="text-slate-500 text-[10px] font-mono">{{ impactError }}</div>
            </div>
          </div>

          <!-- Impact data -->
          <div v-else-if="impactResult" class="space-y-4">
            <!-- Stats Grid -->
            <div class="grid grid-cols-2 gap-4">
              <div class="border border-ms-border bg-ms-deep p-4">
                <div class="text-[9px] text-slate-500 uppercase tracking-wider mb-1 font-mono">
                  受影响链接
                </div>
                <div class="text-2xl text-neon font-bold font-mono">
                  {{ impactResult.total_wikilinks }}
                </div>
              </div>
              <div class="border border-ms-border bg-ms-deep p-4">
                <div class="text-[9px] text-slate-500 uppercase tracking-wider mb-1 font-mono">
                  待修改文件
                </div>
                <div class="text-2xl text-amber-400 font-bold font-mono">
                  {{ impactResult.files_to_modify }}
                </div>
              </div>
            </div>

            <!-- File List -->
            <div v-if="impactResult.affected_files.length > 0">
              <div class="text-[9px] text-slate-500 uppercase tracking-wider mb-2 font-mono">
                受影响文件
              </div>
              <div class="space-y-1">
                <div
                  v-for="file in impactResult.affected_files"
                  :key="file.path"
                  class="flex items-center justify-between text-xs font-mono px-3 py-2 bg-ms-deep border border-ms-border"
                >
                  <span class="text-slate-300 truncate flex-1">{{ file.path }}</span>
                  <span class="text-amber-400/60 text-[10px] ml-3 shrink-0">
                    {{ file.link_count }} 链接
                  </span>
                </div>
              </div>
            </div>

            <!-- No files affected -->
            <div v-else class="text-center py-4">
              <span class="text-[10px] text-slate-600 italic font-mono">
                没有文件会被修改
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Column 3: SURVIVOR -->
      <div class="w-72 border-l border-ms-border flex flex-col bg-ms-void">
        <!-- Section Header -->
        <div class="h-10 flex items-center px-3 border-b border-ms-border shrink-0">
          <span class="text-[10px] font-mono uppercase tracking-widest text-emerald-400">
            SURVIVOR
          </span>
          <span class="text-[10px] text-slate-600 font-mono ml-2">(主节点)</span>
        </div>

        <!-- Search -->
        <div class="px-3 py-2 border-b border-ms-border">
          <input
            v-model="survivorSearchQuery"
            type="text"
            placeholder="搜索主节点..."
            class="w-full bg-ms-deep border border-ms-border px-2 py-1.5 text-xs text-slate-300 font-mono outline-none focus:border-emerald-500/50 placeholder-slate-600"
          />
        </div>

        <!-- Selected Survivor Display -->
        <div v-if="selectedSurvivor" class="px-3 py-2 border-b border-ms-border bg-emerald-500/5">
          <div class="text-[9px] text-emerald-400 font-mono mb-1">已选定</div>
          <div class="text-xs text-slate-300 font-mono truncate">
            {{ cards.find(c => c.id === selectedSurvivor)?.title || "无标题" }}
          </div>
        </div>

        <!-- Card List -->
        <div class="flex-1 overflow-y-auto custom-scrollbar">
          <label
            v-for="card in availableSurvivorCards"
            :key="card.id"
            class="flex items-center gap-2 px-3 py-2 cursor-pointer transition-colors border-b border-ms-border/30"
            :class="
              selectedSurvivor === card.id
                ? 'bg-emerald-500/10 text-emerald-300'
                : 'hover:bg-ms-panel/50 text-slate-400'
            "
          >
            <input
              type="radio"
              :value="card.id"
              :checked="selectedSurvivor === card.id"
              @change="handleSurvivorSelect(card.id)"
              class="survivor-radio"
            />
            <span class="text-xs font-mono truncate flex-1">
              {{ card.title || "无标题" }}
            </span>
          </label>

          <div v-if="availableSurvivorCards.length === 0" class="px-3 py-4 text-center">
            <span class="text-[10px] text-slate-600 italic font-mono">
              无可用卡片
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Bottom Action Bar -->
    <div class="h-16 border-t border-ms-border bg-ms-carbon flex items-center px-4 gap-4 shrink-0">
      <!-- Warning/Status Text -->
      <div class="flex-1 min-w-0">
        <div
          class="text-xs font-mono truncate"
          :class="canShowBlastRadius ? 'text-amber-400' : 'text-slate-600'"
        >
          <span v-if="canShowBlastRadius">⚠</span>
          {{ warningMessage }}
        </div>
        <div v-if="generalError" class="text-[10px] text-red-400 font-mono mt-0.5 truncate">
          {{ generalError }}
        </div>
      </div>

      <!-- Long-Press Initiate Button -->
      <button
        @pointerdown="handlePointerDown"
        @pointerup="handlePointerUp"
        @pointerleave="handlePointerLeave"
        :disabled="!canShowBlastRadius || isExecuting"
        class="relative h-10 min-w-[280px] overflow-hidden transition-colors select-none"
        :class="
          canShowBlastRadius && !isExecuting
            ? 'bg-red-500/10 border border-red-500/30 cursor-pointer'
            : 'bg-ms-surface/50 border border-ms-border cursor-not-allowed opacity-50'
        "
      >
        <!-- Progress Bar Background -->
        <div
          v-if="isHolding"
          class="absolute inset-0 bg-red-500/30 transition-none"
          :style="{ width: progressWidth }"
        />

        <!-- Button Text -->
        <div class="relative z-10 flex items-center justify-center gap-2 px-4">
          <span
            class="text-xs font-mono uppercase tracking-wider"
            :class="canShowBlastRadius && !isExecuting ? 'text-red-400' : 'text-slate-600'"
          >
            <template v-if="isExecuting">执行中...</template>
            <template v-else-if="isHolding">按住 {{ Math.ceil((100 - holdProgress) / 100 * 3) }}s — 启动坍缩</template>
            <template v-else>按住 3 秒 — 启动坍缩</template>
          </span>
        </div>
      </button>
    </div>

    <!-- Success Toast -->
    <Transition name="ms-slide-up">
      <div
        v-if="showSuccessToast"
        class="fixed bottom-20 right-4 z-toast bg-emerald-500/10 border border-emerald-500/30 px-4 py-2 font-mono text-xs text-emerald-400"
      >
        合并完成 — 所有文件写入成功
      </div>
    </Transition>

    <!-- Warning Toast (file write failures) -->
    <Transition name="ms-slide-up">
      <div
        v-if="showWarningToast"
        class="fixed bottom-20 right-4 z-toast bg-amber-500/10 border border-amber-500/30 px-4 py-3 font-mono text-xs text-amber-400 max-w-md"
      >
        <div class="mb-2">数据库合并成功，但文件本地覆写受阻</div>
        <div v-for="w in fileWriteErrors" :key="w.file" class="flex items-center gap-2 mt-1">
          <span class="text-slate-500 truncate max-w-48">{{ w.file }}</span>
          <button
            @click="retryFileWrite(w.file)"
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
.custom-scrollbar::-webkit-scrollbar {
  width: 3px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #222;
  border-radius: 1px;
}

/* Custom checkbox */
.victim-checkbox {
  appearance: none;
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  border: 1px solid #333;
  background: #0d0d0d;
  cursor: pointer;
  transition: all 0.15s ease;
  border-radius: 2px;
  flex-shrink: 0;
}

.victim-checkbox:checked {
  background: #f59e0b;
  border-color: #f59e0b;
}

.victim-checkbox:checked::after {
  content: "✓";
  display: block;
  font-size: 9px;
  color: #0d0d0d;
  text-align: center;
  line-height: 10px;
  font-weight: bold;
}

.victim-checkbox:hover {
  border-color: #f59e0b;
}

/* Custom radio */
.survivor-radio {
  appearance: none;
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  border: 1px solid #333;
  background: #0d0d0d;
  cursor: pointer;
  transition: all 0.15s ease;
  border-radius: 50%;
  flex-shrink: 0;
}

.survivor-radio:checked {
  background: #10b981;
  border-color: #10b981;
}

.survivor-radio:checked::after {
  content: "";
  display: block;
  width: 4px;
  height: 4px;
  background: #0d0d0d;
  border-radius: 50%;
  margin: 3px;
}

.survivor-radio:hover {
  border-color: #10b981;
}
</style>
