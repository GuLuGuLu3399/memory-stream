<script setup lang="ts">
/**
 * DateColumn — 铭文（血肉神殿）
 *
 * 竖排日期铭文，模拟石碑侧面刻字：
 * - 激活日期：金色铭刻
 * - 非激活：暗铜色静默
 */
interface Props {
  dateLabel: string;
  isActive: boolean;
}

defineProps<Props>();
</script>

<template>
  <div class="relative flex items-start justify-end pr-3 pt-2">
    <div
      v-if="dateLabel"
      class="sticky top-4 z-20 font-mono text-2xs tracking-[0.2em] whitespace-nowrap select-none transition-all duration-500"
      :class="isActive ? 'date-inscription--active' : 'date-inscription--rest'"
      style="writing-mode: vertical-rl; transform: rotate(180deg);"
    >
      {{ dateLabel }}
    </div>
  </div>
</template>

<style scoped>
.date-inscription--rest {
    color: rgba(90, 79, 62, 0.4);
}

.date-inscription--active {
    color: #c9a84c;
    text-shadow: 0 0 6px rgba(201, 168, 76, 0.15);
}
</style>
