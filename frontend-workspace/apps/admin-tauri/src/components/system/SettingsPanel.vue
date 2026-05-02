// 设置面板，包含 Vault 路径、服务器地址、S3 存储配置和关于信息
<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { X, FolderOpen, Database, Info, Check, Loader2 } from 'lucide-vue-next'
import type { AppConfig, StorageConfig } from '@memory-stream/types'
import * as configService from '@/services/config'
import { useEditorStore } from '@/stores/editor'
import { useTreeStore } from '@/stores/tree'
import { useToast } from '@/composables/core/useToast'
import debounce from 'lodash-es/debounce'

const emit = defineEmits<{
  close: []
}>()

const toast = useToast()
const treeStore = useTreeStore()
const editorStore = useEditorStore()
const loading = ref(true)
const saving = ref(false)
const testing = ref(false)
const loggingIn = ref(false)
const connectionOk = ref<boolean | null>(null)
const activeSection = ref('general')

const config = reactive<AppConfig>({
  api_base_url: '',
  vault_path: '',
  theme: 'dark',
  storage: null,
})

const storage = reactive<StorageConfig>({
  provider: '',
  endpoint: '',
  region: '',
  bucket: '',
  access_key: '',
  secret_key: '',
  public_domain: '',
  force_path_style: false,
})

onMounted(async () => {
  try {
    const c = await configService.loadConfig()
    Object.assign(config, c)
    if (c.storage) Object.assign(storage, c.storage)
  } catch {
    toast.error('配置加载失败')
  } finally {
    loading.value = false
  }
})

// Auto-save with debounce
const autoSave = debounce(async (patch: Partial<AppConfig>) => {
  saving.value = true
  try {
    await configService.updateConfig(patch)
  } catch {
    toast.error('自动保存失败')
  } finally {
    saving.value = false
  }
}, 800)

function onFieldChange(patch: Partial<AppConfig>) {
  autoSave(patch)
}

// Vault folder picker
async function pickVaultFolder() {
  const selected = await open({ directory: true, title: '选择 Vault 目录' })
  if (selected) {
    const previousPath = config.vault_path
    config.vault_path = selected
    await configService.updateConfig({ vault_path: selected })

    if (selected !== previousPath) {
      editorStore.clear()
      treeStore.setActive(null)
      await treeStore.loadTree()
    }

    toast.success('Vault 路径已更新')
  }
}

// Test connection
async function testConnection() {
  testing.value = true
  connectionOk.value = null
  try {
    const res = await fetch(`${config.api_base_url.replace(/\/+$/, '')}/health`)
    connectionOk.value = res.ok
    if (res.ok) {
      toast.success('连接成功')
    } else {
      toast.error(`服务器返回 ${res.status}`)
    }
  } catch {
    connectionOk.value = false
    toast.error('无法连接服务器')
  } finally {
    testing.value = false
  }
}

// Login to server
async function handleLogin() {
  loggingIn.value = true
  try {
    const { loginToServer } = await import('@/bridge/invoke')
    await loginToServer('admin', 'admin123')
    toast.success('认证成功，同步已就绪')
    connectionOk.value = true
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : String(e))
  } finally {
    loggingIn.value = false
  }
}

// Save storage config
async function saveStorage() {
  saving.value = true
  try {
    await configService.updateConfig({ storage: { ...storage } })
    toast.success('存储配置已保存')
  } catch {
    toast.error('保存失败')
  } finally {
    saving.value = false
  }
}

