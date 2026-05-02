# Memory Stream 设计规范 (Design Specification)

> **Web 端 (web-reader)**：血肉神殿 (Blood Temple) — 东方玄秘与有机学者美学
> **Tauri 端 (admin-tauri)**：机械祭坛 (Mechanical Altar) — 赛博工业与重金属美学

---

## 1. 血肉神殿 (Web 纯读端)

### 1.1 核心设计理念

**东方玄秘美学** —— 将知识视为封印在血肉神殿中的经卷，强调朝圣般的仪式感。

- **有机优于机械**：大量使用曲线而非直线，强调整体的“呼吸感”而非机械闪烁。
- **幽暗基调**：以“玄黑”为底色，辅以“血珀”（暗红）与“金缮”（暗金）作为视觉高光。
- **仪式感交互**：动效克制、有分量、反馈深邃。
- **空间隐喻**：庙门 (侧边栏) → 回廊 (列表区) → 祭坛 (沉浸式阅读区)。

### 1.2 色彩系统 (Color System)

**背景色阶**
| 令牌名称 | 颜色值 | 用途描述 |
| :--- | :--- | :--- |
| `ms-xuan` (玄) | `#0a0806` | 最深底色 / 全局页面背景 |
| `ms-mo` (墨) | `#12100c` | 内容区背景 / 基础卡片底色 |
| `ms-xiang` (香) | `#1c1814` | 悬浮面板 / 核心卡片背景 |
| `ms-zhu` (烛) | `#2a2218` | Hover 悬停状态 / 激活底色 |
| `ms-copper` (铜) | `#3a3228` | 基础边框 / 分割线 |
| `ms-copper-light` | `#4a4238` | 高亮边框 / 聚焦状态 |

**排版色阶**
| 令牌名称 | 颜色值 | 用途描述 |
| :--- | :--- | :--- |
| `ms-smoke` (烟) | `#5a4f3e` | 次要文本 / 图标 / 禁用状态 |
| `ms-ash` (灰) | `#8a7e6e` | 辅助说明 / 时间戳 |
| `ms-bone` (骨) | `#e8dfd0` | 主体文本 / 正文阅读 |
| `ms-bone-dim` | `#c8bfa8` | 次级正文 / 摘要 |
| `ms-ivory` (牙) | `#f5ead0` | 极高亮文本 / 核心大标题 |

**主辅色调**
| 令牌名称 | 颜色值 | 用途描述 |
| :--- | :--- | :--- |
| `xuepo` (血珀) | `#a62626` | 核心主色 (超链接 / 选中状态) |
| `xuepo-bright` | `#c23616` | 极高亮态 (Hover / 焦点) |
| `xuepo-glow` | `rgba(166,38,38,0.2)`| 血珀辉光 (用于 Box-shadow) |
| `ms-gold` (金缮) | `#c9a84c` | 次级高亮 (批注 / 收藏星标) |
| `ms-patina` (铜绿)| `#4a7c6f` | 特殊状态 / 提示语 |
| `ms-meridian` | `#a62626` | 灵脉：图谱中的“序列连线 (Trunk)” |
| `ms-ferry` | `#5a4f3e` | 引渡：图谱中的“引用连线 (Link)” |

### 1.3 排版体系 (Typography)

- **标题 (Display)**: `"Noto Serif SC", Georgia, serif`
- **正文 (Body)**: `"Noto Serif SC", "Source Han Serif SC", serif`
- **代码 (Mono)**: `"JetBrains Mono", "Consolas", monospace`
- **字号与行高**: 大标题 (32-40px, Bold), H1 (24px), 正文 (16px, 行高 1.85), 辅助语 (12px)。

### 1.4 材质与纹理 (Texture System)

- **宣纸纤维 (Meridian Texture)**: 全局叠加 0.02 透明度的 SVG 噪点 (fractalNoise)。
- **烛光暗角 (Smoke Vignette)**: 屏幕边缘叠加淡径向渐变 (`radial-gradient`)。
- **朱砂晕染 (Cinnabar Halo)**: 卡片 Hover 时触发红色微光阴影。

### 1.5 阴影系统 (Shadow System)

- **祭坛辉光 (`shadow-altar-glow`)**: `0 0 8px rgba(166,38,38,0.2), 0 0 20px rgba(166,38,38,0.08)`
- **实体悬浮 (`shadow-raised`)**: `0 2px 8px rgba(0,0,0,0.4)`
- **烛火微光 (`shadow-candle`)**: `0 0 20px rgba(201,168,76,0.08)`

