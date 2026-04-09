<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted } from 'vue';

/**
 * MarkdownViewer - 跨端统一渲染基座
 *
 * 纯展示组件，接收 HTML 字符串，负责：
 * 1. KaTeX 数学公式渲染 (动态按需加载)
 * 2. Shiki 代码高亮 (动态按需加载)
 * 3. Mermaid 流程图渲染 (动态按需加载, 暗色主题 + 可读字号)
 * 4. 图片缩放 (medium-zoom, 神殿黑背景)
 * 5. 锚点跳转拦截 (解决 overflow 容器内 #hash 跳转失效)
 */

const props = defineProps<{
    htmlContent: string;
}>();

const viewerRef = ref<HTMLElement | null>(null);

// ========== medium-zoom 实例 ==========
let zoomInstance: ReturnType<typeof import('medium-zoom').default> | null = null;

// ========== Shiki 高亮器（懒加载单例） ==========
let highlighter: import('shiki').Highlighter | null = null;

// ========== Mermaid 初始化标记 ==========
let mermaidInitialized = false;

// ========== 渲染竞态保护 ==========
let renderId = 0;

async function getShikiHighlighter() {
    if (!highlighter) {
        const { createHighlighter } = await import('shiki');
        highlighter = await createHighlighter({
            themes: ['vitesse-dark', 'vitesse-light'],
            langs: ['rust', 'go', 'javascript', 'typescript', 'vue', 'html', 'css', 'json', 'bash', 'markdown']
        });
    }
    return highlighter;
}

// ========== 富文本后处理钩子 ==========
async function applyRichTextRendering(container: HTMLElement) {
    // 1. KaTeX 数学公式渲染
    const hasMath = container.querySelector('.math-inline, .math-block');
    const hasMathDelimiters = container.innerHTML.includes('\\(') || container.innerHTML.includes('\\[');
    if (hasMath || hasMathDelimiters) {
        try {
            const { default: renderMathInElement } = await import('katex/dist/contrib/auto-render.mjs');
            renderMathInElement(container, {
                delimiters: [
                    { left: '\\[', right: '\\]', display: true },
                    { left: '\\(', right: '\\)', display: false }
                ],
                throwOnError: false
            });
        } catch (e) {
            console.warn('KaTeX 渲染警告:', e);
        }
    }

    // 2. Shiki 代码高亮
    const codeBlocks = container.querySelectorAll('pre > code');
    if (codeBlocks.length > 0) {
        try {
            const shiki = await getShikiHighlighter();
            const loadedLangs = shiki.getLoadedLanguages();

            codeBlocks.forEach((el) => {
                const codeEl = el as HTMLElement;
                const match = codeEl.className.match(/language-([a-zA-Z0-9]+)/);
                let lang = match ? match[1].toLowerCase() : 'text';

                if (lang === 'mermaid') return;

                if (!loadedLangs.includes(lang)) {
                    lang = 'text';
                }

                const code = codeEl.textContent || '';
                try {
                    const html = shiki.codeToHtml(code, {
                        lang,
                        theme: 'vitesse-dark'
                    });
                    if (codeEl.parentElement) {
                        codeEl.parentElement.outerHTML = html;
                    }
                } catch (err) {
                    console.error(`Shiki 渲染 [${lang}] 时崩溃:`, err);
                }
            });
        } catch (e) {
            console.warn('Shiki 渲染警告:', e);
        }
    }

    // 3. Mermaid 流程图渲染（仅初始化一次配置）
    const mermaidNodes = Array.from(container.querySelectorAll('.mermaid')) as HTMLElement[];
    if (mermaidNodes.length > 0) {
        try {
            const { default: mermaid } = await import('mermaid');
            if (!mermaidInitialized) {
                mermaid.initialize({
                    startOnLoad: false,
                    theme: 'dark',
                    securityLevel: 'loose',
                    themeVariables: {
                        fontSize: '16px',
                        fontFamily: '"JetBrains Mono", "Fira Code", Consolas, monospace'
                    }
                });
                mermaidInitialized = true;
            }
            mermaidNodes.forEach(n => n.removeAttribute('data-processed'));
            await mermaid.run({ nodes: mermaidNodes });
        } catch (e) {
            console.warn('Mermaid 渲染警告:', e);
        }
    }

    // 4. 图片缩放 (medium-zoom)
    try {
        const mediumZoom = (await import('medium-zoom')).default;
        if (zoomInstance) {
            zoomInstance.detach();
            zoomInstance = null;
        }
        zoomInstance = mediumZoom(container.querySelectorAll('img'), {
            background: 'rgba(10, 10, 10, 0.95)',
            margin: 24
        });
    } catch (e) {
        console.warn('medium-zoom 初始化警告:', e);
    }
}

