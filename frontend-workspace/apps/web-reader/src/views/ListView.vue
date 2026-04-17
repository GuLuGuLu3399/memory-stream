<script setup lang="ts">
/**
 * ListView — 血脉脊骨 (Blood Spine)
 *
 * 三列网格：铭文日期 | 灵穴脊柱 | 符咒卡片
 * - 单行描述 + 大间距：呼吸感的纵向节奏
 * - 隐藏式时间：仅 hover 节点时显现
 * - 节点分级：Genesis 双环 / 普通 2px 微点
 * - Vim 风格键盘导航：j/k 键上下移动
 * - Shift+Click 打开 ZenReader
 */
import { ref, onMounted, computed, watchEffect, onUnmounted } from 'vue';
import { storeToRefs } from 'pinia';
import { useVirtualizer } from '@tanstack/vue-virtual';
import { useCards } from '../composables/useCards';
import { useGraphStore } from '../store/useGraphStore';
import { useBreakpoints } from '../composables/useBreakpoints';
import SkeletonLine from '../components/ui/SkeletonLine.vue';
import StatsWidget from '../components/StatsWidget.vue';
import EmptyState from '@memory-stream/ui-shared/components/EmptyState.vue';
import ListCardRow from './list/ListCardRow.vue';
import SpineNode from './list/SpineNode.vue';
import DateColumn from './list/DateColumn.vue';
import ListViewHeader from './list/ListViewHeader.vue';

const store = useGraphStore();
const { sortBy, selectedId, categoryFilter } = storeToRefs(store);
const { cardIndex, loadIndex, loading } = useCards();
const { isMobile } = useBreakpoints();

const searchQuery = ref('');
const focusedIndex = ref<number>(-1);
const mobileGap = 20;
const UNCATEGORIZED_FILTER = '__uncategorized__';

const categoryOptions = computed(() => {
  const groups = new Map<string, { id: string; name: string; count: number }>();

  for (const card of cardIndex.value) {
    if (card.category_id == null) {
      const prev = groups.get(UNCATEGORIZED_FILTER);
      groups.set(UNCATEGORIZED_FILTER, {
        id: UNCATEGORIZED_FILTER,
        name: '未分类',
        count: (prev?.count ?? 0) + 1,
      });
      continue;
    }

    const id = String(card.category_id);
    const prev = groups.get(id);
    groups.set(id, {
      id,
      name: card.category_name || `分类 ${id}`,
      count: (prev?.count ?? 0) + 1,
    });
  }

  return Array.from(groups.values()).sort((a, b) => a.name.localeCompare(b.name, 'zh-CN'));
});

function onCategoryFilterChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value;
  store.setCategoryFilter(value === '' ? null : value);
}

interface CardRow {
  type: 'card';
  data: {
    id: string;
    title: string;
    excerpt: string;
    hot_score: number;
    updated_at: string;
    relation: 'sequence' | 'reference';
  };
  isFirstInDay: boolean;
  dateLabel: string;
}

const filteredCards = computed(() => {
  let cards = cardIndex.value;

  if (categoryFilter.value) {
    if (categoryFilter.value === UNCATEGORIZED_FILTER) {
      cards = cards.filter((card) => card.category_id == null);
    } else {
      cards = cards.filter((card) => String(card.category_id ?? '') === categoryFilter.value);
    }
  }

  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase();
    cards = cards.filter(
      (card) =>
        card.title.toLowerCase().includes(q) ||
        (card.excerpt && card.excerpt.toLowerCase().includes(q)),
    );
  }
  if (sortBy.value === 'hot') {
    cards = [...cards].sort((a, b) => (b.hot_score || 0) - (a.hot_score || 0));
  } else {
    cards = [...cards].sort(
      (a, b) => Date.parse(b.updated_at || '') - Date.parse(a.updated_at || ''),
    );
  }
  return cards;
});

const flatItems = computed<CardRow[]>(() => {
  const items: CardRow[] = [];
  let lastDate = '';

  for (const card of filteredCards.value) {
    let isFirstInDay = false;
    let dateLabel = '';

    if (sortBy.value === 'updated') {
      const dateStr = card.updated_at?.slice(0, 10) || 'unknown';
      if (dateStr !== lastDate) {
        isFirstInDay = true;
        dateLabel = formatDateLabel(dateStr);
        lastDate = dateStr;
      }
    }

    items.push({
      type: 'card',
      data: card,
      isFirstInDay,
      dateLabel,
    });
  }
  return items;
});

function formatDateLabel(dateStr: string): string {
  if (dateStr === 'unknown') return '未知';
  const d = new Date(dateStr + 'T00:00:00');
  const month = d.getMonth() + 1;
  const day = d.getDate();
  return `${month}.${day}`;
}

