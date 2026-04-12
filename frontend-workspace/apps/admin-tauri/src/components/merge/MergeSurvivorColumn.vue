<script setup lang="ts">
import { computed } from "vue";

interface CardData {
  id: string;
  title: string;
}

const props = defineProps<{
  cards: CardData[];
  selectedId: string | null;
  searchQuery: string;
}>();

const emit = defineEmits<{
  (e: "select", cardId: string): void;
  (e: "update:searchQuery", value: string): void;
}>();

const availableCards = computed(() => {
  if (!props.searchQuery) return props.cards;
  const q = props.searchQuery.toLowerCase();
  return props.cards.filter((c) => c.title.toLowerCase().includes(q));
});

function handleSelect(cardId: string) {
  emit("select", cardId);
}

function updateSearchQuery(e: Event) {
  emit("update:searchQuery", (e.target as HTMLInputElement).value);
}

const selectedCard = computed(() => {
  return props.cards.find(c => c.id === props.selectedId);
});
</script>

<template>
  <div class="merge-survivor-column">
    <!-- Section Header with Rivet Dots -->
    <div class="merge-survivor-column__header">
      <span class="rivet rivet--tl" />
      <span class="rivet rivet--tr" />
      <span class="rivet rivet--bl" />
      <span class="rivet rivet--br" />

      <span class="merge-survivor-column__title">
        SURVIVOR
      </span>
      <span class="merge-survivor-column__subtitle">(主节点)</span>
    </div>

    <!-- Search -->
    <div class="merge-survivor-column__search">
      <input
        :value="searchQuery"
        @input="updateSearchQuery"
        type="text"
        placeholder="搜索主节点..."
        class="merge-survivor-column__input"
      />
    </div>

    <!-- Selected Survivor Display -->
    <div v-if="selectedId" class="merge-survivor-column__selected">
      <div class="merge-survivor-column__selected-label">已选定</div>
      <div class="merge-survivor-column__selected-title">
        {{ selectedCard?.title || "无标题" }}
      </div>
    </div>

    <!-- Card List -->
    <div class="merge-survivor-column__list">
      <label
        v-for="card in availableCards"
        :key="card.id"
        class="merge-survivor-card"
        :class="{ 'merge-survivor-card--selected': selectedId === card.id }"
      >
        <input
          type="radio"
          :value="card.id"
          :checked="selectedId === card.id"
          @change="handleSelect(card.id)"
          class="survivor-radio"
        />
        <span class="merge-survivor-card__title">
          {{ card.title || "无标题" }}
        </span>
      </label>

      <div v-if="availableCards.length === 0" class="merge-survivor-column__empty">
        <span class="merge-survivor-column__empty-text">
          无可用卡片
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.merge-survivor-column {
  width: 288px;
  border-left: 1px solid theme('colors.ms-border');
  display: flex;
  flex-direction: column;
  background: theme('colors.ms-void');
}

.merge-survivor-column__header {
  height: 40px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  border-bottom: 1px solid theme('colors.ms-border');
  flex-shrink: 0;
  position: relative;
}

.rivet {
  position: absolute;
  width: 4px;
  height: 4px;
  border-radius: 50%;
}

.rivet--tl { top: 6px; left: 6px; background: rgba(222, 184, 135, 0.6); }
.rivet--tr { top: 6px; right: 6px; background: rgba(222, 184, 135, 0.6); }
.rivet--bl { bottom: 6px; left: 6px; background: rgba(222, 184, 135, 0.6); }
.rivet--br { bottom: 6px; right: 6px; background: rgba(222, 184, 135, 0.6); }

.merge-survivor-column__title {
  font-size: 10px;
  font-family: ui-monospace, monospace;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: theme('colors.ms-brass-light');
}

.merge-survivor-column__subtitle {
  font-size: 10px;
  color: theme('colors.ms-slate.600');
  font-family: ui-monospace, monospace);
  margin-left: 8px;
}

.merge-survivor-column__search {
  padding: 8px 12px;
  border-bottom: 1px solid theme('colors.ms-border');
}

.merge-survivor-column__input {
  width: 100%;
  background: theme('colors.ms-deep');
  border: 1px solid theme('colors.ms-border');
  padding: 6px 8px;
  font-size: 12px;
  color: theme('colors.ms-slate.300');
  font-family: ui-monospace, monospace;
  outline: none;
}

.merge-survivor-column__input:focus {
  border-color: rgba(222, 184, 135, 0.5);
}

.merge-survivor-column__input::placeholder {
  color: theme('colors.ms-slate.600');
}

.merge-survivor-column__selected {
  padding: 8px 12px;
  border-bottom: 1px solid theme('colors.ms-border');
  background: rgba(222, 184, 135, 0.05);
}

.merge-survivor-column__selected-label {
  font-size: 9px;
  color: theme('colors.ms-brass-light');
  font-family: ui-monospace, monospace;
  margin-bottom: 4px;
}

.merge-survivor-column__selected-title {
  font-size: 12px;
  color: theme('colors.ms-slate.300');
  font-family: ui-monospace, monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.merge-survivor-column__list {
  flex: 1;
  overflow-y: auto;
}

.merge-survivor-column__list::-webkit-scrollbar {
  width: 3px;
}

.merge-survivor-column__list::-webkit-scrollbar-track {
  background: transparent;
}

.merge-survivor-column__list::-webkit-scrollbar-thumb {
  background: #222;
  border-radius: 1px;
}

.merge-survivor-card {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  transition: background-color 150ms ease, color 150ms ease;
  border-bottom: 1px solid rgba(51, 65, 85, 0.3);
  color: theme('colors.ms-slate.400');
}

.merge-survivor-card:hover {
  background: rgba(30, 41, 59, 0.5);
}

.merge-survivor-card--selected {
  background: rgba(222, 184, 135, 0.1);
  color: theme('colors.ms-brass-light');
}

.merge-survivor-card__title {
  font-size: 12px;
  font-family: ui-monospace, monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.merge-survivor-column__empty {
  padding: 16px 12px;
  text-align: center;
}

.merge-survivor-column__empty-text {
  font-size: 10px;
  color: theme('colors.ms-slate.600');
  font-style: italic;
  font-family: ui-monospace, monospace;
}

.survivor-radio {
  appearance: none;
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  border: 1px solid rgba(184, 134, 11, 0.4);
  background: #0d0d0d;
  cursor: pointer;
  transition: all 0.15s ease;
  border-radius: 50%;
  flex-shrink: 0;
}

.survivor-radio:checked {
  background: #0d0d0d;
  border-color: #b8860b;
}

.survivor-radio:checked::after {
  content: "";
  display: block;
  width: 4px;
  height: 4px;
  background: #b8860b;
  border-radius: 50%;
  margin: 3px;
  box-shadow: 0 0 4px rgba(184, 134, 11, 0.3);
}

.survivor-radio:hover {
  border-color: #b8860b;
  box-shadow: 0 0 4px rgba(184, 134, 11, 0.2);
}
</style>
