<script setup lang="ts">
/**
 * ForgeEmptyState — Empty state when no card is active
 *
 * Features:
 * - Animated SVG crosshair/forge anvil
 * - Subtle brass pulse animation
 * - Instructional text
 */
</script>

<template>
  <div class="forge-empty">
    <!-- Crosshair Reticle -->
    <div class="forge-empty__reticle">
      <!-- Outer ring -->
      <div class="forge-empty__ring forge-empty__ring--outer"></div>

      <!-- Cross lines -->
      <div class="forge-empty__cross forge-empty__cross--h"></div>
      <div class="forge-empty__cross forge-empty__cross--v"></div>

      <!-- Center point -->
      <div class="forge-empty__center"></div>

      <!-- Corner markers -->
      <div class="forge-empty__corner forge-empty__corner--tl"></div>
      <div class="forge-empty__corner forge-empty__corner--tr"></div>
      <div class="forge-empty__corner forge-empty__corner--bl"></div>
      <div class="forge-empty__corner forge-empty__corner--br"></div>
    </div>

    <!-- Instructions -->
    <div class="forge-empty__text">
      <p class="forge-empty__title">选择或新建一张卡片开始锻造</p>
      <p class="forge-empty__subtitle">CTRL+K 搜索 · 左侧 + 新建卡片</p>
    </div>
  </div>
</template>

<style scoped>
.forge-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 24px;
  user-select: none;
}

/* Reticle container */
.forge-empty__reticle {
  position: relative;
  width: 96px;
  height: 96px;
  animation: float 4s ease-in-out infinite;
}

@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-6px); }
}

/* Outer ring */
.forge-empty__ring--outer {
  position: absolute;
  inset: 0;
  border: 1px solid rgba(42, 42, 42, 0.4);
}

/* Cross lines */
.forge-empty__cross--h {
  position: absolute;
  top: 50%;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(to right, transparent, rgba(0, 229, 255, 0.3), transparent);
}

.forge-empty__cross--v {
  position: absolute;
  left: 50%;
  top: 0;
  bottom: 0;
  width: 1px;
  background: linear-gradient(to bottom, transparent, rgba(0, 229, 255, 0.3), transparent);
}

/* Center point with glow */
.forge-empty__center {
  position: absolute;
  top: 50%;
  left: 50%;
  width: 6px;
  height: 6px;
  background: rgba(0, 229, 255, 0.5);
  border-radius: 50%;
  transform: translate(-50%, -50%);
  box-shadow:
    0 0 8px rgba(0, 229, 255, 0.3),
    0 0 20px rgba(0, 229, 255, 0.1);
  animation: center-pulse 2s ease-in-out infinite;
}

@keyframes center-pulse {
  0%, 100% {
    box-shadow:
      0 0 8px rgba(0, 229, 255, 0.3),
      0 0 20px rgba(0, 229, 255, 0.1);
  }
  50% {
    box-shadow:
      0 0 12px rgba(0, 229, 255, 0.4),
      0 0 28px rgba(0, 229, 255, 0.15);
  }
}

/* Corner markers */
.forge-empty__corner {
  position: absolute;
  width: 8px;
  height: 8px;
}

.forge-empty__corner::before,
.forge-empty__corner::after {
  content: '';
  position: absolute;
  background: rgba(0, 229, 255, 0.2);
}

.forge-empty__corner--tl {
  top: 8px;
  left: 8px;
}
.forge-empty__corner--tl::before {
  top: 0;
  left: 0;
  width: 8px;
  height: 1px;
}
.forge-empty__corner--tl::after {
  top: 0;
  left: 0;
  width: 1px;
  height: 8px;
}

.forge-empty__corner--tr {
  top: 8px;
  right: 8px;
}
.forge-empty__corner--tr::before {
  top: 0;
  right: 0;
  width: 8px;
  height: 1px;
}
.forge-empty__corner--tr::after {
  top: 0;
  right: 0;
  width: 1px;
  height: 8px;
}

.forge-empty__corner--bl {
  bottom: 8px;
  left: 8px;
}
.forge-empty__corner--bl::before {
  bottom: 0;
  left: 0;
  width: 8px;
  height: 1px;
}
.forge-empty__corner--bl::after {
  bottom: 0;
  left: 0;
  width: 1px;
  height: 8px;
}

.forge-empty__corner--br {
  bottom: 8px;
  right: 8px;
}
.forge-empty__corner--br::before {
  bottom: 0;
  right: 0;
  width: 8px;
  height: 1px;
}
.forge-empty__corner--br::after {
  bottom: 0;
  right: 0;
  width: 1px;
  height: 8px;
}

/* Text */
.forge-empty__text {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.forge-empty__title {
  font-family: 'Space Grotesk', sans-serif;
  font-size: 14px;
  color: #9ca3af;
  letter-spacing: 0.05em;
  margin: 0;
}

.forge-empty__subtitle {
  font-size: 11px;
  font-family: 'JetBrains Mono', monospace;
  color: #374151;
  letter-spacing: 0.1em;
  margin: 0;
}
</style>
