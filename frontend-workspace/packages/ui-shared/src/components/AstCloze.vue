// ────────────────────────────────────────────────────────────────
// AstCloze.vue — memory mask: click/hover to reveal hidden text
// AstCloze 记忆遮罩：点击/悬停以显示隐藏文本
// ────────────────────────────────────────────────────────────────

<script setup lang="ts">
import { ref } from 'vue'
import type { AstNode } from '@memory-stream/types'
import AstRenderer from './AstRenderer.vue'
import { useIsTouchDevice } from '../composables/useIsTouchDevice'

defineProps<{ children: AstNode[] }>()

const { isTouchDevice } = useIsTouchDevice()
const revealed = ref(false)

function toggle() {
  revealed.value = !revealed.value
}
</script>

<template>
  <span class="ms-cloze" :class="{ 'is-revealed': revealed, 'is-touch': isTouchDevice }" @click="toggle">
    <span v-if="revealed" class="ms-cloze-content">
      <AstRenderer v-for="(child, i) in children" :key="i" :node="child" />
    </span>
    <span v-else class="ms-cloze-mask" aria-hidden="true">
      <span v-if="isTouchDevice" class="ms-cloze-hint">···</span>
    </span>
  </span>
</template>

<style scoped>
.ms-cloze {
  display: inline;
  cursor: pointer;
  user-select: none;
  border-radius: 2px;
  transition: background 200ms var(--ease-snap, cubic-bezier(0.33, 0, 0.2, 1));
}

.ms-cloze-mask {
  display: inline-block;
  min-width: 3em;
  height: 1.15em;
  vertical-align: baseline;
  background: oklch(0.22 0.02 75);
  border: 1px solid oklch(0.32 0.04 75);
  border-radius: 2px;
  position: relative;
  top: 0.1em;
}

.ms-cloze:hover .ms-cloze-mask {
  background: oklch(0.28 0.04 75);
  border-color: oklch(0.42 0.06 75);
}

.ms-cloze.is-revealed {
  color: #00e5ff;
  background: oklch(0.18 0.06 200 / 0.15);
  text-shadow: 0 0 4px oklch(0.78 0.17 200 / 0.25);
}

.ms-cloze-content {
  padding: 0 0.15em;
}

/* Touch device: larger mask + hint indicator */
.ms-cloze.is-touch .ms-cloze-mask {
  min-width: 3.5em;
  height: 1.3em;
}

.ms-cloze-hint {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.7em;
  color: oklch(0.5 0.03 75);
  letter-spacing: 0.1em;
}
</style>
