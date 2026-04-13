# Memory Stream 双主题设计规范

> 版本: 1.0 | 日期: 2026-04-10
> web-reader: **血肉神殿** (东方邪典) | admin-tauri: **机械祭坛** (精炼进化)

---

## 一、血肉神殿 (web-reader)

### 1.1 设计哲学

东方邪典美学——知识如封印于血肉神殿中的经文，每一次阅读都是一场穿越经脉的朝圣。

- **有机而非机械**：曲线取代直线，呼吸取代脉冲
- **暗色主导**：以玄色为基底，血珀与金缮为高亮
- **仪式感**：交互如祭祀——缓慢、庄重、有反馈
- **层次隐喻**：殿门→回廊→祭坛，对应首页→列表→沉浸阅读

### 1.2 色彩体系

```
┌──────────────────────────────────────────────────────┐
│  玄色 Xuan    #0a0806  ██████  主背景（最深）          │
│  墨色 Mo      #12100c  ██████  内容区背景              │
│  香灰 Xiang   #1c1814  ██████  面板/卡片背景           │
│  烛影 Zhu     #2a2218  ██████  悬浮/激活态             │
│  铜锈 Lv      #3a3228  ██████  边框/分隔               │
│  烟烬 Yan     #5a4f3e  ██████  次要文字/图标           │
│  香灰淡       #8a7e6e  ██████  辅助文字                │
│  骨白 Gu      #e8dfd0  ██████  主文字                  │
│  象牙 XiangYa #f5ead0  ██████  高亮文字/标题           │
│                                                      │
│  ── 主色调 ──                                         │
│  血珀 XuePo     #a62626  ██████  主强调（链接/选中）    │
│  朱砂 ZhuSha    #c23616  ██████  高亮强调（悬停/焦点）  │
│  血珀暗         #6b1616  ██████  血珀暗态              │
│  血珀光         rgba(166,38,38,0.2)  血珀辉光          │
│                                                      │
│  ── 辅助色 ──                                         │
│  金缮 JinShan   #c9a84c  ██████  次强调（标注/星标）   │
│  铜绿 TongLv    #4a7c6f  ██████  三级强调（特殊状态）   │
│                                                      │
│  ── 语义色 ──                                         │
│  危险           #d44040                                 │
│  警告           #d4a040                                 │
│  成功           #5a9c60                                 │
│                                                      │
│  ── 图谱专属 ──                                       │
│  经脉（序列边） #a62626                                 │
│  引渡（引用边） #5a4f3e                                 │
│  灵穴（节点）   #c23616                                 │
└──────────────────────────────────────────────────────┘
```

### 1.3 排版体系

```css
fontFamily: {
  /* 宋体为主，营造古典肃穆感 */
  body:     ['"Noto Serif SC"', '"Source Han Serif SC"', "Georgia", "serif"],
  display:  ['"Noto Serif SC"', "Georgia", "serif"],
  serif:    ['"Noto Serif SC"', '"Playfair Display"', "Georgia", "serif"],
  mono:     ['"JetBrains Mono"', '"Fira Code"', "Consolas", "monospace"],
}
```

| 层级 | 大小 | 字重 | 用途 |
|------|------|------|------|
| Display | 32-40px | Bold (700) | ZenReader 标题 |
| H1 | 24px | Bold (700) | 章节标题 |
| H2 | 20px | Semibold (600) | 小节标题 |
| H3 | 18px | Medium (500) | 子标题 |
| Body | 16px | Regular (400) | 正文 |
| Small | 14px | Regular (400) | 辅助信息 |
| Caption | 12px | Regular (400) | 标签/时间 |
| Micro | 10-11px | Regular (400) | 徽章/角标 |

行高: 1.85 (正文), 1.5 (标题)
字间距: tracking-tight (标题), tracking-normal (正文)

### 1.4 纹理系统

```
┌─────────────────────────────────────────────┐
│  纹理层                        不透明度      │
│  ─────────────────────────────────────────── │
│  经脉纹理 (SVG fractalNoise)    0.02         │
│  → 类似宣纸纤维，有机而不规则                  │
│                                              │
│  烟雾渐变 (radial-gradient)    0.04          │
│  → 边角处暗角，营造殿内烛光氛围               │
│                                              │
│  朱砂光晕 (box-shadow)         交互时         │
│  → 0 0 12px rgba(166,38,38,0.25)            │
└─────────────────────────────────────────────┘
```

