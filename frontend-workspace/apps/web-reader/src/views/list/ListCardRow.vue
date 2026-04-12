<script setup lang="ts">
/**
 * ListCardRow — 神殿石碑 (Flesh Temple · Stone Tablet)
 *
 * 每张卡片是一座小型石碑/木牌：
 * - 碑顶装饰线（金缮/铜质）
 * - 碑体：微弱石纹质感 + 雕刻凹陷
 * - 选中态：灯照碑文 — 暖光从左侧投射
 * - 热度以余烬（ember）符号呈现
 */
import StatusBadge from '@memory-stream/ui-shared/components/StatusBadge.vue';

export interface CardIndex {
  id: string;
  title: string;
  excerpt: string;
  hot_score: number;
  updated_at: string;
  relation: 'sequence' | 'reference';
}

interface Props {
  card: CardIndex;
  isSelected: boolean;
  isActive: boolean;
}

defineProps<Props>();

const emit = defineEmits<{
  select: [];
}>();
</script>

<template>
  <div
    class="tablet-row group relative pl-6 pr-12 py-5 cursor-pointer"
    :class="[isSelected ? 'is-selected' : '']"
    @click="emit('select')">
    <!-- Connection line to spine -->
    <div
      class="absolute left-0 top-4.5 w-3 h-0.5 z-10 transition-opacity duration-300 overflow-hidden"
      :class="[
        card.relation === 'sequence'
          ? (isSelected ? 'opacity-80' : 'opacity-0 group-hover:opacity-60')
          : (isSelected ? 'opacity-50' : 'opacity-0 group-hover:opacity-30'),
      ]">
      <div
        v-if="card.relation === 'sequence'"
        class="w-full h-full bg-gradient-to-r from-xuepo/60 via-xuepo/20 to-transparent"
      />
      <div
        v-else
        class="w-full h-full bg-gradient-to-r from-ms-copper/40 via-ms-copper/15 to-transparent"
        style="background-image: repeating-linear-gradient(90deg, rgba(90,79,62,0.4) 0px, rgba(90,79,62,0.4) 3px, transparent 3px, transparent 6px);"
      />
    </div>

    <!-- Tablet body -->
    <div class="tablet" :class="[isSelected ? 'tablet--lit' : 'tablet--rest']">
      <!-- 碑顶装饰线 -->
      <div class="tablet__capping">
        <div class="tablet__capping-line" />
        <div v-if="isSelected" class="tablet__capping-gem" />
      </div>

      <!-- Title row -->
      <div class="flex items-center gap-3 mb-2">
        <h3
          class="text-base font-serif font-semibold tracking-wide transition-colors duration-300 truncate flex-1"
          :class="isSelected ? 'text-ms-ivory' : 'text-ms-bone-dim group-hover:text-ms-bone'"
        >
          {{ card.title || '无标题' }}
        </h3>

        <!-- Relation badge -->
        <StatusBadge :variant="card.relation" size="xs">
          {{ card.relation === 'sequence' ? 'SEQ' : 'REF' }}
        </StatusBadge>

        <!-- Hot score — 余烬符印 -->
        <div
          v-if="card.hot_score > 0"
          class="shrink-0 flex items-center gap-1 px-2 py-0.5"
          :class="isSelected ? 'ember-stamp--lit' : 'ember-stamp'"
        >
          <!-- 余烬符号 -->
          <svg width="8" height="10" viewBox="0 0 8 10" fill="none" class="shrink-0">
            <path d="M4 0C4 0 7 3 7 5.5C7 7.5 5.5 9 4 9C2.5 9 1 7.5 1 5.5C1 3 4 0 4 0Z"
              :fill="isSelected ? 'rgba(166,38,38,0.7)' : 'rgba(90,79,62,0.5)'" />
          </svg>
          <span class="text-2xs font-mono" :class="isSelected ? 'text-xuepo/80' : 'text-ms-ash'">
            {{ Math.round(card.hot_score) }}
          </span>
        </div>
      </div>

      <!-- Excerpt — 碑文 -->
      <p class="text-ms-smoke/60 text-sm truncate leading-relaxed tracking-wide group-hover:text-ms-bone-dim/80 transition-colors duration-500">
        {{ card.excerpt || '暂无摘要' }}
      </p>
    </div>
  </div>
