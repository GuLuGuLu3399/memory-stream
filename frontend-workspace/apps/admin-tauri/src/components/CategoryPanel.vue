<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { storeToRefs } from "pinia";
import { useLayoutStore } from "../stores/layout";
import { useCategoryStore } from "../stores/useCategoryStore";
import { useKnowledgeStore } from "../stores/knowledge";
import { useConfirmDialog } from "../composables/useConfirmDialog";
import CategoryTreeNode from "./CategoryTreeNode.vue";
import { X, Plus, Settings } from "lucide-vue-next";
import { THEME_DICT, THEME_KEYS } from "../composables/useCategoryTheme";

const layoutStore = useLayoutStore();
const { isCategoryPanelOpen } = storeToRefs(layoutStore);
const categoryStore = useCategoryStore();
const { categories } = storeToRefs(categoryStore);
const knowledgeStore = useKnowledgeStore();
const { orphanCards, recentCards } = storeToRefs(knowledgeStore);
const { confirm } = useConfirmDialog();

// Selected category for right panel
const selectedId = ref<number | null>(null);
const editName = ref("");
const editDescription = ref("");
const movingCardId = ref<string | null>(null);
const editThemeColor = ref<string | null>(null);

// Build tree from flat categories
interface TreeNode {
  id: number;
  name: string;
  description: string;
  parent_id: number | null;
  children: TreeNode[];
}
const treeData = computed<TreeNode[]>(() => {
  const map = new Map<number, TreeNode>();
  const roots: TreeNode[] = [];
  for (const cat of categories.value) {
    map.set(cat.id, {
      id: cat.id,
      name: cat.name,
      description: "",
      parent_id: cat.parent_id,
      children: [],
    });
  }
  for (const cat of categories.value) {
    const node = map.get(cat.id)!;
    if (cat.parent_id && map.has(cat.parent_id)) {
      map.get(cat.parent_id)!.children.push(node);
    } else {
      roots.push(node);
    }
  }
  // Sort roots and children alphabetically
  roots.sort((a, b) => a.name.localeCompare(b.name, "zh-CN"));
  for (const node of map.values()) {
    node.children.sort((a, b) => a.name.localeCompare(b.name, "zh-CN"));
  }
  return roots;
});

// Cards belonging to the selected category
const categoryCards = computed(() => {
  if (!selectedId.value) return [];
  const allCards = [...recentCards.value, ...orphanCards.value];
  const seen = new Set<string>();
  return allCards.filter(c => {
    if (seen.has(c.id) || c.category_id !== selectedId.value) return false;
    seen.add(c.id);
    return true;
  });
});

const categoryCardCount = computed(() => categoryCards.value.length);

const categoryChildCount = computed(() => {
  if (!selectedId.value) return 0;
  return categories.value.filter(c => c.parent_id === selectedId.value).length;
});

// New category input
const newCategoryName = ref("");
const newCategoryParentId = ref<number | null>(null);

// Esc key handler
function handleEsc(e: KeyboardEvent) {
  if (e.key === "Escape") layoutStore.closeCategoryPanel();
}

onMounted(() => {
  window.addEventListener("keydown", handleEsc);
  categoryStore.loadCategories();
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleEsc);
});

// When selecting a category in the tree
function handleSelect(id: number) {
  selectedId.value = id;
  const cat = categories.value.find((c) => c.id === id);
  if (cat) {
    editName.value = cat.name;
    editDescription.value = cat.description || "";
    editThemeColor.value = cat.theme_color || null;
  }
}

// Create new category
async function handleCreateCategory() {
  const name = newCategoryName.value.trim();
  if (!name) return;
  await categoryStore.createCategory(name, "");
  newCategoryName.value = "";
  newCategoryParentId.value = null;
}

// Save edited category
async function handleSave() {
  if (selectedId.value && editName.value.trim()) {
    await categoryStore.updateCategory(
      selectedId.value,
      editName.value.trim(),
      editDescription.value,
      editThemeColor.value,
    );
  }
}

// Delete with confirmation
async function handleDelete(id: number, name: string) {
  const ok = await confirm(`确定要删除分类「${name}」吗？该分类下的卡片将变为未分类。`, {
    title: "删除分类",
    confirmText: "删除",
    danger: true,
  });
  if (ok) {
    await categoryStore.deleteCategory(id);
    if (selectedId.value === id) selectedId.value = null;
  }
}

// Edit category — populate form with existing data
function handleEdit(id: number, name: string) {
  selectedId.value = id;
  editName.value = name;
  editDescription.value = categories.value.find(c => c.id === id)?.description || "";
  editThemeColor.value = categories.value.find(c => c.id === id)?.theme_color || null;
}

// Create child category
function handleCreateChild(parentId: number) {
  newCategoryParentId.value = parentId;
  newCategoryName.value = "";
}