### 1.5 阴影体系

```css
boxShadow: {
  /* 血珀辉 — 替代原有 cinnabar-glow */
  'altar-glow':      '0 0 8px rgba(166,38,38,0.2), 0 0 20px rgba(166,38,38,0.08)',
  'altar-glow-sm':   '0 0 4px rgba(166,38,38,0.15)',
  'altar-glow-lg':   '0 0 12px rgba(166,38,38,0.25), 0 0 30px rgba(166,38,38,0.1)',
  'altar-glow-active':'0 0 8px rgba(166,38,38,0.3)',

  /* 浮雕 — 卡片/面板浮起 */
  'raised':          '0 2px 8px rgba(0,0,0,0.4), 0 4px 16px rgba(0,0,0,0.3)',
  'raised-sm':       '0 1px 4px rgba(0,0,0,0.3)',
  'raised-md':       '0 4px 16px rgba(0,0,0,0.4), 0 2px 6px rgba(0,0,0,0.3)',
  'raised-lg':       '0 8px 32px rgba(0,0,0,0.5), 0 4px 12px rgba(0,0,0,0.4)',

  /* 烛光 — 悬浮态 */
  'candle':          '0 0 20px rgba(201,168,76,0.08), 0 4px 12px rgba(0,0,0,0.3)',
}
```

### 1.6 动画体系

```
缓动曲线:
  --ease-altar:    cubic-bezier(0.25, 0.1, 0.25, 1)   /* 默认：平滑过渡 */
  --ease-breathe:  cubic-bezier(0.4, 0, 0.6, 1)       /* 呼吸：有机脉动 */
  --ease-unveil:   cubic-bezier(0.16, 1, 0.3, 1)      /* 揭幕：弹出道具 */

时长:
  --dur-instant:   100ms    即时反馈
  --dur-swift:     200ms    开关/按钮
  --dur-smooth:    350ms    面板/抽屉
  --dur-ritual:    600ms    页面过渡/入场（仪式感）
```

核心动画:
1. **呼吸脉动**: 卡片悬浮时 scale(1.01) + 血珀辉周期性明暗 (2s 周期)
2. **揭幕展开**: DetailDrawer 从右侧展开，带有轻微的 Y 轴偏移 (4px→0)
3. **烟升消隐**: 组件退出时 opacity↓ + 轻微 translateY(-4px)
4. **灵穴亮灭**: 图谱节点选中时，血珀辉从无到有，伴随 scale(1.02)
5. **经脉流动**: 图谱边线的虚线动画（strokeDashoffset 动画）

### 1.7 组件设计

#### LeftDock — 殿门旌旗
```
┌──────────┐
│          │  ← 半透明玄色底 (bg-xuan/80 backdrop-blur)
│  ▣ 列表  │     左侧固定，悬浮于内容之上
│  ◎ 图谱  │     图标用骨白色，激活态用血珀色
│          │
│  ┄┄┄┄┄  │  ← 香灰分隔线
│          │
│  ◫ 控制  │  ← 展开后的控制面板
│          │     bg-xiang, 骨白文字
│          │     控件用朱砂色高亮
└──────────┘
```

#### ListView — 血脉脊骨
```
┌──┬──┬────────────────────────────────┐
│  │  │  ← 三栏: 经脉脊 | 节点 | 卡片内容
│日│灵│  ┌──────────────────────────┐   │
│期│穴│  │ 玄色卡片 (bg-xiang)       │   │
│脊│  │  │ 标题: 骨白, serif        │   │
│  │●│  │ 摘要: 烟烬色             │   │
│  │  │  │ 左侧: 血珀边框 (2px)     │   │
│  │●│  │ 悬浮: candle阴影 + 微上浮  │   │
│  │  │  └──────────────────────────┘   │
│  │  │                                 │
└──┴──┴────────────────────────────────┘
日期脊: 垂直血珀渐变线
节点: ● 灵穴圆点 (hover→血珀辉)
```

