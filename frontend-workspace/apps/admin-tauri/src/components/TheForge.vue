<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { useKnowledgeStore } from "../stores/knowledge";
import { useLayoutStore } from "../stores/layout";
import { storeToRefs } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import CodemirrorEditor from "./CodemirrorEditor.vue";
import MarkdownViewer from "@memory-stream/ui-shared/components/MarkdownViewer.vue";
import type { EditorView } from "@codemirror/view";

const store = useKnowledgeStore();
const layoutStore = useLayoutStore();
const { activeCard, isLoading, isSaving, isDirty, justSaved, categories, backlinks } = storeToRefs(store);
const { isRightPanelOpen } = storeToRefs(layoutStore);

const codemirrorRef = ref<{ editorView: EditorView | null } | null>(null);

async function handleImagePaste(e: ClipboardEvent) {
  const items = e.clipboardData?.items;
  if (!items) return;

  for (const item of items) {
    if (item.type.indexOf('image') !== -1) {
      e.preventDefault();
      const blob = item.getAsFile();
      if (!blob) continue;

      const view = codemirrorRef.value?.editorView;
      if (!view || !activeCard.value) return;

      const { from, to } = view.state.selection.main;
      const placeholder = `\n![上传中...]()\n`;

      view.dispatch({
        changes: { from, to, insert: placeholder },
        selection: { anchor: from + placeholder.length },
      });

      const reader = new FileReader();
      reader.onload = async () => {
        const result = reader.result as string;
        const base64Data = result.split(',')[1];

        try {
          const uploadResult = await invoke<{ url: string; key: string }>(
            'upload_clipboard_image',
            { base64Data },
          );
          const imageMd = `\n![image](${uploadResult.url})\n`;
          const doc = view.state.doc.toString();
          const pos = doc.indexOf(placeholder);
          if (pos !== -1) {
            view.dispatch({
              changes: { from: pos, to: pos + placeholder.length, insert: imageMd },
              selection: { anchor: pos + imageMd.length },
            });
          }
        } catch (error) {
          console.error('[Forge] 图片上传失败:', error);
          const doc = view.state.doc.toString();
          const pos = doc.indexOf(placeholder);
          if (pos !== -1) {
            view.dispatch({
              changes: {
                from: pos,
                to: pos + placeholder.length,
                insert: `\n> ❌ 图片上传失败: ${error}\n`,
              },
            });
          }
        }
      };
      reader.readAsDataURL(blob);
      break;
    }
  }
}

type ViewMode = "edit" | "split" | "preview";
const viewMode = ref<ViewMode>("split");
const renderedHtml = ref("");
const renderError = ref("");
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

// ===== 表单验证 =====
const validationError = ref("");

// Debounced preview rendering
async function renderPreview(content: string) {
  if (!content.trim()) {
    renderedHtml.value = "";
    renderError.value = "";
    return;
  }
  try {
    const result = await invoke<{ html: string; ast_json: string }>(
      "process_markdown",
      { content },
    );
    renderError.value = "";
    renderedHtml.value = result.html;
  } catch (e) {
    console.error("[Forge] render failed:", e);
    renderError.value = String(e);
  }
}

function scheduleRender(content: string) {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => renderPreview(content), 300);
}

function handlePreviewClick(e: MouseEvent) {
  const target = e.target as HTMLElement;
  const link = target.closest('a.wikilink') as HTMLAnchorElement | null;
  if (!link) return;
  e.preventDefault();
  const href = link.getAttribute('href');
  const title = href != null ? href : (link.textContent ?? "");
  if (title) {
    store.loadAndActivateCardByTitle(title);
  }
}

// Watch content changes for live preview + dirty state + validation
watch(
  () => activeCard.value?.content,
  (newContent) => {
    store.checkDirty();
    if (validationError.value && newContent?.trim()) {
      validationError.value = "";
    }
    if (newContent !== undefined) {
      scheduleRender(newContent);
    }
  },
);

watch(
  () => activeCard.value?.title,
  () => {
    store.checkDirty();
    if (validationError.value && activeCard.value?.title.trim()) {
      validationError.value = "";
    }
  },
);

// Initial render when card changes
watch(
  () => activeCard.value?.id,
  () => {
    if (activeCard.value?.content) {
      renderPreview(activeCard.value.content);
    } else {
      renderedHtml.value = "";
    }
  },
);

// Keyboard shortcuts
function handleKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === "s") {
    e.preventDefault();
    if (canSave.value) handleSave();
  }
}

