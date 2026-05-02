// 用途：编辑器核心状态，管理当前卡片内容、AST 解析和自动保存
import { defineStore } from "pinia";
import { ref, shallowRef, watch } from "vue";
import type { AstNode, CardMeta, DocAnalysis, ParsedDocument } from "@memory-stream/types";
import { parseLiveMarkdown, saveDocumentIO } from "@/bridge/invoke";
import debounce from "lodash-es/debounce";

export const useEditorStore = defineStore("editor", () => {
  const currentUuid = ref<string | null>(null);
  const currentMeta = ref<CardMeta | null>(null);
  const rawContent = ref("");
  const currentAst = shallowRef<AstNode | null>(null);
  const currentAnalysis = shallowRef<DocAnalysis | null>(null);
  const isSaving = ref(false);
  const realTimeStats = ref({ words: 0, chars: 0, lines: 0 });
  const lastSaveError = ref("");
  const isHydrating = ref(false);

  // ⚡ Gear 1: visual render (150ms) — body only
  const triggerLiveRender = debounce(async (text: string) => {
    try {
      const ast = await parseLiveMarkdown(text);
      currentAst.value = ast as AstNode;
    } catch (e) {
      console.error("[live-render] parse failed:", e);
    }
  }, 150);

  // 💾 Gear 2: disk IO (1500ms) — merge YAML back before saving
  const triggerDiskSave = debounce(async (text: string) => {
    if (!currentUuid.value || !currentMeta.value || isHydrating.value) return;
    isSaving.value = true;
    lastSaveError.value = "";
    try {
      const updatedMeta = await saveDocumentIO(currentMeta.value, text);
      currentMeta.value = updatedMeta;
    } catch (e) {
      lastSaveError.value = e instanceof Error ? e.message : String(e);
      console.error("[disk-save] failed:", e);
    } finally {
      isSaving.value = false;
    }
  }, 1500);

  // Core watch: every keystroke triggers both clutches
  watch(rawContent, (text) => {
    if (!currentUuid.value || isHydrating.value) return;
    triggerLiveRender(text);
    triggerDiskSave(text);
    updateRealTimeStats(text);
  });

  function updateRealTimeStats(text: string) {
    const lines = text.split("\n").length;
    const chars = text.length;
    const cnMatches = text.match(/[一-龥㐀-䶿]/g)?.length ?? 0;
    const enMatches = text.match(/[a-zA-Z0-9]+/g)?.length ?? 0;
    realTimeStats.value = { words: cnMatches + enMatches, chars, lines };
  }

  // Public: load a parsed article for editing
  function loadArticle(uuid: string, document: ParsedDocument) {
    currentUuid.value = uuid;
    currentMeta.value = document.meta;
    isHydrating.value = true;
    rawContent.value = document.content;
    currentAst.value = document.ast;
    currentAnalysis.value = document.analysis;
    isHydrating.value = false;
    updateRealTimeStats(document.content);
  }

  function clear() {
    currentUuid.value = null;
    currentMeta.value = null;
    rawContent.value = "";
    currentAst.value = null;
    currentAnalysis.value = null;
    realTimeStats.value = { words: 0, chars: 0, lines: 0 };
    isSaving.value = false;
    lastSaveError.value = "";
    isHydrating.value = false;
    triggerLiveRender.cancel();
    triggerDiskSave.cancel();
  }

  async function manualSave(): Promise<boolean> {
    if (!currentUuid.value || !currentMeta.value || isHydrating.value) return false;
    triggerDiskSave.cancel();
    isSaving.value = true;
    lastSaveError.value = "";
    try {
      const updatedMeta = await saveDocumentIO(currentMeta.value, rawContent.value);
      currentMeta.value = updatedMeta;
      return true;
    } catch (e) {
      lastSaveError.value = e instanceof Error ? e.message : String(e);
      console.error("[manual-save] failed:", e);
      return false;
    } finally {
      isSaving.value = false;
    }
  }

  return {
    currentUuid,
    currentMeta,
    rawContent,
    currentAst,
    currentAnalysis,
    realTimeStats,
    isSaving,
    lastSaveError,
    loadArticle,
    clear,
    manualSave,
  };
});