#### GraphView — 经脉星图
```
背景: 玄色 + 经脉纹理 (opacity 0.02)
节点: 圆角矩形, bg-xiang, 骨白标题
  - 悬浮: altar-glow + scale(1.02)
  - 选中: 血珀左边框 + altar-glow-lg
  - 孤立: 更暗 (bg-mo), 虚线边框
  - 层级: outline(紧凑)/summary(中等)/detail(展开)
边: 序列边(血珀实线) / 引用边(烟烬虚线)
聚光灯: 未关联节点→opacity 0.08
```

#### DetailDrawer — 经文卷轴
```
从右侧展开 (45% 宽度), 玄色基底

┌──────────────────────────┐
│ ✕                        │  ← 骨白关闭按钮
│                          │
│  ═════════════════════   │  ← 金缮装饰线
│  卡片标题                 │  ← 象牙色, 24px, serif
│  ═════════════════════   │  ← 金缮装饰线
│                          │
│  经文内容 (prose)         │  ← 骨白正文, 香灰底代码块
│  ...                     │     血珀链接色, 金缮代码高亮
│                          │
│  ── 反链 ──              │  ← 铜绿标题
│  ● 引用卡片 1             │  ← 骨白文字, 烟烬摘要
│  ● 引用卡片 2             │
└──────────────────────────┘

过渡: ease-unveil 350ms
遮罩: bg-xuan/60 backdrop-blur-sm
```

#### ZenReader — 内殿祭坛
```
全屏沉浸 (z-70), 玄色背景

┌────────────────────────────────────────┐
│                                        │
│    ╔══════════════════════════════╗     │
│    ║  金缮顶线装饰               ║     │
│    ║                              ║     │
│    ║  max-width: 72ch            ║     │
│    ║  骨白正文, 1.85 行高         ║     │
│    ║  血珀链接, 金缮强调          ║     │
│    ║                              ║     │
│    ║  ← FloatingCompass →        ║     │
│    ║     (圆形进度环 + TOC)       ║     │
│    ╚══════════════════════════════╝     │
│                                        │
└────────────────────────────────────────┘

进度指示: 顶部金缮细线 (2px)
退出: Esc 键
```

#### SearchBar — 铜镜搜索
```
居中模态, 玄色半透背景

┌────────────────────────────────┐
│  ╔══════════════════════════╗  │
│  ║  🔍 搜索经文...          ║  │  ← 骨白输入框, 香灰底
│  ╚══════════════════════════╝  │
│                                │
│  ● 搜索结果 1                  │  ← 骨白标题, 烟烬摘要
│    摘要预览...                 │     选中态: 左侧血珀边框
│  ● 搜索结果 2                  │
│    摘要预览...                 │
└────────────────────────────────┘

过渡: ease-unveil, opacity + translateY
```

#### FloatingCompass — 罗盘导航
```
右下角, 双态切换

收缩态:
  ┌──────┐
  │  ◎   │  ← 圆形进度环, 金缮色描边
  │  65% │     血珀填充弧度
  └──────┘

展开态:
  ┌──────────────────┐
  │  TOC 树形导航     │  ← 香灰底, 骨白文字
  │  ▸ 第一章         │     激活项: 血珀左边框 + 骨白
  │    ▸ 第二节       │
  │  ▸ 第二章         │
  └──────────────────┘
```

#### StatsWidget — 灵签面板
```
右下角 (FloatingCompass 上方), 可折叠

┌────────────────────┐
│  ◫ 灵签            │  ← 骨白标题
│                    │
│  穴位: 128         │  ← 烟烬标签: 骨白数值
│  今日: 3           │
│  均热: 0.72        │
│                    │
│  ▁▂▃▅▆▇          │  ← 迷你热力图 (血珀渐变)
└────────────────────┘

玄色底, 圆角 4px, raised-md 阴影
```

