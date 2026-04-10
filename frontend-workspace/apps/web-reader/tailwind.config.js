/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{vue,js,ts,jsx,tsx}",
        "../../packages/ui-shared/*.{vue,ts}",
        "../../packages/ui-shared/components/**/*.{vue,ts}",
        "../../packages/ui-shared/types/**/*.{vue,ts}",
    ],
    theme: {
        extend: {
            colors: {
                // ── 深炭灰基底（与 admin-tauri 统一）──
                "ms-deep": "#0d0d0d",
                "ms-carbon": "#141414",
                "ms-panel": "#222222",
                "ms-surface": "#2a2a2a",
                "ms-border": "#333333",
                "ms-void": "#050505",
                "ms-border-light": "#2a2a2a",
                "ms-border-active": "#333333",

                // ── 霓虹青高亮系 ──
                neon: {
                    DEFAULT: "#00e5ff",
                    50: "#e0faff",
                    100: "#b3f5ff",
                    200: "#80eeff",
                    300: "#4de6ff",
                    400: "#1adeff",
                    500: "#00e5ff",
                    600: "#00b8cc",
                    700: "#008a99",
                    800: "#005c66",
                    900: "#002e33",
                },

                // ── 图谱专属色（保留）──
                "ms-spine": "#6366f1",
                "ms-spine-active": "#818cf8",
                "ms-branch": "#a1a1aa",

                // ── 语义色 ──
                "ms-primary": "#00e5ff",
                "ms-primary-hover": "#1adeff",
                "ms-danger": "#ff4444",
                "ms-warning": "#ffaa00",
                "ms-success": "#00e676",
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
                mono: ['"JetBrains Mono"', '"Fira Code"', "Consolas", "Monaco", "monospace"],
                body: ['"JetBrains Mono"', '"Fira Code"', "Consolas", "Monaco", "monospace"],
                display: ['"Space Grotesk"', '"Inter"', "system-ui", "sans-serif"],
                serif: ['"Noto Serif SC"', '"Playfair Display"', "Georgia", "serif"],
            },
            borderRadius: {
                sharp: '0px',
                industrial: '2px',
            },
            fontSize: {
                '3xs': '9px',
                '2xs': '10px',
                '1.5xs': '11px',
            },
            letterSpacing: {
                'spine': '0.15em',
            },
            boxShadow: {
                'neon-glow': '0 0 8px rgba(0, 229, 255, 0.3), 0 0 20px rgba(0, 229, 255, 0.1)',
                'neon-glow-sm': '0 0 4px rgba(0, 229, 255, 0.2)',
                'neon-glow-lg': '0 0 12px rgba(0, 229, 255, 0.4), 0 0 30px rgba(0, 229, 255, 0.15)',

                // ── White glow (spine nodes) ──
                'white-glow': '0 0 10px rgba(255, 255, 255, 0.6)',
                'white-glow-sm': '0 0 8px rgba(255, 255, 255, 0.3)',
                'white-glow-lg': '0 0 12px rgba(255, 255, 255, 0.5)',

                // ── Neon glow variants ──
                'neon-glow-xs': '0 0 6px rgba(0, 229, 255, 0.3)',
                'neon-glow-soft': '0 0 8px rgba(0, 229, 255, 0.3)',
                'neon-glow-ball': '0 0 16px rgba(0, 229, 255, 0.3)',
                'neon-glow-btn': '0 0 8px rgba(0, 229, 255, 0.2)',

                // ── Card state ──
                'card-active': '0 0 12px rgba(0, 229, 255, 0.06), 0 4px 20px rgba(0, 0, 0, 0.25)',
            },
            spacing: {
                '4.5': '18px',
            },
            minWidth: {
                'stats-panel': '260px',
            },
            gridTemplateColumns: {
                'spine': '64px 24px 1fr',
            },
            transitionDuration: {
                drawer: "200ms",
            },
        },
    },
    plugins: [],
};
