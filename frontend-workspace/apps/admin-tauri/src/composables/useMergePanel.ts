/**
 * useMergePanel — Merge panel state and logic
 *
 * Manages card selection, impact preview fetching, and merge execution.
 */

import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { MergePreview } from "@memory-stream/types/ipc";

export interface CardData {
  id: string;
  title: string;
}

interface MergeResult {
  success: boolean;
  survivorId?: string;
  victimIds?: string[];
}

export function useMergePanel(cards: CardData[]) {
  // State
  const selectedSurvivor = ref<string | null>(null);
  const selectedVictims = ref<string[]>([]);
  const victimSearchQuery = ref("");
  const survivorSearchQuery = ref("");

  // Impact preview data
  const impactResult = ref<MergePreview | null>(null);
  const isLoadingImpact = ref(false);
  const impactError = ref<string | null>(null);

  // Toast states
  const showSuccessToast = ref(false);
  const showWarningToast = ref(false);
  const fileWriteErrors = ref<Array<{ file: string; error: string }>>([]);
  const generalError = ref<string | null>(null);

  // Auto-close timers
  let autoCloseTimers: ReturnType<typeof setTimeout>[] = [];

  // Computed
  const availableVictimCards = computed(() => {
    const filtered = selectedSurvivor.value
      ? cards.filter((c) => c.id !== selectedSurvivor.value)
      : cards;

    if (!victimSearchQuery.value) return filtered;
    const q = victimSearchQuery.value.toLowerCase();
    return filtered.filter((c) => c.title.toLowerCase().includes(q));
  });

  const availableSurvivorCards = computed(() => {
    if (!survivorSearchQuery.value) return cards;
    const q = survivorSearchQuery.value.toLowerCase();
    return cards.filter((c) => c.title.toLowerCase().includes(q));
  });

  const canShowBlastRadius = computed(() => {
    return selectedSurvivor.value !== null && selectedVictims.value.length > 0;
  });

  const warningMessage = computed(() => {
    if (!selectedSurvivor.value) return "请选择一个主节点";
    if (selectedVictims.value.length === 0) return "请选择至少一个祭品节点";
    const n = selectedVictims.value.length;
    const m = impactResult.value?.files_to_modify ?? "?";
    return `将永久删除 ${n} 个节点，修改 ${m} 个文件`;
  });

  // Lifecycle
  let unlistenFileWriteFailed: (() => void) | null = null;

  onMounted(async () => {
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

  // Handlers
  function handleSurvivorSelect(cardId: string) {
    selectedSurvivor.value = cardId;
    if (selectedVictims.value.includes(cardId)) {
      selectedVictims.value = selectedVictims.value.filter((id) => id !== cardId);
    }
  }

  function handleVictimToggle(cardId: string) {
    const index = selectedVictims.value.indexOf(cardId);
    if (index > -1) {
      selectedVictims.value = selectedVictims.value.filter((id) => id !== cardId);
    } else {
      selectedVictims.value = [...selectedVictims.value, cardId];
    }
  }

  function selectAllVictims() {
    const newVictims = new Set(selectedVictims.value);
    availableVictimCards.value.forEach((c) => newVictims.add(c.id));
    selectedVictims.value = Array.from(newVictims);
  }

  function clearAllVictims() {
    selectedVictims.value = [];
  }

  async function fetchImpactPreview() {
    if (!selectedSurvivor.value || selectedVictims.value.length === 0) return;

    isLoadingImpact.value = true;
    impactError.value = null;

    const victimTitles = selectedVictims.value
      .map((id) => cards.find((c) => c.id === id)?.title)
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

  async function executeMerge(): Promise<MergeResult> {
    if (!selectedSurvivor.value || selectedVictims.value.length === 0) {
      return { success: false };
    }

    generalError.value = null;
    fileWriteErrors.value = [];
    showSuccessToast.value = false;
    showWarningToast.value = false;

    try {
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

      await invoke("write_back_merged_files");

      if (fileWriteErrors.value.length === 0) {
        showSuccessToast.value = true;
      } else {
        showWarningToast.value = true;
      }

      return {
        success: true,
        survivorId: selectedSurvivor.value as string,
        victimIds: [...selectedVictims.value] as string[],
      };
    } catch (e: unknown) {
      generalError.value = e instanceof Error ? e.message : String(e);
      return { success: false };
    }
  }

  async function retryFileWrite(file: string) {
    try {
      await invoke("retry_file_write", { filePath: file });
      fileWriteErrors.value = fileWriteErrors.value.filter((w) => w.file !== file);

      if (fileWriteErrors.value.length === 0) {
        showWarningToast.value = false;
        showSuccessToast.value = true;
      }
    } catch (e: unknown) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      const entry = fileWriteErrors.value.find((w) => w.file === file);
      if (entry) {
        entry.error = errorMsg;
      }
    }
  }

  function scheduleAutoClose(callback: () => void, delay: number) {
    const timer = setTimeout(() => {
      callback();
      autoCloseTimers = autoCloseTimers.filter(t => t !== timer);
    }, delay);
    autoCloseTimers.push(timer);
  }

  return {
    // State
    selectedSurvivor,
    selectedVictims,
    victimSearchQuery,
    survivorSearchQuery,
    impactResult,
    isLoadingImpact,
    impactError,
    showSuccessToast,
    showWarningToast,
    fileWriteErrors,
    generalError,

    // Computed
    availableVictimCards,
    availableSurvivorCards,
    canShowBlastRadius,
    warningMessage,

    // Methods
    handleSurvivorSelect,
    handleVictimToggle,
    selectAllVictims,
    clearAllVictims,
    fetchImpactPreview,
    executeMerge,
    retryFileWrite,
    scheduleAutoClose,
  };
}
