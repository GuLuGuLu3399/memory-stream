<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { storeToRefs } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { useSysConfigStore } from '../stores/sysconfig'
import { useCategoryStore } from '../stores/useCategoryStore'
import { useLayoutStore } from '../stores/layout'
import { extractMsg } from '../composables/useTempleError'
import { useVaultSync } from '../composables/useVaultSync'
import ChamberHeader from '../components/ChamberHeader.vue'
import { Wifi, Database, Shield, FolderOpen } from 'lucide-vue-next'

const store = useSysConfigStore()
const categoryStore = useCategoryStore()
const layoutStore = useLayoutStore()
const { isSettingsOpen } = storeToRefs(layoutStore)
const { loading, error, syncStatus, syncStats } = storeToRefs(store)
const { categories } = storeToRefs(categoryStore)
const { isWatching, startWatcher, stopWatcher } = useVaultSync()

type ImportCategoryMode = 'auto' | 'none' | 'selected'
const importCategoryMode = ref<ImportCategoryMode>('auto')
const selectedImportCategoryId = ref<number | null>(null)

// Active section tab
type Section = 'vault' | 'network' | 'storage'
const activeSection = ref<Section>('vault')

const sectionTabs: { key: Section; label: string; icon: typeof Wifi }[] = [
  { key: 'vault', label: 'VAULT', icon: FolderOpen },
  { key: 'network', label: 'NETWORK', icon: Wifi },
  { key: 'storage', label: 'STORAGE', icon: Database },
]

// Local form state (not committed until save)
const form = ref({
  api_base_url: 'http://localhost:8080/api/v1',
  ws_url: 'ws://localhost:8080/api/v1/ws',
  s3_endpoint: 'http://localhost:9000',
  s3_region: 'us-east-1',
  s3_bucket: 'memory-stream',
  s3_access_key: 'admin',
  s3_secret_key: 'adminpassword',
  s3_public_url_base: 'http://localhost:9000',
  s3_use_path_style: true,
  vault_path: null as string | null,
})

type TestStatus = 'idle' | 'testing' | 'ok' | 'failed'
const networkStatus = ref<TestStatus>('idle')
const networkError = ref('')
const storageStatus = ref<TestStatus>('idle')
const storageError = ref('')
const saving = ref(false)

// Load config when panel opens or when the component mounts while already open
watch(isSettingsOpen, async (isOpen) => {
  if (isOpen) {
    activeSection.value = 'vault'
    await store.loadConfig()
    await categoryStore.loadCategories()
    if (store.config) {
      form.value = {
        api_base_url: store.config.api_base_url,
        ws_url: store.config.ws_url,
        s3_endpoint: store.config.s3_endpoint,
        s3_region: store.config.s3_region,
        s3_bucket: store.config.s3_bucket,
        s3_access_key: store.config.s3_access_key ?? '',
        s3_secret_key: store.config.s3_secret_key ?? '',
        s3_public_url_base: store.config.s3_public_url_base ?? '',
        s3_use_path_style: store.config.s3_use_path_style,
        vault_path: store.config.vault_path ?? null,
      }
    }
    // Reset test states
    networkStatus.value = 'idle'
    storageStatus.value = 'idle'
  }
}, { immediate: true })

async function testNetwork() {
  networkStatus.value = 'testing'
  networkError.value = ''
  try {
    await store.saveConfig(form.value)
    await invoke('test_api_connection')
    networkStatus.value = 'ok'
  } catch (e) {
    networkStatus.value = 'failed'
    networkError.value = extractMsg(e)
  }
}

async function testStorage() {
  storageStatus.value = 'testing'
  storageError.value = ''
  try {
    await store.saveConfig(form.value)
    const result = await store.testS3Connection()
    storageStatus.value = result ? 'ok' : 'failed'
    if (!result) storageError.value = 'S3 连接失败'
  } catch (e) {
    storageStatus.value = 'failed'
    storageError.value = extractMsg(e)
  }
}

async function testAll() {
  await Promise.all([testNetwork(), testStorage()])
}

async function selectVaultDir() {
  const path = await store.selectVaultDirectory()
  if (path) {
    form.value.vault_path = path
  }
}

async function syncCloudToLocal() {
  await store.syncCloudToVault()
}

