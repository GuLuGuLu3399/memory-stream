<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useLayoutStore } from '../stores/layout'
import { useKnowledgeStore } from '../stores/knowledge'
import { FileArchive, FileText, X, Loader2 } from 'lucide-vue-next'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits<{ (e: 'close'): void }>()

const layoutStore = useLayoutStore()
const knowledgeStore = useKnowledgeStore()
const { isImportPanelOpen } = storeToRefs(layoutStore)

interface ImportCard {
  title: string
  category: string | null
  tags: string[]
  body_md: string
  source_path: string
}

interface PreviewCard extends ImportCard {
  id: string
  status: 'new' | 'duplicate'
  skip: boolean
}

const previewCards = ref<PreviewCard[]>([])
const isImporting = ref(false)
const isLoadingFiles = ref(false)

watch(isImportPanelOpen, (isOpen) => {
  if (!isOpen) {
    previewCards.value = []
  }
})

async function selectMarkdownFiles() {
  try {
    const paths = await open({
      multiple: true,
      filters: [{ name: 'Markdown', extensions: ['md'] }]
    })
    
    if (!paths) return
    
    isLoadingFiles.value = true
    const pathArray = Array.isArray(paths) ? paths : [paths]
    
    const cards = await invoke<ImportCard[]>('import_markdown_files', {
      paths: pathArray
    })
    
    await processImportedCards(cards)
  } catch (e) {
    knowledgeStore.addToast('文件选择失败: ' + String(e), 'error')
  } finally {
    isLoadingFiles.value = false
  }
}

async function selectZipArchive() {
  try {
    const path = await open({
      multiple: false,
      filters: [{ name: 'ZIP', extensions: ['zip'] }]
    })
    
    if (!path) return
    
    isLoadingFiles.value = true
    
    const cards = await invoke<ImportCard[]>('import_zip_archive', { path })
    
    await processImportedCards(cards)
  } catch (e) {
    knowledgeStore.addToast('ZIP 解压失败: ' + String(e), 'error')
  } finally {
    isLoadingFiles.value = false
  }
}

async function processImportedCards(cards: ImportCard[]) {
  const existingTitles = new Set<string>()
  
  const allCards = [...knowledgeStore.orphanCards, ...knowledgeStore.recentCards]
  allCards.forEach(card => {
    if (card.title) existingTitles.add(card.title.toLowerCase())
  })
  
  previewCards.value = cards.map((card, idx) => ({
    ...card,
    id: `import-${idx}`,
    status: existingTitles.has(card.title.toLowerCase()) ? 'duplicate' : 'new',
    skip: existingTitles.has(card.title.toLowerCase())
  }))
}

function toggleSkip(cardId: string) {
  const card = previewCards.value.find(c => c.id === cardId)
  if (card) {
    card.skip = !card.skip
  }
}

const cardsToImport = computed(() => 
  previewCards.value.filter(c => !c.skip)
)

const duplicateCount = computed(() => 
  previewCards.value.filter(c => c.status === 'duplicate' && !c.skip).length
)

async function importCards() {
  if (cardsToImport.value.length === 0) return
  
  isImporting.value = true
  
  try {
    let importedCount = 0
    let errorCount = 0
    
    for (const card of cardsToImport.value) {
      try {
        const renderResult = await invoke<{
          html: string
          ast_json: string
          excerpt: string
          extracted_links: string[]
        }>('process_markdown', { content: card.body_md })
        
        let tocData: unknown = null
        try {
          tocData = await invoke('extract_toc', {
            astJson: renderResult.ast_json
          })
        } catch (e) {
          console.warn('[ImportPanel] extract_toc failed:', e)
        }
        
        await invoke('api_request', {
          method: 'POST',
          endpoint: '/cards',
          body: {
            title: card.title,
            raw_md: card.body_md,
            excerpt: renderResult.excerpt,
            ast_data: renderResult.ast_json,
            toc_data: tocData,
            parent_id: null,
            relation_type: null
          }
        })
        
        importedCount++
      } catch (e) {
        console.error(`[ImportPanel] Failed to import "${card.title}":`, e)
        errorCount++
      }
    }
    
    await knowledgeStore.refreshWorkspace()
    
    const message = errorCount > 0
      ? `导入完成: ${importedCount} 张卡片成功, ${errorCount} 张失败`
      : `已导入 ${importedCount} 张卡片`
    
    knowledgeStore.addToast(message, importedCount > 0 ? 'success' : 'error')
    
    if (importedCount > 0) {
      close()
    }
  } catch (e) {
    knowledgeStore.addToast('导入失败: ' + String(e), 'error')
  } finally {
    isImporting.value = false
  }
}

function close() {
  layoutStore.closeImportPanel()
  emit('close')
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') close()
}

onMounted(() => window.addEventListener('keydown', handleKeydown))
onUnmounted(() => window.removeEventListener('keydown', handleKeydown))
</script>

