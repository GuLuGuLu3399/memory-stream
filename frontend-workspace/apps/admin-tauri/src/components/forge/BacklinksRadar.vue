<script setup lang="ts">
/**
 * ConnectionRadar — Bidirectional connection display
 *
 * Shows inbound, outbound, and bidirectional connections with
 * clear direction indicators. Bidirectional links (A→B AND B→A)
 * are highlighted with amber glow and ⇌ marker.
 */

import { computed } from 'vue'

export interface ConnectionItem {
  cardId: string
  cardTitle: string
  direction: 'inbound' | 'outbound' | 'bidirectional'
  relationType: string
  contextSnippet?: string
}

/** @deprecated Use ConnectionItem instead */
export type BacklinkItem = ConnectionItem

interface Props {
  connections: ConnectionItem[]
  loading: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  navigate: [cardId: string]
}>()

const hasConnections = computed(() => props.connections.length > 0)

const inboundCount = computed(() =>
  props.connections.filter(c => c.direction === 'inbound' || c.direction === 'bidirectional').length
)
const outboundCount = computed(() =>
  props.connections.filter(c => c.direction === 'outbound' || c.direction === 'bidirectional').length
)
const bidirCount = computed(() =>
  props.connections.filter(c => c.direction === 'bidirectional').length
)

// Sort: bidirectional first, then inbound, then outbound
const sortedConnections = computed(() => {
  const order = { bidirectional: 0, inbound: 1, outbound: 2 }
  return [...props.connections].sort((a, b) => order[a.direction] - order[b.direction])
})

function getRelationColor(type: string): string {
  return type === 'sequence' ? '#00e5ff' : '#6b7280'
}
</script>

<template>
  <div v-if="hasConnections" class="connection-radar">
    <div class="cr-header">
      <span class="cr-prefix">&#9670;</span>
      <span class="cr-label">CONNECTIONS</span>
      <span class="cr-sep">::</span>
      <span class="cr-total">{{ connections.length }}</span>
      <div class="cr-dirs">
        <span class="cr-dir cr-dir--in">&larr; {{ inboundCount }}</span>
        <span class="cr-dir-sep">&middot;</span>
        <span class="cr-dir cr-dir--out">&rarr; {{ outboundCount }}</span>
        <template v-if="bidirCount > 0">
          <span class="cr-dir-sep">&middot;</span>
          <span class="cr-dir cr-dir--bidi">&harr; {{ bidirCount }}</span>
        </template>
      </div>
    </div>

    <div class="cr-list">
      <div
        v-for="(conn, index) in sortedConnections"
        :key="conn.cardId"
        class="cr-item"
        :class="[`cr-item--${conn.direction}`]"
      >
        <div class="cr-pill">
          <div class="cr-meta">
            <span class="cr-arrow" :class="`cr-arrow--${conn.direction}`">
              {{ conn.direction === 'bidirectional' ? '\u21CC' : conn.direction === 'inbound' ? '\u2190' : '\u2192' }}
            </span>
            <span class="cr-relation" :style="{ color: getRelationColor(conn.relationType) }">
              {{ conn.relationType }}
            </span>
          </div>
          <button
            @click="emit('navigate', conn.cardId)"
            class="cr-title"
          >
            {{ conn.cardTitle }}
          </button>
          <p v-if="conn.contextSnippet" class="cr-snippet">
            ...{{ conn.contextSnippet }}...
          </p>
        </div>
        <div v-if="index < sortedConnections.length - 1" class="cr-divider" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.connection-radar {
  flex-shrink: 0;
  background: rgba(5, 5, 5, 0.95);
  backdrop-filter: blur(8px);
  border-top: 1px dashed #374151;
  padding: 12px 24px;
}

/* ===== Header ===== */
.cr-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  font-family: 'JetBrains Mono', monospace;
  margin-bottom: 10px;
  color: #6b7280;
}

.cr-prefix {
  color: #d97706;
  text-shadow: 0 0 8px rgba(217, 119, 6, 0.5);
  animation: radar-pulse 3s ease-in-out infinite;
}

@keyframes radar-pulse {
  0%, 100% { opacity: 1; text-shadow: 0 0 8px rgba(217, 119, 6, 0.5); }
  50% { opacity: 0.6; text-shadow: 0 0 4px rgba(217, 119, 6, 0.2); }
}

.cr-label {
  letter-spacing: 0.15em;
  text-transform: uppercase;
}

.cr-sep {
  color: #374151;
  margin: 0 2px;
}

.cr-total {
  color: #d97706;
  font-weight: 700;
  text-shadow: 0 0 8px rgba(217, 119, 6, 0.4);
}

.cr-dirs {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 4px;
}

.cr-dir {
  font-size: 10px;
  letter-spacing: 0.05em;
}

.cr-dir--in { color: #00e5ff; }
.cr-dir--out { color: #9ca3af; }
.cr-dir--bidi { color: #d97706; font-weight: 600; }

.cr-dir-sep {
  color: #374151;
  font-size: 10px;
}

/* ===== List ===== */
.cr-list {
  display: flex;
  gap: 16px;
  overflow-x: auto;
  padding-bottom: 4px;
}

.cr-list::-webkit-scrollbar {
  height: 3px;
}

.cr-list::-webkit-scrollbar-thumb {
  background: rgba(217, 119, 6, 0.3);
  border-radius: 2px;
}

/* ===== Items ===== */
.cr-item {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  flex-shrink: 0;
}

.cr-pill {
  padding: 6px 0;
  position: relative;
}

/* Bidirectional amber left-bar + glow */
.cr-item--bidirectional .cr-pill::before {
  content: '';
  position: absolute;
  inset: -2px -6px;
  background: rgba(217, 119, 6, 0.04);
  border-left: 2px solid #d97706;
  border-radius: 2px;
}

.cr-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
}

.cr-arrow {
  font-size: 12px;
  font-weight: 700;
  line-height: 1;
}

.cr-arrow--bidirectional {
  color: #d97706;
  text-shadow: 0 0 6px rgba(217, 119, 6, 0.4);
}

.cr-arrow--inbound {
  color: #00e5ff;
  opacity: 0.7;
}

.cr-arrow--outbound {
  color: #6b7280;
  opacity: 0.7;
}

.cr-relation {
  font-size: 10px;
  font-family: 'JetBrains Mono', monospace;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}

.cr-title {
  font-size: 13px;
  color: #d1d5db;
  text-align: left;
  background: transparent;
  border: none;
  padding: 0;
  cursor: pointer;
  transition: color 0.15s;
}

.cr-title:hover {
  color: #d97706;
}

.cr-snippet {
  font-size: 11px;
  color: #6b7280;
  font-style: italic;
  margin: 4px 0 0 0;
  max-width: 250px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cr-divider {
  width: 1px;
  height: 32px;
  background: linear-gradient(to bottom, transparent, rgba(217, 119, 6, 0.3), transparent);
  flex-shrink: 0;
}
</style>
