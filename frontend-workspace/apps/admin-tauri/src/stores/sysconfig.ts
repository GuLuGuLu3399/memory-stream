import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { SysConfig } from "@memory-stream/types/ipc";
import { extractMsg } from "../composables/useTempleError";

export type { SysConfig };

export type ConnectionStatus = "idle" | "testing" | "ok" | "failed";
export type SyncStatus = "idle" | "syncing" | "ok" | "failed";
export type ImportCategoryMode = "auto" | "none" | "selected";

export interface ImportLocalOptions {
  categoryMode: ImportCategoryMode;
  selectedCategoryId?: number | null;
}

function createDefaultConfig(): SysConfig {
  return {
    api_base_url: "http://localhost:8080/api/v1",
    ws_url: "ws://localhost:8080/api/v1/ws",
    s3_endpoint: "http://localhost:9000",
    s3_region: "us-east-1",
    s3_bucket: "memory-stream",
    s3_access_key: "admin",
    s3_secret_key: "adminpassword",
    s3_public_url_base: "http://localhost:9000",
    s3_use_path_style: true,
    vault_path: null,
  };
}

export const useSysConfigStore = defineStore("sysconfig", () => {
  const config = ref<SysConfig | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const connectionStatus = ref<ConnectionStatus>("idle");
  const syncStatus = ref<SyncStatus>("idle");
  const syncStats = ref({
    synced: 0,
    skipped: 0,
    uploaded: 0,
    created: 0,
    errors: [] as string[],
  });

  const isConfigured = computed(() => config.value !== null);

  async function loadConfig() {
    loading.value = true;
    error.value = null;
    try {
      config.value = await invoke<SysConfig>("get_sys_config");
    } catch (e: unknown) {
      config.value = createDefaultConfig();
      error.value = extractMsg(e);
      // 读取失败时保留当前内存配置，避免 UI 被默认值覆盖造成“重置”错觉。
    } finally {
      loading.value = false;
    }
  }

  async function saveConfig(newConfig: SysConfig) {
    loading.value = true;
    error.value = null;
    try {
      // save_sys_config 现在返回保存后的配置
      const savedConfig = await invoke<SysConfig>("save_sys_config", {
        config: newConfig,
      });
      config.value = savedConfig ?? { ...newConfig };
    } catch (e: unknown) {
      error.value = extractMsg(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function testApiConnection(): Promise<boolean> {
    try {
      await invoke("test_api_connection");
      return true;
    } catch (e: unknown) {
      error.value = extractMsg(e);
      return false;
    }
  }

  async function testS3Connection(): Promise<boolean> {
    try {
      await invoke("test_s3_connection");
      return true;
    } catch (e: unknown) {
      error.value = extractMsg(e);
      return false;
    }
  }

  async function testConnection(): Promise<boolean> {
    connectionStatus.value = "testing";
    const apiOk = await testApiConnection();
    const s3Ok = await testS3Connection();
    const ok = apiOk && s3Ok;
    connectionStatus.value = ok ? "ok" : "failed";
    return ok;
  }

  async function reloadConfig() {
    try {
      await invoke("reload_sys_config");
    } catch (e: unknown) {
      error.value = extractMsg(e);
      throw e;
    }
  }

  /** 唤起原生文件夹选择器，选定后持久化 vault_path 到配置 */
  async function selectVaultDirectory(): Promise<string | null> {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "选择知识库目录",
      });
      // open() returns null when user cancels, string for single, string[] for multiple
      if (!selected) return null;
      const dirPath = typeof selected === "string" ? selected : selected[0];
      if (!dirPath) return null;

      const updated = await invoke<SysConfig>("set_vault_path", {
        path: dirPath,
      });
      config.value = { ...updated };
      return dirPath;
    } catch (e: unknown) {
      error.value = extractMsg(e);
      return null;
    }
  }

  async function syncCloudToVault() {
    if (!config.value?.vault_path) return;
    syncStatus.value = "syncing";
    try {
      const result = await invoke<{
        synced: number;
        skipped: number;
        errors: string[];
      }>("sync_cloud_to_vault");
      syncStats.value.synced = result.synced;
      syncStats.value.skipped = result.skipped;
      syncStats.value.errors = result.errors;
      syncStatus.value = result.errors.length > 0 ? "failed" : "ok";
    } catch (e) {
      syncStatus.value = "failed";
      syncStats.value.errors = [extractMsg(e)];
    }
  }

  async function importLocalVaultToCloud(
    options: ImportLocalOptions = { categoryMode: "auto" },
  ) {
    if (!config.value?.vault_path) return;
    syncStatus.value = "syncing";
    try {
      const result = await invoke<{
        uploaded: number;
        created: number;
        errors: string[];
      }>("import_local_vault_to_cloud", {
        categoryMode: options.categoryMode,
        selectedCategoryId: options.selectedCategoryId ?? null,
      });
      syncStats.value.uploaded = result.uploaded;
      syncStats.value.created = result.created;
      syncStats.value.errors = result.errors;
      syncStatus.value = result.errors.length > 0 ? "failed" : "ok";
    } catch (e) {
      syncStatus.value = "failed";
      syncStats.value.errors = [extractMsg(e)];
    }
  }

  return {
    config,
    loading,
    error,
    connectionStatus,
    syncStatus,
    syncStats,
    isConfigured,
    loadConfig,
    saveConfig,
    testConnection,
    testApiConnection,
    testS3Connection,
    reloadConfig,
    selectVaultDirectory,
    syncCloudToVault,
    importLocalVaultToCloud,
  };
});
