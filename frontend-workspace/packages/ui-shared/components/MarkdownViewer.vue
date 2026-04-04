<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';

/**
 * 🌟 MarkdownViewer - 跨端渲染核武器
 * 
 * 纯展示组件，只接收 HTML 字符串，负责：
 * 1. KaTeX 数学公式渲染 (动态按需加载)
 * 2. Shiki 代码高亮 (动态按需加载)
 * 3. Mermaid 流程图渲染 (动态按需加载)
 * 
 * 被 MarkdownEditor 和 web-reader 共同使用
 */

const props = defineProps<{
    htmlContent: string;
}>();

const viewerRef = ref<HTMLElement | null>(null);

// ========== Shiki 高亮器（懒加载单例） ==========
let highlighter: any = null;

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
    // 1. KaTeX 数学公式渲染 (动态按需加载)
    // 🌟 只认 Rust 输出的标准 LaTeX 定界符 \(...\) 和 \[...\]
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

    // 2. Shiki 代码高亮 (动态按需加载)
    // 🌟 工业级健壮版：大小写标准化 + 语言包校验
    const codeBlocks = container.querySelectorAll('pre > code');
    if (codeBlocks.length > 0) {
        try {
            const shiki = await getShikiHighlighter();
            const loadedLangs = shiki.getLoadedLanguages();

            codeBlocks.forEach((el) => {
                const codeEl = el as HTMLElement;
                const match = codeEl.className.match(/language-([a-zA-Z0-9]+)/);
                // 强制转小写，防止大小写不匹配
                let lang = match ? match[1].toLowerCase() : 'text';

                // 跳过 Mermaid 块
                if (lang === 'mermaid') return;

                // 检查 Shiki 是否真的加载了这个语言
                if (!loadedLangs.includes(lang)) {
                    console.warn(`Shiki 没有加载语言 [${lang}]，回退到 text 模式`);
                    lang = 'text';
                }

                const code = codeEl.textContent || '';
                try {
                    const html = shiki.codeToHtml(code, {
                        lang,
                        theme: 'vitesse-dark'
                    });
                    // 替换整个 pre 元素
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

    // 3. Mermaid 流程图渲染 (动态按需加载)
    const mermaidNodes = Array.from(container.querySelectorAll('.mermaid')) as HTMLElement[];
    if (mermaidNodes.length > 0) {
        try {
            const { default: mermaid } = await import('mermaid');
            mermaid.initialize({
                startOnLoad: false,
                theme: 'dark',
                securityLevel: 'loose'
            });
            // 重置已处理状态
            mermaidNodes.forEach(n => n.removeAttribute('data-processed'));
            await mermaid.run({ nodes: mermaidNodes });
        } catch (e) {
            console.warn('Mermaid 渲染警告:', e);
        }
    }
}

// ========== 监听传入的 HTML，触发渲染流水线 ==========
watch(() => props.htmlContent, async () => {
    await nextTick();
    const container = viewerRef.value;
    if (!container) return;
    await applyRichTextRendering(container);
}, { immediate: true });
</script>

<template>
    <div ref="viewerRef" class="prose prose-invert max-w-none" v-html="htmlContent"></div>
</template>

<style>
/* 🚨 核心修复：强制隐藏 KaTeX 的 MathML 辅助文本 */
.katex-html {
    display: none !important;
}

/* ==================== 🎨 视觉附魔：极客美感样式 ==================== */

/* --- 1. 排版基石：重塑 prose 质感 --- */
.prose {
    @apply text-zinc-300 leading-relaxed max-w-none;
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

/* 优雅的超链接 */
.prose a {
    @apply text-indigo-400 no-underline border-b border-indigo-400/30 transition-colors;
}

.prose a:hover {
    @apply border-indigo-400 text-indigo-300;
}

/* --- 2. 代码块的"极客装甲" --- */
.prose pre.shiki {
    @apply relative p-4 rounded-xl border border-zinc-800 bg-[#121212] my-6 text-sm shadow-2xl overflow-x-auto;
    font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

/* 自定义滚动条 */
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

/* 行内代码 */
.prose code:not(pre code) {
    @apply bg-zinc-800/70 text-indigo-300 px-1.5 py-0.5 rounded text-sm font-medium;
}

/* --- 3. 引用块的现代化蜕变 --- */
.prose blockquote {
    @apply relative my-6 pl-5 pr-4 py-3 border-l-4 border-indigo-500 bg-indigo-500/10 rounded-r-lg;
    font-style: normal;
    color: #d4d4d8;
}

.prose blockquote p {
    @apply my-1;
}

/* --- 4. 表格与复选框 --- */
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

/* 任务列表复选框 */
.prose input[type="checkbox"] {
    @apply appearance-none w-4 h-4 border border-zinc-600 rounded-sm bg-zinc-900 align-middle mr-2 focus:ring-0 cursor-pointer;
}

.prose input[type="checkbox"]:checked {
    @apply bg-indigo-500 border-indigo-500;
}

/* --- 5. 列表优化 --- */
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

/* --- 6. 图片优化 --- */
.prose img {
    @apply rounded-lg shadow-lg my-6;
}

/* --- 7. 分割线 --- */
.prose hr {
    @apply border-zinc-800 my-8;
}
</style>