#### CardNode — 符咒玉牌
```
图谱节点, 三种语义层级

┌─────────────┐  outline (紧凑)
│  标题        │  仅标题, bg-mo, 1px 铜锈边框
└─────────────┘

┌─────────────┐  summary (中等)
│  标题        │  标题+摘要首行, bg-xiang
│  摘要预览..  │  血珀左边框 (序列) / 虚线边框 (引用)
└─────────────┘

┌─────────────┐  detail (展开)
│  标题        │  完整卡片, bg-xiang
│  摘要        │  血珀左边框 + 热力徽章
│  [热力: 0.8] │  handle 位置: 序列(上下), 引用(左右)
└─────────────┘

悬浮: altar-glow + scale(1.02)
孤立: bg-mo + dashed 铜锈边框 + opacity 0.7
选中: 血珀左边框(3px) + altar-glow-lg
```

#### BacklinksPanel — 反链经幡
```
DetailDrawer 内的底部区域

┌────────────────────────────┐
│  ── 引渡经幡 ──             │  ← 铜绿标题, 装饰线
│                            │
│  ┌──────────────────────┐  │
│  │ ● 来源卡片 1          │  │  ← 骨白标题, 烟烬摘要
│  │   摘要预览...         │  │     悬浮: altar-glow-sm
│  └──────────────────────┘  │
│  ┌──────────────────────┐  │
│  │ ● 来源卡片 2          │  │
│  │   摘要预览...         │  │
│  └──────────────────────┘  │
└────────────────────────────┘
```

### 1.8 全局样式要点

```css
/* 背景: 玄色 + 经脉纹理 + 暗角 */
body {
  background: #0a0806;
  color: #e8dfd0;
  font-family: "Noto Serif SC", Georgia, serif;
}

/* 纹理叠加 */
body::before {
  /* 经脉纹理 — fractalNoise SVG */
  opacity: 0.02;
  /* fractalNoise 纹理 */
}

/* 暗角 */
body::after {
  /* radial-gradient 边角暗化 */
  opacity: 0.3;
  background: radial-gradient(ellipse at center, transparent 50%, rgba(10,8,6,0.6) 100%);
}

/* 选中高亮 */
::selection {
  background: rgba(166, 38, 38, 0.25);
  color: #f5ead0;
}

/* 滚动条 */
* { scrollbar-color: rgba(232,223,208,0.1) transparent; }
::-webkit-scrollbar-thumb { background-color: rgba(232,223,208,0.1); }
::-webkit-scrollbar-thumb:hover { background-color: rgba(232,223,208,0.2); }
```

---

## 二、机械祭坛 (admin-tauri)

### 2.1 设计哲学

精炼进化——在保留暗黑+霓虹青基底上，提升三个维度：
1. **层次深度**: 更细腻的明度阶梯，空间感更强
2. **机械纹理**: 引入铆钉/蚀刻/黄铜细节，强化祭坛意象
3. **统一语言**: 收敛不一致的视觉处理，建立更严格的规范

### 2.2 色彩体系 (精炼)

```
保留基底，新增辅助色阶:

┌──────────────────────────────────────────────────────┐
│  ── 基底 (保留) ──                                    │
│  void     #050505   最深（侧栏/工具栏）                │
│  deep     #0d0d0d   主背景                            │
│  carbon   #141414   内容区                            │
│  panel    #1a1a1a   面板                              │
│  surface  #222222   表面 (hover)                      │
│  border   #1e1e1e   结构线                            │
│                                                      │
│  ── 新增层次 ──                                       │
│  deep-hover  #181818   deep 的悬浮态                  │
│  surface-raised #2a2a2a 浮起面板                      │
│  engrave     #333333   蚀刻/凹陷效果                  │
│                                                      │
│  ── 霓虹青 (保留，新增层次) ──                         │
│  neon        #00e5ff   主强调                         │
│  neon-dim    #00e5ff/60  弱化强调                     │
│  neon-ghost  #00e5ff/15  极弱底色                     │
│                                                      │
│  ── 新增: 黄铜辅助 ──                                 │
│  brass       #b8860b   黄铜高亮（次强调）              │
│  brass-light #d4a853   黄铜亮态                       │
│  brass-dim   #b8860b/40 黄铜弱化                      │
│                                                      │
│  ── 语义色 (微调) ──                                  │
│  danger      #ff4444   保留                           │
│  warning     #ffaa00   保留                           │
│  success     #00e676   保留                           │
└──────────────────────────────────────────────────────┘
```

