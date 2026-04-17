<script setup lang="ts">
import { ref, computed } from "vue";
import { ChevronRight, ChevronDown, Plus, Pencil, Trash2 } from "lucide-vue-next";

/**
 * CategoryTreeNode — 递归树形组件（机械祭坛·分类索引）
 *
 * 每个节点显示分类名 + 主题色指示点。
 * 激活态左侧黄铜高亮条 + 微辉。
 * 悬浮操作按钮：添加子分类 / 编辑 / 删除。
 */

interface CategoryTreeNodeData {
  id: number;
  name: string;
  description?: string;
  parent_id: number | null;
  theme_color?: string | null;
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

// 计算左侧缩进
const indentStyle = computed(() => {
  const basePadding = 12;
  const indentPerLevel = 20;
  return {
    paddingLeft: `${basePadding + props.depth * indentPerLevel}px`,
  };
});

// 连接线（子树左侧黄铜竖线）
const traceLineStyle = computed(() => {
  if (props.depth === 0) return {};
  return {
    borderColor: "rgba(184, 134, 11, 0.2)",
  };
});
</script>

<template>
  <div class="tree-node">
    <!-- Node row -->
    <div
      class="cat-node__row"
      :class="{ 'cat-node__row--active': activeId === node.id }"
      :style="indentStyle"
      @click="toggleExpand"
    >
      <!-- Active indicator -->
      <div
        v-if="activeId === node.id"
        class="cat-node__active-bar"
      />

      <!-- Tree trace line (depth > 0) -->
      <div
        v-if="depth > 0"
        class="cat-node__trace"
        :style="traceLineStyle"
      />

      <!-- Expand/collapse arrow or leaf dot -->
      <ChevronDown
        v-if="hasChildren && expanded"
        :size="13"
        class="shrink-0 text-brass/60 transition-transform duration-200"
      />
      <ChevronRight
        v-else-if="hasChildren"
        :size="13"
        class="shrink-0 text-slate-600 transition-transform duration-200"
      />
      <div v-else class="w-[13px] h-[13px] flex items-center justify-center shrink-0">
        <div class="w-[5px] h-[5px] rounded-full bg-slate-700" />
      </div>

      <!-- Theme color swatch -->
      <div
        v-if="node.theme_color"
        class="cat-node__swatch"
        :style="{ backgroundColor: `var(--cat-color, #555)` }"
      />
      <div v-else class="cat-node__swatch cat-node__swatch--empty" />

      <!-- Name -->
      <span
        class="cat-node__label"
        :class="{ 'cat-node__label--active': activeId === node.id }"
      >
        {{ node.name }}
      </span>

      <!-- Child count badge -->
      <span
        v-if="hasChildren"
        class="cat-node__count"
      >
        {{ node.children.length }}
      </span>

      <!-- Hover actions -->
      <div class="cat-node__actions">
        <button
          @click.stop="$emit('create-child', node.id)"
          class="cat-node__action-btn"
          title="添加子分类"
        >
          <Plus :size="12" />
        </button>
        <button
          @click.stop="$emit('edit', node.id, node.name)"
          class="cat-node__action-btn"
          title="编辑"
        >
          <Pencil :size="12" />
        </button>
        <button
          @click.stop="$emit('delete', node.id, node.name)"
          class="cat-node__action-btn cat-node__action-btn--danger"
          title="删除"
        >
          <Trash2 :size="12" />
        </button>
      </div>
    </div>

    <!-- Children -->
    <Transition name="tree-expand">
      <div v-if="hasChildren && expanded && depth < 10" class="mt-px">
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
.cat-node__row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  cursor: pointer;
  position: relative;
  transition: background-color 120ms ease;
  border-left: 2px solid transparent;
}

.cat-node__row:hover {
  background: rgba(255, 255, 255, 0.02);
}

.cat-node__row:hover .cat-node__actions {
  display: flex;
}

.cat-node__row:hover .cat-node__count {
  display: none;
}

.cat-node__row--active {
  background: rgba(184, 134, 11, 0.04);
  border-left-color: theme('colors.brass.DEFAULT');
}

.cat-node__active-bar {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 2px;
  background: theme('colors.brass.DEFAULT');
  box-shadow: 0 0 6px rgba(184, 134, 11, 0.3);
}

.cat-node__trace {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 1px;
  border-left: 1px dashed rgba(184, 134, 11, 0.2);
}

.cat-node__swatch {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
  box-shadow: 0 0 4px rgba(0, 0, 0, 0.3);
}

.cat-node__swatch--empty {
  background: #3a3a3a;
  border: 1px solid #4a4a4a;
}

.cat-node__label {
  font-size: 12px;
  color: theme('colors.slate.400');
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  transition: color 120ms ease;
  font-family: ui-monospace, monospace;
}

.cat-node__label--active {
  color: theme('colors.brass.DEFAULT');
  font-weight: 600;
}

.cat-node__count {
  font-size: 9px;
  font-family: ui-monospace, monospace;
  color: theme('colors.slate.600');
  background: rgba(255, 255, 255, 0.03);
  padding: 1px 5px;
  border-radius: 2px;
  flex-shrink: 0;
}

.cat-node__actions {
  display: none;
  align-items: center;
  gap: 1px;
  flex-shrink: 0;
}

.cat-node__action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  color: theme('colors.slate.500');
  transition: color 100ms ease, background-color 100ms ease;
}

.cat-node__action-btn:hover {
  color: theme('colors.neon.DEFAULT');
  background: rgba(0, 229, 255, 0.06);
}

.cat-node__action-btn--danger:hover {
  color: theme('colors.red.400');
  background: rgba(239, 68, 68, 0.06);
}

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
