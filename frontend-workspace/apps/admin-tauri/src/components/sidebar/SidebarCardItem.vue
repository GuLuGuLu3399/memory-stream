<script setup lang="ts">
import { computed } from 'vue';
import { Settings } from 'lucide-vue-next';

interface Props {
  card: {
    id: string;
    title: string;
    content?: string;
    category_id?: number | null;
  };
  isSelected: boolean;
  categoryInfo?: Map<number, string>;
  index?: number;
}

interface Emits {
  (e: 'select', id: string): void;
  (e: 'delete', id: string, title: string): void;
}

const props = withDefaults(defineProps<Props>(), {
  index: 0,
});

const emit = defineEmits<Emits>();

const categoryName = computed(() => {
  if (!props.card.category_id || !props.categoryInfo) return null;
  return props.categoryInfo.get(props.card.category_id) || null;
});

const preview = computed(() => {
  if (!props.card.content) return '';
  const firstLine = props.card.content.split('\n').find((l) => l.trim()) || '';
  const cleaned = firstLine
    .replace(/\[\[([^\]]+)\]\]/g, '$1')
    .replace(/\*\*([^*]+)\*\*/g, '$1')
    .replace(/\*([^*]+)\*/g, '$1')
    .replace(/`([^`]+)`/g, '$1')
    .replace(/#{1,6}\s/g, '')
    .trim();
  if (cleaned.length <= 40) return cleaned;
  const truncated = cleaned.slice(0, 40);
  const lastSpace = truncated.lastIndexOf(' ');
  return (lastSpace > 20 ? truncated.slice(0, lastSpace) : truncated) + '…';
});
</script>

<template>
  <div
    @click="emit('select', card.id)"
    class="group relative p-2.5 rounded-sm cursor-pointer transition-all border-l-2"
    :class="isSelected
      ? 'bg-brass/10 text-brass border-l-brass shadow-[inset_0_1px_0_0_rgba(255,255,255,0.04),2px_2px_0_0_rgba(0,0,0,0.4)]'
      : 'bg-transparent hover:bg-ms-panel/50 text-slate-400 border-l-slate-700 hover:border-l-slate-500 hover:shadow-[1px_1px_0_0_rgba(0,0,0,0.3)] active:translate-x-[1px] active:translate-y-[1px] active:shadow-none'
    "
  >
    <div class="flex items-center justify-between">
      <div class="truncate text-xs flex-1 mr-2">
        {{ card.title || '无标题' }}
        <span v-if="card.title && !card.content" class="text-3xs text-slate-700 ml-1">∅</span>
      </div>
      <span
        v-if="categoryName"
        class="shrink-0 text-3xs bg-ms-deep text-slate-500 px-1.5 py-0.5 rounded-sm font-mono mr-1 border border-ms-border shadow-[1px_1px_0_0_rgba(0,0,0,0.3)]"
      >
        {{ categoryName }}
      </span>
      <button
        @click.stop="emit('delete', card.id, card.title)"
        class="shrink-0 opacity-0 group-hover:opacity-100 text-slate-600 hover:text-brass transition-all px-0.5 rounded-sm"
        title="删除卡片"
      >
        <Settings :size="12" />
      </button>
    </div>
    <div
      v-if="card.content"
      class="truncate text-2xs mt-0.5 font-mono"
      :class="isSelected ? 'text-brass/40' : 'text-slate-600'"
    >
      {{ preview }}
    </div>
  </div>
</template>
