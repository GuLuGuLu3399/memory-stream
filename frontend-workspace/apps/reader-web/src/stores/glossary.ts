// ────────────────────────────────────────────────────────────────
// glossary.ts — Preloaded glossary store
// ────────────────────────────────────────────────────────────────

import { defineStore } from 'pinia'
import { ref, shallowRef } from 'vue'
import * as glossaryApi from '@/api/glossary'

export const useGlossaryStore = defineStore('glossary', () => {
  const terms = shallowRef<Map<string, string>>(new Map())
  const loaded = ref(false)

  async function loadGlossary(): Promise<void> {
    if (loaded.value) return
    try {
      const data = await glossaryApi.fetchGlossarySlim()
      terms.value = new Map(Object.entries(data))
    } finally {
      loaded.value = true
    }
  }

  function getDefinition(term: string): string | undefined {
    return terms.value.get(term)
  }

  return { terms, loaded, loadGlossary, getDefinition }
})
