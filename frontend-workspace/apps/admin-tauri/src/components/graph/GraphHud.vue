<script setup lang="ts">
// 用途：图谱HUD控制面板，切换显示Trunk、Link和Orphan节点
const showTrunk = defineModel<boolean>('showTrunk', { required: true })
const showLink = defineModel<boolean>('showLink', { required: true })
const showOrphan = defineModel<boolean>('showOrphan', { required: true })
</script>

<template>
  <div class="graph-hud">
    <button
      class="hud-item"
      :class="{ active: showTrunk }"
      @click="showTrunk = !showTrunk"
    >
      <svg class="hud-icon" width="14" height="6" viewBox="0 0 14 6">
        <line x1="0" y1="3" x2="14" y2="3" stroke="currentColor" stroke-width="1.5" />
      </svg>
      <span class="hud-label">TRUNK</span>
    </button>
    <button
      class="hud-item"
      :class="{ active: showLink }"
      @click="showLink = !showLink"
    >
      <svg class="hud-icon" width="14" height="6" viewBox="0 0 14 6">
        <line x1="0" y1="3" x2="14" y2="3" stroke="currentColor" stroke-width="1.5" stroke-dasharray="3 2" />
      </svg>
      <span class="hud-label">LINK</span>
    </button>
    <button
      class="hud-item"
      :class="{ active: showOrphan }"
      @click="showOrphan = !showOrphan"
    >
      <svg class="hud-icon" width="10" height="10" viewBox="0 0 10 10">
        <circle cx="5" cy="5" r="3.5" fill="none" stroke="currentColor" stroke-width="1" />
      </svg>
      <span class="hud-label">ORPHAN</span>
    </button>
  </div>
</template>

<style scoped>
.graph-hud {
  display: flex;
  flex-direction: column;
  gap: 6px;
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  background: color-mix(in oklch, var(--ms-void) 85%, transparent);
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  backdrop-filter: blur(8px);
  padding: 6px;
  z-index: 10;
}

.hud-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 4px;
  border: none;
  border-radius: 1px;
  background: transparent;
  cursor: pointer;
  color: var(--ms-smoke);
  opacity: 0.4;
  transition: color var(--duration-fast) var(--ease-hydraulic),
    opacity var(--duration-fast) var(--ease-hydraulic),
    filter var(--duration-fast) var(--ease-hydraulic);
}

.hud-item:hover:not(.active) {
  opacity: 0.6;
  background: var(--ms-surface);
}

.hud-item.active {
  opacity: 1;
}

.hud-icon {
  flex-shrink: 0;
}

.hud-label {
  font-family: var(--font-mono);
  font-size: 9px;
  letter-spacing: 0.08em;
}

/* Type-specific active colors */
.hud-item.active:nth-child(1) {
  color: var(--neon);
  filter: drop-shadow(0 0 4px oklch(0.78 0.17 200 / 0.5));
}

.hud-item.active:nth-child(2) {
  color: var(--brass);
  filter: drop-shadow(0 0 4px oklch(0.75 0.13 80 / 0.5));
}

.hud-item.active:nth-child(3) {
  color: var(--text-primary);
  filter: drop-shadow(0 0 4px oklch(1 0 0 / 0.3));
}
</style>
