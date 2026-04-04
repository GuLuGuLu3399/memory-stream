<script setup lang="ts">
import { ref, watch } from 'vue';
import type { ParseEngine, RenderResult } from '../types';
import MarkdownViewer from './MarkdownViewer.vue';

// ========== Props & Emits ==========
const props = defineProps<{
    parseEngine: ParseEngine;
    initialValue?: string;
}>();

const emit = defineEmits<{
    (e: 'save', rawMd: string, astJson: string): void;
    (e: 'rendered', result: RenderResult): void;
}>();

// ========== State ==========
const markdownInput = ref(props.initialValue || `# 🎨 富文本演示

## 代码高亮 (Shiki)

\`\`\`rust
fn main() {
    println!("Hello, Memory Stream!");
}
\`\`\`

## 数学公式 (KaTeX)

行内公式: $E = mc^2$

块级公式:
$$
\\int_{-\\infty}^{\\infty} e^{-x^2} dx = \\sqrt{\\pi}
$$

## 流程图 (Mermaid)

\`\`\`mermaid
graph LR
    A[Vue 前端] --> B[Rust 引擎]
    B --> C[WASM]
    B --> D[Tauri]
    C --> E[Web 端]
    D --> F[桌面端]
\`\`\`

---
开始编辑，体验实时渲染！支持粘贴图片自动压缩！
`);
const previewHtml = ref('');
const currentAstJson = ref('');
const isRendering = ref(false);
const isUploading = ref(false);

// ========== Textarea DOM 引用 ==========
const textareaRef = ref<HTMLTextAreaElement | null>(null);

// ========== 防抖工具函数 ==========
function useDebounce<T extends (...args: any[]) => any>(fn: T, delay: number): T {
    let timeoutId: ReturnType<typeof setTimeout> | null = null;
    return ((...args: any[]) => {
        if (timeoutId) clearTimeout(timeoutId);
        timeoutId = setTimeout(() => fn(...args), delay);
    }) as T;
}

// ========== 渲染逻辑 ==========
const renderMarkdown = useDebounce(async (text: string) => {
    if (!text.trim()) {
        previewHtml.value = '';
        currentAstJson.value = '';
        return;
    }

    isRendering.value = true;
    try {
        const res = await props.parseEngine(text);
        previewHtml.value = res.html;
        currentAstJson.value = res.ast_json;
        emit('rendered', res);
        // 🌟 富文本渲染已由 MarkdownViewer 组件内部处理
    } catch (error) {
        console.error('引擎报错:', error);
        previewHtml.value = `<p style="color: red;">渲染崩溃: ${error}</p>`;
    } finally {
        isRendering.value = false;
    }
}, 200);

// ========== 光标处插入文本 ==========
function insertTextAtCursor(textToInsert: string) {
    const el = textareaRef.value;
    if (!el) {
        markdownInput.value += textToInsert;
        return;
    }
    const start = el.selectionStart;
    const end = el.selectionEnd;
    const text = markdownInput.value;

    // 拼接：光标前的内容 + 插入的图片语法 + 光标后的内容
    markdownInput.value = text.slice(0, start) + textToInsert + text.slice(end);

    // 恢复光标位置到插入内容之后
    setTimeout(() => {
        el.selectionStart = el.selectionEnd = start + textToInsert.length;
        el.focus();
    }, 0);
}

