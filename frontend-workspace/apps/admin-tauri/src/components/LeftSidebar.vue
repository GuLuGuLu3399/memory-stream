<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useKnowledgeStore } from "../stores/knowledge";
import { useLayoutStore } from "../stores/layout";
import { storeToRefs } from "pinia";
import { Compass, ScrollText } from "lucide-vue-next";
import { hexForKey } from "../composables/useCategoryTheme";

const store = useKnowledgeStore();
const layoutStore = useLayoutStore();
const { isLeftDrawerOpen } = storeToRefs(layoutStore);
const {
  filteredOrphans,
  filteredRecent,
  activeCard,
  categories,
  selectedCategoryId,
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

onMounted(() => {
  store.loadCategories();
  store.loadOrphans();
  store.loadRecent();
});

function contentPreview(content: string): string {
  if (!content) return "";
  const firstLine = content.split("\n").find((l) => l.trim()) || "";
  const cleaned = firstLine
    .replace(/\[\[([^\]]+)\]\]/g, "$1")
    .replace(/\*\*([^*]+)\*\*/g, "$1")
    .replace(/\*([^*]+)\*/g, "$1")
    .replace(/`([^`]+)`/g, "$1")
    .replace(/#{1,6}\s/g, "")
    .trim();
  if (cleaned.length <= 40) return cleaned;
  const truncated = cleaned.slice(0, 40);
  const lastSpace = truncated.lastIndexOf(" ");
  return (lastSpace > 20 ? truncated.slice(0, lastSpace) : truncated) + "…";
}

async function handleDelete(cardId: string, title: string) {
  const { confirm } = await import("../composables/useConfirmDialog").then(m => m.useConfirmDialog());
  const ok = await confirm(`确定要删除「${title || "无标题"}」吗？此操作不可撤销。`, {
    title: "删除卡片",
    confirmText: "删除",
    danger: true,
  });
  if (ok) store.deleteCard(cardId);
}
</script>

<template>
  <aside class="shrink-0 bg-ms-void border-r border-ms-border flex flex-col overflow-hidden transition-[width] duration-200 ease-out"
    :class="collapsed ? 'w-[40px]' : 'w-[280px]'">

    <!-- ═══ SPINE MODE (collapsed) ═══ -->
    <template v-if="collapsed">
      <div class="flex-1 flex flex-col items-center pt-3 gap-1.5 overflow-y-auto custom-scrollbar">
        <button @click="store.selectedCategoryId = null"
          class="w-6 h-6 rounded-sm flex items-center justify-center text-[8px] font-mono font-bold transition-colors"
          :class="selectedCategoryId === null ? 'bg-neon/20 text-neon' : 'bg-ms-panel text-slate-600 hover:text-slate-400'"
          title="ALL">
          A
        </button>
        <button v-for="cat in categories" :key="cat.id"
          @click="store.selectedCategoryId = cat.id"
          class="w-6 h-6 rounded-sm flex items-center justify-center text-[8px] font-mono font-bold transition-colors"
          :style="selectedCategoryId === cat.id && hexForKey(cat.theme_color)
            ? { backgroundColor: hexForKey(cat.theme_color)! + '33', color: hexForKey(cat.theme_color)! }
            : undefined"
          :class="selectedCategoryId === cat.id
            ? (hexForKey(cat.theme_color) ? '' : 'bg-neon/20 text-neon')
            : 'bg-ms-panel text-slate-600 hover:text-slate-400'"
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
          <div class="w-5 h-5 bg-neon/10 border border-neon/30 flex items-center justify-center text-neon text-2xs font-bold font-display select-none">M</div>
          <span class="text-slate-600 font-mono text-2xs tracking-spine uppercase">Memory_Bay</span>
        </div>
        <span class="text-3xs text-slate-700 font-mono border border-ms-border px-1.5 py-0.5 rounded-sm select-none">Ctrl+K</span>
      </div>

      <!-- Category Ribbon -->
      <div v-if="categories.length > 0" class="flex px-3 py-1.5 gap-3 overflow-x-auto border-b border-ms-border/50 no-scrollbar">
        <button @click="store.selectedCategoryId = null"
          class="shrink-0 font-mono text-2xs tracking-wider uppercase pb-0.5 border-b-2 transition-colors"
          :class="selectedCategoryId === null ? 'text-neon border-b-neon' : 'text-slate-600 border-b-transparent hover:text-slate-400'">
          ALL
        </button>
        <button v-for="cat in categories" :key="cat.id" @click="store.selectedCategoryId = cat.id"
          class="shrink-0 font-mono text-2xs tracking-wider uppercase pb-0.5 border-b-2 transition-all"
          :style="(selectedCategoryId === cat.id && hexForKey(cat.theme_color))
            ? { color: hexForKey(cat.theme_color)!, borderBottomColor: hexForKey(cat.theme_color)! }
            : undefined"
          :class="selectedCategoryId === cat.id
            ? (hexForKey(cat.theme_color) ? '' : 'text-neon border-b-neon')
            : 'text-slate-600 border-b-transparent hover:text-slate-400'">
          {{ cat.name }}
        </button>
      </div>

      <!-- View Tabs -->
      <div class="flex px-3 pt-2 gap-0.5">
        <button v-for="tab in tabs" :key="tab.key" @click="activeTab = tab.key"
          class="flex-1 text-2xs py-1.5 transition-all text-center font-mono tracking-wider" :class="activeTab === tab.key
            ? 'text-neon border-b border-neon/50'
            : 'text-slate-600 border-b border-transparent hover:text-slate-400'
            ">
          <component :is="tab.icon" :size="10" class="inline-block align-middle mr-0.5" />{{ tab.label }}
        </button>
      </div>

      <div class="flex-1 overflow-y-auto custom-scrollbar">
        <!-- Tab: 孤岛雷达 -->
        <div v-if="activeTab === 'orphans'" class="p-2 space-y-2">
          <div v-for="(card, idx) in filteredOrphans" :key="card.id" @click="store.loadAndActivateCard(card.id)"
            class="levitation-card group relative p-2.5 rounded-sm cursor-pointer transition-all border-l-2"
            :class="activeCard?.id === card.id
              ? 'bg-neon/5 text-neon border-l-neon'
              : 'bg-transparent hover:bg-ms-panel/50 text-slate-400 border-l-slate-700 hover:border-l-slate-500'
              "
            :style="{ '--delay': `${idx * 0.18}s` }">
            <div class="flex items-center justify-between">
              <div class="truncate text-xs flex-1 mr-2">
                {{ card.title || "无标题" }}
                <span v-if="card.title && !card.content" class="text-3xs text-slate-700 ml-1">∅</span>
              </div>
              <button @click.stop="handleDelete(card.id, card.title)"
                class="shrink-0 opacity-0 group-hover:opacity-100 text-slate-600 hover:text-red-400 transition-all px-0.5 rounded-sm"
                title="删除卡片">
                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            <div v-if="card.content" class="truncate text-2xs mt-0.5 font-mono" :class="activeCard?.id === card.id
              ? 'text-neon/40'
              : 'text-slate-600'
              ">
              {{ contentPreview(card.content) }}
            </div>
          </div>
          <div v-if="filteredOrphans.length === 0" class="text-2xs text-slate-700 italic py-2 font-mono">
            暂无孤岛卡片
          </div>
        </div>

        <!-- Tab: 全部卡片 -->
        <div v-if="activeTab === 'all'" class="p-2 space-y-2">
          <div v-for="(card, idx) in allCards" :key="card.id" @click="store.loadAndActivateCard(card.id)"
            class="levitation-card group relative p-2.5 rounded-sm cursor-pointer transition-all"
            :class="activeCard?.id === card.id
              ? 'bg-neon/5 text-neon border-l-2 border-l-neon'
              : 'bg-transparent hover:bg-ms-panel/50 text-slate-400'
              "
            :style="{ '--delay': `${idx * 0.18}s` }">
            <div class="flex items-center justify-between">
              <div class="truncate text-xs flex-1 mr-2">
                {{ card.title || "无标题" }}
                <span v-if="card.title && !card.content" class="text-3xs text-slate-700 ml-1">∅</span>
              </div>
              <span v-if="card.category_id && categoryMap.get(card.category_id)"
                class="shrink-0 text-3xs bg-ms-deep text-slate-500 px-1.5 py-0.5 rounded-sm font-mono mr-1">
                {{ categoryMap.get(card.category_id) }}
              </span>
              <button @click.stop="handleDelete(card.id, card.title)"
                class="shrink-0 opacity-0 group-hover:opacity-100 text-slate-600 hover:text-red-400 transition-all px-0.5 rounded-sm"
                title="删除卡片">
                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            <div v-if="card.content" class="truncate text-2xs mt-0.5 font-mono" :class="activeCard?.id === card.id
              ? 'text-neon/40'
              : 'text-slate-600'
              ">
              {{ contentPreview(card.content) }}
            </div>
          </div>
          <div v-if="allCards.length === 0" class="text-2xs text-slate-700 italic py-2 font-mono">
            暂无卡片
          </div>
        </div>
      </div>
    </template>
  </aside>
</template>

<style scoped>
@keyframes magnetic-float {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-5px);
  }
}

.levitation-card {
  animation: magnetic-float 5s ease-in-out infinite;
  animation-delay: var(--delay);
  transition: border-color 0.25s, box-shadow 0.25s, background-color 0.2s;
}

.levitation-card:hover {
  animation-play-state: paused;
  transform: translateY(-2px);
  box-shadow: 0 6px 20px -6px rgba(0, 229, 255, 0.12);
}

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
