<script setup lang="ts">
/**
 * TocNode — 递归 TOC 树节点（血肉神殿）
 *
 * 支持活跃状态左侧血珀边框动画、古铜色缩进指引线、悬停背景增强。
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
    <li class="relative">
        <button
            class="node-button w-full text-left text-xs py-1.5 px-2 truncate block relative transition-all duration-200"
            :class="activeSlug === item.slug
                ? 'text-xuepo bg-xuepo/5 font-medium'
                : 'text-ms-smoke hover:text-ms-bone hover:bg-ms-mo/50'"
            :style="{ paddingLeft: `${(depth ?? 0) * 8 + 8}px` }"
            @click.prevent="emit('jump', item.slug)">
            <!-- 活跃状态的左侧血珀边框 -->
            <span
                v-if="activeSlug === item.slug"
                class="active-border"
            />
            {{ item.text }}
        </button>

        <!-- 子节点列表 -->
        <ul v-if="item.children?.length" class="space-y-0.5 ml-3 border-l border-ms-copper/30 pl-2 relative">
            <!-- 缩进连接点 -->
            <li
                v-for="child in item.children"
                :key="child.slug"
                class="relative child-node">
                <TocNode
                    :item="child"
                    :depth="(depth ?? 0) + 1"
                    :active-slug="activeSlug"
                    @jump="(s: string) => emit('jump', s)"
                />
                <!-- 古铜色连接点 -->
                <span class="connection-dot" />
            </li>
        </ul>
    </li>
</template>

<style scoped>
/* ── 节点按钮基础样式 ── */
.node-button {
    position: relative;
    overflow: hidden;
}

/* ── 活跃状态左侧血珀边框动画 ── */
.active-border {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 2px;
    background: theme('colors.xuepo.DEFAULT');
    transform-origin: top;
    animation: border-slide-in 0.3s cubic-bezier(0.16, 1, 0.3, 1) forwards;
    box-shadow: 0 0 8px rgba(166, 38, 38, 0.4);
}

@keyframes border-slide-in {
    from {
        transform: scaleY(0);
    }

    to {
        transform: scaleY(1);
    }
}

/* ── 子节点容器 ── */
.child-node {
    position: relative;
}

/* ── 古铜色连接点 ── */
.connection-dot {
    position: absolute;
    left: -5px;
    top: 0.75em;
    width: 4px;
    height: 4px;
    background: theme('colors.ms-copper');
    border-radius: 50%;
    opacity: 0.6;
    transition: all 0.2s ease;
}

.child-node:hover .connection-dot {
    opacity: 1;
    background: theme('colors.ms-gold');
    box-shadow: 0 0 4px rgba(201, 168, 76, 0.4);
}

/* ── 子节点列表的缩进线增强 ── */
ul > li {
    position: relative;
}
</style>
