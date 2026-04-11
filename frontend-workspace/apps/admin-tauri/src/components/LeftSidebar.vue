<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useKnowledgeStore } from "../stores/knowledge";
import { useLayoutStore } from "../stores/layout";
import { storeToRefs } from "pinia";
import { Compass, ScrollText, Search } from "lucide-vue-next";
import { hexForKey } from "../composables/useCategoryTheme";
import SidebarCardItem from "./sidebar/SidebarCardItem.vue";
import CategoryRibbon from "./sidebar/CategoryRibbon.vue";
import TabSelector from "./sidebar/TabSelector.vue";

const store = useKnowledgeStore();
const layoutStore = useLayoutStore();
const { isLeftDrawerOpen } = storeToRefs(layoutStore);
const {
  filteredOrphans,
  filteredRecent,
  activeCard,
  categories,
  selectedCategoryId,
  searchQuery,
} = storeToRefs(store);

const collapsed = computed(() => !isLeftDrawerOpen.value);

// ===== View Tabs =====
type ViewTab = "orphans" | "all";
const activeTab = ref<ViewTab>("orphans");

const tabs: { key: ViewTab; label: string; icon: typeof Compass }[] = [
  { key: "orphans", label: "孤岛", icon: Compass },
  { key: "all", label: "全部", icon: ScrollText },
];

const allCards = computed(() => {
  const seen = new Set<string>();
  const result = [...filteredRecent.value, ...filteredOrphans.value];
  return result.filter((c) => {
    if (seen.has(c.id)) return false;
    seen.add(c.id);
    return true;
  });
});

const categoryMap = computed(() => {
  const map = new Map<number, string>();
  for (const cat of categories.value) {
    map.set(cat.id, cat.name);
  }
  return map;
});

const displayedCards = computed(() => {
  return activeTab.value === "orphans" ? filteredOrphans.value : allCards.value;
});

onMounted(() => {
  store.loadCategories();
  store.loadOrphans();
  store.loadRecent();
});

async function handleDelete(cardId: string, title: string) {
  const { confirm } = await import("../composables/useConfirmDialog").then(m => m.useConfirmDialog());
  const ok = await confirm(`确定要删除「${title || "无标题"}」吗？此操作不可撤销。`, {
    title: "删除卡片",
    confirmText: "删除",
    danger: true,
  });
  if (ok) store.deleteCard(cardId);
}

function handleCardSelect(cardId: string) {
  store.loadAndActivateCard(cardId);
}

function handleCategorySelect(categoryId: number | null) {
  store.selectedCategoryId = categoryId;
}
</script>

<template>
  <aside class="shrink-0 bg-ms-void border-r border-ms-border flex flex-col overflow-hidden transition-[width] duration-200 ease-out"
    :class="collapsed ? 'w-[40px]' : 'w-[280px]'">

    <!-- ═══ SPINE MODE (collapsed) ═══ -->
    <template v-if="collapsed">
      <div class="flex-1 flex flex-col items-center pt-3 gap-1.5 overflow-y-auto custom-scrollbar brushed-metal">
        <button @click="handleCategorySelect(null)"
          class="w-6 h-6 rounded-sm flex items-center justify-center text-[8px] font-mono font-bold transition-all"
          :class="selectedCategoryId === null ? 'bg-brass/20 text-brass shadow-[1px_1px_0_0_rgba(0,0,0,0.4)]' : 'bg-ms-panel text-slate-600 hover:text-slate-400 hover:shadow-[1px_1px_0_0_rgba(0,0,0,0.3)]'"
          title="ALL">
          A
        </button>
        <button v-for="cat in categories" :key="cat.id"
          @click="handleCategorySelect(cat.id)"
          class="w-6 h-6 rounded-sm flex items-center justify-center text-[8px] font-mono font-bold transition-all"
          :style="selectedCategoryId === cat.id && hexForKey(cat.theme_color)
            ? { backgroundColor: hexForKey(cat.theme_color)! + '33', color: hexForKey(cat.theme_color)! }
            : undefined"
          :class="selectedCategoryId === cat.id
            ? (hexForKey(cat.theme_color) ? 'shadow-[1px_1px_0_0_rgba(0,0,0,0.4)]' : 'bg-brass/20 text-brass shadow-[1px_1px_0_0_rgba(0,0,0,0.4)]')
            : 'bg-ms-panel text-slate-600 hover:text-slate-400 hover:shadow-[1px_1px_0_0_rgba(0,0,0,0.3)]'"
          :title="cat.name">
          {{ cat.name.charAt(0).toUpperCase() }}
        </button>
      </div>
      <div class="h-6 flex items-center justify-center border-t border-ms-border/50">
        <span class="text-[7px] text-slate-700 font-mono">⌘K</span>
      </div>
    </template>

    <!-- ═══ FULL MODE (expanded) ═══ -->
    <template v-else>
      <!-- Ctrl+K hint bar -->
      <div class="h-10 flex items-center justify-between px-3 border-b border-ms-border shrink-0">
        <div class="flex items-center space-x-2">
          <div class="w-5 h-5 bg-ms-deep border border-brass/30 flex items-center justify-center text-brass text-2xs font-bold font-display select-none shadow-[1px_1px_0_0_rgba(0,0,0,0.4)]">M</div>
          <span class="text-slate-600 font-mono text-2xs tracking-spine uppercase">Memory_Bay</span>
        </div>
        <span class="text-3xs text-slate-700 font-mono border border-ms-border px-1.5 py-0.5 rounded-sm select-none shadow-[1px_1px_0_0_rgba(0,0,0,0.4)]">Ctrl+K</span>
      </div>

      <!-- Category Ribbon -->
      <CategoryRibbon
        :categories="categories"
        :selected-category-id="selectedCategoryId"
        @select="handleCategorySelect"
      />

      <!-- Search Input -->
      <div class="px-3 pt-2 pb-1">
        <div class="relative">
          <Search :size="12" class="absolute left-2 top-1/2 -translate-y-1/2 text-slate-600" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索卡片..."
            class="w-full pl-7 pr-2 py-1.5 text-2xs font-mono terminal-input-engrave text-slate-300 placeholder:text-slate-700"
          />
        </div>
      </div>

      <!-- View Tabs -->
      <TabSelector
        :tabs="tabs"
        :active-tab="activeTab"
        @update:active-tab="(val: any) => activeTab = val as ViewTab"
      />

      <!-- Card List -->
      <div class="flex-1 overflow-y-auto custom-scrollbar">
        <div class="p-2 space-y-2">
          <SidebarCardItem
            v-for="(card, idx) in displayedCards"
            :key="card.id"
            :card="card"
            :is-selected="activeCard?.id === card.id"
            :category-info="categoryMap"
            :index="idx"
            @select="handleCardSelect"
            @delete="handleDelete"
          />
          <div v-if="displayedCards.length === 0" class="text-2xs text-slate-700 italic py-2 font-mono">
            {{ searchQuery ? '无匹配结果' : (activeTab === 'orphans' ? '暂无孤岛卡片' : '暂无卡片') }}
          </div>
        </div>
      </div>
    </template>
  </aside>
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

.no-scrollbar::-webkit-scrollbar {
  display: none;
}

.no-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
