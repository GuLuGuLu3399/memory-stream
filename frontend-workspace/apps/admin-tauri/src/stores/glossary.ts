// 用途：术语表状态管理，加载和维护概念定义映射
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { readGlossary } from '@/bridge/invoke'

export const useGlossaryStore = defineStore('glossary', () => {
  const terms = ref<Record<string, string>>({})
  const loading = ref(false)

  async function load() {
    loading.value = true
    try {
      terms.value = await readGlossary()
    } catch {
      terms.value = {}
    } finally {
      loading.value = false
    }
  }

  function lookup(term: string): string | undefined {
    return terms.value[term]
  }

  return { terms, loading, load, lookup }
})