function selectCard(id: string) {
  store.selectNode(id);
  focusedIndex.value = flatItems.value.findIndex(item => item.data.id === id);
}

/** Type-safe row accessor — eliminates repeated `as CardRow` casts in template */
function getRow(index: number): CardRow | undefined {
  return flatItems.value[index] as CardRow | undefined;
}

function handleCardClick(index: number, event: MouseEvent | KeyboardEvent) {
  const item = flatItems.value[index];
  if (!item?.data) return;

  // Shift+Click opens ZenReader
  if (event.shiftKey) {
    store.toggleZenMode();
  }

  selectCard(item.data.id);
}

function toggleSort() {
  store.setSortBy(sortBy.value === 'hot' ? 'updated' : 'hot');
}

// Vim-style keyboard navigation
function handleKeydown(event: KeyboardEvent) {
  // Ignore if typing in input
  if ((event.target as HTMLElement).tagName === 'INPUT' ||
    (event.target as HTMLElement).tagName === 'TEXTAREA') {
    return;
  }

  if (event.key === 'j' || event.key === 'ArrowDown') {
    event.preventDefault();
    const nextIndex = Math.min(focusedIndex.value + 1, flatItems.value.length - 1);
    if (nextIndex >= 0) {
      focusedIndex.value = nextIndex;
      const item = flatItems.value[nextIndex];
      if (item?.data) {
        selectCard(item.data.id);
        scrollToItem(nextIndex);
      }
    }
  } else if (event.key === 'k' || event.key === 'ArrowUp') {
    event.preventDefault();
    const prevIndex = Math.max(focusedIndex.value - 1, 0);
    if (prevIndex < flatItems.value.length) {
      focusedIndex.value = prevIndex;
      const item = flatItems.value[prevIndex];
      if (item?.data) {
        selectCard(item.data.id);
        scrollToItem(prevIndex);
      }
    }
  } else if (event.key === 'Enter' && focusedIndex.value >= 0) {
    event.preventDefault();
    const item = flatItems.value[focusedIndex.value];
    if (item?.data) {
      store.toggleZenMode();
    }
  }
}

function scrollToItem(index: number) {
  // Virtual scrolling handles scroll position, but we can ensure the item is visible
  const virtualRow = virtualizer.value.getVirtualItems().find(v => v.index === index);
  if (virtualRow) {
    virtualizer.value.scrollToIndex(index, {
      align: 'center',
      behavior: 'smooth',
    });
  }
}

onMounted(() => {
  loadIndex();
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});

