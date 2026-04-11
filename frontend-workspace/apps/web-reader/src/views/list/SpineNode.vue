<script setup lang="ts">
/**
 * SpineNode — Spine node indicator for ListView
 *
 * Displays either a genesis node (animated layered rings) or a normal dot.
 * Genesis nodes feature blood-amber glow trails with 2s pulse animation.
 * Normal dots are subtle bone-dim that glow blood-amber on hover.
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
  <div class="group/node relative flex items-start justify-center pt-5 z-10">
    <!-- Normal node: small dot -->
    <div
      v-if="!isGenesis"
      class="w-1 h-1 rounded-full transition-all duration-700 cursor-pointer"
      :class="isSelected ? 'bg-xuepo shadow-altar-glow-sm' : 'bg-ms-ash group-hover/node:bg-xuepo/70 group-hover/node:shadow-altar-glow-sm'"
    />

    <!-- Genesis node: animated layered rings -->
    <div
      v-else
      class="genesis-node w-4 h-4 rounded-full transition-all duration-300 cursor-pointer relative"
      :class="isSelected ? 'genesis-selected shadow-altar-glow-lg' : 'group-hover/node:shadow-altar-glow'"
    >
      <!-- Outer energy ring (conic-gradient rotation) -->
      <div class="genesis-ring absolute inset-0 rounded-full" />
      <!-- Selected state: extra rotating outer ring -->
      <div
        v-if="isSelected"
        class="genesis-spin-ring absolute -inset-[3px] rounded-full border border-xuepo/30"
      />
      <!-- Core -->
      <div
        class="absolute inset-[3px] rounded-full transition-all duration-300"
        :class="isSelected ? 'bg-xuepo/60' : 'bg-xuepo/20 group-hover/node:bg-xuepo/60'"
      />
    </div>

    <!-- Time tooltip (visible on hover) -->
    <div
      v-if="date"
      class="absolute left-1/2 -translate-x-1/2 -top-2 opacity-0 group-hover/node:opacity-100 group-hover:opacity-80 transition-opacity duration-200 pointer-events-none z-30"
    >
      <span class="text-2xs font-mono text-ms-bone-dim bg-ms-xuan/95 px-1.5 py-0.5 border border-ms-copper shadow-[1px_1px_0_0_rgba(0,0,0,0.5)] whitespace-nowrap">
        {{ formatTime(date) }}
      </span>
    </div>
  </div>
</template>

<style scoped>
/* Genesis node energy flow animation */
@keyframes genesis-energy-flow {
  0% {
    background: conic-gradient(from 0deg,
      transparent 0%,
      rgba(166, 38, 38, 0.06) 20%,
      rgba(166, 38, 38, 0.2) 50%,
      rgba(166, 38, 38, 0.06) 80%,
      transparent 100%);
    filter: drop-shadow(0 0 3px rgba(166, 38, 38, 0.15));
  }
  50% {
    filter: drop-shadow(0 0 8px rgba(166, 38, 38, 0.3));
  }
  100% {
    background: conic-gradient(from 360deg,
      transparent 0%,
      rgba(166, 38, 38, 0.06) 20%,
      rgba(166, 38, 38, 0.2) 50%,
      rgba(166, 38, 38, 0.06) 80%,
      transparent 100%);
    filter: drop-shadow(0 0 3px rgba(166, 38, 38, 0.15));
  }
}

@keyframes genesis-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes genesis-pulse-glow {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 0.8; }
}

.genesis-node {
  animation: genesis-energy-flow 3s linear infinite;
}

.genesis-node .genesis-ring {
  background: conic-gradient(from 0deg,
    transparent 0%,
    rgba(166, 38, 38, 0.06) 20%,
    rgba(166, 38, 38, 0.2) 50%,
    rgba(166, 38, 38, 0.06) 80%,
    transparent 100%);
  animation: genesis-spin 3s linear infinite;
}

.genesis-node.genesis-selected .genesis-ring {
  animation-duration: 1.5s;
}

.genesis-node .genesis-spin-ring {
  animation: genesis-spin 2s linear infinite, genesis-pulse-glow 1.5s ease-in-out infinite;
}

/* Hover accelerates energy flow */
.group:hover .genesis-node {
  animation-duration: 1.5s;
}

.group:hover .genesis-node .genesis-ring {
  animation-duration: 1s;
}
</style>
