// 用途：文件树和分类结构状态管理，维护卡片索引和标题匹配
import { defineStore } from "pinia";
import { ref, shallowRef, computed } from "vue";
import type { TreeNode } from "@memory-stream/core";
import * as vaultService from "@/services/vault";
import { listen } from "@tauri-apps/api/event";

export interface TitleMatch {
  uuid: string
  category: string
}

export const useTreeStore = defineStore("tree", () => {
  const categories = shallowRef<TreeNode[]>([]);
  const expandedIds = ref(new Set<string>());
  const activeCardUuid = ref<string | null>(null);
  const loading = ref(false);

  let refreshTimer: ReturnType<typeof setTimeout> | null = null;
  let unlisten: (() => void) | null = null;
  let initDone = false;

  async function loadTree() {
    loading.value = true;
    try {
      categories.value = await vaultService.scanVault();
    } catch (error) {
      categories.value = [];
      console.error("[tree] load failed:", error);
    } finally {
      loading.value = false;
    }
  }

  function scheduleRefresh() {
    if (refreshTimer) clearTimeout(refreshTimer);
    refreshTimer = setTimeout(() => loadTree(), 300);
  }

  function toggleExpand(id: string) {
    if (expandedIds.value.has(id)) {
      expandedIds.value.delete(id);
    } else {
      expandedIds.value.add(id);
    }
  }

  function setActive(uuid: string | null) {
    activeCardUuid.value = uuid;
  }

  async function init() {
    if (initDone) return;
    initDone = true;
    const fn = await listen("fs:change", () => {
      scheduleRefresh();
    });
    unlisten = fn;
  }

  function dispose() {
    if (unlisten) {
      unlisten();
      unlisten = null;
    }
    if (refreshTimer) {
      clearTimeout(refreshTimer);
      refreshTimer = null;
    }
    initDone = false;
  }

  // Auto-init on first store usage
  init();

  const titleIndex = computed(() => {
    const index = new Map<string, TitleMatch[]>()
    function walk(nodes: TreeNode[], parentName: string) {
      for (const node of nodes) {
        if (node.is_dir) {
          walk(node.children, node.name)
        } else {
          const list = index.get(node.name) ?? []
          list.push({ uuid: node.id, category: parentName })
          index.set(node.name, list)
        }
      }
    }
    walk(categories.value, '')
    return index
  })

  function lookupByTitle(title: string): TitleMatch[] {
    return titleIndex.value.get(title) ?? []
  }

  return {
    categories,
    expandedIds,
    activeCardUuid,
    loading,
    loadTree,
    toggleExpand,
    setActive,
    dispose,
    lookupByTitle,
  };
});
