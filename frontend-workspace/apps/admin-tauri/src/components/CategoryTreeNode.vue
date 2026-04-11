<script setup lang="ts">
import { ref, computed } from "vue";
import { ChevronRight, ChevronDown, Plus, Pencil, Trash2 } from "lucide-vue-next";

/**
 * CategoryTreeNode — 递归树形组件
 *
 * 用于渲染分类的层级结构，支持展开/折叠、选中高亮、悬停操作。
 */

interface CategoryTreeNodeData {
  id: number;
  name: string;
  description?: string;
  parent_id: number | null;
  children: CategoryTreeNodeData[];
}

const props = withDefaults(
  defineProps<{
    node: CategoryTreeNodeData;
    depth?: number;
    activeId?: number | null;
  }>(),
  {
    depth: 0,
    activeId: null,
  }
);

const emit = defineEmits<{
  select: [id: number];
  "create-child": [parentId: number];
  delete: [id: number, name: string];
  edit: [id: number, name: string];
}>();

// 展开状态
const expanded = ref(false);

// 是否有子节点
const hasChildren = computed(() => props.node.children?.length > 0);

// 切换展开状态
function toggleExpand() {
  if (hasChildren.value) {
    expanded.value = !expanded.value;
  }
  emit("select", props.node.id);
}

// 计算左侧缩进和树连接线
const treeLineStyle = computed(() => {
  const basePadding = 8;
  const indentPerLevel = 16;
  return {
    paddingLeft: `${basePadding + props.depth * indentPerLevel}px`,
    borderLeft: props.depth > 0 ? `1px solid rgba(184, 134, 11, 0.3)` : 'none',
  };
});
</script>

<template>
  <div class="tree-node">
    <!-- Node row -->
    <div
      class="flex items-center gap-1.5 py-1.5 px-2 cursor-pointer group transition-all duration-150 relative"
      :class="[
        activeId === node.id
          ? 'bg-brass/5 shadow-brass-glow-sm'
          : 'hover:bg-neon/5',
      ]"
      :style="treeLineStyle"
      @click="toggleExpand"
    >
      <!-- Active indicator border -->
      <div
        v-if="activeId === node.id"
        class="absolute left-0 top-0 bottom-0 w-0.5 bg-brass shadow-brass-glow-sm"
      />

      <!-- Arrow or dot indicator -->
      <ChevronDown
        v-if="hasChildren && expanded"
        :size="14"
        class="text-slate-400 transition-transform duration-200 shrink-0"
      />
      <ChevronRight
        v-else-if="hasChildren"
        :size="14"
        class="text-slate-400 transition-transform duration-200 shrink-0"
      />
      <div v-else class="w-3.5 h-3.5 flex items-center justify-center shrink-0">
        <div class="w-1.5 h-1.5 bg-slate-600" />
      </div>

      <!-- Name -->
      <span
        class="text-xs truncate flex-1 transition-colors duration-150"
        :class="activeId === node.id ? 'text-brass font-medium' : 'text-slate-300'"
      >
        {{ node.name }}
      </span>

      <!-- Action buttons (hover only) -->
      <div class="hidden group-hover:flex items-center gap-0.5">
        <button
          @click.stop="$emit('create-child', node.id)"
          class="p-1 rounded-sharp text-slate-500 hover:text-neon hover:bg-neon/10 transition-colors"
          title="添加子分类"
        >
          <Plus :size="12" />
        </button>
        <button
          @click.stop="$emit('edit', node.id, node.name)"
          class="p-1 rounded-sharp text-slate-500 hover:text-neon hover:bg-neon/10 transition-colors"
          title="编辑"
        >
          <Pencil :size="12" />
        </button>
        <button
          @click.stop="$emit('delete', node.id, node.name)"
          class="p-1 rounded-sharp text-slate-500 hover:text-red-400 hover:bg-red-400/10 transition-colors"
          title="删除"
        >
          <Trash2 :size="12" />
        </button>
      </div>
    </div>

    <!-- Children with expand transition -->
    <Transition name="tree-expand">
      <div v-if="hasChildren && expanded && depth < 10" class="mt-0.5">
        <CategoryTreeNode
          v-for="child in node.children"
          :key="child.id"
          :node="child"
          :depth="depth + 1"
          :active-id="activeId"
          @select="$emit('select', $event)"
          @create-child="$emit('create-child', $event)"
          @delete="(id, name) => $emit('delete', id, name)"
          @edit="(id, name) => $emit('edit', id, name)"
        />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
/* Tree expand transition */
.tree-expand-enter-active,
.tree-expand-leave-active {
  transition: max-height 200ms ease, opacity 200ms ease;
  overflow: hidden;
}

.tree-expand-enter-from,
.tree-expand-leave-to {
  max-height: 0;
  opacity: 0;
}

.tree-expand-enter-to,
.tree-expand-leave-from {
  max-height: 500px;
  opacity: 1;
}
</style>