<template>
  <Transition name="import-panel">
    <div
      v-if="isImportPanelOpen"
      class="fixed inset-0 z-50 flex items-center justify-center bg-ms-deep/95 backdrop-blur-sm"
      @click.self="close"
    >
      <div class="w-full max-w-4xl max-h-[90vh] overflow-y-auto bg-ms-panel border border-ms-border shadow-2xl">
        
        <div class="h-14 flex items-center justify-between px-6 border-b border-ms-border bg-ms-carbon shrink-0">
          <div class="flex items-center gap-3">
            <span class="text-neon text-lg">◆</span>
            <span class="text-sm font-mono font-bold text-slate-300 tracking-wider">
              IMPORT MODULE
            </span>
            <span class="text-xs text-slate-600 font-mono">— 知识导入舱</span>
          </div>
          <button
            @click="close"
            class="text-slate-500 hover:text-slate-300 transition-colors p-1"
            title="关闭"
          >
            <X class="w-5 h-5" />
          </button>
        </div>

        <div class="p-6 space-y-6">
          
          <div class="border border-ms-border bg-ms-deep">
            <div class="h-10 flex items-center gap-2 px-4 border-b border-ms-border bg-ms-carbon">
              <FileText class="w-3.5 h-3.5 text-cyan-400" />
              <span class="text-cyan-400 text-xs tracking-widest uppercase font-bold font-mono">
                [ SOURCE ]
              </span>
            </div>
            <div class="p-4 space-y-3">
              <div class="flex gap-3">
                <button
                  @click="selectMarkdownFiles"
                  :disabled="isLoadingFiles || isImporting"
                  class="flex-1 flex items-center justify-center gap-2 px-4 py-3 text-xs font-mono transition-all border border-cyan-500/30 bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-400 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <FileText class="w-4 h-4" />
                  IMPORT MARKDOWN
                </button>
                <button
                  @click="selectZipArchive"
                  :disabled="isLoadingFiles || isImporting"
                  class="flex-1 flex items-center justify-center gap-2 px-4 py-3 text-xs font-mono transition-all border border-cyan-500/30 bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-400 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <FileArchive class="w-4 h-4" />
                  IMPORT ZIP
                </button>
              </div>
            </div>
          </div>

          <div v-if="isLoadingFiles" class="flex items-center justify-center py-8">
            <Loader2 class="w-6 h-6 text-neon animate-spin" />
            <span class="ml-3 text-sm font-mono text-slate-400">正在解析文件...</span>
          </div>

          <div v-if="previewCards.length > 0" class="border border-ms-border bg-ms-deep">
            <div class="h-10 flex items-center gap-2 px-4 border-b border-ms-border bg-ms-carbon">
              <span class="text-cyan-400 text-xs tracking-widest uppercase font-bold font-mono">
                [ PREVIEW ]
              </span>
              <span class="text-xs text-slate-600 font-mono">
                {{ cardsToImport.length }} / {{ previewCards.length }} cards selected
              </span>
            </div>
            <div class="overflow-x-auto">
              <table class="w-full text-xs font-mono">
                <thead>
                  <tr class="border-b border-ms-border text-slate-500">
                    <th class="px-4 py-2 text-left">FILENAME</th>
                    <th class="px-4 py-2 text-left">TITLE</th>
                    <th class="px-4 py-2 text-left">CATEGORY</th>
                    <th class="px-4 py-2 text-center">STATUS</th>
                    <th class="px-4 py-2 text-center">SKIP</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="card in previewCards"
                    :key="card.id"
                    class="border-b border-ms-border/50 hover:bg-ms-panel/30 transition-colors"
                    :class="{ 'bg-amber-500/5': card.status === 'duplicate' }"
                  >
                    <td class="px-4 py-2 text-slate-400 truncate max-w-[200px]">
                      {{ card.source_path.split('/').pop() || card.source_path.split('\\').pop() }}
                    </td>
                    <td class="px-4 py-2 text-slate-300 truncate max-w-[200px]">
                      {{ card.title || '—' }}
                    </td>
                    <td class="px-4 py-2 text-slate-400 truncate max-w-[150px]">
                      {{ card.category || '—' }}
                    </td>
                    <td class="px-4 py-2 text-center">
                      <span
                        class="px-2 py-0.5 text-[10px] tracking-wider uppercase"
                        :class="card.status === 'duplicate' 
                          ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30' 
                          : 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'"
                      >
                        {{ card.status }}
                      </span>
                    </td>
                    <td class="px-4 py-2 text-center">
                      <button
                        @click="toggleSkip(card.id)"
                        :disabled="isImporting"
                        class="w-5 h-5 border transition-all"
                        :class="card.skip
                          ? 'bg-red-500/20 border-red-500/50 text-red-400'
                          : 'border-ms-border hover:border-neon/50'"
                        :title="card.skip ? '点击取消跳过' : '点击跳过此卡片'"
                      >
                        <span v-if="card.skip" class="text-[10px]">✕</span>
                      </button>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

        </div>

        <div class="h-16 flex items-center justify-between px-6 border-t border-ms-border bg-ms-carbon">
          <div class="text-xs font-mono text-slate-500">
            <span v-if="duplicateCount > 0" class="text-amber-400">
              {{ duplicateCount }} duplicates detected
            </span>
          </div>
          <div class="flex items-center gap-3">
            <button
              @click="close"
              :disabled="isImporting"
              class="px-4 py-2 text-xs font-mono border border-ms-border text-slate-500 hover:text-slate-300 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              CANCEL
            </button>
            <button
              @click="importCards"
              :disabled="cardsToImport.length === 0 || isImporting"
              class="flex items-center gap-2 px-6 py-2 text-xs font-mono transition-all"
              :class="cardsToImport.length > 0 && !isImporting
                ? 'bg-cyan-500/10 border border-cyan-500/30 text-cyan-400 hover:bg-cyan-500/20'
                : 'bg-ms-surface border border-ms-border text-slate-600 opacity-30 cursor-not-allowed'"
            >
              <Loader2 v-if="isImporting" class="w-3.5 h-3.5 animate-spin" />
              <span v-else class="text-neon">◆</span>
              {{ isImporting ? 'IMPORTING...' : `IMPORT ${cardsToImport.length} CARDS` }}
            </button>
          </div>
        </div>

      </div>
    </div>
  </Transition>
</template>

<style scoped>
.import-panel-enter-active {
  transition: opacity 0.2s ease-out;
}

.import-panel-leave-active {
  transition: opacity 0.15s ease-in;
}

.import-panel-enter-from,
.import-panel-leave-to {
  opacity: 0;
}
</style>
