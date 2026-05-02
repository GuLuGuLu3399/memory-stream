<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core'

defineProps(['data'])
</script>

<template>
  <div class="altar-node">
    <Handle type="target" :position="Position.Left" id="trunk-target" class="altar-handle h-left" />
    <Handle type="target" :position="Position.Left" id="link-target" class="altar-handle h-left" />
    <div class="altar-label">{{ data.label }}</div>
    <Handle type="source" :position="Position.Right" id="trunk-source" class="altar-handle h-right" />
    <Handle type="source" :position="Position.Right" id="link-source" class="altar-handle h-right" />
  </div>
</template>

<style scoped>
.altar-node {
  padding: 4px 10px;
  border-radius: 2px;
  font-size: 11px;
  position: relative;
  background: var(--ms-surface, #222222);
  border: 1px solid var(--ms-border, color-mix(in oklch, var(--accent-dim, #4A6B5A) 30%, transparent));
  color: var(--text-primary, #E0D8C8);
}

.altar-label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 120px;
}

.altar-handle {
  width: 6px;
  height: 6px;
  background: var(--neon, #8b9eb7);
  border: none;
  border-radius: 50%;
  opacity: 0;
  transition: opacity 150ms ease, box-shadow 150ms ease;
}

.altar-node:hover .altar-handle {
  opacity: 1;
  box-shadow: 0 0 6px color-mix(in oklch, var(--neon, #8b9eb7) 60%, transparent);
}

.h-left { left: -3px; }
.h-right { right: -3px; }
</style>
