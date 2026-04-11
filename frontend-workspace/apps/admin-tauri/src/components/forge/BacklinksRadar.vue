<script setup lang="ts">
/**
 * BacklinksRadar — Horizontal scrolling backlinks panel
 *
 * Features:
 * - Horizontal scroll with card pills
 * - Shows backlink source, relation type, and snippet
 * - Brass glow border styling
 */

import { computed } from 'vue'

export interface BacklinkItem {
  source_id: string
  source_title: string
  relation_type: string
  context_snippet?: string
}

interface Props {
  backlinks: BacklinkItem[]
  loading: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  navigate: [cardId: string]
}>()

const hasBacklinks = computed(() => props.backlinks && props.backlinks.length > 0)

function getRelationColor(type: string): string {
  return type === 'sequence' ? '#00e5ff' : '#6b7280'
}
</script>

<template>
  <div v-if="hasBacklinks" class="backlinks-radar">
    <div class="backlinks-radar__header">
      <span class="backlinks-radar__prefix">❯</span>
      <span class="backlinks-radar__label">INCOMING_LINKS</span>
      <span class="backlinks-radar__separator">::</span>
      <span class="backlinks-radar__count">{{ backlinks.length }}</span>
    </div>

    <div class="backlinks-radar__list">
      <div
        v-for="(link, index) in backlinks"
        :key="link.source_id"
        class="backlinks-radar__item"
      >
        <div class="backlinks-radar__pill">
          <div class="backlinks-radar__relation" :style="{ color: getRelationColor(link.relation_type) }">
            {{ link.relation_type }}
          </div>
          <button
            @click="emit('navigate', link.source_id)"
            class="backlinks-radar__title"
          >
            {{ link.source_title }}
          </button>
          <p v-if="link.context_snippet" class="backlinks-radar__snippet">
            ...{{ link.context_snippet }}...
          </p>
        </div>
        <div v-if="index < backlinks.length - 1" class="backlinks-radar__separator-v" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.backlinks-radar {
  flex-shrink: 0;
  background: rgba(5, 5, 5, 0.95);
  backdrop-filter: blur(8px);
  border-top: 1px dashed #374151;
  padding: 12px 24px;
}

.backlinks-radar__header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  font-family: 'JetBrains Mono', monospace;
  margin-bottom: 8px;
  color: #6b7280;
}

.backlinks-radar__prefix {
  color: #00e5ff;
  animation: pulse-glow 2s ease-in-out infinite;
}

@keyframes pulse-glow {
  0%, 100% { opacity: 1; text-shadow: 0 0 8px rgba(0, 229, 255, 0.4); }
  50% { opacity: 0.7; text-shadow: 0 0 4px rgba(0, 229, 255, 0.2); }
}

.backlinks-radar__label {
  letter-spacing: 0.15em;
  text-transform: uppercase;
}

.backlinks-radar__separator {
  color: #374151;
  margin: 0 4px;
}

.backlinks-radar__count {
  color: #00e5ff;
  font-weight: 700;
  text-shadow: 0 0 8px rgba(0, 229, 255, 0.4);
}

.backlinks-radar__list {
  display: flex;
  gap: 16px;
  overflow-x: auto;
  padding-bottom: 4px;
}

.backlinks-radar__list::-webkit-scrollbar {
  height: 4px;
}

.backlinks-radar__list::-webkit-scrollbar-thumb {
  background: rgba(184, 134, 11, 0.3);
  border-radius: 2px;
}

.backlinks-radar__item {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  flex-shrink: 0;
}

.backlinks-radar__pill {
  padding: 6px 0;
}

.backlinks-radar__relation {
  font-size: 10px;
  font-family: 'JetBrains Mono', monospace;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: 4px;
}

.backlinks-radar__title {
  font-size: 13px;
  color: #d1d5db;
  text-align: left;
  background: transparent;
  border: none;
  padding: 0;
  cursor: pointer;
  transition: color 0.15s;
}

.backlinks-radar__title:hover {
  color: #00e5ff;
}

.backlinks-radar__snippet {
  font-size: 11px;
  color: #6b7280;
  font-style: italic;
  margin: 4px 0 0 0;
  max-width: 250px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.backlinks-radar__separator-v {
  width: 1px;
  height: 32px;
  background: linear-gradient(to bottom, transparent, rgba(184, 134, 11, 0.3), transparent);
  flex-shrink: 0;
}
</style>
