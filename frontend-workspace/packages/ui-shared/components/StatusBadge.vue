<script setup lang="ts">
/**
 * StatusBadge — Semantic status indicator
 *
 * Replaces scattered inline badge markup across both apps.
 * Uses CSS custom properties that resolve per-app theme.
 *
 * @example
 * <StatusBadge variant="sequence" size="xs" />
 * <StatusBadge variant="orphan" size="sm">Disconnected</StatusBadge>
 */

withDefaults(defineProps<{
  variant: 'sequence' | 'reference' | 'orphan' | 'draft' | 'active'
  size?: 'xs' | 'sm'
}>(), {
  size: 'xs',
})
</script>

<template>
  <span
    class="status-badge"
    :class="[`status-badge--${variant}`, `status-badge--${size}`]"
  >
    <slot />
  </span>
</template>

<style scoped>
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-family: "JetBrains Mono", "Fira Code", Consolas, monospace;
  letter-spacing: 0.04em;
  border-radius: 2px;
  white-space: nowrap;
  user-select: none;
  line-height: 1;
}

.status-badge--xs {
  padding: 2px 6px;
  font-size: 9px;
}

.status-badge--sm {
  padding: 3px 8px;
  font-size: 10px;
}

/* Variant: sequence — warm amber/red */
.status-badge--sequence {
  background: rgba(166, 38, 38, 0.15);
  color: #c23616;
  border: 1px solid rgba(166, 38, 38, 0.25);
}

/* Variant: reference — muted copper */
.status-badge--reference {
  background: rgba(90, 79, 62, 0.2);
  color: #8a7e6e;
  border: 1px solid rgba(90, 79, 62, 0.3);
}

/* Variant: orphan — dim ghost */
.status-badge--orphan {
  background: rgba(255, 255, 255, 0.04);
  color: rgba(255, 255, 255, 0.35);
  border: 1px dashed rgba(255, 255, 255, 0.12);
}

/* Variant: draft — brass tint */
.status-badge--draft {
  background: rgba(184, 134, 11, 0.12);
  color: #d4a853;
  border: 1px solid rgba(184, 134, 11, 0.2);
}

/* Variant: active — neon pulse (admin) / xuepo pulse (web-reader) */
.status-badge--active {
  background: rgba(0, 229, 255, 0.1);
  color: #00e5ff;
  border: 1px solid rgba(0, 229, 255, 0.2);
}
</style>
