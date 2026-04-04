/**
 * useAuth — Tauri 桌面端静默登录
 *
 * 应用启动时自动调用 Rust 侧的 login command，
 * 拿到 JWT 后注入 AuthState，后续所有 API 请求自动携带。
 */
import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";

/** 登录完成后设为 true，控制子组件渲染时机 */
const isReady = ref(false);

/** 是否正在登录中 */
const isLoading = ref(false);

/** 登录错误信息 */
const authError = ref("");

export function useAuth() {
  /**
   * 静默登录：使用默认 admin 凭据调用 Rust login command
   *
   * Rust 侧会向 Go 后端 POST /auth/login，
   * 拿到 JWT 后存入 Arc<AuthState>，后续所有请求自动携带。
   */
  async function silentLogin(): Promise<void> {
    if (isReady.value || isLoading.value) return;

    isLoading.value = true;
    authError.value = "";

    try {
      await invoke("login", {
        username: "admin",
        password: "admin123",
      });
      console.log("[Auth] ✅ logged in successfully");
    } catch (e) {
      console.warn("[Auth] login failed:", e);
      authError.value = String(e);
    } finally {
      isLoading.value = false;
      isReady.value = true;
    }
  }

  /**
   * 手动登录（用于登录失败后重试或切换账号）
   */
  async function login(username: string, password: string): Promise<boolean> {
    try {
      await invoke("login", { username, password });
      authError.value = "";
      isReady.value = true;
      return true;
    } catch (e) {
      authError.value = String(e);
      return false;
    }
  }

  return {
    isReady,
    isLoading,
    authError,
    silentLogin,
    login,
  };
}
