<script setup lang="ts">
import { computed } from "vue";

interface CardData {
  id: string;
  title: string;
}

const props = defineProps<{
  cards: CardData[];
  selectedIds: string[];
  searchQuery: string;
}>();

const emit = defineEmits<{
  (e: "toggle", cardId: string): void;
  (e: "selectAll"): void;
  (e: "clearAll"): void;
  (e: "update:searchQuery", value: string): void;
}>();

const availableCards = computed(() => {
  if (!props.searchQuery) return props.cards;
  const q = props.searchQuery.toLowerCase();
  return props.cards.filter((c) => c.title.toLowerCase().includes(q));
});

function handleToggle(cardId: string) {
  emit("toggle", cardId);
}

function handleSelectAll() {
  emit("selectAll");
}

function handleClearAll() {
  emit("clearAll");
}

function updateSearchQuery(e: Event) {
  emit("update:searchQuery", (e.target as HTMLInputElement).value);
}
</script>

<template>
  <div class="merge-victims-column">
    <!-- Section Header with Rivet Dots -->
    <div class="merge-victims-column__header">
      <span class="rivet rivet--tl" />
      <span class="rivet rivet--tr" />
      <span class="rivet rivet--bl" />
      <span class="rivet rivet--br" />

      <div class="flex items-center gap-2">
        <span class="merge-victims-column__title">
          SACRIFICES
        </span>
        <span class="merge-victims-column__subtitle">(祭品)</span>
      </div>
      <div class="flex items-center gap-1">
        <button
          @click="handleSelectAll"
          class="merge-victims-column__action">
          ALL
        </button>
        <button
          @click="handleClearAll"
          class="merge-victims-column__action">
          CLEAR
        </button>
      </div>
    </div>

    <!-- Search -->
    <div class="merge-victims-column__search">
      <input
        :value="searchQuery"
        @input="updateSearchQuery"
        type="text"
        placeholder="搜索祭品..."
        class="merge-victims-column__input"
      />
    </div>

    <!-- Victim Counter -->
    <div v-if="selectedIds.length > 0" class="merge-victims-column__counter">
      <span class="merge-victims-column__count">
        {{ selectedIds.length }} 个待献祭
      </span>
    </div>

    <!-- Card List -->
    <div class="merge-victims-column__list">
      <label
        v-for="card in availableCards"
        :key="card.id"
        class="merge-victims-card"
        :class="{ 'merge-victims-card--selected': selectedIds.includes(card.id) }"
      >
        <input
          type="checkbox"
          :checked="selectedIds.includes(card.id)"
          @change="handleToggle(card.id)"
          class="victim-checkbox"
        />
        <span class="merge-victims-card__title">
          {{ card.title || "无标题" }}
        </span>
      </label>

      <div v-if="availableCards.length === 0" class="merge-victims-column__empty">
        <span class="merge-victims-column__empty-text">
          无可用卡片
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.merge-victims-column {
  width: 320px;
  border-right: 1px solid theme('colors.ms-border');
  display: flex;
  flex-direction: column;
  background: theme('colors.ms-void');
}

.merge-victims-column__header {
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  border-bottom: 1px solid theme('colors.ms-border');
  flex-shrink: 0;
  position: relative;
}

.rivet {
  position: absolute;
  width: 4px;
  height: 4px;
  background: rgba(184, 134, 11, 0.6);
  border-radius: 50%;
}

.rivet--tl { top: 6px; left: 6px; }
.rivet--tr { top: 6px; right: 6px; }
.rivet--bl { bottom: 6px; left: 6px; }
.rivet--br { bottom: 6px; right: 6px; }

.merge-victims-column__title {
  font-size: 10px;
  font-family: ui-monospace, monospace;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: theme('colors.brass.DEFAULT');
}

.merge-victims-column__subtitle {
  font-size: 10px;
  color: theme('colors.ms-engrave');
  font-family: ui-monospace, monospace;
}

.merge-victims-column__action {
  font-size: 9px;
  color: theme('colors.ms-engrave');
  padding: 2px 6px;
  transition: color 150ms ease;
}

.merge-victims-column__action:hover {
  color: theme('colors.ms-surface-raised');
}

.merge-victims-column__search {
  padding: 8px 12px;
  border-bottom: 1px solid theme('colors.ms-border');
}

.merge-victims-column__input {
  width: 100%;
  background: theme('colors.ms-deep');
  border: 1px solid theme('colors.ms-border');
  padding: 6px 8px;
  font-size: 12px;
  color: theme('colors.ms-surface-raised');
  font-family: ui-monospace, monospace;
  outline: none;
}

.merge-victims-column__input:focus {
  border-color: rgba(184, 134, 11, 0.5);
}

.merge-victims-column__input::placeholder {
  color: theme('colors.ms-engrave');
}

.merge-victims-column__counter {
  padding: 6px 12px;
  border-bottom: 1px solid rgba(251, 146, 60, 0.5);
  background: rgba(251, 146, 60, 0.05);
}

.merge-victims-column__count {
  font-size: 10px;
  color: theme('colors.ms-warning');
  font-family: ui-monospace, monospace;
}

.merge-victims-column__list {
  flex: 1;
  overflow-y: auto;
}

.merge-victims-column__list::-webkit-scrollbar {
  width: 3px;
}

.merge-victims-column__list::-webkit-scrollbar-track {
  background: transparent;
}

.merge-victims-column__list::-webkit-scrollbar-thumb {
  background: #222;
  border-radius: 1px;
}

.merge-victims-card {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  transition: background-color 150ms ease, color 150ms ease;
  border-bottom: 1px solid rgba(51, 65, 85, 0.3);
  color: theme('colors.ms-surface-raised');
}

.merge-victims-card:hover {
  background: rgba(30, 41, 59, 0.5);
}

.merge-victims-card--selected {
  background: rgba(184, 134, 11, 0.1);
  color: theme('colors.brass.light');
}

.merge-victims-card__title {
  font-size: 12px;
  font-family: ui-monospace, monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.merge-victims-column__empty {
  padding: 16px 12px;
  text-align: center;
}

.merge-victims-column__empty-text {
  font-size: 10px;
  color: theme('colors.ms-engrave');
  font-style: italic;
  font-family: ui-monospace, monospace;
}

.victim-checkbox {
  appearance: none;
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  border: 1px solid rgba(184, 134, 11, 0.4);
  background: #0d0d0d;
  cursor: pointer;
  transition: all 0.15s ease;
  border-radius: 2px;
  flex-shrink: 0;
}

.victim-checkbox:checked {
  background: #b8860b;
  border-color: #b8860b;
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
  border-color: #b8860b;
  box-shadow: 0 0 4px rgba(184, 134, 11, 0.2);
}
</style>
