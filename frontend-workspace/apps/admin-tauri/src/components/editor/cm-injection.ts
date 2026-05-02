// 用途：CodeMirror 依赖注入键，提供 EditorView 的 Vue provide/inject 类型
import type { InjectionKey, ShallowRef } from 'vue'
import type { EditorView } from '@codemirror/view'

export const EditorViewKey: InjectionKey<ShallowRef<EditorView | null>> = Symbol('cm-editor-view')
