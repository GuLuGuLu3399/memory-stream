<script setup lang="ts">
/**
 * SpineNode — 灵穴节点（血肉神殿）
 *
 * 灵穴脊柱上的节点指示器：
 * - 普通节点：余烬微点，hover 泛红
 * - 起始节点 (Genesis)：能量旋转环
 * - 选中态：血琥珀辉光 + 外环旋转
 * - 隐藏式时间提示：hover 显示
 */
interface Props {
  isGenesis: boolean;
  date?: string;
  isSelected: boolean;
}

defineProps<Props>();

const formatTime = (iso: string): string => {
  if (!iso) return '';
  const d = new Date(iso);
  const hour = String(d.getHours()).padStart(2, '0');
  const min = String(d.getMinutes()).padStart(2, '0');
  return `${hour}:${min}`;
};
</script>

<template>
  <div class="group/node relative flex flex-col items-center z-10">
    <!-- 上方连线 (香灰脉络) -->
    <div class="w-px flex-1 min-h-[8px] transition-colors duration-500"
      :class="isSelected ? 'bg-xuepo/20' : 'bg-ms-copper/15'" />

    <!-- 节点 -->
    <div class="relative flex items-center justify-center py-1">
      <!-- 普通节点：余烬微点 -->
      <div
        v-if="!isGenesis"
        class="spine-dot"
        :class="isSelected ? 'spine-dot--lit' : 'spine-dot--rest'"
      />

      <!-- Genesis 节点：能量旋转环 -->
      <div
        v-else
        class="genesis-node"
        :class="isSelected ? 'genesis-node--lit' : ''">
        <!-- 外圈旋转 -->
        <div class="genesis-ring" />
        <!-- 选中额外旋转环 -->
        <div v-if="isSelected" class="genesis-outer-ring" />
        <!-- 核心 -->
        <div class="genesis-core" :class="isSelected ? 'genesis-core--lit' : ''" />
      </div>

      <!-- 时间提示 (hover) -->
      <div
        v-if="date"
        class="absolute left-1/2 -translate-x-1/2 -top-1 opacity-0 group-hover/node:opacity-100 transition-opacity duration-200 pointer-events-none z-30">
        <span class="text-2xs font-mono text-ms-bone-dim bg-ms-xuan/95 px-1.5 py-0.5 border border-ms-copper/40 shadow-[1px_1px_0_0_rgba(0,0,0,0.5)] whitespace-nowrap">
          {{ formatTime(date) }}
        </span>
      </div>
    </div>

    <!-- 下方连线 -->
    <div class="w-px flex-1 min-h-[8px] transition-colors duration-500"
      :class="isSelected ? 'bg-xuepo/20' : 'bg-ms-copper/15'" />
  </div>
</template>

<style scoped>
/* ── 普通节点：余烬微点 ── */
.spine-dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    transition: all 400ms ease;
}

.spine-dot--rest {
    background: rgba(90, 79, 62, 0.4);
    box-shadow: 0 0 2px rgba(90, 79, 62, 0.1);
}

.group:hover .spine-dot--rest {
    background: rgba(166, 38, 38, 0.5);
    box-shadow: 0 0 6px rgba(166, 38, 38, 0.2);
}

.spine-dot--lit {
    background: rgba(166, 38, 38, 0.8);
    box-shadow: 0 0 8px rgba(166, 38, 38, 0.3);
    animation: dotEmber 2.5s ease-in-out infinite;
}

@keyframes dotEmber {
    0%, 100% { box-shadow: 0 0 5px rgba(166, 38, 38, 0.2); }
    50% { box-shadow: 0 0 10px rgba(166, 38, 38, 0.4); }
}

/* ── Genesis 节点：能量旋转环 ── */
.genesis-node {
    position: relative;
    width: 16px;
    height: 16px;
    animation: genesisFlow 3s linear infinite;
}

.genesis-node--lit {
    animation-duration: 1.5s;
}

.group:hover .genesis-node {
    animation-duration: 1.5s;
}

@keyframes genesisFlow {
    0% {
        filter: drop-shadow(0 0 3px rgba(166, 38, 38, 0.15));
    }
    50% {
        filter: drop-shadow(0 0 8px rgba(166, 38, 38, 0.3));
    }
    100% {
        filter: drop-shadow(0 0 3px rgba(166, 38, 38, 0.15));
    }
}

.genesis-ring {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: conic-gradient(from 0deg,
        transparent 0%,
        rgba(166, 38, 38, 0.06) 20%,
        rgba(166, 38, 38, 0.2) 50%,
        rgba(166, 38, 38, 0.06) 80%,
        transparent 100%);
    animation: genesisSpin 3s linear infinite;
}

.group:hover .genesis-ring {
    animation-duration: 1s;
}

.genesis-node--lit .genesis-ring {
    animation-duration: 1.5s;
}

@keyframes genesisSpin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
}

.genesis-outer-ring {
    position: absolute;
    inset: -3px;
    border-radius: 50%;
    border: 1px solid rgba(166, 38, 38, 0.25);
    animation: genesisSpin 2s linear infinite;
}

.genesis-core {
    position: absolute;
    inset: 4px;
    border-radius: 50%;
    transition: all 300ms ease;
}

.genesis-core:not(.genesis-core--lit) {
    background: rgba(166, 38, 38, 0.15);
}

.group:hover .genesis-core:not(.genesis-core--lit) {
    background: rgba(166, 38, 38, 0.4);
}

.genesis-core--lit {
    background: rgba(166, 38, 38, 0.6);
    box-shadow: 0 0 6px rgba(166, 38, 38, 0.3);
}
</style>
