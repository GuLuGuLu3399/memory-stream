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
import SkeletonLine from '../components/ui/SkeletonLine.vue';
import StatsWidget from '../components/StatsWidget.vue';
import EmptyState from '@memory-stream/ui-shared/components/EmptyState.vue';
import ListCardRow from './list/ListCardRow.vue';
import SpineNode from './list/SpineNode.vue';
import DateColumn from './list/DateColumn.vue';
import ListViewHeader from './list/ListViewHeader.vue';

const store = useGraphStore();
const { sortBy, selectedId } = storeToRefs(store);
const { cardIndex, loadIndex, loading } = useCards();

const searchQuery = ref('');
const focusedIndex = ref<number>(-1);

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

// ── Virtual scrolling (large gap: 32px) ──
const listRef = ref<HTMLElement>();
const itemCount = computed(() => flatItems.value.length);
const virtualizer = useVirtualizer({
  count: itemCount.value,
  getScrollElement: () => listRef.value ?? null,
  estimateSize: () => 100,
  overscan: 10,
  gap: 32,
});

watchEffect(() => {
  virtualizer.value.options.count = itemCount.value;
});
</script>

<template>
  <div class="list-view bg-ms-xuan h-full flex flex-col pt-8 pb-0">
    <!-- Loading state -->
    <div v-if="loading" class="flex-1 px-8 py-4 space-y-3 max-w-3xl mx-auto w-full">
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
    <div v-else-if="filteredCards.length > 0" ref="listRef" class="flex-1 min-h-0 overflow-y-auto pb-6 relative">

      <!-- Global spine beam - blood amber glow -->
      <div class="spine-beam absolute left-24 top-0 bottom-0 w-24 z-0 pointer-events-none" />

      <!-- Header -->
      <div class="max-w-4xl mx-auto">
        <ListViewHeader />
      </div>

      <div class="max-w-4xl mx-auto">
        <div :style="{ height: `${virtualizer.getTotalSize()}px`, position: 'relative' }">
          <div
            v-for="row in virtualizer.getVirtualItems()"
            :key="row.index"
            :style="{
              position: 'absolute',
              top: 0,
              transform: `translateY(${row.start}px)`,
              width: '100%',
            }"
            class="grid grid-cols-spine items-start group cursor-pointer w-full transition-all duration-300 ease-out hover:-translate-y-0.5"
            role="button"
            tabindex="0"
            @click="handleCardClick(row.index, $event)"
            @keyup.enter="handleCardClick(row.index, $event)"
          >
            <!-- Column 1: DateColumn -->
            <DateColumn
              :date-label="(flatItems[row.index] as CardRow)?.dateLabel || ''"
              :is-active="(flatItems[row.index] as CardRow)?.isFirstInDay || false"
            />

            <!-- Column 2: SpineNode -->
            <SpineNode
              :is-genesis="(flatItems[row.index] as CardRow)?.isFirstInDay || false"
              :date="(flatItems[row.index] as CardRow)?.data?.updated_at"
              :is-selected="selectedId === (flatItems[row.index] as CardRow)?.data?.id"
            />

            <!-- Column 3: ListCardRow -->
            <ListCardRow
              v-if="(flatItems[row.index] as CardRow)?.data"
              :card="(flatItems[row.index] as CardRow).data"
              :is-selected="selectedId === (flatItems[row.index] as CardRow).data.id"
              :is-active="focusedIndex === row.index"
              @select="selectCard((flatItems[row.index] as CardRow).data.id)"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <div v-else class="flex-1 flex items-center justify-center">
      <EmptyState
        :title="searchQuery ? '没有找到匹配的卡片' : '记忆流是空的'"
        :description="!searchQuery ? '快去桌面端写几张卡片吧。创建卡片后图谱会自动生成。' : undefined"
      />
    </div>

    <!-- 灵签长条 -->
    <StatsWidget
      v-if="filteredCards.length > 0"
      :total-nodes="filteredCards.length"
      :today-count="todayCount"
      :avg-hot="avgHot"
      :sort-label="sortLabel"
      :sparkline-data="filteredCards.slice(0, 20).map(c => c.hot_score || 0)"
      @toggle-sort="toggleSort"
    />
  </div>
</template>

<style scoped>
/* ── Spine beam — blood amber column ── */
.spine-beam {
  background: radial-gradient(ellipse at center, rgba(166, 38, 38, 0.06) 0%, transparent 70%);
}
</style>