// Clear storage config
async function clearStorage() {
  saving.value = true
  try {
    const { clearStorageConfig } = await import('@/bridge/invoke')
    await clearStorageConfig()
    Object.assign(storage, {
      provider: '', endpoint: '', region: '', bucket: '',
      access_key: '', secret_key: '', public_domain: '', force_path_style: false,
    })
    config.storage = null
    toast.success('存储配置已清除')
  } catch {
    toast.error('清除失败')
  } finally {
    saving.value = false
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
}
</script>

<template>
  <Teleport to="body">
    <div class="settings-overlay" @click.self="emit('close')" @keydown="handleKeydown">
      <div class="settings-panel animate-slide-in-right">
        <!-- Header -->
        <div class="settings-header">
          <h2 class="settings-title">SETTINGS</h2>
          <div class="settings-header-right">
            <span v-if="saving" class="settings-saving">保存中...</span>
            <button class="settings-close" @click="emit('close')">
              <X :size="16" :stroke-width="1.5" />
            </button>
          </div>
        </div>

        <!-- Nav -->
        <div class="settings-nav">
          <button v-for="tab in [
            { id: 'general', icon: 'folder', label: '通用' },
            { id: 'storage', icon: 'database', label: '存储' },
            { id: 'about', icon: 'info', label: '关于' },
          ]" :key="tab.id" class="settings-tab" :class="{ active: activeSection === tab.id }"
            @click="activeSection = tab.id">
            <FolderOpen v-if="tab.icon === 'folder'" :size="14" />
            <Database v-else-if="tab.icon === 'database'" :size="14" />
            <Info v-else-if="tab.icon === 'info'" :size="14" />
            {{ tab.label }}
          </button>
        </div>

        <!-- Content -->
        <div class="settings-body">
          <div v-if="loading" class="settings-loading animate-neon-pulse">加载中...</div>

          <!-- General: Vault + Server in one section -->
          <template v-else-if="activeSection === 'general'">
            <!-- Vault -->
            <div class="settings-group">
              <div class="settings-group-title">Vault</div>
              <div class="settings-field">
                <label class="settings-label">数据目录</label>
                <div class="settings-path-row">
                  <input :value="config.vault_path" class="settings-input settings-input-path"
                    placeholder="点击右侧按钮选择目录..." readonly />
                  <button class="settings-pick-btn" @click="pickVaultFolder">
                    <FolderOpen :size="14" />
                  </button>
                </div>
              </div>
            </div>

            <!-- Server -->
            <div class="settings-group">
              <div class="settings-group-title">服务器</div>
              <div class="settings-field">
                <label class="settings-label">API 地址</label>
                <input v-model="config.api_base_url" class="settings-input" placeholder="http://localhost:8080"
                  @blur="onFieldChange({ api_base_url: config.api_base_url })" />
              </div>
              <button class="settings-test-btn" :disabled="testing || !config.api_base_url" @click="testConnection">
                <Loader2 v-if="testing" :size="14" class="animate-spin-slow" />
                <Check v-else-if="connectionOk === true" :size="14" />
                <span v-else>测试连接</span>
              </button>
              <button class="settings-test-btn" :disabled="loggingIn || !config.api_base_url" @click="handleLogin">
                <Loader2 v-if="loggingIn" :size="14" class="animate-spin-slow" />
                <span v-else>登录认证</span>
              </button>
            </div>
          </template>

          <!-- Storage -->
          <template v-else-if="activeSection === 'storage'">
            <div class="settings-group">
              <div class="settings-group-title">S3 兼容存储</div>
              <div class="settings-field">
                <label class="settings-label">Endpoint</label>
                <input v-model="storage.endpoint" class="settings-input" placeholder="https://s3.example.com" />
              </div>
              <div class="settings-row">
                <div class="settings-field settings-field-half">
                  <label class="settings-label">Bucket</label>
                  <input v-model="storage.bucket" class="settings-input" placeholder="my-bucket" />
                </div>
                <div class="settings-field settings-field-half">
                  <label class="settings-label">Region</label>
                  <input v-model="storage.region" class="settings-input" placeholder="us-east-1" />
                </div>
              </div>
              <div class="settings-row">
                <div class="settings-field settings-field-half">
                  <label class="settings-label">Access Key</label>
                  <input v-model="storage.access_key" class="settings-input" type="password" />
                </div>
                <div class="settings-field settings-field-half">
                  <label class="settings-label">Secret Key</label>
                  <input v-model="storage.secret_key" class="settings-input" type="password" />
                </div>
              </div>
              <div class="settings-field">
                <label class="settings-label">Public Domain</label>
                <input v-model="storage.public_domain" class="settings-input" placeholder="https://cdn.example.com" />
              </div>
            </div>
            <div class="settings-actions">
              <button class="settings-clear-btn" @click="clearStorage">清除配置</button>
              <button class="settings-save-btn" :disabled="saving" @click="saveStorage">
                {{ saving ? '保存中...' : '保存' }}
              </button>
            </div>
          </template>

          <!-- About -->
          <template v-else-if="activeSection === 'about'">
            <div class="settings-about">
              <div class="settings-about-name">Memory Stream</div>
              <div class="settings-about-row">
                <span>版本</span><span class="settings-about-mono">0.1.0</span>
              </div>
              <div class="settings-about-row">
                <span>框架</span><span>Tauri v2 + Vue 3</span>
              </div>
              <div class="settings-about-row">
                <span>主题</span><span>Mechanical Altar</span>
              </div>
              <div class="settings-about-row">
                <span>引擎</span><span class="settings-about-mono">ms-ast / ms-io / ms-graph</span>
              </div>
            </div>
          </template>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.settings-overlay {
  position: fixed;
  inset: 0;
  background: oklch(0 0 0 / 0.5);
  display: flex;
  justify-content: flex-end;
  z-index: var(--z-overlay);
}

.settings-panel {
  width: 380px;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ms-void);
  border-left: 1px solid var(--ms-border);
  box-shadow: -4px 0 24px oklch(0 0 0 / 0.4);
}