### 2.3 新增纹理

```css
/* 铆钉纹理 — 面板边缘装饰 */
.rivet-top {
  background-image:
    radial-gradient(circle 1.5px at 8px 0, rgba(255,255,255,0.06) 100%, transparent 100%),
    radial-gradient(circle 1.5px at calc(100% - 8px) 0, rgba(255,255,255,0.06) 100%, transparent 100%);
}

/* 蚀刻线 — 内嵌效果 */
.engrave {
  box-shadow:
    inset 0 1px 0 rgba(255,255,255,0.04),
    inset 0 -1px 0 rgba(0,0,0,0.3);
}

/* 金属拉丝 — 面板底纹 (仅大面积面板) */
.brushed-metal {
  background-image:
    repeating-linear-gradient(
      90deg,
      transparent,
      transparent 2px,
      rgba(255,255,255,0.008) 2px,
      rgba(255,255,255,0.008) 4px
    );
}

/* 网格纹理 (保留，微调) */
.grid-texture {
  background-image:
    linear-gradient(rgba(255,255,255,0.015) 1px, transparent 1px),
    linear-gradient(90deg, rgba(255,255,255,0.015) 1px, transparent 1px);
  background-size: 24px 24px;
}
```

### 2.4 阴影体系 (精炼)

```css
boxShadow: {
  /* 霓虹辉 (保留) */
  'neon-glow':       '0 0 8px rgba(0,229,255,0.3), 0 0 20px rgba(0,229,255,0.1)',
  'neon-glow-sm':    '0 0 4px rgba(0,229,255,0.2)',
  'neon-glow-lg':    '0 0 12px rgba(0,229,255,0.4), 0 0 30px rgba(0,229,255,0.15)',

  /* 新增: 黄铜辉 */
  'brass-glow':      '0 0 8px rgba(184,134,11,0.2), 0 0 20px rgba(184,134,11,0.08)',
  'brass-glow-sm':   '0 0 4px rgba(184,134,11,0.15)',

  /* 新增: 机械浮雕 */
  'mech-raised':     '0 2px 4px rgba(0,0,0,0.4), 0 4px 12px rgba(0,0,0,0.3)',
  'mech-raised-md':  '0 4px 16px rgba(0,0,0,0.5), 0 2px 6px rgba(0,0,0,0.3)',
  'mech-inset':      'inset 0 1px 3px rgba(0,0,0,0.4), inset 0 -1px 0 rgba(255,255,255,0.03)',
}
```

### 2.5 动画精炼

```
保留现有时长，新增:

  --ease-hydraulic: cubic-bezier(0.33, 0, 0.2, 1)  /* 液压: 面板展开 */
  --ease-snap:      cubic-bezier(0.68, -0.3, 0.32, 1.3)  /* 卡扣: 按钮反馈 */

新增动画:
1. 面板展开: 带 engrave 边框逐步显现
2. 按钮按下: scale(0.97) + mech-inset 阴影
3. 状态切换: neon ↔ brass 渐变过渡 (0.3s)
```

### 2.6 组件精炼

#### TitleBar — 精炼
- 保留结构，按钮增加 brass-dim hover 态
- 窗口控制按钮增加 hover 颜色区分 (关闭→red, 最小化→yellow)
- 新增 engrave 底边效果

#### LeftSidebar — 精炼
- 折叠态 spine 图标增加 neon-ghost 底色圆形背景
- 展开态卡片列表增加 brushed-metal 底纹
- 卡片悬浮: neon-glow-sm → brass-glow-sm 交替暗示选中/普通
- 分类 ribbon 颜色增加 brass 高亮态

#### TheForge — 精炼
- 编辑器/预览切换按钮增加 snap 反馈
- 空状态十字准星增加 brass 色
- 保存按钮: neon 脉冲保留，增加 brass 边框环
- 标题输入框增加 engrave 效果

#### RightAstrolabe — 精炼
- VueFlow 背景增加 brushed-metal 底纹
- 节点增加 brass 边框选中态
- 边增加 neon 流动动画 (strokeDasharray)
- 召唤按钮增加 hydraulic 缓动

