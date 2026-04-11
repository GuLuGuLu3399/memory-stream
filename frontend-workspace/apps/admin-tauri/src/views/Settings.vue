<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useSysConfigStore } from '../stores/sysconfig'
import { useLayoutStore } from '../stores/layout'
import { Wifi, Database, Shield, FolderOpen, X } from 'lucide-vue-next'

const emit = defineEmits<{ (e: 'close'): void }>()

const store = useSysConfigStore()
const layoutStore = useLayoutStore()
const { isSettingsOpen } = storeToRefs(layoutStore)
const { loading, error } = storeToRefs(store)

// Local form state (not committed until save)
const form = ref({
  api_base_url: '',
  ws_url: '',
  s3_endpoint: '',
  s3_region: 'us-east-1',
  s3_bucket: '',
  s3_access_key: '',
  s3_secret_key: '',
  s3_public_url_base: '',
  s3_use_path_style: false,
  vault_path: null as string | null,
})

type TestStatus = 'idle' | 'testing' | 'ok' | 'failed'
const networkStatus = ref<TestStatus>('idle')
const storageStatus = ref<TestStatus>('idle')
const saving = ref(false)

// Load config when panel opens
watch(isSettingsOpen, async (isOpen) => {
  if (isOpen) {
    await store.loadConfig()
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
    } else {
      // First run defaults
      form.value = {
        api_base_url: 'http://localhost:8080/api/v1',
        ws_url: 'ws://localhost:8080/api/v1/ws',
        s3_endpoint: '',
        s3_region: 'us-east-1',
        s3_bucket: '',
        s3_access_key: '',
        s3_secret_key: '',
        s3_public_url_base: '',
        s3_use_path_style: false,
        vault_path: null,
      }
    }
    // Reset test states
    networkStatus.value = 'idle'
    storageStatus.value = 'idle'
  }
})

async function testNetwork() {
  networkStatus.value = 'testing'
  try {
    const controller = new AbortController()
    const timeoutId = setTimeout(() => controller.abort(), 5000)
    const url = new URL(form.value.api_base_url)

    await fetch(`${url.origin}/health`, {
      signal: controller.signal,
    })

    clearTimeout(timeoutId)
    networkStatus.value = 'ok'
  } catch {
    networkStatus.value = 'failed'
  }
}

async function testStorage() {
  storageStatus.value = 'testing'
  try {
    await store.saveConfig(form.value)
    const result = await store.testS3Connection()
    storageStatus.value = result ? 'ok' : 'failed'
  } catch {
    storageStatus.value = 'failed'
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
    await store.saveConfig(form.value)
    await store.reloadConfig()
    close()
  } catch (e: unknown) {
    // Error is stored in store.error
    console.error('[Settings] Save failed:', e)
  } finally {
    saving.value = false
  }
}

function close() {
  layoutStore.isSettingsOpen = false
  emit('close')
}

// Close settings panel with ESC key
function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') close()
}

onMounted(() => window.addEventListener('keydown', handleKeydown))
onUnmounted(() => window.removeEventListener('keydown', handleKeydown))

