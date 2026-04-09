// @vitest-environment jsdom
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useSysConfigStore, type SysConfig } from '../sysconfig'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'

const mockInvoke = vi.mocked(invoke)

const defaultConfig: SysConfig = {
  api_base_url: 'http://localhost:8080/api/v1',
  ws_url: 'ws://localhost:8080/api/v1/ws',
  s3_endpoint: '',
  s3_region: 'us-east-1',
  s3_bucket: '',
  s3_access_key: '',
  s3_secret_key: '',
  s3_public_url_base: '',
  s3_use_path_style: false,
}

const validConfig: SysConfig = {
  api_base_url: 'https://api.example.com/api/v1',
  ws_url: 'wss://api.example.com/api/v1/ws',
  s3_endpoint: 'https://s3.example.com',
  s3_region: 'us-west-2',
  s3_bucket: 'my-bucket',
  s3_access_key: 'access123',
  s3_secret_key: 'secret456',
  s3_public_url_base: 'https://cdn.example.com',
  s3_use_path_style: false,
}

describe('useSysConfigStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('should have null config initially', () => {
      const store = useSysConfigStore()

      expect(store.config).toBeNull()
      expect(store.loading).toBe(false)
      expect(store.error).toBeNull()
      expect(store.connectionStatus).toBe('idle')
    })

    it('should compute isConfigured as false when config is null', () => {
      const store = useSysConfigStore()

      expect(store.isConfigured).toBe(false)
    })
  })

  describe('loadConfig', () => {
    it('should load config from backend successfully', async () => {
      mockInvoke.mockResolvedValueOnce(validConfig)

      const store = useSysConfigStore()
      await store.loadConfig()

      expect(mockInvoke).toHaveBeenCalledWith('get_sys_config')
      expect(store.config).toEqual(validConfig)
      expect(store.loading).toBe(false)
      expect(store.error).toBeNull()
    })

    it('should set loading state during load', async () => {
      let resolveLoad: (value: SysConfig) => void
      mockInvoke.mockImplementation(() => new Promise((resolve) => {
        resolveLoad = resolve
      }))

      const store = useSysConfigStore()
      const loadPromise = store.loadConfig()

      expect(store.loading).toBe(true)

      resolveLoad!(validConfig)
      await loadPromise

      expect(store.loading).toBe(false)
    })

    it('should fallback to default config when invoke fails', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Config not found'))

      const store = useSysConfigStore()
      await store.loadConfig()

      expect(store.config).toEqual(defaultConfig)
      expect(store.loading).toBe(false)
    })

    it('should compute isConfigured as true after loading', async () => {
      mockInvoke.mockResolvedValueOnce(validConfig)

      const store = useSysConfigStore()
      await store.loadConfig()

      expect(store.isConfigured).toBe(true)
    })
  })

  describe('saveConfig', () => {
    it('should save config successfully', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useSysConfigStore()
      await store.saveConfig(validConfig)

      expect(mockInvoke).toHaveBeenCalledWith('save_sys_config', { config: validConfig })
      expect(store.config).toEqual(validConfig)
      expect(store.loading).toBe(false)
      expect(store.error).toBeNull()
    })

    it('should set error on save failure', async () => {
      const error = new Error('Save failed')
      mockInvoke.mockRejectedValueOnce(error)

      const store = useSysConfigStore()

      await expect(store.saveConfig(validConfig)).rejects.toThrow('Save failed')
      expect(store.error).toBe('Save failed')
    })

    it('should set loading state during save', async () => {
      let resolveSave: (value?: unknown) => void
      mockInvoke.mockImplementation(() => new Promise((resolve) => {
        resolveSave = resolve
      }))

      const store = useSysConfigStore()
      const savePromise = store.saveConfig(validConfig)

      expect(store.loading).toBe(true)

      resolveSave!()
      await savePromise

      expect(store.loading).toBe(false)
    })

    it('should handle non-Error thrown values', async () => {
      mockInvoke.mockRejectedValueOnce('String error')

      const store = useSysConfigStore()

      await expect(store.saveConfig(validConfig)).rejects.toThrow()
      expect(store.error).toBe('String error')
    })
  })

  describe('testConnection', () => {
    it('should return true and set status to ok when both pass', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useSysConfigStore()
      const result = await store.testConnection()

      expect(result).toBe(true)
      expect(store.connectionStatus).toBe('ok')
      expect(mockInvoke).toHaveBeenCalledWith('test_api_connection')
      expect(mockInvoke).toHaveBeenCalledWith('test_s3_connection')
    })

    it('should return false when API connection fails', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('API unreachable'))
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useSysConfigStore()
      const result = await store.testConnection()

      expect(result).toBe(false)
      expect(store.connectionStatus).toBe('failed')
    })

    it('should return false when S3 connection fails', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockRejectedValueOnce(new Error('S3 connection failed'))

      const store = useSysConfigStore()
      const result = await store.testConnection()

      expect(result).toBe(false)
      expect(store.connectionStatus).toBe('failed')
      expect(store.error).toBe('S3 connection failed')
    })

    it('should set status to testing during test', async () => {
      let resolveApi!: () => void
      mockInvoke
        .mockImplementationOnce(() => new Promise<void>((resolve) => { resolveApi = resolve }))
        .mockResolvedValueOnce(undefined)

      const store = useSysConfigStore()
      const testPromise = store.testConnection()

      expect(store.connectionStatus).toBe('testing')

      resolveApi!()
      await testPromise

      expect(store.connectionStatus).toBe('ok')
    })
  })

  describe('testApiConnection', () => {
    it('should return true on success', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useSysConfigStore()
      const result = await store.testApiConnection()

      expect(result).toBe(true)
      expect(mockInvoke).toHaveBeenCalledWith('test_api_connection')
    })

    it('should return false on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('API unreachable'))

      const store = useSysConfigStore()
      const result = await store.testApiConnection()

      expect(result).toBe(false)
      expect(store.error).toBe('API unreachable')
    })
  })

  describe('testS3Connection', () => {
    it('should return true on success', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useSysConfigStore()
      const result = await store.testS3Connection()

      expect(result).toBe(true)
      expect(mockInvoke).toHaveBeenCalledWith('test_s3_connection')
    })

    it('should return false on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('S3 failed'))

      const store = useSysConfigStore()
      const result = await store.testS3Connection()

      expect(result).toBe(false)
      expect(store.error).toBe('S3 failed')
    })
  })

  describe('reloadConfig', () => {
    it('should reload config from backend', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useSysConfigStore()
      await store.reloadConfig()

      expect(mockInvoke).toHaveBeenCalledWith('reload_sys_config')
    })

    it('should set error on reload failure', async () => {
      const error = new Error('Reload failed')
      mockInvoke.mockRejectedValueOnce(error)

      const store = useSysConfigStore()

      await expect(store.reloadConfig()).rejects.toThrow('Reload failed')
      expect(store.error).toBe('Reload failed')
    })
  })

  describe('state transitions', () => {
    it('should handle full config lifecycle', async () => {
      mockInvoke
        .mockResolvedValueOnce(defaultConfig)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce(undefined)

      const store = useSysConfigStore()

      await store.loadConfig()
      expect(store.config).toEqual(defaultConfig)

      await store.saveConfig(validConfig)
      expect(store.config).toEqual(validConfig)

      const connectionResult = await store.testConnection()
      expect(connectionResult).toBe(true)
      expect(store.connectionStatus).toBe('ok')

      await store.reloadConfig()
    })
  })
})
