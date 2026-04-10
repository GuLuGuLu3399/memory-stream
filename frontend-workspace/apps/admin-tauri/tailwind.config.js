/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{vue,js,ts,jsx,tsx}",
        "../../packages/ui-shared/components/**/*.{vue,ts}",
        "../../packages/ui-shared/types/**/*.{vue,ts}",
    ],
    theme: {
        extend: {
            colors: {
                // ── 机械祭坛基底 ──
                'ms-void': '#050505',         // 最深（侧栏/工具栏）
                'ms-deep': '#0d0d0d',         // 主背景
                'ms-carbon': '#141414',       // 内容区
                'ms-panel': '#1a1a1a',        // 面板色
                'ms-surface': '#222222',      // 表面色（hover 态）
                'ms-border': '#1e1e1e',       // 结构线（极暗，明确分隔）
                'ms-border-light': '#2a2a2a', // 高亮边框（交互元素）
                'ms-border-active': '#333333',// 活跃态边框

                // ── 霓虹青高亮系 ──
                'neon': {
                    DEFAULT: '#00e5ff',
                    '50': '#e0faff',
                    '100': '#b3f5ff',
                    '200': '#80eeff',
                    '300': '#4de6ff',
                    '400': '#1adeff',
                    '500': '#00e5ff',
                    '600': '#00b8cc',
                    '700': '#008a99',
                    '800': '#005c66',
                    '900': '#002e33',
                },

                // ── 语义色 ──
                'ms-primary': '#00e5ff',
                'ms-primary-hover': '#1adeff',
                'ms-danger': '#ff4444',
                'ms-warning': '#ffaa00',
                'ms-success': '#00e676',
            },
            zIndex: {
                'base': '0',
                'chrome': '20',
                'drawer': '30',
                'panel': '40',
                'overlay': '50',
                'modal': '60',
                'fullscreen': '70',
                'toast': '80',
                'dropdown': '90',
                'entrance': '100',
                'error': '110',
            },
            fontFamily: {
                mono: ['"JetBrains Mono"', '"Fira Code"', 'Consolas', 'Monaco', 'monospace'],
                display: ['"Space Grotesk"', '"Inter"', 'system-ui', 'sans-serif'],
                body: ['"JetBrains Mono"', '"Fira Code"', 'Consolas', 'Monaco', 'monospace'],
            },
            borderRadius: {
                'sharp': '0px',
                'industrial': '2px',
            },
            fontSize: {
                '2xs': '10px',
                '3xs': '9px',
            },
            letterSpacing: {
                'spine': '0.15em',
            },
            boxShadow: {
                'neon-glow': '0 0 8px rgba(0, 229, 255, 0.3), 0 0 20px rgba(0, 229, 255, 0.1)',
                'neon-glow-sm': '0 0 4px rgba(0, 229, 255, 0.2)',
                'neon-glow-lg': '0 0 12px rgba(0, 229, 255, 0.4), 0 0 30px rgba(0, 229, 255, 0.15)',
            },
            spacing: {
                'titlebar': '36px',
            },
            minWidth: {
                'merge-btn': '280px',
            },
            transitionDuration: {
                'drawer': '200ms',
                'panel': '300ms',
            },
        },
    },
    plugins: [],
}