onMounted(() => {
  if (activeCard.value?.content) {
    renderPreview(activeCard.value.content);
  }
});

onUnmounted(() => {
  if (debounceTimer) clearTimeout(debounceTimer);
});

const viewModes: { key: ViewMode; label: string }[] = [
  { key: "edit", label: "编辑" },
  { key: "split", label: "分屏" },
  { key: "preview", label: "预览" },
];

const canSave = computed(() => {
  if (!activeCard.value) return false;
  if (isSaving.value || isLoading.value || !isDirty.value) return false;
  if (!activeCard.value.title.trim()) return false;
  if (!activeCard.value.content.trim()) return false;
  return true;
});

function validateBeforeSave(): boolean {
  if (!activeCard.value) return false;
  if (!activeCard.value.title.trim()) {
    validationError.value = "标题不能为空";
    return false;
  }
  if (!activeCard.value.content.trim()) {
    validationError.value = "内容不能为空 — 请输入 Markdown 正文";
    return false;
  }
  validationError.value = "";
  return true;
}

function handleSave() {
  if (!validateBeforeSave()) return;
  store.saveCard();
}
</script>

<template>
  <main class="h-full bg-ms-carbon flex flex-col relative min-w-0" @keydown="handleKeydown">
    <!-- Top Bar -->
    <div class="h-14 flex items-center px-6 border-b border-ms-border justify-between shrink-0 bg-ms-panel">
      <template v-if="activeCard">
        <div class="flex items-center gap-3 min-w-0">
          <span class="text-sm font-medium text-slate-300 truncate max-w-xs">
            {{ activeCard.title || "无标题" }}
          </span>
          <span v-if="isDirty"
            class="shrink-0 inline-flex items-center gap-1 text-xs text-ms-warning bg-ms-warning/10 px-2 py-0.5 rounded-full">
            <span class="w-1.5 h-1.5 bg-amber-500 rounded-full animate-pulse"></span>
            未保存
          </span>
          <span v-else-if="activeCard.id" class="shrink-0 text-xs text-slate-500">
            已保存
          </span>
        </div>

        <div class="flex items-center gap-3">
          <!-- Category Selector -->
          <select v-if="activeCard.id" :value="activeCard.category_id ?? ''"
            @change="(e: Event) => { activeCard!.category_id = (e.target as HTMLSelectElement).value ? Number((e.target as HTMLSelectElement).value) : null; store.checkDirty(); }"
            class="text-xs bg-ms-deep text-slate-400 rounded-sm px-2 py-1 border border-ms-border outline-none focus:border-neon transition">
            <option value="">未分类</option>
            <option v-for="cat in categories" :key="cat.id" :value="cat.id">{{ cat.name }}</option>
          </select>

          <!-- View Mode Toggle -->
          <div class="flex border border-ms-border rounded-none p-0 text-xs">
            <button v-for="mode in viewModes" :key="mode.key" @click="viewMode = mode.key"
              class="px-3 py-1 rounded-none transition-all border-b-2" :class="viewMode === mode.key
                ? 'text-neon font-medium border-b-neon bg-transparent'
                : 'text-slate-500 hover:text-slate-300 border-b-transparent'">
              {{ mode.label }}
            </button>
          </div>

          <!-- Graph Toggle -->
          <button @click="layoutStore.toggleRightPanel()" title="切换图谱面板 (Ctrl+\)"
            class="w-8 h-8 flex items-center justify-center rounded-sm transition-all border"
            :class="isRightPanelOpen
              ? 'text-neon border-neon/30 bg-neon/5'
              : 'text-slate-600 border-ms-border hover:text-slate-400 hover:border-slate-500'">
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path d="M12 2a9 9 0 0 0 0 18 9 9 0 0 0 0-18z"/>
              <path d="M12 7a4.5 4.5 0 0 0 0 9 4.5 4.5 0 0 0 0-9z"/>
              <circle cx="12" cy="12" r="1.5" fill="currentColor"/>
            </svg>
          </button>

          <!-- 验证提示 -->
          <span v-if="validationError" class="text-xs text-ms-danger animate-pulse mr-2">
            {{ validationError }}
          </span>
          <div class="relative">
            <button @click="handleSave" :disabled="!canSave"
              class="relative px-4 py-1.5 text-sm rounded-sm transition-all z-10" :class="justSaved
                ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
                : canSave
                  ? 'bg-neon-600 text-ms-deep hover:bg-neon shadow-sm'
                  : 'bg-ms-surface text-slate-600 cursor-not-allowed'">
              {{ justSaved ? '✓ 已保存' : isSaving ? '保存中...' : '保存 ⌘S' }}
            </button>
            <!-- 霓虹脉冲扩散动效 -->
            <div v-if="justSaved" class="save-neon-pulse"></div>
          </div>
        </div>
      </template>
      <!-- 无 activeCard 时 Top Bar 保持空白底色，只占位 -->
    </div>

    <!-- 编辑态 / 空状态切换（带 Transition） -->
    <Transition name="forge-content" mode="out-in">
      <div v-if="activeCard" key="editor" class="flex-1 flex flex-col min-h-0">
        <!-- Content Area -->
        <div class="flex-1 flex min-h-0 relative">
          <!-- Editor Pane -->
          <div class="flex flex-col min-h-0"
            :class="viewMode === 'edit' ? 'flex-1' : viewMode === 'split' ? 'w-1/2 border-r border-ms-border' : 'hidden'">
            <input v-model="activeCard.title" placeholder="无标题..."
              class="w-full px-6 pt-8 text-2xl font-bold text-slate-100 border-none outline-none bg-transparent placeholder-slate-600 shrink-0" />
            <div class="flex-1 min-h-0 overflow-hidden" @paste="handleImagePaste">
              <CodemirrorEditor
                ref="codemirrorRef"
                v-model="activeCard.content"
                placeholder="开始锻造知识... (支持 Markdown，可粘贴图片)"
                @save="handleSave"
              />
            </div>
          </div>

          <!-- Preview Pane -->
          <div class="flex flex-col min-h-0"
            :class="viewMode === 'preview' ? 'flex-1' : viewMode === 'split' ? 'w-1/2' : 'hidden'">
            <div class="px-6 pt-8 pb-2 shrink-0">
              <h1 v-if="viewMode === 'preview' && activeCard.title" class="text-2xl font-bold text-slate-100 mb-4">
                {{ activeCard.title }}
              </h1>
            </div>
            <div class="flex-1 overflow-y-auto px-6 py-6">
              <!-- Render error (safe text binding, no XSS) -->
              <div v-if="renderError" class="p-4 bg-red-50 border border-red-200 text-red-600 rounded-md">
                <strong>渲染引擎错误:</strong>
                <pre class="text-xs mt-2 whitespace-pre-wrap">{{ renderError }}</pre>
              </div>
              <MarkdownViewer v-else-if="renderedHtml" :html-content="renderedHtml" @click="handlePreviewClick" />
              <div v-else class="text-slate-500 text-sm italic">
                {{ activeCard.content ? "渲染中..." : "在编辑区输入 Markdown，这里会实时预览" }}
              </div>
            </div>
          </div>

          <!-- Loading Overlay -->
          <Transition name="fade">
            <div v-if="isLoading"
              class="absolute inset-0 bg-ms-carbon/80 backdrop-blur-sm flex items-center justify-center z-10">
              <div class="flex flex-col items-center gap-2 text-slate-500">
                <svg class="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                </svg>
                <span class="text-xs">加载中...</span>
              </div>
            </div>
          </Transition>

        </div>

          <!-- Reverse Gravity Radar: Backlinks -->
          <div v-if="activeCard.id && backlinks.length > 0"
            class="shrink-0 bg-ms-void/95 backdrop-blur border-t border-dashed border-slate-800">
            <div class="px-6 py-3">
              <div class="flex items-center text-xs font-mono mb-2 text-slate-500">
                <span class="text-neon mr-2 animate-pulse">❯</span>
                <span class="tracking-widest uppercase">INCOMING_LINKS</span>
                <span class="mx-2 text-slate-700">::</span>
                <span class="text-neon font-bold" style="text-shadow: 0 0 8px rgba(0,229,255,0.4);">
                  {{ backlinks.length }}
                </span>
              </div>
              <div class="flex gap-4 overflow-x-auto pb-1 no-scrollbar">
                <div v-for="link in backlinks" :key="link.source_id" class="py-1.5 shrink-0">
                  <div class="flex items-center gap-2">
                    <span class="text-xs font-mono uppercase"
                      :class="link.relation_type === 'sequence' ? 'text-neon' : 'text-slate-600'">
                      {{ link.relation_type }}
                    </span>
                    <button @click="store.loadAndActivateCard(link.source_id)"
                      class="text-sm text-slate-300 hover:text-neon transition-colors text-left">
                      {{ link.source_title }}
                    </button>
                  </div>
                  <p v-if="link.context_snippet" class="text-xs text-slate-500 italic pl-4 mt-0.5 truncate max-w-xs">
                    ...{{ link.context_snippet }}...
                  </p>
                </div>
              </div>
            </div>
          </div>

      </div>

      <!-- 空状态：待引燃的火种 -->
      <div v-else key="empty" class="flex-1 flex flex-col items-center justify-center gap-6 select-none">
        <!-- 十字准星图腾 -->
        <div class="relative w-24 h-24 animate-[float_4s_ease-in-out_infinite]">
          <!-- 外圈 -->
          <div class="absolute inset-0 rounded-none border border-ms-border-light/40"></div>
          <!-- 十字线 -->
          <div class="absolute top-1/2 left-0 right-0 h-px bg-gradient-to-r from-transparent via-neon/30 to-transparent"></div>
          <div class="absolute left-1/2 top-0 bottom-0 w-px bg-gradient-to-b from-transparent via-neon/30 to-transparent"></div>
          <!-- 中心点 -->
          <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-1.5 h-1.5 bg-neon/50 rounded-full"
            style="box-shadow: 0 0 8px rgba(0,229,255,0.3), 0 0 20px rgba(0,229,255,0.1);">
          </div>
          <!-- 角标 -->
          <div class="absolute top-2 left-2 w-2 h-px bg-neon/20"></div>
          <div class="absolute top-2 left-2 h-2 w-px bg-neon/20"></div>
          <div class="absolute top-2 right-2 w-2 h-px bg-neon/20"></div>
          <div class="absolute top-2 right-2 h-2 w-px bg-neon/20"></div>
          <div class="absolute bottom-2 left-2 w-2 h-px bg-neon/20"></div>
          <div class="absolute bottom-2 left-2 h-2 w-px bg-neon/20"></div>
          <div class="absolute bottom-2 right-2 w-2 h-px bg-neon/20"></div>
          <div class="absolute bottom-2 right-2 h-2 w-px bg-neon/20"></div>
        </div>

        <div class="flex flex-col items-center gap-2">
          <span class="text-sm font-display text-slate-400 tracking-wide">选择或新建一张卡片开始锻造</span>
          <span class="text-[11px] text-slate-700 font-mono tracking-wider">CTRL+K 搜索 &middot; 左侧 + 新建卡片</span>
        </div>
      </div>
    </Transition>
  </main>