async function importLocalToCloud() {
  await store.importLocalVaultToCloud({
    categoryMode: importCategoryMode.value,
    selectedCategoryId: selectedImportCategoryId.value,
  })
  const { useKnowledgeStore } = await import('../stores/knowledge')
  useKnowledgeStore().refreshWorkspace()
}

async function toggleWatcher() {
  if (!form.value.vault_path) return
  if (isWatching.value) {
    await stopWatcher()
  } else {
    await startWatcher(form.value.vault_path)
  }
}

const canSave = computed(() =>
  networkStatus.value === 'ok' &&
  storageStatus.value === 'ok' &&
  !saving.value &&
  !loading.value
)

async function save() {
  if (!canSave.value) return
  saving.value = true
  try {
    // saveConfig 现在返回保存后的完整配置
    await store.saveConfig(form.value)
    // 强制重新加载后端运行态配置，然后整页刷新，确保 HTTP/WS/鉴权全部按新配置重建
    await store.reloadConfig()
    window.location.reload()
  } catch (e: unknown) {
    console.error('[Settings] Save failed:', e)
  } finally {
    saving.value = false
  }
}

function close() {
  layoutStore.closeChamber()
}

// Close settings panel with ESC key
function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') close()
}

onMounted(() => window.addEventListener('keydown', handleKeydown))
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  void stopWatcher()
})

// Status indicator helpers
function getStatusIcon(status: TestStatus): string {
  switch (status) {
    case 'ok': return '✓'
    case 'failed': return '✗'
    case 'testing': return '⋯'
    default: return ''
  }
}

function getStatusColor(status: TestStatus): string {
  switch (status) {
    case 'ok': return 'text-brass'
    case 'failed': return 'text-neon'
    case 'testing': return 'text-brass animate-pulse'
    default: return 'text-slate-600'
  }
}
</script>

