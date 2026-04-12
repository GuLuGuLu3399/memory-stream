<script setup lang="ts">
const props = defineProps<{
  isActive: boolean;
}>();

const emit = defineEmits<{
  (e: "click"): void;
}>();
</script>

<template>
  <button
    class="zen-seal"
    :class="{ 'zen-seal--entering': isActive }"
    @click.stop="emit('click')"
    title="进入禅模式"
  >
    禅
  </button>
</template>

<style scoped>
/* ── 朱印禅章 — 移动端禅模式入口 ── */
.zen-seal {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 2px;
  background: rgba(166, 38, 38, 0.08);
  border: 1px solid rgba(166, 38, 38, 0.2);
  color: rgba(166, 38, 38, 0.5);
  font-size: 11px;
  font-weight: 700;
  font-family: 'Noto Serif SC', serif;
  cursor: pointer;
  transition: all 250ms cubic-bezier(0.16, 1, 0.3, 1);
  -webkit-tap-highlight-color: transparent;
  letter-spacing: 0;
  line-height: 1;
  flex-shrink: 0;
}

.zen-seal:active {
  background: rgba(166, 38, 38, 0.2);
  border-color: rgba(166, 38, 38, 0.5);
  color: rgba(166, 38, 38, 0.95);
  transform: scale(0.88);
  box-shadow: 0 0 12px rgba(166, 38, 38, 0.25);
}

.zen-seal--entering {
  background: rgba(166, 38, 38, 0.15);
  border-color: rgba(166, 38, 38, 0.4);
  color: rgba(166, 38, 38, 0.8);
  animation: sealBreathe 2.5s ease-in-out infinite;
}

@keyframes sealBreathe {
  0%, 100% { box-shadow: 0 0 4px rgba(166, 38, 38, 0.1); }
  50% { box-shadow: 0 0 10px rgba(166, 38, 38, 0.25); }
}
</style>
