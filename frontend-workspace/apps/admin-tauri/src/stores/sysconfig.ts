import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { SysConfig } from "@memory-stream/types/ipc";

export type { SysConfig };

export type ConnectionStatus = 'idle' | 'testing' | 'ok' | 'failed'

export const useSysConfigStore = defineStore('sysconfig', () => {
  const config = ref<SysConfig | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const connectionStatus = ref<ConnectionStatus>('idle')

  const isConfigured = computed(() => config.value !== null)

  async function loadConfig() {
    loading.value = true
    error.value = null
    try {
      config.value = await invoke<SysConfig>('get_sys_config')
    } catch (e) {
      // Fall back to defaults if store is not yet available
      config.value = {
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
    } finally {
      loading.value = false
    }
  }

  async function saveConfig(newConfig: SysConfig) {
    loading.value = true
    error.value = null
    try {
      await invoke('save_sys_config', { config: newConfig })
      config.value = { ...newConfig }
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function testApiConnection(): Promise<boolean> {
    try {
      await invoke('test_api_connection')
      return true
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e)
      return false
    }
  }

  async function testS3Connection(): Promise<boolean> {
    try {
      await invoke('test_s3_connection')
      return true
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e)
      return false
    }
  }

  async function testConnection(): Promise<boolean> {
    connectionStatus.value = 'testing'
    const apiOk = await testApiConnection()
    const s3Ok = await testS3Connection()
    const ok = apiOk && s3Ok
    connectionStatus.value = ok ? 'ok' : 'failed'
    return ok
  }

  async function reloadConfig() {
    try {
      await invoke('reload_sys_config')
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    }
  }

  /** 唤起原生文件夹选择器，选定后持久化 vault_path 到配置 */
  async function selectVaultDirectory(): Promise<string | null> {
    try {
      const selected = await open({ directory: true, multiple: false, title: '选择知识库目录' })
      // open() returns null when user cancels, string for single, string[] for multiple
      if (!selected) return null
      const dirPath = typeof selected === 'string' ? selected : selected[0]
      if (!dirPath) return null

      const updated = await invoke<SysConfig>('set_vault_path', { path: dirPath })
      config.value = { ...updated }
      return dirPath
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    }
  }

  return {
    config,
    loading,
    error,
    connectionStatus,
    isConfigured,
    loadConfig,
    saveConfig,
    testConnection,
    testApiConnection,
    testS3Connection,
    reloadConfig,
    selectVaultDirectory,
  }
})