<template>
  <div v-if="isSettingsOpen" class="fixed inset-x-0 bottom-0 top-titlebar z-panel bg-ms-deep flex flex-col">
    <!-- Header -->
    <ChamberHeader title="SYSTEM CONFIG" subtitle="系统配置舱" @close="close" />

    <!-- Body: Left nav + Right content -->
    <div class="flex flex-1 min-h-0">

      <!-- Left: Section navigation -->
      <div class="w-48 border-r border-ms-border bg-ms-carbon flex flex-col shrink-0">
        <div class="flex-1 py-3">
          <div class="px-3 mb-3">
            <span class="text-2xs font-mono text-slate-600 uppercase tracking-widest">Sections</span>
          </div>
          <button v-for="tab in sectionTabs" :key="tab.key" @click="activeSection = tab.key"
            class="w-full flex items-center gap-2.5 px-4 py-2.5 text-xs font-mono transition-all border-l-2" :class="activeSection === tab.key
              ? 'border-neon text-neon bg-neon/5'
              : 'border-transparent text-slate-500 hover:text-slate-300 hover:bg-ms-surface/30'">
            <component :is="tab.icon" :size="14" />
            <span>{{ tab.label }}</span>
          </button>
        </div>

        <!-- Global actions at bottom -->
        <div class="p-3 border-t border-ms-border space-y-2">
          <button @click="testAll" :disabled="networkStatus === 'testing' || storageStatus === 'testing'"
            class="w-full px-3 py-2 text-xs font-mono border border-ms-border text-slate-400 hover:text-slate-300 hover:border-slate-500 transition-all disabled:opacity-50 disabled:cursor-not-allowed">
            TEST ALL
          </button>
          <button @click="save" :disabled="!canSave"
            class="w-full flex items-center justify-center gap-2 px-3 py-2 text-xs font-mono transition-all" :class="canSave
              ? 'bg-brass/10 border border-brass/30 text-brass hover:bg-brass/20'
              : 'bg-ms-surface border border-ms-border text-slate-600 opacity-30 cursor-not-allowed'">
            <span class="text-neon">◆</span>
            SAVE CONFIG
          </button>
        </div>
      </div>

      <!-- Right: Content area -->
      <div class="flex-1 min-w-0 overflow-y-auto custom-scrollbar p-8">

        <!-- VAULT Section -->
        <div v-show="activeSection === 'vault'" class="max-w-2xl">
          <div class="flex items-center gap-2 mb-6">
            <FolderOpen class="w-4 h-4 text-brass" />
            <h2 class="text-sm font-mono font-bold text-slate-300 tracking-wider uppercase">Vault Configuration</h2>
          </div>

          <div class="space-y-6">
            <!-- Vault Path Selection -->
            <div>
              <label class="block text-xs text-slate-500 font-mono mb-2">知识库目录</label>
              <div class="flex items-center gap-3">
                <div class="flex-1 bg-ms-carbon border border-ms-border px-4 py-3 min-w-0">
                  <span v-if="form.vault_path" class="text-sm text-slate-300 font-mono truncate block"
                    :title="form.vault_path">
                    {{ form.vault_path }}
                  </span>
                  <span v-else class="text-sm text-slate-600 font-mono italic">未设置</span>
                </div>
                <button @click="selectVaultDir"
                  class="text-brass border border-brass/30 bg-brass/10 hover:bg-brass/20 px-4 py-3 text-xs font-mono transition-all whitespace-nowrap">
                  {{ form.vault_path ? 'CHANGE' : 'SELECT' }}
                </button>
              </div>
            </div>

            <!-- Sync Controls (only show when vault is configured) -->
            <div v-if="form.vault_path" class="space-y-6">
              <!-- Cloud to Local Sync -->
              <div class="border border-ms-border bg-ms-carbon/30 p-4 space-y-3">
                <div class="flex items-center justify-between">
                  <h3 class="text-xs font-mono font-bold text-slate-400 uppercase">Cloud → Local</h3>
                  <button @click="syncCloudToLocal" :disabled="syncStatus === 'syncing'"
                    class="text-neon border border-neon/30 bg-neon/10 hover:bg-neon/20 px-3 py-1.5 text-2xs font-mono transition-all disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap">
                    {{ syncStatus === 'syncing' ? 'SYNCING...' : 'SYNC NOW' }}
                  </button>
                </div>

                <!-- Sync Status -->
                <div v-if="syncStatus !== 'idle'" class="space-y-2">
                  <div class="flex items-center gap-3 text-2xs font-mono">
                    <span
                      :class="syncStatus === 'ok' ? 'text-ms-success' : syncStatus === 'failed' ? 'text-ms-danger' : 'text-brass'">
                      {{ syncStatus === 'ok' ? '✓ SYNCED' : syncStatus === 'failed' ? '✗ FAILED' : '⋯ SYNCING' }}
                    </span>
                    <span v-if="syncStats.synced > 0 || syncStats.skipped > 0" class="text-slate-500">
                      {{ syncStats.synced }} synced, {{ syncStats.skipped }} skipped
                    </span>
                  </div>

                  <!-- Error List -->
                  <div v-if="syncStats.errors.length > 0" class="space-y-1">
                    <div v-for="(err, idx) in syncStats.errors" :key="idx"
                      class="text-2xs font-mono text-ms-danger truncate">
                      {{ err }}
                    </div>
                  </div>
                </div>
              </div>

              <!-- Local to Cloud Full Import -->
              <div class="border border-ms-border bg-ms-carbon/30 p-4 space-y-3">
                <div class="flex items-center justify-between">
                  <h3 class="text-xs font-mono font-bold text-slate-400 uppercase">Local → Cloud Import</h3>
                  <button @click="importLocalToCloud" :disabled="syncStatus === 'syncing'"
                    class="text-brass border border-brass/30 bg-brass/10 hover:bg-brass/20 px-3 py-1.5 text-2xs font-mono transition-all disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap">
                    {{ syncStatus === 'syncing' ? 'IMPORTING...' : 'IMPORT ALL .MD' }}
                  </button>
                </div>
                <div class="text-2xs font-mono text-slate-500">
                  将当前 Vault 目录下全部 Markdown 一次性导入到当前云端服务（按标题更新或创建）。
                </div>

                <div class="grid grid-cols-1 md:grid-cols-3 gap-2">
                  <label class="flex items-center gap-2 text-2xs font-mono text-slate-400">
                    <input v-model="importCategoryMode" type="radio" value="auto" class="accent-brass" />
                    自动按文件夹分类
                  </label>
                  <label class="flex items-center gap-2 text-2xs font-mono text-slate-400">
                    <input v-model="importCategoryMode" type="radio" value="none" class="accent-brass" />
                    不设置分类
                  </label>
                  <label class="flex items-center gap-2 text-2xs font-mono text-slate-400">
                    <input v-model="importCategoryMode" type="radio" value="selected" class="accent-brass" />
                    指定分类
                  </label>
                </div>

                <div v-if="importCategoryMode === 'selected'" class="space-y-2">
                  <label class="block text-2xs text-slate-500 font-mono">导入目标分类</label>
                  <select
                    v-model="selectedImportCategoryId"
                    class="w-full bg-ms-carbon border border-ms-border px-3 py-2 text-2xs text-slate-200 font-mono outline-none focus:border-brass/50"
                  >
                    <option :value="null">未分类</option>
                    <option v-for="cat in categories" :key="cat.id" :value="cat.id">
                      {{ cat.name }}
                    </option>
                  </select>
                </div>
              </div>

              <!-- Local to Cloud Watcher -->
              <div class="border border-ms-border bg-ms-carbon/30 p-4 space-y-3">
                <div class="flex items-center justify-between">
                  <h3 class="text-xs font-mono font-bold text-slate-400 uppercase">Local → Cloud</h3>
                  <button @click="toggleWatcher" class="px-3 py-1.5 text-2xs font-mono transition-all whitespace-nowrap"
                    :class="isWatching
                      ? 'text-ms-danger border border-ms-danger/30 bg-ms-danger/10 hover:bg-ms-danger/20'
                      : 'text-brass border border-brass/30 bg-brass/10 hover:bg-brass/20'">
                    {{ isWatching ? 'STOP WATCHER' : 'ENABLE WATCHER' }}
                  </button>
                </div>

                <!-- Watcher Status -->
                <div class="flex items-center gap-2 text-2xs font-mono">
                  <span class="w-1.5 h-1.5 rounded-full"
                    :class="isWatching ? 'bg-neon animate-pulse' : 'bg-slate-600'" />
                  <span :class="isWatching ? 'text-neon' : 'text-slate-600'">
                    {{ isWatching ? 'WATCHING' : 'IDLE' }}
                  </span>
                  <span v-if="syncStats.uploaded > 0 || syncStats.created > 0" class="text-slate-500">
                    · {{ syncStats.uploaded }} uploaded, {{ syncStats.created }} created
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- NETWORK Section -->
        <div v-show="activeSection === 'network'" class="max-w-2xl">
          <div class="flex items-center gap-2 mb-6">
            <Wifi class="w-4 h-4 text-neon" />
            <h2 class="text-sm font-mono font-bold text-slate-300 tracking-wider uppercase">Network Configuration</h2>
            <span v-if="networkStatus !== 'idle'" class="ml-2 font-mono text-sm" :class="getStatusColor(networkStatus)">
              {{ getStatusIcon(networkStatus) }}
            </span>
          </div>

          <div class="space-y-5">
            <div>
              <label class="block text-xs text-slate-500 font-mono mb-2">API Base URL</label>
              <div class="flex items-center gap-3">
                <input v-model="form.api_base_url" type="text" placeholder="http://localhost:8080/api/v1"
                  class="flex-1 bg-ms-carbon border border-ms-border px-4 py-3 text-xs text-slate-200 font-mono outline-none focus:border-brass/50 transition-colors" />
              </div>
            </div>

            <div>
              <label class="block text-xs text-slate-500 font-mono mb-2">WebSocket URL</label>
              <div class="flex items-center gap-3">
                <input v-model="form.ws_url" type="text" placeholder="ws://localhost:8080/api/v1/ws"
                  class="flex-1 bg-ms-carbon border border-ms-border px-4 py-3 text-xs text-slate-200 font-mono outline-none focus:border-brass/50 transition-colors" />
                <button @click="testNetwork" :disabled="networkStatus === 'testing'"
                  class="text-neon border border-neon/30 bg-neon/10 hover:bg-neon/20 px-4 py-3 text-xs font-mono transition-all disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap">
                  TEST
                </button>
              </div>
              <p v-if="networkError" class="mt-2 text-2xs font-mono text-ms-danger">{{ networkError }}</p>
            </div>
          </div>
        </div>

        <!-- STORAGE Section -->
        <div v-show="activeSection === 'storage'" class="max-w-2xl">
          <div class="flex items-center gap-2 mb-6">
            <Database class="w-4 h-4 text-neon" />
            <h2 class="text-sm font-mono font-bold text-slate-300 tracking-wider uppercase">Storage Configuration</h2>
            <span v-if="storageStatus !== 'idle'" class="ml-2 font-mono text-sm" :class="getStatusColor(storageStatus)">
              {{ getStatusIcon(storageStatus) }}
            </span>
          </div>

          <div class="space-y-4">
            <div>
              <label class="block text-xs text-slate-500 font-mono mb-2">S3 Endpoint</label>
              <input v-model="form.s3_endpoint" type="text" placeholder="https://s3.amazonaws.com"
                class="w-full bg-ms-carbon border border-ms-border px-4 py-3 text-xs text-slate-200 font-mono outline-none focus:border-brass/50 transition-colors" />
            </div>

            <div>
              <label class="block text-xs text-slate-500 font-mono mb-2">S3 Region</label>
              <input v-model="form.s3_region" type="text" placeholder="us-east-1"
                class="w-full bg-ms-carbon border border-ms-border px-4 py-3 text-xs text-slate-200 font-mono outline-none focus:border-brass/50 transition-colors" />
            </div>

            <div>
              <label class="block text-xs text-slate-500 font-mono mb-2">S3 Bucket</label>
              <input v-model="form.s3_bucket" type="text" placeholder="my-bucket"
                class="w-full bg-ms-carbon border border-ms-border px-4 py-3 text-xs text-slate-200 font-mono outline-none focus:border-brass/50 transition-colors" />
            </div>

            <div>
              <label class="block text-xs text-slate-500 font-mono mb-2">Access Key</label>
              <div class="flex items-center gap-3">
                <input v-model="form.s3_access_key" type="password" placeholder="••••••••"
                  class="flex-1 bg-ms-carbon border border-ms-border px-4 py-3 text-xs text-slate-200 font-mono outline-none focus:border-brass/50 transition-colors" />
                <Shield class="w-4 h-4 text-slate-600 shrink-0" />
              </div>
            </div>

            <div>
              <label class="block text-xs text-slate-500 font-mono mb-2">Secret Key</label>
              <div class="flex items-center gap-3">
                <input v-model="form.s3_secret_key" type="password" placeholder="••••••••••••••••"
                  class="flex-1 bg-ms-carbon border border-ms-border px-4 py-3 text-xs text-slate-200 font-mono outline-none focus:border-brass/50 transition-colors" />
                <Shield class="w-4 h-4 text-slate-600 shrink-0" />
              </div>
            </div>

            <div>
              <label class="block text-xs text-slate-500 font-mono mb-2">Path Style</label>
              <div class="flex items-center gap-3">
                <button type="button" @click="form.s3_use_path_style = !form.s3_use_path_style"
                  class="relative w-9 h-5 rounded-full transition-colors"
                  :class="form.s3_use_path_style ? 'bg-neon/40' : 'bg-slate-700'">
                  <span class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-slate-300 transition-transform"
                    :class="form.s3_use_path_style ? 'translate-x-4' : ''" />
                </button>
                <span class="text-xs text-slate-600 font-mono">{{ form.s3_use_path_style ? 'MinIO' : 'S3 / OSS'
                  }}</span>
              </div>
            </div>

            <div>
              <label class="block text-xs text-slate-500 font-mono mb-2">Public URL Base</label>
              <div class="flex items-center gap-3">
                <input v-model="form.s3_public_url_base" type="text" placeholder="https://cdn.example.com"
                  class="flex-1 bg-ms-carbon border border-ms-border px-4 py-3 text-xs text-slate-200 font-mono outline-none focus:border-brass/50 transition-colors" />
                <button @click="testStorage" :disabled="storageStatus === 'testing'"
                  class="text-neon border border-neon/30 bg-neon/10 hover:bg-neon/20 px-4 py-3 text-xs font-mono transition-all disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap">
                  TEST
                </button>
              </div>
              <p v-if="storageError" class="mt-2 text-2xs font-mono text-ms-danger">{{ storageError }}</p>
            </div>
          </div>
        </div>

        <!-- Error Message -->
        <div v-if="error"
          class="mt-6 bg-red-500/10 border border-red-500/30 px-4 py-3 text-xs text-red-400 font-mono max-w-2xl">
          {{ error }}
        </div>

      </div>
    </div>
  </div>
</template>

<style scoped>
input[type="password"] {
  -webkit-text-security: disc;
}

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