const todayCount = computed(() => {
  const today = new Date();
  const todayStr = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`;
  return filteredCards.value.filter((c) => c.updated_at?.startsWith(todayStr)).length;
});

const avgHot = computed(() => {
  if (filteredCards.value.length === 0) return '0';
  const sum = filteredCards.value.reduce((acc, c) => acc + (c.hot_score || 0), 0);
  return String(Math.round(sum / filteredCards.value.length));
});

const sortLabel = computed(() => (sortBy.value === 'hot' ? '热度排序' : '时间排序'));

// ── Virtual scrolling ──
const listRef = ref<HTMLElement>();
const itemCount = computed(() => flatItems.value.length);
const virtualGap = computed(() => isMobile.value ? mobileGap : 32);
const virtualizer = useVirtualizer({
  count: itemCount.value,
  getScrollElement: () => listRef.value ?? null,
  estimateSize: () => isMobile.value ? 88 : 100,
  overscan: isMobile.value ? 5 : 10,
  gap: virtualGap.value,
});

watchEffect(() => {
  virtualizer.value.options.count = itemCount.value;
  virtualizer.value.options.gap = virtualGap.value;
});
</script>

<template>
  <div class="list-view bg-ms-xuan h-full flex flex-col pb-0" :class="isMobile ? 'pt-4' : 'pt-8'">
    <!-- Loading state -->
    <div v-if="loading" class="flex-1 py-4 space-y-3 max-w-3xl mx-auto w-full" :class="isMobile ? 'px-4' : 'px-8'">
      <div v-for="i in 5" :key="i" class="flex items-center gap-4 p-6 rounded-altar border border-ms-copper-light">
        <SkeletonLine width="4px" height="60px" />
        <div class="flex-1 space-y-3">
          <SkeletonLine width="60%" height="16px" />
          <SkeletonLine width="90%" height="12px" />
          <SkeletonLine width="40%" height="10px" />
        </div>
      </div>
    </div>

    <!-- Card list (blood spine) -->
    <div v-else-if="filteredCards.length > 0" ref="listRef" class="flex-1 min-h-0 overflow-y-auto relative"
      :class="isMobile ? 'pb-20' : 'pb-6'">

      <!-- Global spine beam - blood amber glow (desktop only) -->
      <div v-if="!isMobile" class="spine-beam absolute left-24 top-0 bottom-0 w-24 z-0 pointer-events-none" />

      <!-- Header -->
      <div :class="isMobile ? 'px-4 pb-2 mb-0' : 'max-w-4xl mx-auto px-8'">
        <ListViewHeader />
        <div class="list-filters" :class="isMobile ? 'mt-2' : 'mt-3'">
          <label class="list-filters__label" for="category-filter">分类筛选</label>
          <select id="category-filter" class="list-filters__select" :value="categoryFilter ?? ''"
            @change="onCategoryFilterChange">
            <option value="">全部分类</option>
            <option v-for="item in categoryOptions" :key="item.id" :value="item.id">
              {{ item.name }} ({{ item.count }})
            </option>
          </select>
        </div>
      </div>

      <div :class="isMobile ? 'px-3' : 'max-w-4xl mx-auto'">
        <div :style="{ height: `${virtualizer.getTotalSize()}px`, position: 'relative' }">
          <div v-for="row in virtualizer.getVirtualItems()" :key="row.index" :style="{
            position: 'absolute',
            top: 0,
            transform: `translateY(${row.start}px)`,
            width: '100%',
          }" class="items-start group cursor-pointer w-full transition-all duration-300 ease-out" :class="[
              isMobile ? 'grid grid-cols-spine-mobile' : 'grid grid-cols-spine hover:-translate-y-0.5',
            ]" role="button" tabindex="0" @click="handleCardClick(row.index, $event)"
            @keyup.enter="handleCardClick(row.index, $event)">
            <!-- Column 1: DateColumn (desktop only) -->
            <DateColumn v-if="!isMobile" :date-label="getRow(row.index)?.dateLabel || ''"
              :is-active="getRow(row.index)?.isFirstInDay || false" />

            <!-- Column 2: SpineNode (desktop only) -->
            <SpineNode v-if="!isMobile" :is-genesis="getRow(row.index)?.isFirstInDay || false"
              :date="getRow(row.index)?.data?.updated_at" :is-selected="selectedId === getRow(row.index)?.data?.id" />

            <!-- Column 3: ListCardRow -->
            <ListCardRow v-if="getRow(row.index)?.data" :card="getRow(row.index)!.data"
              :is-selected="selectedId === getRow(row.index)!.data.id" :is-active="focusedIndex === row.index"
              :is-mobile="isMobile" @select="selectCard(getRow(row.index)!.data.id)" />
          </div>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <div v-else class="flex-1 flex items-center justify-center">
      <EmptyState :title="searchQuery ? '没有找到匹配的卡片' : '记忆流是空的'"
        :description="!searchQuery ? '快去桌面端写几张卡片吧。创建卡片后图谱会自动生成。' : undefined" />
    </div>

    <!-- 灵签长条 -->
    <StatsWidget v-if="filteredCards.length > 0" :total-nodes="filteredCards.length" :today-count="todayCount"
      :avg-hot="avgHot" :sort-label="sortLabel" :sparkline-data="filteredCards.slice(0, 20).map(c => c.hot_score || 0)"
      @toggle-sort="toggleSort" />
  </div>
</template>

<style scoped>
.list-filters {
  display: flex;
  align-items: center;
  gap: 10px;
}

.list-filters__label {
  font-size: 12px;
  color: #a1a1aa;
  letter-spacing: 0.04em;
}

.list-filters__select {
  appearance: none;
  min-width: 180px;
  height: 30px;
  padding: 0 10px;
  border: 1px solid rgba(201, 168, 76, 0.28);
  border-radius: 6px;
  background: #141416;
  color: #e4e4e7;
  font-size: 12px;
}

.list-filters__select:focus {
  outline: none;
  border-color: rgba(201, 168, 76, 0.6);
  box-shadow: 0 0 0 2px rgba(201, 168, 76, 0.15);
}

/* ── Spine beam — 血珀香柱光柱 ── */
.spine-beam {
  background:
    radial-gradient(ellipse 30% 100% at 50% 50%,
      rgba(166, 38, 38, 0.04) 0%,
      rgba(166, 38, 38, 0.08) 50%,
      transparent 100%),
    linear-gradient(180deg,
      transparent 0%,
      rgba(166, 38, 38, 0.03) 10%,
      rgba(166, 38, 38, 0.05) 50%,
      rgba(166, 38, 38, 0.03) 90%,
      transparent 100%);
}
</style>
