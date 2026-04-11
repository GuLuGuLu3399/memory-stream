<script setup lang="ts">
/**
 * ListCardRow — 符咒卡片 (Blood Temple · Neo-Brutalism)
 *
 * 卡片行组件：光学造影 + 机械实体感
 * - 硬边实体阴影（无高斯模糊）
 * - 顶部高光模拟物理厚度
 * - 悬浮抬升 / 按压下沉机械反馈
 * - 血珀选中态 + 脉动左侧条
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
    class="list-card-row group relative pl-6 pr-12 py-6 cursor-pointer"
    :class="[isSelected ? 'is-selected' : '']"
    @click="emit('select')"
  >
    <!-- Connection line to spine -->
    <div
      class="absolute left-0 top-4.5 w-3 h-0.5 z-10 transition-opacity duration-300 overflow-hidden"
      :class="[
        card.relation === 'sequence'
          ? (isSelected ? 'opacity-80' : 'opacity-0 group-hover:opacity-60')
          : (isSelected ? 'opacity-50 conn-reference' : 'opacity-0 group-hover:opacity-30 conn-reference'),
      ]"
    >
      <div
        v-if="card.relation === 'sequence'"
        class="w-full h-full bg-gradient-to-r from-xuepo/60 via-xuepo/20 to-transparent"
      />
      <div
        v-else
        class="w-full h-full bg-gradient-to-r from-ms-smoke/40 via-ms-smoke/15 to-transparent"
      />
      <div
        v-if="card.relation === 'sequence'"
        class="sweep-light absolute inset-0 bg-gradient-to-r from-transparent via-ms-bone/40 to-transparent"
      />
    </div>

    <!-- Card body — optical elevation -->
    <div
      class="relative max-w-2xl ml-2 px-5 py-4 rounded-industrial card-shell"
      :class="[isSelected ? 'card-shell--selected' : 'card-shell--rest']"
    >
      <!-- Top-light reflection -->
      <div
        class="absolute top-0 left-[8%] right-[8%] h-px pointer-events-none transition-opacity duration-300"
        :class="[
          isSelected
            ? 'bg-gradient-to-r from-transparent via-xuepo/30 to-transparent'
            : 'bg-gradient-to-r from-transparent via-white/[0.04] to-transparent group-hover:via-white/[0.07]'
        ]"
      />

      <!-- Selected: pulse left bar -->
      <div v-if="isSelected" class="absolute left-0 top-2 bottom-2 w-[3px] rounded-full bg-xuepo pulse-bar" />

      <!-- Title row -->
      <div class="flex items-center gap-3 mb-2.5">
        <h3
          class="text-base font-serif font-semibold tracking-spine transition-colors duration-300 truncate flex-1"
          :class="isSelected ? 'text-ms-ivory' : 'text-ms-bone-dim group-hover:text-ms-bone'"
        >
          {{ card.title || '无标题' }}
        </h3>

        <!-- Relation badge -->
        <StatusBadge :variant="card.relation" size="xs">
          {{ card.relation === 'sequence' ? 'SEQ' : 'REF' }}
        </StatusBadge>

        <!-- Hot score — metal stamp -->
        <div
          v-if="card.hot_score > 0"
          class="shrink-0 text-2xs font-mono tracking-widest uppercase px-2 py-0.5 border border-ms-copper/30"
        >
          <span class="mr-1 text-ms-ash">Vol</span>
          <span class="text-xuepo">{{ Math.round(card.hot_score) }}</span>
        </div>
      </div>

      <!-- Excerpt -->
      <p class="text-ms-smoke/70 text-sm truncate leading-loose tracking-wide group-hover:text-ms-bone-dim transition-colors duration-500">
        {{ card.excerpt || '暂无摘要' }}
      </p>
    </div>
  </div>
</template>

<style scoped>
/* ═══ Card shell — optical elevation tokens ═══ */
.card-shell {
    --bg-deep: #12100c;
    --bg-warm: #1c1814;
    --border: rgba(58, 50, 40, 0.4);
    --border-bright: rgba(74, 66, 56, 0.6);
    --border: rgba(58, 50, 40, 0.4);
    --shadow-hard: rgba(0, 0, 0, 0.5);
    --xuepo-rgb: 166, 38, 38;
}

/* ── Rest state: cold jade plate ── */
.card-shell--rest {
    background: var(--bg-deep);
    border: 1px solid var(--border);
    box-shadow:
        inset 0 1px 0 0 rgba(255, 255, 255, 0.03),
        2px 2px 0 0 var(--shadow-hard);
    transition:
        transform 0.15s ease,
        box-shadow 0.15s ease,
        border-color 0.2s ease,
        background 0.2s ease;
}

.card-shell--rest:hover {
    background: var(--bg-warm);
    border-color: var(--border-bright);
    transform: translate(-1px, -1px);
    box-shadow:
        inset 0 1px 0 0 rgba(255, 255, 255, 0.05),
        3px 3px 0 0 var(--shadow-hard);
}

.card-shell--rest:active {
    transform: translate(1px, 1px);
    box-shadow:
        inset 0 1px 0 0 rgba(255, 255, 255, 0.02),
        0px 0px 0 0 var(--shadow-hard);
}

/* ── Selected state: blood-amber lit ── */
.card-shell--selected {
    background: var(--bg-warm);
    border: 1px solid rgba(var(--xuepo-rgb), 0.3);
    box-shadow:
        inset 0 1px 0 0 rgba(255, 255, 255, 0.04),
        0 0 10px rgba(var(--xuepo-rgb), 0.1),
        2px 2px 0 0 var(--shadow-hard);
}

/* ── Pulse bar animation ── */
.pulse-bar {
    animation: bloodPulse 2s ease-in-out infinite;
}

@keyframes bloodPulse {
    0%, 100% {
        opacity: 0.5;
        box-shadow: 0 0 4px rgba(166, 38, 38, 0.15);
    }
    50% {
        opacity: 1;
        box-shadow: 0 0 8px rgba(166, 38, 38, 0.3);
    }
}

/* ── Connection line styles ── */
.conn-reference {
    border-style: dashed;
    border-width: 1px 0 0 0;
    border-color: rgba(90, 79, 62, 0.4);
}

@keyframes sweep-light {
    from { transform: translateX(-200%); }
    to { transform: translateX(200%); }
}

.sweep-light {
    animation: sweep-light 1.2s ease-in-out infinite;
}
</style>
