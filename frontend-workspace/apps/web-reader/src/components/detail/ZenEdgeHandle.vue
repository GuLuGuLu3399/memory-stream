<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  isActive: boolean;
  isMobile: boolean;
}>();

const emit = defineEmits<{
  (e: "click"): void;
}>();

const shouldShow = computed(() => !props.isMobile);
</script>

<template>
  <button
    v-if="shouldShow"
    class="zen-edge-handle z-chrome"
    :class="{ 'zen-edge-handle--active': isActive }"
    :title="isActive ? '退出禅模式' : '进入禅模式'"
    @click.stop="emit('click')"
  />
</template>

<style scoped>
/* ── 左侧隐形机械把手 — 有机-机械缝合线 ── */
.zen-edge-handle {
  position: fixed;
  right: 45%;
  top: 50%;
  transform: translateY(-50%) translateX(-50%);
  width: 10px;
  height: 96px;
  border-radius: 5px;
  background: linear-gradient(
    180deg,
    rgba(90, 79, 62, 0.15) 0%,
    rgba(138, 126, 110, 0.3) 30%,
    rgba(138, 126, 110, 0.35) 50%,
    rgba(138, 126, 110, 0.3) 70%,
    rgba(90, 79, 62, 0.15) 100%
  );
  border: 1px solid rgba(58, 50, 40, 0.4);
  border-left: none;
  border-radius: 0 5px 5px 0;
  padding: 0;
  cursor: w-resize;
  transition: all 350ms cubic-bezier(0.16, 1, 0.3, 1);
  box-shadow:
    inset 1px 0 0 0 rgba(138, 126, 110, 0.08),
    1px 0 4px rgba(0, 0, 0, 0.3);
  animation: zen-handle-breathe 4s ease-in-out infinite;
}

/* 中心机械凹槽 */
.zen-edge-handle::before {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: rgba(138, 126, 110, 0.4);
  box-shadow:
    0 -10px 0 rgba(138, 126, 110, 0.2),
    0 10px 0 rgba(138, 126, 110, 0.2),
    0 -20px 0 rgba(138, 126, 110, 0.1),
    0 20px 0 rgba(138, 126, 110, 0.1);
  transition: all 350ms cubic-bezier(0.16, 1, 0.3, 1);
}

/* 呼吸脉动 */
@keyframes zen-handle-breathe {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 0.65; }
}

.zen-edge-handle:hover {
  opacity: 1;
  width: 12px;
  height: 108px;
  background: linear-gradient(
    180deg,
    rgba(90, 79, 62, 0.3) 0%,
    rgba(200, 191, 168, 0.6) 30%,
    rgba(200, 191, 168, 0.7) 50%,
    rgba(200, 191, 168, 0.6) 70%,
    rgba(90, 79, 62, 0.3) 100%
  );
  border-color: rgba(200, 191, 168, 0.3);
  transform: translateY(-50%) translateX(-60%);
  box-shadow:
    inset 1px 0 0 0 rgba(200, 191, 168, 0.15),
    -3px 0 12px rgba(200, 191, 168, 0.12),
    1px 0 8px rgba(0, 0, 0, 0.4);
  animation: none;
}

.zen-edge-handle:hover::before {
  background: rgba(232, 223, 208, 0.7);
  box-shadow:
    0 -10px 0 rgba(232, 223, 208, 0.4),
    0 10px 0 rgba(232, 223, 208, 0.4),
    0 -20px 0 rgba(232, 223, 208, 0.2),
    0 20px 0 rgba(232, 223, 208, 0.2);
}

.zen-edge-handle:active {
  width: 14px;
  height: 104px;
  background: linear-gradient(
    180deg,
    rgba(166, 38, 38, 0.4) 0%,
    rgba(166, 38, 38, 0.85) 40%,
    rgba(166, 38, 38, 0.9) 50%,
    rgba(166, 38, 38, 0.85) 60%,
    rgba(166, 38, 38, 0.4) 100%
  );
  border-color: rgba(166, 38, 38, 0.5);
  transform: translateY(-50%) translateX(-70%);
  box-shadow:
    -6px 0 20px rgba(166, 38, 38, 0.35),
    -2px 0 8px rgba(166, 38, 38, 0.5);
  transition-duration: 80ms;
}

.zen-edge-handle:active::before {
  background: rgba(255, 255, 255, 0.6);
  box-shadow: 0 0 6px rgba(255, 255, 255, 0.3);
}

/* 禅模式激活 — 血珀缝合线脉动 */
.zen-edge-handle--active {
  background: linear-gradient(
    180deg,
    rgba(166, 38, 38, 0.15) 0%,
    rgba(166, 38, 38, 0.45) 30%,
    rgba(166, 38, 38, 0.5) 50%,
    rgba(166, 38, 38, 0.45) 70%,
    rgba(166, 38, 38, 0.15) 100%
  );
  border-color: rgba(166, 38, 38, 0.35);
  box-shadow:
    -2px 0 8px rgba(166, 38, 38, 0.2),
    1px 0 4px rgba(0, 0, 0, 0.3);
  animation: zen-handle-pulse 3s ease-in-out infinite;
}

.zen-edge-handle--active::before {
  background: rgba(166, 38, 38, 0.6);
  box-shadow:
    0 -10px 0 rgba(166, 38, 38, 0.3),
    0 10px 0 rgba(166, 38, 38, 0.3),
    0 -20px 0 rgba(166, 38, 38, 0.15),
    0 20px 0 rgba(166, 38, 38, 0.15);
}

@keyframes zen-handle-pulse {
  0%, 100% {
    opacity: 0.55;
    box-shadow: -2px 0 8px rgba(166, 38, 38, 0.15), 1px 0 4px rgba(0, 0, 0, 0.3);
  }
  50% {
    opacity: 0.8;
    box-shadow: -3px 0 14px rgba(166, 38, 38, 0.3), 1px 0 4px rgba(0, 0, 0, 0.3);
  }
}

.zen-edge-handle--active:hover {
  opacity: 1;
  background: linear-gradient(
    180deg,
    rgba(194, 54, 22, 0.4) 0%,
    rgba(194, 54, 22, 0.8) 30%,
    rgba(194, 54, 22, 0.9) 50%,
    rgba(194, 54, 22, 0.8) 70%,
    rgba(194, 54, 22, 0.4) 100%
  );
  border-color: rgba(194, 54, 22, 0.5);
  animation: none;
}

.zen-edge-handle--active:hover::before {
  background: rgba(224, 112, 112, 0.8);
  box-shadow:
    0 -10px 0 rgba(224, 112, 112, 0.5),
    0 10px 0 rgba(224, 112, 112, 0.5);
}

.zen-edge-handle--active:active {
  background: linear-gradient(
    180deg,
    rgba(201, 168, 76, 0.4) 0%,
    rgba(201, 168, 76, 0.9) 40%,
    rgba(201, 168, 76, 1) 50%,
    rgba(201, 168, 76, 0.9) 60%,
    rgba(201, 168, 76, 0.4) 100%
  );
  border-color: rgba(201, 168, 76, 0.5);
  box-shadow: -6px 0 24px rgba(201, 168, 76, 0.45);
}

.zen-edge-handle--active:active::before {
  background: rgba(255, 255, 255, 0.7);
  box-shadow: 0 0 8px rgba(201, 168, 76, 0.5);
}
</style>
