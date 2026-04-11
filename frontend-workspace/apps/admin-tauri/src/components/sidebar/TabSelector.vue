<script setup lang="ts">
import type { Component } from 'vue';

interface Tab {
  key: string;
  label: string;
  icon: Component;
}

interface Props {
  tabs: Tab[];
  activeTab: string;
}

interface Emits {
  (e: 'update:activeTab', value: any): void;
}

defineProps<Props>();
const emit = defineEmits<Emits>();

function selectTab(key: string) {
  emit('update:activeTab', key);
}
</script>

<template>
  <div class="flex px-3 pt-2 gap-0.5">
    <button
      v-for="tab in tabs"
      :key="tab.key"
      @click="selectTab(tab.key)"
      class="flex-1 text-2xs py-1.5 transition-all text-center font-mono tracking-wider relative"
      :class="activeTab === tab.key
        ? 'text-neon after:absolute after:bottom-0 after:left-2 after:right-2 after:h-px after:bg-neon/80 after:shadow-[0_0_8px_rgba(0,229,255,0.6)]'
        : 'text-slate-600 hover:text-slate-400'
      "
    >
      <component :is="tab.icon" :size="10" class="inline-block align-middle mr-0.5" />
      {{ tab.label }}
    </button>
  </div>
</template>
