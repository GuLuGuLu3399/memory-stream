<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { storeToRefs } from "pinia";
import { useLayoutStore } from "../stores/layout";
import { useCategoryStore } from "../stores/useCategoryStore";
import { useKnowledgeStore } from "../stores/knowledge";
import { useConfirmDialog } from "../composables/useConfirmDialog";
import CategoryTreeNode from "./CategoryTreeNode.vue";
import ChamberHeader from "./ChamberHeader.vue";
import { Plus, FolderSearch, FileText, GripVertical } from "lucide-vue-next";
import { THEME_DICT, THEME_KEYS, hexForKey, toRgba } from "../composables/useCategoryTheme";

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

// Build tree from flat categories — includes theme_color
interface TreeNode {
  id: number;
  name: string;
  description: string;
  parent_id: number | null;
  theme_color: string | null;
  children: TreeNode[];
}
const treeData = computed<TreeNode[]>(() => {
  const map = new Map<number, TreeNode>();
  const roots: TreeNode[] = [];
  for (const cat of categories.value) {
    map.set(cat.id, {
      id: cat.id,
      name: cat.name,
      description: cat.description || "",
      parent_id: cat.parent_id,
      theme_color: cat.theme_color || null,
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
  roots.sort((a, b) => a.name.localeCompare(b.name, "zh-CN"));
  for (const node of map.values()) {
    node.children.sort((a, b) => a.name.localeCompare(b.name, "zh-CN"));
  }
  return roots;
});

// Total categories count
const totalCategories = computed(() => categories.value.length);

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

// Selected category's theme color as CSS variables for the diagnostic panel
const selectedThemeHex = computed(() => hexForKey(editThemeColor.value));
const themeGlowStyle = computed(() => {
  const hex = selectedThemeHex.value;
  if (!hex) return {};
  return {
    '--cat-color': hex,
    '--cat-color-10': toRgba(hex, 0.1),
    '--cat-color-20': toRgba(hex, 0.2),
    '--cat-color-05': toRgba(hex, 0.05),
  } as Record<string, string>;
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
  <div
    v-if="isCategoryPanelOpen"
    class="fixed inset-x-0 bottom-0 top-titlebar z-panel bg-ms-deep flex flex-col"
  >
    <!-- Header -->
    <ChamberHeader
      title="CATEGORIES"
      subtitle="分类档案库"
      accent="text-brass"
      @close="layoutStore.closeCategoryPanel()"
    />

    <!-- Split view body -->
    <div class="flex flex-1 min-h-0">

      <!-- ==================== LEFT: Specimen Index ==================== -->
      <div class="cat-index flex flex-col border-r border-ms-border bg-ms-carbon">
        <!-- Index sub-header -->
        <div class="cat-index__header">
          <div class="flex items-center gap-2">
            <GripVertical :size="12" class="text-brass/40" />
            <span class="text-2xs font-mono text-slate-500 uppercase tracking-[0.15em] font-bold">Specimen Index</span>
          </div>
          <span class="text-2xs font-mono text-brass/40 tabular-nums">{{ totalCategories }}</span>
        </div>

        <!-- Search / Create input -->
        <div class="cat-index__input-bar">
          <div class="cat-index__input-wrap">
            <input
              v-model="newCategoryName"
              placeholder="新建分类..."
              class="cat-index__input"
              @keyup.enter="handleCreateCategory"
            />
            <div class="cat-index__input-accent" />
          </div>
          <button
            @click="handleCreateCategory"
            :disabled="!newCategoryName.trim()"
            class="cat-index__add-btn"
            title="创建分类"
          >
            <Plus :size="14" />
          </button>
        </div>

        <!-- Parent indicator -->
        <div v-if="newCategoryParentId" class="cat-index__parent-hint">
          <span class="text-brass/50 mr-1">↳</span>
          <span>在「{{ categories.find((c) => c.id === newCategoryParentId)?.name }}」下创建</span>
          <button @click="newCategoryParentId = null" class="cat-index__parent-cancel">取消</button>
        </div>

        <!-- Tree -->
        <div class="flex-1 overflow-y-auto custom-scrollbar p-2">
          <div v-if="treeData.length > 0">
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
          <div v-else class="cat-index__empty">
            <FolderSearch :size="24" class="text-brass/20 mb-2" />
            <span>尚无分类条目</span>
            <span class="text-slate-700 text-2xs">在上方输入名称创建</span>
          </div>
        </div>
      </div>

      <!-- ==================== RIGHT: Diagnostic Station ==================== -->
      <div class="flex-1 flex flex-col min-w-0" :style="themeGlowStyle">
        <div v-if="selectedId" class="flex-1 overflow-y-auto custom-scrollbar">

          <!-- Diagnostic: Identity Banner -->
          <div class="diag-banner">
            <div class="diag-banner__line" />
            <div class="diag-banner__content">
              <div
                v-if="editThemeColor && THEME_DICT[editThemeColor as keyof typeof THEME_DICT]"
                class="diag-banner__swatch"
                :style="{ backgroundColor: THEME_DICT[editThemeColor as keyof typeof THEME_DICT].hex }"
              />
              <div class="diag-banner__swatch diag-banner__swatch--empty" v-else />
              <span class="diag-banner__name">{{ editName }}</span>
              <span class="diag-banner__id">ID:{{ selectedId }}</span>
            </div>
            <div class="diag-banner__line" />
          </div>

          <!-- Diagnostic: Gauges -->
          <div class="diag-section">
            <div class="diag-section__header">
              <span class="text-brass text-[8px]">&#9672;</span>
              <h3 class="diag-section__title">DATA READOUT</h3>
            </div>
            <div class="diag-gauges">
              <div class="diag-gauge">
                <div class="diag-gauge__value">{{ categoryCardCount }}</div>
                <div class="diag-gauge__label">
                  <FileText :size="10" class="diag-gauge__icon" />
                  CARDS
                </div>
                <div class="diag-gauge__bar">
                  <div class="diag-gauge__fill" :style="{ width: `${Math.min(categoryCardCount * 10, 100)}%` }" />
                </div>
              </div>
              <div class="diag-gauge">
                <div class="diag-gauge__value">{{ categoryChildCount }}</div>
                <div class="diag-gauge__label">
                  <FolderSearch :size="10" class="diag-gauge__icon" />
                  CHILDREN
                </div>
                <div class="diag-gauge__bar">
                  <div class="diag-gauge__fill diag-gauge__fill--brass" :style="{ width: `${Math.min(categoryChildCount * 20, 100)}%` }" />
                </div>
              </div>
            </div>
          </div>

          <!-- Diagnostic: Card Index -->
          <div class="diag-section">
            <div class="diag-section__header">
              <span class="text-brass text-[8px]">&#9672;</span>
              <h3 class="diag-section__title">CARD INDEX</h3>
              <span class="diag-section__count">{{ categoryCards.length }}</span>
            </div>
            <div v-if="categoryCards.length > 0" class="diag-cards">
              <div v-for="(card, idx) in categoryCards" :key="card.id"
                class="diag-card group">
                <span class="diag-card__idx">{{ String(idx + 1).padStart(2, '0') }}</span>
                <span class="diag-card__title">{{ card.title || '无标题' }}</span>
                <div class="diag-card__actions">
                  <button @click.stop="movingCardId = movingCardId === card.id ? null : card.id"
                    class="diag-card__action" title="迁移到其他分类">
                    MOVE
                  </button>
                  <button @click="handleUnlinkCard(card.id)"
                    class="diag-card__action diag-card__action--danger" title="从分类中移除">
                    UNLINK
                  </button>
                </div>
                <!-- Move dropdown -->
                <div v-if="movingCardId === card.id" @click.stop
                  class="diag-card__dropdown">
                  <button v-for="cat in categories.filter(c => c.id !== selectedId)" :key="cat.id"
                    @click="handleMoveCard(card.id, cat.id)"
                    class="diag-card__dropdown-item">
                    {{ cat.name }}
                  </button>
                  <div v-if="categories.filter(c => c.id !== selectedId).length === 0"
                    class="diag-card__dropdown-empty">
                    无其他分类
                  </div>
                </div>
              </div>
            </div>
            <div v-else class="diag-empty-hint">
              NO CARDS IN THIS CATEGORY
            </div>
          </div>

          <!-- Diagnostic: Edit Controls -->
          <div class="diag-section">
            <div class="diag-section__header">
              <span class="text-brass text-[8px]">&#9672;</span>
              <h3 class="diag-section__title">EDIT</h3>
            </div>
            <div class="diag-edit">
              <div>
                <label class="diag-edit__label">分类名称</label>
                <input v-model="editName"
                  class="diag-edit__input"
                  placeholder="输入分类名称" />
              </div>
              <div>
                <label class="diag-edit__label">分类描述</label>
                <textarea v-model="editDescription"
                  class="diag-edit__textarea"
                  placeholder="输入分类描述（可选）" />
              </div>
              <div>
                <label class="diag-edit__label">专属能量色</label>
                <div class="diag-edit__colors">
                  <button v-for="key in THEME_KEYS" :key="key" @click="editThemeColor = key"
                    class="diag-edit__color-btn"
                    :class="{ 'diag-edit__color-btn--active': editThemeColor === key }"
                    :style="{ backgroundColor: THEME_DICT[key].hex }"
                    :title="THEME_DICT[key].label">
                  </button>
                  <button v-if="editThemeColor" @click="editThemeColor = null"
                    class="diag-edit__color-clear" title="清除颜色">
                    <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              </div>
              <button @click="handleSave" :disabled="!editName.trim()"
                class="diag-edit__save">
                保存更改
              </button>
            </div>
          </div>

          <!-- Danger Zone -->
          <div class="diag-section diag-section--danger">
            <div class="diag-section__header">
              <span class="text-red-500/50 text-[8px]">&#9672;</span>
              <h3 class="diag-section__title text-red-500/50">DANGER ZONE</h3>
            </div>
            <button @click="handleDelete(selectedId, categories.find(c => c.id === selectedId)?.name || '')"
              class="diag-danger-btn">
              删除分类
            </button>
          </div>

        </div>

        <!-- Empty state -->
        <div v-else class="flex-1 flex items-center justify-center">
          <div class="diag-placeholder">
            <svg class="diag-placeholder__icon" viewBox="0 0 48 48" fill="none">
              <circle cx="24" cy="24" r="18" stroke="rgba(184,134,11,0.15)" stroke-width="1" />
              <circle cx="24" cy="24" r="12" stroke="rgba(184,134,11,0.1)" stroke-width="1" />
              <circle cx="24" cy="24" r="3" fill="rgba(184,134,11,0.2)" />
              <line x1="24" y1="6" x2="24" y2="12" stroke="rgba(184,134,11,0.15)" stroke-width="1" />
              <line x1="24" y1="36" x2="24" y2="42" stroke="rgba(184,134,11,0.15)" stroke-width="1" />
              <line x1="6" y1="24" x2="12" y2="24" stroke="rgba(184,134,11,0.15)" stroke-width="1" />
              <line x1="36" y1="24" x2="42" y2="24" stroke="rgba(184,134,11,0.15)" stroke-width="1" />
            </svg>
            <span class="diag-placeholder__text">选择左侧分类条目查看诊断数据</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ═══════════════════════════════════════════════════════════════
   LEFT PANEL — Specimen Index
   ═══════════════════════════════════════════════════════════════ */
.cat-index {
  width: 320px;
  flex-shrink: 0;
}

.cat-index__header {
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  border-bottom: 1px solid theme('colors.ms-border');
  background: theme('colors.ms-void');
  flex-shrink: 0;
}

.cat-index__input-bar {
  display: flex;
  gap: 8px;
  padding: 12px 12px 8px;
  flex-shrink: 0;
}

.cat-index__input-wrap {
  flex: 1;
  position: relative;
}

.cat-index__input {
  width: 100%;
  background: transparent;
  color: theme('colors.slate.300');
  font-size: 12px;
  font-family: ui-monospace, monospace;
  padding: 6px 0;
  border: none;
  border-bottom: 1px solid theme('colors.ms-border');
  outline: none;
  transition: border-color 200ms ease;
}

.cat-index__input:focus {
  border-color: rgba(184, 134, 11, 0.5);
}

.cat-index__input:focus ~ .cat-index__input-accent {
  transform: scaleX(1);
}

.cat-index__input::placeholder {
  color: theme('colors.slate.700');
}

.cat-index__input-accent {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: theme('colors.brass.DEFAULT');
  box-shadow: 0 0 6px rgba(184, 134, 11, 0.3);
  transform: scaleX(0);
  transition: transform 200ms ease;
  transform-origin: left;
}

.cat-index__add-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  color: theme('colors.neon.DEFAULT');
  background: rgba(0, 229, 255, 0.06);
  border: 1px solid rgba(0, 229, 255, 0.2);
  transition: all 150ms ease;
  flex-shrink: 0;
}

.cat-index__add-btn:hover:not(:disabled) {
  background: rgba(0, 229, 255, 0.12);
  box-shadow: 0 0 8px rgba(0, 229, 255, 0.15);
}

.cat-index__add-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.cat-index__parent-hint {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px 8px;
  font-size: 11px;
  font-family: ui-monospace, monospace;
  color: theme('colors.amber.400');
}

.cat-index__parent-cancel {
  margin-left: auto;
  font-size: 10px;
  color: theme('colors.amber.300');
  font-family: ui-monospace, monospace;
}

.cat-index__parent-cancel:hover {
  color: theme('colors.amber.200');
}

.cat-index__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 16px;
  font-size: 12px;
  font-family: ui-monospace, monospace;
  color: theme('colors.slate.600');
  text-align: center;
  gap: 4px;
}

/* ═══════════════════════════════════════════════════════════════
   RIGHT PANEL — Diagnostic Station
   ═══════════════════════════════════════════════════════════════ */

/* --- Identity Banner --- */
.diag-banner {
  padding: 20px 24px 0;
}

.diag-banner__line {
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(184, 134, 11, 0.2) 20%, rgba(184, 134, 11, 0.2) 80%, transparent);
}

.diag-banner__content {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 0;
}

.diag-banner__swatch {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
  box-shadow: 0 0 8px var(--cat-color-20, transparent);
}

.diag-banner__swatch--empty {
  background: #3a3a3a;
  border: 1px solid #4a4a4a;
  box-shadow: none;
}

.diag-banner__name {
  font-size: 16px;
  font-weight: 600;
  font-family: ui-monospace, monospace;
  color: theme('colors.slate.200');
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.diag-banner__id {
  font-size: 10px;
  font-family: ui-monospace, monospace;
  color: theme('colors.slate.700');
  flex-shrink: 0;
}

/* --- Section --- */
.diag-section {
  padding: 16px 24px;
}

.diag-section--danger {
  margin-bottom: 24px;
}

.diag-section__header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.diag-section__title {
  font-size: 10px;
  font-family: ui-monospace, monospace;
  color: theme('colors.slate.500');
  letter-spacing: 0.12em;
  font-weight: 700;
  text-transform: uppercase;
}

.diag-section__count {
  font-size: 10px;
  font-family: ui-monospace, monospace;
  color: theme('colors.slate.700');
  margin-left: auto;
}

/* --- Gauges --- */
.diag-gauges {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.diag-gauge {
  background: theme('colors.ms-carbon');
  border: 1px solid theme('colors.ms-border');
  padding: 12px;
  position: relative;
  overflow: hidden;
}

.diag-gauge__value {
  font-size: 28px;
  font-family: ui-monospace, monospace;
  font-weight: 700;
  color: var(--cat-color, theme('colors.brass.DEFAULT'));
  line-height: 1;
  text-shadow: 0 0 20px var(--cat-color-20, transparent);
}

.diag-gauge__label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 9px;
  font-family: ui-monospace, monospace;
  color: theme('colors.slate.600');
  letter-spacing: 0.1em;
  text-transform: uppercase;
  margin-top: 6px;
}

.diag-gauge__icon {
  color: theme('colors.slate.700');
}

.diag-gauge__bar {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: rgba(255, 255, 255, 0.03);
}

.diag-gauge__fill {
  height: 100%;
  background: var(--cat-color, theme('colors.brass.DEFAULT'));
  opacity: 0.6;
  transition: width 300ms ease;
}

.diag-gauge__fill--brass {
  background: theme('colors.brass.DEFAULT');
}

/* --- Card Index --- */
.diag-cards {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.diag-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 12px;
  background: rgba(255, 255, 255, 0.01);
  position: relative;
  transition: background 120ms ease;
}

.diag-card:hover {
  background: rgba(255, 255, 255, 0.03);
}

.diag-card:hover .diag-card__actions {
  opacity: 1;
}

.diag-card__idx {
  font-size: 10px;
  font-family: ui-monospace, monospace;
  color: theme('colors.slate.700');
  width: 20px;
  flex-shrink: 0;
  text-align: right;
}

.diag-card__title {
  flex: 1;
  font-size: 12px;
  font-family: ui-monospace, monospace;
  color: theme('colors.slate.400');
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.diag-card__actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 120ms ease;
  flex-shrink: 0;
}

.diag-card__action {
  font-size: 9px;
  font-family: ui-monospace, monospace;
  color: theme('colors.slate.600');
  padding: 2px 6px;
  transition: color 100ms ease, background-color 100ms ease;
  cursor: pointer;
  background: none;
  border: none;
}

.diag-card__action:hover {
  color: theme('colors.neon.DEFAULT');
  background: rgba(0, 229, 255, 0.06);
}

.diag-card__action--danger:hover {
  color: theme('colors.red.400');
  background: rgba(239, 68, 68, 0.06);
}

.diag-card__dropdown {
  position: absolute;
  right: 12px;
  top: 100%;
  width: 180px;
  background: theme('colors.ms-carbon');
  border: 1px solid theme('colors.ms-border');
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  z-index: 20;
  max-height: 160px;
  overflow-y: auto;
}

.diag-card__dropdown-item {
  display: block;
  width: 100%;
  text-align: left;
  padding: 6px 10px;
  font-size: 11px;
  font-family: ui-monospace, monospace;
  color: theme('colors.slate.400');
  background: none;
  border: none;
  cursor: pointer;
  transition: all 100ms ease;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.diag-card__dropdown-item:hover {
  background: rgba(0, 229, 255, 0.06);
  color: theme('colors.neon.DEFAULT');
}

.diag-card__dropdown-empty {
  padding: 8px 10px;
  font-size: 11px;
  font-family: ui-monospace, monospace;
  color: theme('colors.slate.600');
}

.diag-empty-hint {
  font-size: 10px;
  font-family: ui-monospace, monospace;
  color: theme('colors.slate.700');
  font-style: italic;
  padding: 8px 0;
}

/* --- Edit Form --- */
.diag-edit {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.diag-edit__label {
  display: block;
  font-size: 10px;
  font-family: ui-monospace, monospace;
  color: theme('colors.slate.500');
  margin-bottom: 6px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.diag-edit__input {
  width: 100%;
  background: theme('colors.ms-carbon');
  color: theme('colors.slate.200');
  font-size: 13px;
  font-family: ui-monospace, monospace;
  padding: 8px 12px;
  border: 1px solid theme('colors.ms-border');
  outline: none;
  transition: border-color 200ms ease;
}

.diag-edit__input:focus {
  border-color: rgba(184, 134, 11, 0.5);
}

.diag-edit__textarea {
  width: 100%;
  background: theme('colors.ms-carbon');
  color: theme('colors.slate.200');
  font-size: 13px;
  font-family: ui-monospace, monospace;
  padding: 8px 12px;
  border: 1px solid theme('colors.ms-border');
  outline: none;
  transition: border-color 200ms ease;
  resize: none;
  height: 72px;
}

.diag-edit__textarea:focus {
  border-color: rgba(184, 134, 11, 0.5);
}

.diag-edit__colors {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.diag-edit__color-btn {
  width: 28px;
  height: 28px;
  border: 2px solid transparent;
  transition: all 150ms ease;
  cursor: pointer;
}

.diag-edit__color-btn:hover {
  border-color: rgba(255, 255, 255, 0.3);
}

.diag-edit__color-btn--active {
  border-color: white;
  transform: scale(1.1);
  box-shadow: 0 0 8px var(--cat-color-20, transparent);
}

.diag-edit__color-clear {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: theme('colors.slate.600');
  border: 1px solid theme('colors.ms-border');
  background: none;
  cursor: pointer;
  transition: all 150ms ease;
}

.diag-edit__color-clear:hover {
  color: theme('colors.slate.400');
}

.diag-edit__save {
  width: 100%;
  background: transparent;
  color: theme('colors.brass.DEFAULT');
  font-size: 12px;
  font-family: ui-monospace, monospace;
  font-weight: 500;
  padding: 8px;
  border: 1px solid rgba(184, 134, 11, 0.4);
  cursor: pointer;
  transition: all 150ms ease;
}

.diag-edit__save:hover:not(:disabled) {
  background: rgba(184, 134, 11, 0.08);
}

.diag-edit__save:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

/* --- Danger Button --- */
.diag-danger-btn {
  width: 100%;
  padding: 8px;
  font-size: 12px;
  font-family: ui-monospace, monospace;
  color: theme('colors.red.400');
  background: transparent;
  border: 1px solid rgba(239, 68, 68, 0.25);
  cursor: pointer;
  transition: all 150ms ease;
}

.diag-danger-btn:hover {
  background: rgba(239, 68, 68, 0.06);
  border-color: rgba(239, 68, 68, 0.4);
}

/* --- Empty Placeholder --- */
.diag-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.diag-placeholder__icon {
  width: 80px;
  height: 80px;
  animation: diag-pulse 4s ease-in-out infinite;
}

.diag-placeholder__text {
  font-size: 12px;
  font-family: ui-monospace, monospace;
  color: theme('colors.slate.600');
}

@keyframes diag-pulse {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 0.8; }
}

/* ═══════════════════════════════════════════════════════════════
   Shared
   ═══════════════════════════════════════════════════════════════ */
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #333;
  border-radius: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #444;
}
</style>