// ========== 锚点跳转拦截 ==========
function handleViewerClick(e: MouseEvent) {
    const target = (e.target as HTMLElement).closest('a');
    if (!target) return;

    const href = target.getAttribute('href');
    if (!href || !href.startsWith('#')) return;

    e.preventDefault();

    let id: string;
    try {
        id = decodeURIComponent(href.slice(1));
    } catch {
        return; // 无效编码，忽略
    }
    if (!id || !viewerRef.value) return;

    const element = viewerRef.value.querySelector(`[id="${CSS.escape(id)}"]`) as HTMLElement | null;
    if (element) {
        element.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
}

// ========== 渲染管线（带竞态保护） ==========
async function triggerRender() {
    const currentId = ++renderId;
    await nextTick();
    const container = viewerRef.value;
    if (!container || currentId !== renderId) return;
    await applyRichTextRendering(container);
}

onMounted(triggerRender);

watch(() => props.htmlContent, triggerRender);

onUnmounted(() => {
    if (zoomInstance) zoomInstance.detach();
    highlighter = null;
});
</script>

<template>
    <div ref="viewerRef" class="markdown-body prose prose-invert max-w-none" v-html="htmlContent"
        @click="handleViewerClick" />
</template>

<style>
/* NOTE: <style> intentionally unscoped — v-html content does not receive Vue's scoped
   data-v- attributes, so scoped selectors would not match rendered markdown elements.
   All styles are class-scoped via .markdown-body / .prose to prevent global leakage. */

/* KaTeX MathML 辅助文本（屏幕阅读器用，视觉上已由 KaTeX 自身隐藏） */

/* ==================== 排版基石 ==================== */
.markdown-body {
    font-size: 1.05rem;
    line-height: 1.8;
}

.prose p {
    @apply my-5;
}

.prose h1,
.prose h2,
.prose h3,
.prose h4 {
    @apply text-zinc-100 font-semibold tracking-tight;
    margin-top: 2em;
    margin-bottom: 0.8em;
    scroll-margin-top: 80px;
}

.prose h1 {
    @apply text-3xl border-b border-zinc-800 pb-3;
}

.prose h2 {
    @apply text-2xl border-b border-zinc-800/50 pb-2;
}

.prose h3 {
    @apply text-xl;
}

.prose a {
    @apply text-indigo-400 no-underline border-b border-indigo-400/30 transition-colors;
}

.prose a:hover {
    @apply border-indigo-400 text-indigo-300;
}

/* ==================== 代码块 ==================== */
.prose pre.shiki {
    @apply relative p-4 rounded-xl border border-zinc-800 bg-[#121212] my-6 text-sm shadow-2xl overflow-x-auto;
    font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

.prose pre.shiki::-webkit-scrollbar {
    height: 8px;
}

.prose pre.shiki::-webkit-scrollbar-track {
    @apply bg-transparent;
}

.prose pre.shiki::-webkit-scrollbar-thumb {
    @apply bg-zinc-700/50 rounded-full;
}

.prose pre.shiki::-webkit-scrollbar-thumb:hover {
    @apply bg-zinc-600;
}

.prose code:not(pre code) {
    @apply bg-zinc-800/70 text-indigo-300 px-1.5 py-0.5 rounded text-sm font-medium;
}

/* ==================== 引用块 ==================== */
.prose blockquote {
    @apply relative my-6 pl-5 pr-4 py-3 border-l-4 border-indigo-500 bg-indigo-500/10 rounded-r-lg;
    font-style: normal;
    color: #d4d4d8;
}

.prose blockquote p {
    @apply my-1;
}

/* ==================== 表格与复选框 ==================== */
.prose table {
    @apply w-full text-left border-collapse my-8 text-sm;
}

.prose th {
    @apply bg-zinc-800/50 p-3 font-medium text-zinc-200 border-b border-zinc-700;
}

.prose td {
    @apply p-3 border-b border-zinc-800/50 text-zinc-400;
}

.prose tr:hover td {
    @apply bg-zinc-800/30 transition-colors;
}

.prose input[type="checkbox"] {
    @apply appearance-none w-4 h-4 border border-zinc-600 rounded-sm bg-zinc-900 align-middle mr-2 focus:ring-0 cursor-pointer;
}

.prose input[type="checkbox"]:checked {
    @apply bg-indigo-500 border-indigo-500;
}

/* ==================== 列表 ==================== */
.prose ul,
.prose ol {
    @apply my-4 pl-6;
}

.prose li {
    @apply my-1;
}

.prose ul li {
    @apply list-disc;
}

.prose ol li {
    @apply list-decimal;
}

/* ==================== Mermaid 图表可读性 ==================== */
.markdown-body .mermaid {
    overflow-x: auto;
    text-align: center;
    padding: 1rem 0;
}

.markdown-body .mermaid svg {
    min-width: 600px;
    max-width: 100%;
    height: auto;
}

/* ==================== 图片缩放 ==================== */
.prose img {
    @apply rounded-lg shadow-lg my-6;
    cursor: zoom-in;
    transition: transform 0.3s cubic-bezier(0.2, 0, 0.2, 1) !important;
}

.medium-zoom-image--opened {
    cursor: zoom-out;
}

/* ==================== 分割线 ==================== */
.prose hr {
    @apply border-zinc-800 my-8;
}
</style>