### 1.6 动效系统 (Animation System)

- **呼吸律动 (`ease-breathe`)**: `cubic-bezier(0.4, 0, 0.6, 1)`，用于微放大 (`scale-101`) 与辉光交替。
- **卷轴展开 (`ease-unveil`)**: `cubic-bezier(0.16, 1, 0.3, 1)`，用于抽屉与模态框。

---

## 2. 机械祭坛 (Tauri 全能控制端)

### 2.1 核心设计理念

**精致工业进化** —— 基于深渊黑与霓虹青色，通过极其细腻的景深、机械纹理强化极客控制台隐喻。

- **层级深度**：利用高对比的黑灰色阶区分操作面板和底座。
- **统一语言**：所有交互元件带有硬朗的工程机械感。

### 2.2 色彩系统 (Color System)

**背景色阶**
| 令牌名称 | 颜色值 | 用途描述 |
| :--- | :--- | :--- |
| `ms-void` (虚空) | `#050505` | 极深邃底色 (侧边栏 / 顶栏) |
| `ms-deep` (深渊) | `#0d0d0d` | 主干工作区背景 |
| `ms-carbon` (碳纤)| `#141414` | 内容区域 / 编辑器输入区 |
| `ms-panel` (面板) | `#1a1a1a` | 浮动面板 / 弹窗底色 |
| `ms-surface` (表面)| `#222222` | Hover 或轻量级凸起底色 |

**边框与高亮**
| 令牌名称 | 颜色值 | 用途描述 |
| :--- | :--- | :--- |
| `ms-border` | `#1e1e1e` | 基础结构线 |
| `ms-border-light` | `#2a2a2a` | 高亮结构线 / 分割器 Hover |
| `neon` (霓虹青) | `#00e5ff` | 核心主色 (选中、激活、光晕) |
| `brass` (黄铜) | `#b8860b` | 次级辅助色 (等待状态、警告、次要高亮) |

### 2.3 排版体系 (Typography)

- **界面 UI**: `"Space Grotesk", "Inter", sans-serif`
- **代码/编辑器**: `"JetBrains Mono", "Fira Code", monospace`

### 2.4 材质与纹理 (Texture System)

- **拉丝金属 (`brushed-metal`)**: 细微横向条纹渐变，用于大型面板底色。
- **战术网格 (`grid-texture`)**: 24x24px 点阵网格线，用于图谱背景。
- **机械雕刻 (`engrave`)**: `inset 0 1px 0 rgba(255,255,255,0.04), inset 0 -1px 0 rgba(0,0,0,0.3)`。

### 2.5 阴影系统 (Shadow System)

- **霓虹辉光 (`shadow-neon-glow`)**: `0 0 8px rgba(0,229,255,0.3)`
- **黄铜微光 (`shadow-brass-glow`)**: `0 0 8px rgba(184,134,11,0.2)`
- **机械浮雕 (`shadow-mech-raised`)**: `0 2px 4px rgba(0,0,0,0.4), 0 4px 12px rgba(0,0,0,0.3)`
- **内嵌凹槽 (`shadow-mech-inset`)**: 物理凹陷感，用于输入框及按下状态。

### 2.6 动效系统 (Animation System)

- **液压缓动 (`ease-hydraulic`)**: `cubic-bezier(0.33, 0, 0.2, 1)`，用于抽屉展开、面板拉伸。
- **机械回弹 (`ease-snap`)**: `cubic-bezier(0.68, -0.3, 0.32, 1.3)`，用于按钮按下反弹。

---

## 3. 共享设计令牌 (Shared Tokens)

### 3.1 Z 轴层级编排

- `z-base`: 0
- `z-chrome`: 100 (顶栏/侧边栏)
- `z-drawer`: 200 (抽屉层)
- `z-overlay`: 400 (全局遮罩)
- `z-modal`: 800 (弹窗中心)
- `z-fullscreen`: 900 (沉浸阅读区)

### 3.2 空间网格 (基于 8px)

- `xs`: 4px | `sm`: 8px | `md`: 16px | `lg`: 24px | `xl`: 32px | `2xl`: 48px

### 3.3 圆角系统 (Border Radius)

- `rounded-sharp` (0px): 两端通用 —— 强调结构的硬朗切割。
- `rounded-industrial` (2px): **Tauri 端专属** —— 机械零件默认微倒角。
- `rounded-altar` (3px): **Web 端专属** —— 带有手工打磨感的温润边缘。
