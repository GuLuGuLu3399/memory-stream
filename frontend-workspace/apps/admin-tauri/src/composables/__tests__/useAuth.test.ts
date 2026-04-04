// @vitest-environment jsdom
import { describe, it, expect, beforeEach, vi } from 'vitest'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('useAuth', () => {
  let useAuth: typeof import('../useAuth')['useAuth']
  let mockInvoke: ReturnType<typeof vi.fn>

  beforeEach(async () => {
    vi.resetModules()
    vi.doMock('@tauri-apps/api/core', () => ({
      invoke: vi.fn(),
    }))
    const mod = await import('../useAuth')
    useAuth = mod.useAuth
    const core = await import('@tauri-apps/api/core')
    mockInvoke = vi.mocked(core.invoke)
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('should have isReady false initially', () => {
      const { isReady } = useAuth()

      expect(isReady.value).toBe(false)
    })

    it('should have isLoading false initially', () => {
      const { isLoading } = useAuth()

      expect(isLoading.value).toBe(false)
    })

    it('should have empty authError initially', () => {
      const { authError } = useAuth()

      expect(authError.value).toBe('')
    })
  })

  describe('silentLogin', () => {
    it('should call invoke with default credentials', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const { silentLogin } = useAuth()
      await silentLogin()

      expect(mockInvoke).toHaveBeenCalledWith('login', {
        username: 'admin',
        password: 'admin123',
      })
    })

    it('should set isReady to true on success', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const { silentLogin, isReady } = useAuth()
      await silentLogin()

      expect(isReady.value).toBe(true)
    })

    it('should set isLoading during login', async () => {
      let resolveLogin: () => void
      mockInvoke.mockImplementation(() => new Promise((resolve) => {
        resolveLogin = resolve
      }))

      const { silentLogin, isLoading } = useAuth()
      const loginPromise = silentLogin()

      expect(isLoading.value).toBe(true)

      resolveLogin!()
      await loginPromise

      expect(isLoading.value).toBe(false)
    })

    it('should set authError on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Login failed'))

      const { silentLogin, authError, isReady } = useAuth()
      await silentLogin()

      expect(authError.value).toBe('Error: Login failed')
      expect(isReady.value).toBe(true)
    })

    it('should NOT login if already ready', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const { silentLogin, isReady } = useAuth()
      await silentLogin()

      expect(mockInvoke).toHaveBeenCalledTimes(1)

      await silentLogin()

      expect(mockInvoke).toHaveBeenCalledTimes(1)
    })

    it('should NOT login if currently loading', async () => {
      let resolveLogin: () => void
      mockInvoke.mockImplementation(() => new Promise((resolve) => {
        resolveLogin = resolve
      }))

      const { silentLogin } = useAuth()
      const firstLogin = silentLogin()

      await silentLogin()

      expect(mockInvoke).toHaveBeenCalledTimes(1)

      resolveLogin!()
      await firstLogin
    })
  })

  describe('login', () => {
    it('should call invoke with provided credentials', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const { login } = useAuth()
      await login('testuser', 'testpass')

      expect(mockInvoke).toHaveBeenCalledWith('login', {
        username: 'testuser',
        password: 'testpass',
      })
    })

    it('should return true on success', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const { login } = useAuth()
      const result = await login('user', 'pass')

      expect(result).toBe(true)
    })

    it('should return false on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid credentials'))

      const { login } = useAuth()
      const result = await login('user', 'wrong')

      expect(result).toBe(false)
    })

    it('should set authError on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid credentials'))

      const { login, authError } = useAuth()
      await login('user', 'wrong')

      expect(authError.value).toBe('Error: Invalid credentials')
    })

    it('should clear authError on success', async () => {
      const { login, authError, silentLogin } = useAuth()

      mockInvoke.mockRejectedValueOnce(new Error('Failed'))
      await silentLogin()
      expect(authError.value).toBeTruthy()

      mockInvoke.mockResolvedValueOnce(undefined)
      await login('user', 'pass')

      expect(authError.value).toBe('')
    })

    it('should set isReady to true on success', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const { login, isReady } = useAuth()
      await login('user', 'pass')

      expect(isReady.value).toBe(true)
    })
  })

  describe('shared state', () => {
    it('should share state across multiple useAuth calls', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const auth1 = useAuth()
      const auth2 = useAuth()

      await auth1.silentLogin()

      expect(auth2.isReady.value).toBe(true)
    })

    it('should reflect authError across instances', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Failed'))

      const auth1 = useAuth()
      const auth2 = useAuth()

      await auth1.silentLogin()

      expect(auth2.authError.value).toBe('Error: Failed')
    })
  })
})
