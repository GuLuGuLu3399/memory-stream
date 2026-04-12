<script setup lang="ts">
const props = defineProps<{
  canShow: boolean;
  isExecuting: boolean;
  isHolding: boolean;
  holdProgress: number;
  progressOffset: number;
  warningMessage: string;
  generalError: string | null;
}>();

const emit = defineEmits<{
  (e: "pointerDown", event: PointerEvent): void;
  (e: "pointerUp"): void;
  (e: "pointerLeave"): void;
}>();

const CIRCUMFERENCE = 2 * Math.PI * 16;
</script>

<template>
  <div class="merge-action-bar">
    <!-- Warning/Status Text -->
    <div class="merge-action-bar__status">
      <div
        class="merge-action-bar__warning"
        :class="canShow ? 'merge-action-bar__warning--active' : ''"
      >
        <span v-if="canShow">⚠</span>
        {{ warningMessage }}
      </div>
      <div v-if="generalError" class="merge-action-bar__error">
        {{ generalError }}
      </div>
    </div>

    <!-- Long-Press Initiate Button with Circular Progress Ring -->
    <button
      @pointerdown="emit('pointerDown', $event)"
      @pointerup="emit('pointerUp')"
      @pointerleave="emit('pointerLeave')"
      :disabled="!canShow || isExecuting"
      class="merge-action-bar__button"
      :class="
        canShow && !isExecuting
          ? 'merge-action-bar__button--active'
          : 'merge-action-bar__button--disabled'
      "
    >
      <!-- SVG Circular Progress Ring -->
      <svg
        v-if="isHolding"
        class="merge-action-bar__progress"
        viewBox="0 0 56 40"
      >
        <!-- Background circle -->
        <circle
          cx="28"
          cy="20"
          r="16"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          class="merge-action-bar__progress-bg"
        />
        <!-- Progress circle -->
        <circle
          cx="28"
          cy="20"
          r="16"
          fill="none"
          stroke="currentColor"
          :stroke-dasharray="CIRCUMFERENCE"
          :stroke-dashoffset="progressOffset"
          :stroke-opacity="0.3 + (holdProgress / 100) * 0.7"
          class="merge-action-bar__progress-circle"
          :class="{ 'merge-action-bar__progress-circle--glow': holdProgress > 50 }"
          style="transform-origin: 28px 20px; transform: rotate(-90deg);"
        />
      </svg>

      <!-- Button Text -->
      <div class="merge-action-bar__button-text">
        <span
          class="merge-action-bar__button-label"
          :class="canShow && !isExecuting ? '' : 'merge-action-bar__button-label--disabled'"
        >
          <template v-if="isExecuting">执行中...</template>
          <template v-else-if="isHolding">按住 {{ Math.ceil((100 - holdProgress) / 100 * 3) }}s — 启动坍缩</template>
          <template v-else>按住 3 秒 — 启动坍缩</template>
        </span>
      </div>
    </button>
  </div>
</template>

<style scoped>
.merge-action-bar {
  height: 64px;
  border-top: 1px solid theme('colors.ms-border');
  background: theme('colors.ms-carbon');
  display: flex;
  align-items: center;
  padding: 0 16px;
  gap: 16px;
  flex-shrink: 0;
}

.merge-action-bar__status {
  flex: 1;
  min-width: 0;
}

.merge-action-bar__warning {
  font-size: 12px;
  font-family: ui-monospace, monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  color: theme('colors.ms-engrave');
}

.merge-action-bar__warning--active {
  color: theme('colors.ms-warning');
}

.merge-action-bar__error {
  font-size: 10px;
  color: theme('colors.ms-danger');
  font-family: ui-monospace, monospace;
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.merge-action-bar__button {
  position: relative;
  height: 40px;
  min-width: 180px;
  overflow: hidden;
  transition: all 150ms ease;
  user-select: none;
}

.merge-action-bar__button--active {
  background: rgba(184, 134, 11, 0.05);
  border: 1px solid rgba(184, 134, 11, 0.3);
  cursor: pointer;
}

.merge-action-bar__button--disabled {
  background: rgba(30, 41, 59, 0.5);
  border: 1px solid theme('colors.ms-border');
  cursor: not-allowed;
  opacity: 0.5;
}

.merge-action-bar__progress {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.merge-action-bar__progress-bg {
  color: rgba(184, 134, 11, 0.1);
}

.merge-action-bar__progress-circle {
  color: theme('colors.brass.DEFAULT');
  transition: all 75ms ease;
}

.merge-action-bar__progress-circle--glow {
  filter: drop-shadow(0 0 8px rgba(184, 134, 11, 0.4));
}

.merge-action-bar__button-text {
  position: relative;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 0 16px;
}

.merge-action-bar__button-label {
  font-size: 12px;
  font-family: ui-monospace, monospace;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: theme('colors.brass.DEFAULT');
}

.merge-action-bar__button-label--disabled {
  color: theme('colors.ms-engrave');
}
</style>
