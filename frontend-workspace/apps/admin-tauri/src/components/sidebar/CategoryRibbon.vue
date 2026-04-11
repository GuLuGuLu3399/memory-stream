<script setup lang="ts">
import { hexForKey } from '../../composables/useCategoryTheme';

interface Category {
  id: number;
  name: string;
  theme_color?: string | null;
}

interface Props {
  categories: Category[];
  selectedCategoryId: number | null;
}

interface Emits {
  (e: 'select', id: number | null): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

function handleSelect(id: number | null) {
  emit('select', id);
}
</script>

<template>
  <div v-if="categories.length > 0" class="flex px-3 py-1.5 gap-3 overflow-x-auto border-b border-ms-border/50 no-scrollbar">
    <button
      @click="handleSelect(null)"
      class="shrink-0 font-mono text-2xs tracking-wider uppercase pb-0.5 border-b-2 transition-all relative"
      :class="selectedCategoryId === null
        ? 'text-brass border-b-brass after:absolute after:bottom-0 after:left-0 after:right-0 after:h-px after:bg-brass/60'
        : 'text-slate-600 border-b-transparent hover:text-slate-400'
      "
    >
      ALL
    </button>
    <button
      v-for="cat in categories"
      :key="cat.id"
      @click="handleSelect(cat.id)"
      class="shrink-0 font-mono text-2xs tracking-wider uppercase pb-0.5 border-b-2 transition-all relative"
      :style="(selectedCategoryId === cat.id && hexForKey(cat.theme_color))
        ? {
            color: hexForKey(cat.theme_color)!,
            borderBottomColor: hexForKey(cat.theme_color)!
          }
        : undefined"
      :class="selectedCategoryId === cat.id
        ? (hexForKey(cat.theme_color) ? 'after:absolute after:bottom-0 after:left-0 after:right-0 after:h-px' : 'text-brass border-b-brass after:absolute after:bottom-0 after:left-0 after:right-0 after:h-px after:bg-brass/60')
        : 'text-slate-600 border-b-transparent hover:text-slate-400'
      "
      :style-after="(selectedCategoryId === cat.id && hexForKey(cat.theme_color))
        ? { backgroundColor: hexForKey(cat.theme_color)! + '99' }
        : undefined"
    >
      {{ cat.name }}
    </button>
  </div>
</template>
