<script setup lang="ts">
import type { MergePreview } from "@memory-stream/types/ipc";

const props = defineProps<{
  canShow: boolean;
  isLoading: boolean;
  error: string | null;
  impactResult: MergePreview | null;
}>();
</script>

<template>
  <div class="merge-blast-radius">
    <!-- Section Header with Rivet Dots -->
    <div class="merge-blast-radius__header">
      <span class="rivet rivet--tl" />
      <span class="rivet rivet--tr" />
      <span class="rivet rivet--bl" />
      <span class="rivet rivet--br" />

      <span class="merge-blast-radius__title">
        BLAST RADIUS
      </span>
      <span class="merge-blast-radius__subtitle">(爆炸半径)</span>
    </div>

    <!-- Content -->
    <div class="merge-blast-radius__content">
      <!-- Not ready state -->
      <div v-if="!canShow" class="merge-blast-radius__not-ready">
        <div class="merge-blast-radius__not-ready-text">
          请选择一个主节点和至少一个祭品
        </div>
        <div class="merge-blast-radius__not-ready-sub">
          以预览合并影响范围
        </div>
      </div>

      <!-- Loading state -->
      <div v-else-if="isLoading" class="merge-blast-radius__loading">
        <div class="flex items-center gap-2 text-slate-500">
          <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
          </svg>
          <span class="text-xs font-mono">计算影响范围...</span>
        </div>
      </div>

      <!-- Error state -->
      <div v-else-if="error" class="merge-blast-radius__error">
        <div class="text-red-400 text-xs font-mono mb-2">影响预览失败</div>
        <div class="text-slate-500 text-2xs font-mono">{{ error }}</div>
      </div>

      <!-- Impact data -->
      <div v-else-if="impactResult" class="merge-blast-radius__data">
        <!-- Stats Grid -->
        <div class="merge-blast-radius__stats">
          <div class="merge-blast-radius__stat">
            <div class="merge-blast-radius__stat-label">
              受影响链接
            </div>
            <div class="merge-blast-radius__stat-value merge-blast-radius__stat-value--neon">
              {{ impactResult.total_wikilinks }}
            </div>
          </div>
          <div class="merge-blast-radius__stat">
            <div class="merge-blast-radius__stat-label">
              待修改文件
            </div>
            <div class="merge-blast-radius__stat-value merge-blast-radius__stat-value--brass">
              {{ impactResult.files_to_modify }}
            </div>
          </div>
        </div>

        <!-- File List -->
        <div v-if="impactResult.affected_files.length > 0" class="merge-blast-radius__files">
          <div class="merge-blast-radius__files-title">
            受影响文件
          </div>
          <div class="merge-blast-radius__files-list">
            <div
              v-for="file in impactResult.affected_files"
              :key="file.path"
              class="merge-blast-radius__file"
            >
              <span class="merge-blast-radius__file-path">{{ file.path }}</span>
              <span class="merge-blast-radius__file-count">
                {{ file.link_count }} 链接
              </span>
            </div>
          </div>
        </div>

        <!-- No files affected -->
        <div v-else class="merge-blast-radius__no-files">
          <span class="merge-blast-radius__no-files-text">
            没有文件会被修改
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.merge-blast-radius {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: theme('colors.ms-panel');
}

.merge-blast-radius__header {
  height: 40px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  border-bottom: 1px solid theme('colors.ms-border');
  flex-shrink: 0;
  position: relative;
}

.rivet {
  position: absolute;
  width: 4px;
  height: 4px;
  border-radius: 50%;
}

.rivet--tl { top: 6px; left: 6px; background: rgba(45, 212, 191, 0.6); }
.rivet--tr { top: 6px; right: 6px; background: rgba(45, 212, 191, 0.6); }
.rivet--bl { bottom: 6px; left: 6px; background: rgba(45, 212, 191, 0.6); }
.rivet--br { bottom: 6px; right: 6px; background: rgba(45, 212, 191, 0.6); }

.merge-blast-radius__title {
  font-size: 10px;
  font-family: ui-monospace, monospace;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: theme('colors.ms-primary');
}

.merge-blast-radius__subtitle {
  font-size: 10px;
  color: theme('colors.ms-engrave');
  font-family: ui-monospace, monospace;
  margin-left: 8px;
}

.merge-blast-radius__content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.merge-blast-radius__content::-webkit-scrollbar {
  width: 3px;
}

.merge-blast-radius__content::-webkit-scrollbar-track {
  background: transparent;
}

.merge-blast-radius__content::-webkit-scrollbar-thumb {
  background: #222;
  border-radius: 1px;
}

.merge-blast-radius__not-ready {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
}

.merge-blast-radius__not-ready-text {
  color: theme('colors.ms-engrave');
  font-size: 12px;
  font-family: ui-monospace, monospace;
  margin-bottom: 8px;
}

.merge-blast-radius__not-ready-sub {
  color: theme('colors.ms-deep-hover');
  font-size: 10px;
  font-family: ui-monospace, monospace;
}

.merge-blast-radius__loading {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.merge-blast-radius__error {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.merge-blast-radius__data {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.merge-blast-radius__stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.merge-blast-radius__stat {
  border: 1px solid theme('colors.ms-border');
  background: theme('colors.ms-deep');
  padding: 16px;
  transition: box-shadow 150ms ease;
}

.merge-blast-radius__stat:hover {
  box-shadow: 0 0 8px rgba(184, 134, 11, 0.2);
}

.merge-blast-radius__stat-label {
  font-size: 9px;
  color: theme('colors.ms-engrave');
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-family: ui-monospace, monospace;
  margin-bottom: 4px;
}

.merge-blast-radius__stat-value {
  font-size: 24px;
  font-weight: bold;
  font-family: ui-monospace, monospace;
}

.merge-blast-radius__stat-value--neon {
  color: theme('colors.ms-primary');
}

.merge-blast-radius__stat-value--brass {
  color: theme('colors.brass.DEFAULT');
}

.merge-blast-radius__files-title {
  font-size: 9px;
  color: theme('colors.ms-engrave');
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-family: ui-monospace, monospace;
  margin-bottom: 8px;
}

.merge-blast-radius__files-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.merge-blast-radius__file {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  font-family: ui-monospace, monospace;
  padding: 8px 12px;
  background: theme('colors.ms-deep');
  border: 1px solid theme('colors.ms-border');
  transition: box-shadow 150ms ease;
}

.merge-blast-radius__file:hover {
  box-shadow: 0 0 8px rgba(184, 134, 11, 0.2);
}

.merge-blast-radius__file-path {
  color: theme('colors.ms-surface-raised');
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.merge-blast-radius__file-count {
  color: rgba(184, 134, 11, 0.6);
  font-size: 10px;
  margin-left: 12px;
  flex-shrink: 0;
}

.merge-blast-radius__no-files {
  text-align: center;
  padding: 16px 0;
}

.merge-blast-radius__no-files-text {
  font-size: 10px;
  color: theme('colors.ms-engrave');
  font-style: italic;
  font-family: ui-monospace, monospace;
}
</style>