// Status indicator component
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
  <Transition name="ms-fade">
    <div
      v-if="isSettingsOpen"
      class="fixed inset-x-0 bottom-0 top-titlebar z-panel flex items-center justify-center bg-ms-deep/95 backdrop-blur-sm"
      @click.self="close"
    >
      <div class="w-full max-w-2xl max-h-[90vh] overflow-y-auto bg-ms-panel border border-ms-border shadow-2xl" style="box-shadow: inset 0 1px 3px rgba(0,0,0,0.3), inset 0 0 0 1px rgba(42,42,42,0.5);">

        <!-- Header -->
        <div class="h-14 flex items-center justify-between px-6 border-b border-ms-border bg-ms-carbon shrink-0">
          <div class="flex items-center gap-3">
            <span class="text-neon text-lg">◆</span>
            <span class="text-sm font-mono font-bold text-slate-300 tracking-wider">
              SYSTEM CONFIG
            </span>
            <span class="text-xs text-slate-600 font-mono">— 系统配置舱</span>
          </div>
          <button
            @click="close"
            class="text-slate-500 hover:text-slate-300 transition-colors p-1"
            title="关闭"
          >
            <X class="w-5 h-5" />
          </button>
        </div>

        <!-- Content -->
        <div class="p-6 space-y-6">

          <!-- VAULT Section -->
          <div class="border border-ms-border bg-ms-deep">
            <div class="h-10 flex items-center gap-2 px-4 border-b border-ms-border bg-ms-carbon">
              <FolderOpen class="w-3.5 h-3.5 text-brass" />
              <span class="text-brass text-xs tracking-widest uppercase font-bold font-mono">
                [ VAULT ]
              </span>
            </div>
            <div class="p-4 space-y-3">
              <div class="grid grid-cols-[140px_1fr_auto] items-center gap-3">
                <label class="text-xs text-slate-500 font-mono text-right">知识库目录</label>
                <div class="flex items-center gap-2 min-w-0">
                  <span
                    v-if="form.vault_path"
                    class="text-xs text-slate-300 font-mono truncate"
                    :title="form.vault_path"
                  >
                    {{ form.vault_path }}
                  </span>
                  <span v-else class="text-xs text-slate-600 font-mono italic">
                    未设置
                  </span>
                </div>
                <div class="flex items-center gap-2">
                  <button
                    @click="selectVaultDir"
                    class="text-brass border border-brass/30 bg-brass/10 hover:bg-brass/20 px-3 py-1.5 text-xs font-mono transition-all whitespace-nowrap"
                  >
                    {{ form.vault_path ? 'CHANGE' : 'SELECT' }}
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- NETWORK Section -->
          <div class="border border-ms-border bg-ms-deep">
            <div class="h-10 flex items-center gap-2 px-4 border-b border-ms-border bg-ms-carbon">
              <Wifi class="w-3.5 h-3.5 text-neon" />
              <span class="text-neon text-xs tracking-widest uppercase font-bold font-mono">
                [ NETWORK ]
              </span>
            </div>
            <div class="p-4 space-y-3">
              <div class="grid grid-cols-[140px_1fr_auto] items-center gap-3">
                <label class="text-xs text-slate-500 font-mono text-right">API Base URL</label>
                <input
                  v-model="form.api_base_url"
                  type="text"
                  placeholder="http://localhost:8080/api/v1"
                  class="bg-ms-panel border border-ms-border px-3 py-2 text-xs text-slate-200 font-mono outline-none focus:border-cyan-500/50 transition-colors"
                />
                <div class="w-8"></div>
              </div>
              <div class="grid grid-cols-[140px_1fr_auto] items-center gap-3">
                <label class="text-xs text-slate-500 font-mono text-right">WebSocket URL</label>
                <input
                  v-model="form.ws_url"
                  type="text"
                  placeholder="ws://localhost:8080/api/v1/ws"
                  class="bg-ms-panel border border-ms-border px-3 py-2 text-xs text-slate-200 font-mono outline-none focus:border-cyan-500/50 transition-colors"
                />
                <div class="flex items-center gap-2">
                  <button
                    @click="testNetwork"
                    :disabled="networkStatus === 'testing'"
                    class="text-cyan-400 border border-cyan-500/30 bg-cyan-500/10 hover:bg-cyan-500/20 px-3 py-1.5 text-xs font-mono transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    TEST
                  </button>
                  <span
                    v-if="networkStatus !== 'idle'"
                    class="w-5 h-5 flex items-center justify-center font-mono text-sm"
                    :class="getStatusColor(networkStatus)"
                  >
                    {{ getStatusIcon(networkStatus) }}
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- STORAGE Section -->
          <div class="border border-ms-border bg-ms-deep">
            <div class="h-10 flex items-center gap-2 px-4 border-b border-ms-border bg-ms-carbon">
              <Database class="w-3.5 h-3.5 text-neon" />
              <span class="text-neon text-xs tracking-widest uppercase font-bold font-mono">
                [ STORAGE ]
              </span>
            </div>
            <div class="p-4 space-y-3">
              <div class="grid grid-cols-[140px_1fr_auto] items-center gap-3">
                <label class="text-xs text-slate-500 font-mono text-right">S3 Endpoint</label>
                <input
                  v-model="form.s3_endpoint"
                  type="text"
                  placeholder="https://s3.amazonaws.com"
                  class="bg-ms-panel border border-ms-border px-3 py-2 text-xs text-slate-200 font-mono outline-none focus:border-cyan-500/50 transition-colors"
                />
                <div class="w-8"></div>
              </div>
              <div class="grid grid-cols-[140px_1fr_auto] items-center gap-3">
                <label class="text-xs text-slate-500 font-mono text-right">S3 Region</label>
                <input
                  v-model="form.s3_region"
                  type="text"
                  placeholder="us-east-1"
                  class="bg-ms-panel border border-ms-border px-3 py-2 text-xs text-slate-200 font-mono outline-none focus:border-cyan-500/50 transition-colors"
                />
                <div class="w-8"></div>
              </div>
              <div class="grid grid-cols-[140px_1fr_auto] items-center gap-3">
                <label class="text-xs text-slate-500 font-mono text-right">S3 Bucket</label>
                <input
                  v-model="form.s3_bucket"
                  type="text"
                  placeholder="my-bucket"
                  class="bg-ms-panel border border-ms-border px-3 py-2 text-xs text-slate-200 font-mono outline-none focus:border-cyan-500/50 transition-colors"
                />
                <div class="w-8"></div>
              </div>
              <div class="grid grid-cols-[140px_1fr_auto] items-center gap-3">
                <label class="text-xs text-slate-500 font-mono text-right">Access Key</label>
                <input
                  v-model="form.s3_access_key"
                  type="password"
                  placeholder="••••••••"
                  class="bg-ms-panel border border-ms-border px-3 py-2 text-xs text-slate-200 font-mono outline-none focus:border-cyan-500/50 transition-colors"
                />
                <Shield class="w-4 h-4 text-slate-600" />
              </div>
              <div class="grid grid-cols-[140px_1fr_auto] items-center gap-3">
                <label class="text-xs text-slate-500 font-mono text-right">Secret Key</label>
                <input
                  v-model="form.s3_secret_key"
                  type="password"
                  placeholder="••••••••••••••••"
                  class="bg-ms-panel border border-ms-border px-3 py-2 text-xs text-slate-200 font-mono outline-none focus:border-cyan-500/50 transition-colors"
                />
                <div class="flex items-center gap-2">
                  <Shield class="w-4 h-4 text-slate-600" />
                </div>
              </div>
              <div class="grid grid-cols-[140px_1fr_auto] items-center gap-3">
                <label class="text-xs text-slate-500 font-mono text-right">Path Style</label>
                <div class="flex items-center gap-2">
                  <button
                    type="button"
                    @click="form.s3_use_path_style = !form.s3_use_path_style"
                    class="relative w-9 h-5 rounded-full transition-colors"
                    :class="form.s3_use_path_style ? 'bg-cyan-500/40' : 'bg-slate-700'"
                  >
                    <span
                      class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-slate-300 transition-transform"
                      :class="form.s3_use_path_style ? 'translate-x-4' : ''"
                    />
                  </button>
                  <span class="text-xs text-slate-600 font-mono">{{ form.s3_use_path_style ? 'MinIO' : 'S3 / OSS' }}</span>
                </div>
                <div class="w-8"></div>
              </div>
              <div class="grid grid-cols-[140px_1fr_auto] items-center gap-3">
                <label class="text-xs text-slate-500 font-mono text-right">Public URL</label>
                <input
                  v-model="form.s3_public_url_base"
                  type="text"
                  placeholder="https://cdn.example.com"
                  class="bg-ms-panel border border-ms-border px-3 py-2 text-xs text-slate-200 font-mono outline-none focus:border-cyan-500/50 transition-colors"
                />
                <div class="flex items-center gap-2">
                  <button
                    @click="testStorage"
                    :disabled="storageStatus === 'testing'"
                    class="text-cyan-400 border border-cyan-500/30 bg-cyan-500/10 hover:bg-cyan-500/20 px-3 py-1.5 text-xs font-mono transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    TEST
                  </button>
                  <span
                    v-if="storageStatus !== 'idle'"
                    class="w-5 h-5 flex items-center justify-center font-mono text-sm"
                    :class="getStatusColor(storageStatus)"
                  >
                    {{ getStatusIcon(storageStatus) }}
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- Error Message -->
          <div
            v-if="error"
            class="bg-red-500/10 border border-red-500/30 px-4 py-3 text-xs text-red-400 font-mono"
          >
            {{ error }}
          </div>

        </div>

        <!-- Footer Actions -->
        <div class="h-16 flex items-center justify-between px-6 border-t border-ms-border bg-ms-carbon">
          <button
            @click="testAll"
            :disabled="networkStatus === 'testing' || storageStatus === 'testing'"
            class="px-4 py-2 text-xs font-mono border border-ms-border text-slate-400 hover:text-slate-300 hover:border-slate-500 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
          >
            TEST ALL
          </button>
          <div class="flex items-center gap-3">
            <button
              @click="close"
              class="px-4 py-2 text-xs font-mono border border-ms-border text-slate-500 hover:text-slate-300 transition-colors"
            >
              CANCEL
            </button>
            <button
              @click="save"
              :disabled="!canSave"
              class="flex items-center gap-2 px-6 py-2 text-xs font-mono transition-all"
              :class="canSave
                ? 'bg-brass/10 border border-brass/30 text-brass hover:bg-brass/20'
                : 'bg-ms-surface border border-ms-border text-slate-600 opacity-30 cursor-not-allowed'"
            >
              <span class="text-neon">◆</span>
              SAVE CONFIG
            </button>
          </div>
        </div>

      </div>
    </div>
  </Transition>
</template>

<style scoped>
/* Input password field masking */
input[type="password"] {
  -webkit-text-security: disc;
}
</style>