// ========== 🌟 图片粘贴拦截器 ==========
async function handlePaste(event: ClipboardEvent) {
    const items = event.clipboardData?.items;
    if (!items) return;

    for (const item of items) {
        // 如果粘贴的是图片 (PNG/JPG/GIF 等)
        if (item.type.startsWith('image/')) {
            event.preventDefault(); // 阻止浏览器默认行为

            const file = item.getAsFile();
            if (!file) continue;

            isUploading.value = true;

            try {
                // 1. 将前端 File 对象转为底层字节数组
                const arrayBuffer = await file.arrayBuffer();
                const rawBytes = Array.from(new Uint8Array(arrayBuffer));

                // 2. 检测是否有 Tauri 环境（桌面端）
                const hasTauri = typeof window !== 'undefined' && '__TAURI__' in window;

                let imageUrl: string;

                if (hasTauri) {
                    // 🚀 桌面端：调用 Rust 进行极致压缩
                    const { invoke } = await import('@tauri-apps/api/core');

                    console.time("Rust WebP 压缩耗时");
                    const webpBytes = await invoke<number[]>('compress_image_to_webp', { rawBytes });
                    console.timeEnd("Rust WebP 压缩耗时");

                    // 使用压缩后的 WebP 创建本地预览 URL
                    const blob = new Blob([new Uint8Array(webpBytes)], { type: 'image/webp' });
                    imageUrl = URL.createObjectURL(blob);

                    console.log(`✅ 图片压缩完成: ${(file.size / 1024).toFixed(1)}KB → ${(blob.size / 1024).toFixed(1)}KB`);
                } else {
                    // 🌐 Web 端：直接使用原图（未来可接入 OSS）
                    imageUrl = URL.createObjectURL(file);
                    console.log('ℹ️ Web 端暂不支持压缩，使用原图');
                }

                // 3. 组装 Markdown 语法并精准插入光标位置
                const mdImageSyntax = `\n![图片](${imageUrl})\n`;
                insertTextAtCursor(mdImageSyntax);

            } catch (error) {
                console.error("图片处理流水线崩溃:", error);
                alert(`图片处理失败: ${error}`);
            } finally {
                isUploading.value = false;
            }

            break; // 只处理第一张图片
        }
    }
}

// ========== 监听输入 ==========
watch(markdownInput, (newText) => {
    renderMarkdown(newText);
}, { immediate: true });

// ========== 触发保存 ==========
function triggerSave() {
    if (!currentAstJson.value) return;
    emit('save', markdownInput.value, currentAstJson.value);
}

// ========== 暴露方法 ==========
defineExpose({
    getMarkdown: () => markdownInput.value,
    getAstJson: () => currentAstJson.value,
});
</script>

<template>
    <div class="flex flex-col h-screen w-screen">
        <!-- 顶部工具栏 -->
        <div class="flex justify-between items-center px-5 py-2.5 bg-ms-darker text-white">
            <span class="font-bold text-base">📝 Memory Stream Editor</span>
            <div class="flex items-center gap-3">
                <span v-if="isUploading" class="text-xs text-yellow-400 animate-pulse">📷 图片处理中...</span>
                <span v-else-if="isRendering" class="text-xs text-gray-400 animate-pulse">渲染中...</span>
                <slot name="actions" :save="triggerSave" :hasContent="!!currentAstJson">
                    <button
                        class="px-4 py-2 bg-ms-primary text-white border-none rounded cursor-pointer font-bold transition-colors duration-200 hover:bg-ms-primary-hover disabled:bg-gray-500 disabled:cursor-not-allowed"
                        @click="triggerSave" :disabled="!currentAstJson">
                        ☁️ 保存
                    </button>
                </slot>
            </div>
        </div>

        <!-- 主体区域 -->
        <div class="flex flex-1 overflow-hidden">
            <!-- 编辑器 -->
            <textarea ref="textareaRef"
                class="flex-1 p-5 bg-ms-dark text-gray-300 border-none font-mono text-sm leading-relaxed outline-none resize-none placeholder-gray-500"
                v-model="markdownInput" @paste="handlePaste" placeholder="输入 Markdown... (支持粘贴图片)"></textarea>
            <!-- 预览区：使用 MarkdownViewer 组件（暗色背景，与 prose-invert 统一） -->
            <div class="flex-1 p-5 bg-ms-dark text-gray-300 overflow-y-auto leading-relaxed">
                <MarkdownViewer :html-content="previewHtml" />
            </div>
        </div>
    </div>
</template>