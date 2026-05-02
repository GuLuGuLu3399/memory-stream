<script setup lang="ts">
// 用途：切换开关组件，支持双状态切换和禁用态
const props = defineProps<{
  modelValue: boolean
  disabled?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

function toggle() {
  if (!props.disabled) emit('update:modelValue', !props.modelValue)
}
</script>

<template>
  <button
    class="toggle-track"
    :class="{ on: modelValue, disabled }"
    role="switch"
    :aria-checked="modelValue"
    :disabled="disabled"
    @click="toggle"
  >
    <span class="toggle-thumb" />
  </button>
</template>

<style scoped>
.toggle-track {
  position: relative;
  width: 36px;
  height: 20px;
  border: none;
  border-radius: 2px;
  background: var(--ms-carbon);
  box-shadow: var(--shadow-inset);
  cursor: pointer;
  padding: 0;
  transition: background var(--duration-normal) var(--ease-hydraulic),
    box-shadow var(--duration-normal) var(--ease-hydraulic);
}

.toggle-track.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.toggle-thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  border-radius: 1px;
  background: var(--ms-surface);
  box-shadow: 0 1px 2px oklch(0 0 0 / 0.4);
  transition: transform var(--duration-normal) var(--ease-hydraulic),
    background var(--duration-normal) var(--ease-hydraulic),
    box-shadow var(--duration-normal) var(--ease-hydraulic);
}

.toggle-track.on {
  background: color-mix(in oklch, var(--neon) 15%, var(--ms-carbon));
  box-shadow: var(--shadow-inset), 0 0 0 1px color-mix(in oklch, var(--neon) 30%, transparent);
}

.toggle-track.on .toggle-thumb {
  transform: translateX(16px);
  background: var(--neon);
  box-shadow: 0 1px 2px oklch(0 0 0 / 0.3), 0 0 6px oklch(0.78 0.17 200 / 0.3);
}

.toggle-track:not(.disabled):active .toggle-thumb {
  transform: scaleX(1.15);
}

.toggle-track:not(.disabled).on:active .toggle-thumb {
  transform: translateX(16px) scaleX(1.15);
}
</style>
