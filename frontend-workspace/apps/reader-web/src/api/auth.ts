// ────────────────────────────────────────────────────────────────
// auth.ts — Silent guest authentication
// ────────────────────────────────────────────────────────────────

import { getClient } from './client'

export interface LoginResponse {
  access_token: string
  refresh_token: string
}

export async function login(
  username: string,
  password: string,
): Promise<LoginResponse> {
  return getClient().post<LoginResponse>('/auth/login', { username, password })
}
