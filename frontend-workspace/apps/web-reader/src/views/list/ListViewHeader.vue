<script setup lang="ts">
/**
 * ListViewHeader — Top header for ListView
 *
 * Displays sort indicator and item count with minimal styling.
 * Emits toggleSort event when sort indicator is clicked.
 */

interface Props {
  sortBy: 'updated' | 'hot';
  itemCount: number;
}

defineProps<Props>();

const emit = defineEmits<{
  toggleSort: [];
}>();
</script>

<template>
  <div class="flex items-center justify-between px-8 pb-4 mb-2 border-b border-ms-copper/30">
    <!-- Sort indicator -->
    <button
      class="flex items-center gap-2 text-ms-bone-dim hover:text-ms-bone transition-colors duration-200"
      @click="emit('toggleSort')"
    >
      <span class="text-xs font-mono tracking-wider uppercase">
        {{ sortBy === 'hot' ? '热度排序' : '时间排序' }}
      </span>
      <svg
        class="w-3 h-3 transition-transform duration-300"
        :class="sortBy === 'hot' ? 'rotate-180 text-xuepo' : ''"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>

    <!-- Item count -->
    <div class="text-xs font-mono text-ms-ash">
      {{ itemCount }} <span class="text-ms-smoke/50">卡片</span>
    </div>
  </div>
</template>
