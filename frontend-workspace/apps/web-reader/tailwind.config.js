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
                // ── 血肉神殿基底 ──
                'ms-xuan': '#0a0806',         // 玄色（最深/主背景）
                'ms-mo': '#12100c',            // 墨色（内容区）
                'ms-xiang': '#1c1814',         // 香灰（面板/卡片）
                'ms-zhu': '#2a2218',           // 烛影（悬浮/激活）
                'ms-copper': '#3a3228',        // 铜锈（边框/分隔）
                'ms-copper-light': '#4a4238',  // 铜锈亮态
                'ms-smoke': '#5a4f3e',         // 烟烬（次要文字）
                'ms-ash': '#8a7e6e',           // 香灰淡（辅助文字）
                'ms-bone': '#e8dfd0',          // 骨白（主文字）
                'ms-bone-dim': '#c8bfa8',      // 骨白暗（次文字）
                'ms-ivory': '#f5ead0',         // 象牙（高亮文字/标题）

                // ── 血珀红强调系 ──
                xuepo: {
                    DEFAULT: '#a62626',
                    50: '#2a0e0e',
                    100: '#3a1212',
                    200: '#551818',
                    300: '#6b1616',
                    400: '#8a1e1e',
                    500: '#a62626',
                    600: '#c23616',
                    700: '#d44a4a',
                    800: '#e07070',
                    900: '#f0a0a0',
                },

                // ── 金缮辅助色 ──
                'ms-gold': '#c9a84c',
                'ms-gold-dim': '#c9a84c66',

                // ── 铜绿三级色 ──
                'ms-patina': '#4a7c6f',

                // ── 图谱专属 ──
                'ms-spine': '#a62626',         // 经脉（序列边）
                'ms-spine-active': '#c23616',  // 灵穴（激活节点）
                'ms-branch': '#5a4f3e',        // 引渡（引用边）

                // ── 语义色 ──
                'ms-primary': '#a62626',
                'ms-primary-hover': '#c23616',
                'ms-danger': '#d44040',
                'ms-warning': '#d4a040',
                'ms-success': '#5a9c60',
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
                body: ['"Noto Serif SC"', '"Source Han Serif SC"', "Georgia", "serif"],
                display: ['"Noto Serif SC"', "Georgia", "serif"],
                serif: ['"Noto Serif SC"', '"Playfair Display"', "Georgia", "serif"],
            },
            borderRadius: {
                sharp: '0px',
                industrial: '2px',
                altar: '3px',
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
                // ── 血珀辉（altar-glow） ──
                'altar-glow': '0 0 8px rgba(166,38,38,0.2), 0 0 20px rgba(166,38,38,0.08)',
                'altar-glow-sm': '0 0 4px rgba(166,38,38,0.15)',
                'altar-glow-lg': '0 0 12px rgba(166,38,38,0.25), 0 0 30px rgba(166,38,38,0.1)',
                'altar-glow-active': '0 0 8px rgba(166,38,38,0.3)',

                // ── 浮雕（raised） ──
                'raised': '0 2px 8px rgba(0,0,0,0.4), 0 4px 16px rgba(0,0,0,0.3)',
                'raised-sm': '0 1px 4px rgba(0,0,0,0.3)',
                'raised-md': '0 4px 16px rgba(0,0,0,0.4), 0 2px 6px rgba(0,0,0,0.3)',
                'raised-lg': '0 8px 32px rgba(0,0,0,0.5), 0 4px 12px rgba(0,0,0,0.4)',

                // ── 烛光（candle） ──
                'candle': '0 0 20px rgba(201,168,76,0.08), 0 4px 12px rgba(0,0,0,0.3)',

                // ── Card state ──
                'card-active': '0 2px 8px rgba(166,38,38,0.15), 0 4px 16px rgba(0,0,0,0.4)',
            },
            spacing: {
                '4.5': '18px',
            },
            minWidth: {
                'stats-panel': '260px',
            },
            maxWidth: {
                'prose': '72ch',
            },
            maxHeight: {
                'dropdown': '300px',
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