</template>

<style scoped>
/* ═══ Tablet — 神殿石碑 ═══ */
.tablet {
    position: relative;
    max-width: 42rem;
    margin-left: 6px;
    padding: 14px 18px 14px 16px;
    border-radius: 1px;
    overflow: hidden;
}

/* ── 碑体纹理 ── */
.tablet::before {
    content: '';
    position: absolute;
    inset: 0;
    pointer-events: none;
    opacity: 0.03;
    background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.8' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");
}

/* ── Rest state: 冷碑 ── */
.tablet--rest {
    background: #12100c;
    border: 1px solid rgba(58, 50, 40, 0.35);
    box-shadow:
        inset 0 1px 0 rgba(232, 223, 208, 0.02),
        2px 2px 0 0 rgba(0, 0, 0, 0.4);
    transition:
        transform 0.15s ease,
        box-shadow 0.15s ease,
        border-color 0.25s ease,
        background 0.25s ease;
}

.tablet--rest:hover {
    background: #1a1714;
    border-color: rgba(74, 66, 56, 0.5);
    transform: translate(-1px, -1px);
    box-shadow:
        inset 0 1px 0 rgba(232, 223, 208, 0.04),
        3px 3px 0 0 rgba(0, 0, 0, 0.4);
}

.tablet--rest:active {
    transform: translate(1px, 1px);
    box-shadow:
        inset 0 1px 0 rgba(232, 223, 208, 0.01),
        0px 0px 0 0 rgba(0, 0, 0, 0.4);
}

/* ── Selected state: 灯照碑文 ── */
.tablet--lit {
    background: linear-gradient(
        100deg,
        rgba(166, 38, 38, 0.06) 0%,
        #16130f 25%,
        #141210 100%
    );
    border: 1px solid rgba(166, 38, 38, 0.25);
    box-shadow:
        inset 0 1px 0 rgba(232, 223, 208, 0.03),
        -2px 0 12px rgba(166, 38, 38, 0.06),
        2px 2px 0 0 rgba(0, 0, 0, 0.4);
}

/* 左侧灯光投射 */
.tablet--lit::after {
    content: '';
    position: absolute;
    left: 0;
    top: 10%;
    bottom: 10%;
    width: 3px;
    border-radius: 0 2px 2px 0;
    background: rgba(166, 38, 38, 0.5);
    box-shadow: 0 0 8px rgba(166, 38, 38, 0.2);
    animation: tabletGlow 2.5s ease-in-out infinite;
}

@keyframes tabletGlow {
    0%, 100% { box-shadow: 0 0 6px rgba(166, 38, 38, 0.15); opacity: 0.6; }
    50% { box-shadow: 0 0 12px rgba(166, 38, 38, 0.3); opacity: 1; }
}

/* ── 碑顶装饰 ── */
.tablet__capping {
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 10px;
    position: relative;
}

.tablet__capping-line {
    width: 100%;
    height: 1px;
    background: linear-gradient(90deg, transparent 0%, rgba(58, 50, 40, 0.25) 20%, rgba(58, 50, 40, 0.25) 80%, transparent 100%);
    transition: all 300ms ease;
}

.tablet--lit .tablet__capping-line {
    background: linear-gradient(90deg, transparent 0%, rgba(201, 168, 76, 0.2) 20%, rgba(201, 168, 76, 0.25) 50%, rgba(201, 168, 76, 0.2) 80%, transparent 100%);
}

/* 选中态碑顶宝石 */
.tablet__capping-gem {
    position: absolute;
    width: 5px;
    height: 5px;
    background: rgba(201, 168, 76, 0.4);
    transform: rotate(45deg);
    box-shadow: 0 0 4px rgba(201, 168, 76, 0.3);
}

/* ── 余烬符印 ── */
.ember-stamp {
    border: 1px solid rgba(58, 50, 40, 0.25);
}

.ember-stamp--lit {
    background: rgba(166, 38, 38, 0.06);
    border: 1px solid rgba(166, 38, 38, 0.15);
}
</style>
