<script setup lang="ts">
const props = defineProps<{
  detail: { id: string; title: string } | null;
  createdAt: string;
}>();
</script>

<template>
  <div v-if="detail" class="detail-drawer__footer">
    <div v-if="createdAt" class="detail-drawer__footer-item">
      <svg class="detail-drawer__footer-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
        <line x1="16" y1="2" x2="16" y2="6" />
        <line x1="8" y1="2" x2="8" y2="6" />
        <line x1="3" y1="10" x2="21" y2="10" />
      </svg>
      <span class="detail-drawer__footer-text">{{ new Date(createdAt).toLocaleDateString("zh-CN") }}</span>
    </div>
    <div class="detail-drawer__footer-item detail-drawer__footer-item--id">
      <svg class="detail-drawer__footer-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="4" y1="9" x2="20" y2="9" />
        <line x1="4" y1="15" x2="20" y2="15" />
        <line x1="10" y1="3" x2="8" y2="21" />
        <line x1="16" y1="3" x2="14" y2="21" />
      </svg>
      <span class="detail-drawer__footer-text">{{ detail.id.slice(0, 8) }}</span>
    </div>
  </div>
</template>

<style scoped>
/* Footer — minimal seal */
.detail-drawer__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 20px;
  background: theme('colors.ms-mo');
  border-top: 1px solid rgba(58, 50, 40, 0.25);
}

.detail-drawer__footer-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-family: ui-monospace, monospace;
  font-size: 10px;
  color: theme('colors.ms-ash');
}

.detail-drawer__footer-icon {
  width: 10px;
  height: 10px;
  color: theme('colors.ms-copper');
}

.detail-drawer__footer-text {
  color: theme('colors.ms-ash');
}

.detail-drawer__footer-item--id {
  color: theme('colors.ms-ash');
  opacity: 0.4;
}
</style>