/* Header */
.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
  border-bottom: 1px solid var(--ms-border);
  flex-shrink: 0;
}

.settings-title {
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  margin: 0;
  letter-spacing: 0.1em;
}

.settings-header-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.settings-saving {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--brass);
  animation: neon-pulse 1.5s ease-in-out infinite;
}

.settings-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 2px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: background var(--duration-fast) var(--ease-hydraulic),
    color var(--duration-fast) var(--ease-hydraulic);
}

.settings-close:hover {
  background: var(--ms-surface);
  color: var(--text-primary);
}

/* Nav */
.settings-nav {
  display: flex;
  border-bottom: 1px solid var(--ms-border);
  flex-shrink: 0;
}

.settings-tab {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  flex: 1;
  padding: 10px 12px;
  border: none;
  border-bottom: 2px solid transparent;
  background: transparent;
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--text-muted);
  cursor: pointer;
  transition: color var(--duration-fast) var(--ease-hydraulic),
    border-color var(--duration-fast) var(--ease-hydraulic);
}

.settings-tab:hover {
  color: var(--text-secondary);
}

.settings-tab.active {
  color: var(--neon);
  border-bottom-color: var(--neon);
}

/* Body */
.settings-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.settings-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 120px;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--text-muted);
}

/* Groups */
.settings-group {
  margin-bottom: 24px;
}

.settings-group-title {
  font-family: var(--font-sans);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--text-muted);
  margin-bottom: 12px;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--ms-border);
}

/* Fields */
.settings-field {
  margin-bottom: 12px;
}

.settings-label {
  display: block;
  font-family: var(--font-sans);
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 5px;
}

.settings-input {
  width: 100%;
  padding: 7px 10px;
  background: var(--ms-deep);
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--text-primary);
  outline: none;
  box-shadow: var(--shadow-inset);
  transition: border-color var(--duration-fast) var(--ease-hydraulic),
    box-shadow var(--duration-fast) var(--ease-hydraulic);
}

.settings-input:focus {
  border-color: var(--neon);
  box-shadow: var(--shadow-inset), 0 0 8px oklch(0.78 0.17 200 / 0.15);
}

.settings-input::placeholder {
  color: var(--text-muted);
}

.settings-input-path {
  cursor: default;
}

/* Path row (input + pick button) */
.settings-path-row {
  display: flex;
  gap: 6px;
}

.settings-path-row .settings-input {
  flex: 1;
}

.settings-pick-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  background: var(--ms-deep);
  color: var(--text-muted);
  cursor: pointer;
  flex-shrink: 0;
  transition: border-color var(--duration-fast) var(--ease-hydraulic),
    color var(--duration-fast) var(--ease-hydraulic),
    box-shadow var(--duration-fast) var(--ease-hydraulic);
}

.settings-pick-btn:hover {
  border-color: var(--neon);
  color: var(--neon);
  box-shadow: 0 0 8px oklch(0.78 0.17 200 / 0.15);
}

/* Two-column row */
.settings-row {
  display: flex;
  gap: 10px;
}

.settings-field-half {
  flex: 1;
}

/* Test connection button */
.settings-test-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 100%;
  padding: 7px 0;
  margin-top: 4px;
  border: 1px solid var(--ms-border-light);
  border-radius: 2px;
  background: transparent;
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: border-color var(--duration-fast) var(--ease-hydraulic),
    color var(--duration-fast) var(--ease-hydraulic);
}

.settings-test-btn:hover:not(:disabled) {
  border-color: var(--neon);
  color: var(--neon);
}

.settings-test-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* Actions row */
.settings-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.settings-save-btn {
  flex: 1;
  padding: 8px 0;
  border: 1px solid var(--neon);
  border-radius: 2px;
  background: transparent;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--neon);
  cursor: pointer;
  transition: background var(--duration-fast) var(--ease-hydraulic);
}

.settings-save-btn:hover {
  background: oklch(0.78 0.17 200 / 0.1);
}

.settings-save-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.settings-clear-btn {
  padding: 8px 16px;
  border: 1px solid var(--ms-border-light);
  border-radius: 2px;
  background: transparent;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--text-muted);
  cursor: pointer;
  transition: border-color var(--duration-fast) var(--ease-hydraulic),
    color var(--duration-fast) var(--ease-hydraulic);
}

.settings-clear-btn:hover {
  border-color: var(--destructive);
  color: var(--destructive);
}

/* About */
.settings-about {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.settings-about-name {
  font-family: var(--font-sans);
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: 0.04em;
  margin-bottom: 8px;
}

.settings-about-row {
  display: flex;
  justify-content: space-between;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--text-secondary);
  padding: 10px 0;
  border-bottom: 1px solid var(--ms-border);
}

.settings-about-mono {
  font-family: var(--font-mono);
  font-size: 12px;
}
</style>
