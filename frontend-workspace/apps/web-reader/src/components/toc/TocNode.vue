<script setup lang="ts">
/**
 * TocNode — 递归 TOC 树节点
 *
 * 支持任意深度嵌套，竖线辅助线 + 缩进。
 * 点击触发 jump 事件 → 父组件执行 scrollIntoView。
 */

interface TocItem {
    text: string;
    slug: string;
    level: number;
    children?: TocItem[];
}

defineProps<{
    item: TocItem;
    depth?: number;
    activeSlug?: string;
}>();

const emit = defineEmits<{
    jump: [slug: string];
}>();
</script>

<template>
    <li>
        <button class="w-full text-left text-xs py-1 px-2 rounded transition-all duration-200 truncate block" :class="activeSlug === item.slug
            ? 'text-neon bg-neon/10 font-medium'
            : 'text-gray-500 hover:text-gray-300 hover:bg-white/5'"
            :style="{ paddingLeft: `${(depth ?? 0) * 8 + 8}px` }" @click.prevent="emit('jump', item.slug)">
            {{ item.text }}
        </button>
        <ul v-if="item.children?.length" class="space-y-0.5 ml-3 border-l border-gray-800 pl-2">
            <TocNode v-for="child in item.children" :key="child.slug" :item="child" :depth="(depth ?? 0) + 1"
                :active-slug="activeSlug" @jump="(s: string) => emit('jump', s)" />
        </ul>
    </li>
</template>