</template>

<style scoped>
/* Forge content transition (edit ↔ empty state) */
.forge-content-enter-active {
  transition: opacity 0.2s ease-out, transform 0.2s ease-out;
}

.forge-content-leave-active {
  transition: opacity 0.15s ease-in;
}

.forge-content-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.forge-content-leave-to {
  opacity: 0;
}

/* Float animation for crosshair reticle */
@keyframes float {

  0%,
  100% {
    transform: translateY(0);
  }

  50% {
    transform: translateY(-6px);
  }
}

/* ── 保存成功霓虹脉冲扩散动效 ── */
.save-neon-pulse {
  position: absolute;
  inset: -4px;
  border-radius: 4px;
  z-index: 0;
  pointer-events: none;
  animation: neon-burst 1.2s cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

@keyframes neon-burst {
  0% {
    box-shadow:
      0 0 0 0 rgba(0, 229, 255, 0.6),
      0 0 0 0 rgba(0, 229, 255, 0.3);
    opacity: 1;
  }

  50% {
    box-shadow:
      0 0 12px 4px rgba(0, 229, 255, 0.4),
      0 0 24px 8px rgba(0, 229, 255, 0.15);
    opacity: 0.8;
  }

  100% {
    box-shadow:
      0 0 20px 10px rgba(0, 229, 255, 0),
      0 0 40px 20px rgba(0, 229, 255, 0);
    opacity: 0;
  }
}
.markdown-body :deep(a.wikilink) {
  color: #00e5ff;
  border-bottom: 1px dashed rgba(0, 229, 255, 0.3);
  cursor: pointer;
  transition: border-color 0.15s;
}
.markdown-body :deep(a.wikilink:hover) {
  border-bottom-color: rgba(0, 229, 255, 0.8);
}
</style>

<style scoped>
.no-scrollbar::-webkit-scrollbar {
  display: none;
}
.no-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
