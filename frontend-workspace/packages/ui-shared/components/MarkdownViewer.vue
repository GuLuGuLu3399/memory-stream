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
            langs: [
                // 核心语言
                'rust', 'go', 'javascript', 'typescript', 'vue', 'html', 'css', 'json', 'bash', 'markdown',
                // 扩展语言
                'python', 'java', 'c', 'cpp', 'sql', 'yaml', 'toml', 'shell', 'diff',
                'dockerfile', 'protobuf', 'regex', 'xml', 'lua', 'nix', 'ini',
            ]
        });
    }
    return highlighter;
}

// 代码块语言别名映射（用户常用名 → Shiki 内部名）
const LANG_ALIASES: Record<string, string> = {
    js: 'javascript',
    ts: 'typescript',
    py: 'python',
    sh: 'shell',
    shell: 'bash',
    yml: 'yaml',
    rb: 'ruby',
    'c++': 'cpp',
    golang: 'go',
    makefile: 'make',
    docker: 'dockerfile',
    proto: 'protobuf',
};

function resolveLang(raw: string): string {
    const lower = raw.toLowerCase();
    return LANG_ALIASES[lower] ?? lower;
}

// 解析代码块元数据中的行高亮范围，如 `{1,3-5}` → Set(1,3,4,5)
function parseHighlightLines(meta: string): Set<number> {
    const lines = new Set<number>();
    const match = meta.match(/\{([^}]+)\}/);
    if (!match) return lines;
    for (const part of match[1].split(',')) {
        const trimmed = part.trim();
        const range = trimmed.match(/^(\d+)-(\d+)$/);
        if (range) {
            const start = parseInt(range[1], 10);
            const end = parseInt(range[2], 10);
            for (let i = start; i <= end; i++) lines.add(i);
        } else {
            const n = parseInt(trimmed, 10);
            if (!isNaN(n)) lines.add(n);
        }
    }
    return lines;
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
                const match = codeEl.className.match(/language-([a-zA-Z0-9+]+)/);
                const rawLang = match ? match[1] : '';
                let lang = rawLang ? resolveLang(rawLang) : 'text';

                if (rawLang.toLowerCase() === 'mermaid') return;

                // 提取 data-meta 中的行高亮信息
                const meta = codeEl.getAttribute('data-meta') || '';
                const highlightLines = parseHighlightLines(meta);

                if (!loadedLangs.includes(lang)) {
                    lang = 'text';
                }

                const code = codeEl.textContent || '';
                try {
                    let highlighted = shiki.codeToHtml(code, {
                        lang,
                        theme: 'vitesse-dark'
                    });

                    // 后处理：为每一行添加行号和高亮标记
                    let lineNum = 0;
                    highlighted = highlighted.replace(
                        /(<span class="line")/g,
                        () => {
                            lineNum++;
                            const isHighlighted = highlightLines.has(lineNum);
                            return `<span class="line${isHighlighted ? ' highlighted' : ''}" data-line="${lineNum}"`;
                        }
                    );

                    const hasLineNumbers = meta.includes('showLineNumbers') || meta.includes('ln');
                    const wrapperClass = `code-block-wrapper${hasLineNumbers ? ' show-line-numbers' : ''}`;
                    const label = rawLang ? rawLang.toLowerCase() : '';
                    const wrapped = `<div class="${wrapperClass}" data-lang="${label}">
                        <div class="code-block-header">
                            <span class="code-block-lang">${label}</span>
                            <button class="code-copy-btn" title="复制代码" data-code="">复制</button>
                        </div>
                        ${highlighted}
                    </div>`;
                    if (codeEl.parentElement) {
                        codeEl.parentElement.outerHTML = wrapped;
                    }
                } catch (err) {
                    console.error(`Shiki 渲染 [${lang}] 时崩溃:`, err);
                }
            });

            // 设置复制按钮的 data-code 属性（从高亮后的 code 元素提取纯文本）
            container.querySelectorAll('.code-block-wrapper').forEach((wrapper) => {
                const codeEl = wrapper.querySelector('code') || wrapper.querySelector('pre');
                const btn = wrapper.querySelector('.code-copy-btn');
                if (codeEl && btn) {
                    btn.setAttribute('data-code', codeEl.textContent || '');
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

// ========== 锚点跳转拦截 & 复制按钮 ==========
async function handleViewerClick(e: MouseEvent) {
    // 复制按钮处理
    const copyBtn = (e.target as HTMLElement).closest('.code-copy-btn');
    if (copyBtn) {
        const code = (copyBtn as HTMLElement).getAttribute('data-code') || '';
        try {
            await navigator.clipboard.writeText(code);
            copyBtn.textContent = '已复制';
            copyBtn.classList.add('copied');
            setTimeout(() => {
                copyBtn.textContent = '复制';
                copyBtn.classList.remove('copied');
            }, 2000);
        } catch {
            copyBtn.textContent = '失败';
            setTimeout(() => { copyBtn.textContent = '复制'; }, 2000);
        }
        return;
    }

    // 锚点跳转处理
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

/* KaTeX: 隐藏 CSS 视觉层，仅保留 MathML 语义渲染 */
.markdown-body .katex-html {
    display: none !important;
}
/* 行内公式：MathML 必须保持 inline，否则会破坏文本流 */
.markdown-body .katex-mathml {
    display: inline !important;
    overflow: visible !important;
    height: auto !important;
    width: auto !important;
    position: static !important;
    clip: auto !important;
}
/* 块级公式：独立居中显示 */
.markdown-body .katex-display .katex-mathml {
    display: block !important;
    text-align: center !important;
}
.markdown-body .katex {
    font-size: 1.1em;
}
/* 确保行内公式不产生额外间距 */
.markdown-body .math-inline .katex {
    display: inline;
    margin: 0;
    padding: 0;
    vertical-align: baseline;
}

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
.code-block-wrapper {
    @apply relative my-6 rounded-xl border border-zinc-800 bg-[#121212] shadow-2xl overflow-hidden;
}

.code-block-wrapper pre.shiki {
    @apply p-4 text-sm overflow-x-auto;
    margin: 0;
    border: none;
    border-radius: 0;
    background: transparent !important;
    box-shadow: none;
}

.code-block-header {
    @apply flex items-center justify-between px-4 py-1.5 border-b border-zinc-800/60 bg-zinc-900/40;
}

.code-block-lang {
    @apply text-xs font-medium text-zinc-500 tracking-wide uppercase select-none;
    font-family: 'JetBrains Mono', 'Fira Code', Consolas, monospace;
}

.code-copy-btn {
    @apply text-xs text-zinc-500 hover:text-zinc-300 px-2 py-0.5 rounded transition-colors cursor-pointer bg-transparent border-none;
    font-family: system-ui, sans-serif;
}

.code-copy-btn:hover {
    @apply bg-zinc-800;
}

.code-copy-btn.copied {
    @apply text-emerald-400;
}

.prose pre.shiki {
    @apply relative p-4 rounded-xl border border-zinc-800 bg-[#121212] text-sm shadow-2xl overflow-x-auto;
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
    background: rgba(166, 38, 38, 0.12);
    color: #f0b429;
    padding: 2px 6px;
    margin: 0 2px;
    border-radius: 3px;
    border: 1px solid rgba(201, 168, 76, 0.15);
    font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
    font-size: 0.9em;
    font-weight: 500;
    box-shadow: 1px 1px 0 0 rgba(0, 0, 0, 0.3);
}

/* 行高亮 */
.code-block-wrapper .line.highlighted {
    @apply bg-indigo-500/10 border-l-2 border-indigo-400 -ml-1 pl-1;
    display: inline-block;
    width: calc(100% + 8px);
    margin-left: -8px;
    padding-left: 7px;
}

/* 行号 */
.code-block-wrapper.show-line-numbers pre {
    counter-reset: line;
}

.code-block-wrapper.show-line-numbers .line::before {
    counter-increment: line;
    content: counter(line);
    @apply inline-block w-8 mr-4 text-right text-zinc-600 select-none;
    font-variant-numeric: tabular-nums;
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

/* ==================== 脚注 ==================== */
.prose sup.footnote-ref a {
    @apply text-indigo-400 no-underline text-xs font-medium;
}

.prose sup.footnote-ref a:hover {
    @apply text-indigo-300 underline;
}

.prose section.footnote-def {
    @apply my-4 p-4 border border-zinc-800 rounded-lg bg-zinc-900/30 text-sm text-zinc-400;
}

.prose section.footnote-def .footnote-back-ref {
    @apply float-right my-0;
}

.prose section.footnote-def .footnote-back-ref a {
    @apply text-zinc-500 no-underline text-xs;
}

.prose section.footnote-def .footnote-back-ref a:hover {
    @apply text-zinc-300;
}

/* ==================== 定义列表 ==================== */
.prose dl {
    @apply my-6 border-l-4 border-zinc-700 pl-6;
}

.prose dt {
    @apply font-semibold text-zinc-200 mt-4;
}

.prose dt:first-child {
    @apply mt-0;
}

.prose dd {
    @apply text-zinc-400 mt-1 ml-0;
}
</style>