// Unlink card from category
async function handleUnlinkCard(cardId: string) {
  await knowledgeStore.unlinkCardFromCategory(cardId);
}

async function handleMoveCard(cardId: string, targetCategoryId: number) {
  await knowledgeStore.updateCardCategory(cardId, targetCategoryId);
  movingCardId.value = null;
}
</script>

<template>
  <Teleport to="body">
    <Transition name="ms-slide-right">
      <div
        v-if="isCategoryPanelOpen"
        class="fixed inset-0 z-panel"
        @click.self="layoutStore.closeCategoryPanel()"
      >
        <!-- Blur backdrop -->
        <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" />

        <!-- Panel container -->
        <div
          class="absolute inset-4 md:inset-8 bg-ms-panel rounded-sm border border-ms-border shadow-2xl overflow-hidden flex flex-col"
        >
          <!-- Header -->
          <div
            class="h-14 flex items-center px-6 border-b border-ms-border shrink-0"
          >
            <Settings :size="18" class="text-neon mr-2" />
            <h2 class="text-sm font-bold text-white tracking-wider">分类管理</h2>
            <button
              @click="layoutStore.closeCategoryPanel()"
              class="ml-auto w-8 h-8 flex items-center justify-center rounded-sm text-slate-500 hover:text-slate-300 hover:bg-ms-surface transition-all"
              title="关闭"
            >
              <X :size="18" />
            </button>
          </div>

          <!-- Split view body -->
          <div class="flex flex-1 min-h-0">
            <!-- Left: Tree (40%) -->
            <div class="w-2/5 border-r border-ms-border flex flex-col">
              <div class="p-4 flex-1 overflow-y-auto custom-scrollbar">
                <!-- New category input -->
                <div class="flex gap-2 mb-4">
                  <input
                    v-model="newCategoryName"
                    placeholder="新建分类..."
                    class="flex-1 bg-ms-deep text-slate-300 text-sm rounded-sm px-3 py-2 border border-ms-border outline-none focus:border-neon placeholder-slate-600 transition-all"
                    @keyup.enter="handleCreateCategory"
                  />
                  <button
                    @click="handleCreateCategory"
                    :disabled="!newCategoryName.trim()"
                    class="flex items-center gap-1 text-xs bg-neon/10 text-neon px-3 py-2 rounded-sm hover:bg-neon/20 transition-all border border-neon/30 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <Plus :size="14" />
                    <span>新建</span>
                  </button>
                </div>

                <!-- Parent indicator when creating child -->
                <div
                  v-if="newCategoryParentId"
                  class="mb-3 px-3 py-2 bg-amber-500/10 border border-amber-500/30 rounded-sm text-xs text-amber-400"
                >
                  将在「{{
                    categories.find((c) => c.id === newCategoryParentId)?.name
                  }}」下创建子分类
                  <button
                    @click="newCategoryParentId = null"
                    class="ml-2 text-amber-300 hover:text-amber-100"
                  >
                    取消
                  </button>
                </div>

                <!-- Tree -->
                <div v-if="treeData.length > 0" class="space-y-1">
                  <CategoryTreeNode
                    v-for="node in treeData"
                    :key="node.id"
                    :node="node"
                    :active-id="selectedId"
                    @select="handleSelect"
                    @delete="handleDelete"
                    @create-child="handleCreateChild"
                    @edit="handleEdit"
                  />
                </div>
                <div
                  v-else
                  class="text-center py-8 text-slate-600 text-sm"
                >
                  暂无分类，在上方输入框创建第一个分类
                </div>
              </div>
            </div>

            <!-- Right: Category Dashboard (60%) -->
            <div class="w-3/5 flex flex-col">
              <div v-if="selectedId" class="p-6 flex-1 overflow-y-auto custom-scrollbar">

                <!-- Section A: Data Readout -->
                <div class="mb-6">
                  <h3 class="text-[10px] font-mono text-slate-600 uppercase tracking-widest mb-3">DATA READOUT</h3>
                  <div class="grid grid-cols-2 gap-3">
                    <div class="bg-ms-deep border border-ms-border p-3">
                      <div class="text-2xl font-mono text-neon">{{ categoryCardCount }}</div>
                      <div class="text-[10px] text-slate-600 uppercase tracking-widest mt-1">CARDS</div>
                    </div>
                    <div class="bg-ms-deep border border-ms-border p-3">
                      <div class="text-2xl font-mono text-neon">{{ categoryChildCount }}</div>
                      <div class="text-[10px] text-slate-600 uppercase tracking-widest mt-1">CHILDREN</div>
                    </div>
                  </div>
                </div>

                <!-- Section B: Quick Index -->
                <div class="mb-6">
                  <h3 class="text-[10px] font-mono text-slate-600 uppercase tracking-widest mb-3">QUICK INDEX</h3>
                  <div v-if="categoryCards.length > 0" class="space-y-1">
                    <div v-for="card in categoryCards" :key="card.id"
                      class="flex items-center justify-between px-3 py-1.5 bg-ms-deep/50 border border-ms-border/50 group">
                      <span class="text-xs text-slate-400 truncate flex-1 mr-2">{{ card.title || '无标题' }}</span>
                      <button @click="handleUnlinkCard(card.id)"
                        class="shrink-0 opacity-0 group-hover:opacity-100 text-[10px] text-slate-600 hover:text-red-400 transition-all font-mono"
                        title="从分类中移除">
                        UNLINK
                      </button>
                      <div class="relative shrink-0">
                        <button @click.stop="movingCardId = movingCardId === card.id ? null : card.id"
                          class="opacity-0 group-hover:opacity-100 text-[10px] text-slate-600 hover:text-neon transition-all font-mono"
                          title="迁移到其他分类">
                          MOVE
                        </button>
                        <div v-if="movingCardId === card.id" @click.stop
                          class="absolute right-0 top-5 w-40 bg-ms-carbon border border-ms-border rounded-sm shadow-xl z-20 max-h-40 overflow-y-auto">
                          <button v-for="cat in categories.filter(c => c.id !== selectedId)" :key="cat.id"
                            @click="handleMoveCard(card.id, cat.id)"
                            class="w-full text-left px-3 py-1.5 text-xs text-slate-400 hover:bg-neon/10 hover:text-neon transition truncate">
                            {{ cat.name }}
                          </button>
                          <div v-if="categories.filter(c => c.id !== selectedId).length === 0" class="px-3 py-1.5 text-xs text-slate-600">
                            无其他分类
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div v-else class="text-[10px] text-slate-700 italic font-mono py-2">NO CARDS IN THIS CATEGORY</div>
                </div>

                <!-- Section C: Edit + Danger Zone -->
                <div>
                  <h3 class="text-[10px] font-mono text-slate-600 uppercase tracking-widest mb-3">EDIT</h3>
                  <div class="space-y-4">
                    <div>
                      <label class="block text-xs text-slate-400 mb-1.5">分类名称</label>
                      <input v-model="editName"
                        class="w-full bg-ms-deep text-slate-200 text-sm rounded-sm px-4 py-2.5 border border-ms-border outline-none focus:border-neon transition-all"
                        placeholder="输入分类名称" />
                    </div>
                    <div>
                      <label class="block text-xs text-slate-400 mb-1.5">分类描述</label>
                      <textarea v-model="editDescription"
                        class="w-full bg-ms-deep text-slate-200 text-sm rounded-sm px-4 py-2.5 border border-ms-border outline-none focus:border-neon transition-all resize-none h-20"
                        placeholder="输入分类描述（可选）" />
                    </div>
                    <div>
                      <label class="block text-xs text-slate-400 mb-1.5">专属能量色</label>
                      <div class="flex gap-2 flex-wrap">
                        <button v-for="key in THEME_KEYS" :key="key" @click="editThemeColor = key"
                          class="w-7 h-7 border-2 transition-all"
                          :class="editThemeColor === key ? 'border-white scale-110' : 'border-transparent hover:border-slate-500'"
                          :style="{ backgroundColor: THEME_DICT[key].hex }"
                          :title="THEME_DICT[key].label">
                        </button>
                        <button v-if="editThemeColor" @click="editThemeColor = null"
                          class="w-7 h-7 border border-ms-border flex items-center justify-center text-slate-600 hover:text-slate-400 transition-all"
                          title="清除颜色">
                          <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                          </svg>
                        </button>
                      </div>
                    </div>
                    <button @click="handleSave" :disabled="!editName.trim()"
                      class="w-full bg-transparent text-neon text-sm font-medium py-2 rounded-sm border border-neon/50 hover:bg-neon/10 transition-all disabled:opacity-30 disabled:cursor-not-allowed">
                      保存更改
                    </button>
                  </div>

                  <!-- Danger Zone -->
                  <div class="mt-6 pt-4 border-t border-ms-border/50">
                    <h3 class="text-[10px] font-mono text-red-500/60 uppercase tracking-widest mb-3">DANGER ZONE</h3>
                    <button @click="handleDelete(selectedId, categories.find(c => c.id === selectedId)?.name || '')"
                      class="w-full px-4 py-2 text-sm text-red-400 bg-transparent border border-red-500/30 rounded-sm hover:bg-red-500/10 transition-all">
                      删除分类
                    </button>
                  </div>
                </div>
              </div>
              <div v-else class="flex-1 flex items-center justify-center text-slate-600 text-sm">
                <div class="text-center">
                  <Settings :size="32" class="mx-auto mb-3 opacity-30" />
                  <p>选择左侧分类查看仪表盘</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #444;
  border-radius: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #555;
}
</style>
