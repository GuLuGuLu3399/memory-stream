// 用途：Frontmatter 编辑器组合式函数，序列化/反序列化 YAML 元数据
import { ref, watch } from 'vue'
import type { CardMeta } from '@memory-stream/types'

export function useFrontmatter(getMeta: () => CardMeta | null) {
  const isYamlOpen = ref(false)
  const yamlDraft = ref('')

  function serializeYaml(meta: CardMeta | null): string {
    if (!meta) return ''
    return [
      `uuid: ${meta.uuid}`,
      `title: ${meta.title}`,
      `category: ${meta.category}`,
      `created_at: ${meta.created_at}`,
      `updated_at: ${meta.updated_at}`,
    ].join('\n')
  }

  function parseYamlDraft(text: string): Record<string, string> {
    const result: Record<string, string> = {}
    for (const line of text.split(/\r?\n/)) {
      const trimmed = line.trim()
      if (!trimmed || trimmed === '---' || trimmed.startsWith('#')) continue
      const colonIndex = trimmed.indexOf(':')
      if (colonIndex <= 0) continue
      const key = trimmed.slice(0, colonIndex).trim()
      let value = trimmed.slice(colonIndex + 1).trim()
      if ((value.startsWith('"') && value.endsWith('"')) || (value.startsWith("'") && value.endsWith("'"))) {
        value = value.slice(1, -1)
      }
      result[key] = value
    }
    return result
  }

  function syncDraft() {
    if (!isYamlOpen.value) {
      yamlDraft.value = serializeYaml(getMeta())
    }
  }

  function togglePanel() {
    yamlDraft.value = serializeYaml(getMeta())
    isYamlOpen.value = !isYamlOpen.value
  }

  function resetDraft() {
    yamlDraft.value = serializeYaml(getMeta())
  }

  watch(getMeta, () => syncDraft(), { immediate: true })

  return {
    isYamlOpen,
    yamlDraft,
    parseYamlDraft,
    togglePanel,
    resetDraft,
    syncDraft,
  }
}
