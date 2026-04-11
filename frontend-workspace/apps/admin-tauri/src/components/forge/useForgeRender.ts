/**
 * useForgeRender — Composable for debounced preview rendering
 *
 * Features:
 * - Debounced markdown rendering (300ms)
 * - Uses Tauri IPC process_markdown command
 * - Error handling
 * - Loading state tracking
 */

import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export function useForgeRender(rawMd: () => string) {
  const html = ref('')
  const renderError = ref('')
  const isRendering = ref(false)

  let debounceTimer: ReturnType<typeof setTimeout> | null = null

  async function renderPreview(content: string) {
    if (!content.trim()) {
      html.value = ''
      renderError.value = ''
      return
    }

    isRendering.value = true
    try {
      const result = await invoke<{ html: string; ast_json: string }>(
        'process_markdown',
        { content }
      )
      renderError.value = ''
      html.value = result.html
    } catch (e) {
      console.error('[useForgeRender] render failed:', e)
      renderError.value = String(e)
      html.value = ''
    } finally {
      isRendering.value = false
    }
  }

  function scheduleRender() {
    if (debounceTimer) {
      clearTimeout(debounceTimer)
    }
    debounceTimer = setTimeout(() => {
      renderPreview(rawMd())
    }, 300)
  }

  function triggerRender() {
    if (debounceTimer) {
      clearTimeout(debounceTimer)
    }
    renderPreview(rawMd())
  }

  // Watch for content changes
  watch(rawMd, () => {
    scheduleRender()
  })

  return {
    html,
    renderError,
    isRendering,
    triggerRender,
  }
}
