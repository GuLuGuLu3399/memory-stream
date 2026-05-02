// ────────────────────────────────────────────────────────────────
// auth.ts — Silent guest authentication store
// ────────────────────────────────────────────────────────────────

import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as authApi from '@/api/auth'
import { getClient } from '@/api/client'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(null)
  const ready = ref(false)

  async function initAuth(): Promise<void> {
    if (token.value) return
    try {
      const res = await authApi.login('guest', 'guest123')
      token.value = res.access_token
      getClient().setToken(res.access_token)
    } finally {
      ready.value = true
    }
  }

  return { token, ready, initAuth }
})

/** Standalone silent login for 401 retry (avoids circular store dependency). */
export async function silentLogin(): Promise<void> {
  const res = await authApi.login('guest', 'guest123')
  getClient().setToken(res.access_token)
}
