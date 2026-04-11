<script setup lang="ts">
/**
 * EmptyState — Unified empty/error state display
 *
 * Replaces duplicated empty state markup across both apps.
 * Supports icon, title, description, and optional action button.
 *
 * @example
 * <EmptyState
 *   title="No cards found"
 *   description="Create your first card to get started"
 *   :action="{ label: 'Create Card', handler: onCreate }"
 * />
 */

defineProps<{
  title: string
  description?: string
  action?: { label: string; handler: () => void }
}>()
</script>

<template>
  <div class="empty-state">
    <div class="empty-state__icon">
      <slot name="icon">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" opacity="0.25">
          <circle cx="12" cy="12" r="10" />
          <path d="M8 15h8" />
          <path d="M9 9h.01" />
          <path d="M15 9h.01" />
        </svg>
      </slot>
    </div>

    <h3 class="empty-state__title">{{ title }}</h3>
    <p v-if="description" class="empty-state__desc">{{ description }}</p>

    <slot name="action">
      <button
        v-if="action"
        class="empty-state__btn"
        @click="action.handler"
      >
        {{ action.label }}
      </button>
    </slot>
  </div>
</template>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  text-align: center;
  width: 100%;
}

.empty-state__icon {
  margin-bottom: 16px;
  color: rgba(255, 255, 255, 0.15);
}

.empty-state__title {
  font-size: 16px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.5);
  margin: 0 0 8px;
}

.empty-state__desc {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.3);
  margin: 0;
  max-width: 280px;
  line-height: 1.5;
}

.empty-state__btn {
  margin-top: 20px;
  padding: 8px 20px;
  font-size: 13px;
  font-weight: 500;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.04);
  color: rgba(255, 255, 255, 0.6);
  cursor: pointer;
  transition: all 150ms ease;
}

.empty-state__btn:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.15);
  color: rgba(255, 255, 255, 0.8);
}
</style>
