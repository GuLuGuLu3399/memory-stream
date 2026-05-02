// ────────────────────────────────────────────────────────────────
// AstAdmonition.vue — warning / tip / question callout blocks
// AstAdmonition 警告/提示/问题提示块
// ────────────────────────────────────────────────────────────────

<script setup lang="ts">
import type { AstNode, AdmonitionKind } from '@memory-stream/types'
import AstRenderer from './AstRenderer.vue'

const props = defineProps<{
  kind: AdmonitionKind
  children: AstNode[]
}>()

const icons: Record<AdmonitionKind, string> = {
  Warning: '⚠',
  Tip: '💡',
  Question: '❓',
}
</script>

<template>
  <div class="ms-admonition" :class="`ms-admonition--${kind.toLowerCase()}`">
    <div class="ms-admonition__bar" />
    <div class="ms-admonition__body">
      <span class="ms-admonition__icon">{{ icons[props.kind] }}</span>
      <div class="ms-admonition__content">
        <AstRenderer v-for="(child, i) in children" :key="i" :node="child" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.ms-admonition {
  display: flex;
  margin: 1em 0;
  border-radius: 2px;
  overflow: hidden;
  background: var(--ms-carbon, #141414);
  border: 1px solid var(--ms-border, #1e1e1e);
}

.ms-admonition__bar {
  width: 4px;
  flex-shrink: 0;
}

.ms-admonition__body {
  display: flex;
  gap: 0.5em;
  padding: 0.75em 1em;
  flex: 1;
  min-width: 0;
}

.ms-admonition__icon {
  flex-shrink: 0;
  font-size: 0.95em;
  line-height: 1.75;
}

.ms-admonition__content {
  flex: 1;
  min-width: 0;
}

/* Per-kind color variants */
.ms-admonition--warning .ms-admonition__bar {
  background: #e53935;
}

.ms-admonition--warning .ms-admonition__icon {
  color: #e53935;
}

.ms-admonition--tip .ms-admonition__bar {
  background: #00e5ff;
}

.ms-admonition--tip .ms-admonition__icon {
  color: #00e5ff;
}

.ms-admonition--question .ms-admonition__bar {
  background: #d4a017;
}

.ms-admonition--question .ms-admonition__icon {
  color: #d4a017;
}
</style>
