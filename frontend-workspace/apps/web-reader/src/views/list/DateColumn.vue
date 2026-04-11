<script setup lang="ts">
/**
 * DateColumn — Date label in vertical text for ListView
 *
 * Displays date label in vertical-rl writing mode with gold-leaf decoration.
 * Active dates feature ms-gold accent color.
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
      class="sticky top-4 z-20 font-mono text-2xs tracking-widest whitespace-nowrap select-none transition-colors duration-300"
      :class="isActive ? 'text-ms-gold' : 'text-ms-ash'"
      style="writing-mode: vertical-rl; transform: rotate(180deg);"
    >
      {{ dateLabel }}
    </div>
  </div>
</template>