#### MergePanel — 精炼
- 保留 amber/red 主题
- 三栏分隔线增加 rivet-top 纹理
- 长按激活增加 brass-glow 进度指示

#### Settings — 精炼
- 配置区块增加 engrave 内嵌效果
- 测试按钮增加 brass 成功态
- 输入框增加 mech-inset focus 态

#### 底部状态栏 — 精炼
- WS 状态指示灯: disconnected=dim, connecting=brass pulse, connected=neon
- 版本号增加 brass-light 色
- 新增 brass 装饰分隔点

---

## 三、共享设计令牌

### 3.1 Z-Index (统一)
```
base: 0 | chrome: 20 | drawer: 30 | panel: 40
overlay: 50 | modal: 60 | fullscreen: 70 | toast: 80
dropdown: 90 | entrance: 100 | error: 110
```

### 3.2 间距 (8px 基准)
```
xs: 4px | sm: 8px | md: 16px | lg: 24px | xl: 32px | 2xl: 48px
```

### 3.3 圆角
```
sharp: 0px | industrial: 2px (admin)
altar: 3px (web-reader 新增)
```

### 3.4 Tailwind Token 映射

**web-reader 新 token 名 → 旧 token 名:**
```
旧 ms-paper      → 新 ms-xuan (基底)
旧 ms-cream      → 新 ms-mo (内容)
旧 ms-white      → (删除)
旧 ms-shell      → 新 ms-xiang (面板)
旧 ms-border     → 新 ms-copper (铜锈)
旧 ms-ink        → 新 ms-bone (骨白)
旧 ms-ink-light  → 新 ms-bone-dim
旧 ms-stone      → 新 ms-ash (香灰淡)
旧 ms-mist       → 新 ms-smoke (烟烬)
旧 cinnabar      → 新 xuepo (血珀)
```

**admin-tauri 新增 token:**
```
新增 brass / brass-light / brass-dim
新增 deep-hover / surface-raised / engrave
新增 box-shadow: brass-glow, brass-glow-sm, mech-raised, mech-raised-md, mech-inset
```

---

## 四、实施检查清单

### web-reader (血肉神殿)
- [ ] tailwind.config.js — 全部色彩/阴影/字体替换
- [ ] style.css — body 背景、纹理、prose 样式重写
- [ ] App.vue — bg class、字体 class、错误页文案
- [ ] ListView.vue — 脊柱渐变色、节点色、卡片色
- [ ] GraphView.vue — 背景、节点默认样式
- [ ] LeftDock.vue — 背景色、图标色、控制面板
- [ ] DetailDrawer.vue — 背景、prose 样式、关闭按钮
- [ ] ZenReader.vue — 背景、进度线颜色
- [ ] SearchBar.vue — 模态背景、输入框色
- [ ] FloatingCompass.vue — 进度环颜色
- [ ] StatsWidget.vue — 背景、文字色
- [ ] CardNode.vue — 背景、边框、hover 态
- [ ] BacklinksPanel.vue — 背景、标题色
- [ ] SkeletonLine.vue — 动画色
- [ ] EntranceAnimation.vue — 入场视觉
- [ ] TocNode.vue — 激活态颜色

### admin-tauri (机械祭坛精炼)
- [ ] tailwind.config.js — 新增 brass/deep-hover/engrave/shadow tokens
- [ ] style.css — 新增纹理类、精炼 prose 样式
- [ ] App.vue — 状态栏精炼、loading 态
- [ ] TitleBar.vue — hover 态精炼、engrave 边框
- [ ] LeftSidebar.vue — brushed-metal 底纹、分类高亮
- [ ] TheForge.vue — 按钮反馈、空状态
- [ ] RightAstrolabe.vue — 节点 brass 边框、流动边动画
- [ ] MergePanel.vue — rivet 纹理、brass 进度
- [ ] Settings.vue — engrave 输入框、brass 成功态
- [ ] CommandPalette.vue — brass hover 态
- [ ] CategoryPanel.vue — engrave 分隔
- [ ] ConfirmDialog.vue — brass 边框
