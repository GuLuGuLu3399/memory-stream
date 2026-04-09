<script setup lang="ts">
/**
 * 🌟 App.vue — Chrome-less 沉浸式入口
 *
 * 架构：
 * - LeftDock：统一左侧指挥中枢（导航 + 点击展开控制面板）
 * - v-show：零延迟切换且保留 VueFlow 状态
 * - DetailDrawer：右侧阅读抽屉（无 ✕ 按钮，依赖遮罩/Esc/右滑关闭）
 * - ZenReader：全屏禅模式阅读（z-70，WASM TOC 侧边栏）
 * - CommandPalette：Cmd+K 全局搜索
 * - TimelineRuler：列表视图右侧极简时间轴
 */

import { ref, onErrorCaptured } from "vue";
import { storeToRefs } from "pinia";
import ListView from "./views/ListView.vue";
import GraphView from "./views/GraphView.vue";
import DetailDrawer from "./components/DetailDrawer.vue";
import LeftDock from "./components/LeftDock.vue";
import SearchBar from "./components/SearchBar.vue";
import ZenReader from "./components/ZenReader.vue";
import EntranceAnimation from "./components/EntranceAnimation.vue";
import { useGraphStore } from "./store/useGraphStore";
import { useKeyboardNav } from "./composables/useKeyboardNav";

const store = useGraphStore();
const { viewMode } = storeToRefs(store);

// ── 全局键盘导航（含 Cmd+K） ──
useKeyboardNav();

// ── GraphView fitView ref ──
const graphViewRef = ref<InstanceType<typeof GraphView> | null>(null);

// ── 全局错误边界 ──
const hasError = ref(false);
const errorMessage = ref("");

onErrorCaptured((err) => {
  console.error("[App] render error captured:", err);
  hasError.value = true;
  errorMessage.value = err instanceof Error ? err.message : String(err);
  return false; // 阻止错误继续传播
});

function handleRecover() {
  hasError.value = false;
  errorMessage.value = "";
  location.reload();
}

// ── 入场动画（首次加载后自动消失） ──
const showEntrance = ref(true);

function handleFitView() {
  graphViewRef.value?.fitView();
}
</script>

<template>
  <div class="h-screen w-screen bg-ms-deep flex flex-col overflow-hidden font-mono" role="application"
    aria-label="Memory Stream 知识图谱">
    <!-- ── Skip to content（无障碍快捷跳转） ── -->
    <a href="#main-content"
      class="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-error focus:px-4 focus:py-2 focus:bg-neon focus:text-ms-deep focus:rounded-sm focus:text-sm focus:font-bold">
      跳转到主内容
    </a>
    <!-- ── 全局错误边界：祭坛重连 UI ── -->
    <div v-if="hasError"
      class="fixed inset-0 z-error bg-ms-deep/95 backdrop-blur-xl flex flex-col items-center justify-center">
      <div class="text-center max-w-md px-6">
        <div
          class="w-16 h-16 mx-auto mb-6 rounded-none bg-red-500/10 border border-red-500/20 flex items-center justify-center">
          <svg class="w-8 h-8 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
              d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
          </svg>
        </div>
        <h2 class="text-xl font-semibold text-gray-100 mb-2">祭坛能量中断</h2>
        <p class="text-gray-400 text-sm mb-1">渲染层发生了未预期的异常</p>
        <p class="text-gray-600 text-xs font-mono mb-6 truncate">{{ errorMessage }}</p>
        <button @click="handleRecover"
          class="px-6 py-2.5 bg-neon/10 text-neon text-sm rounded-sm border border-neon/20 hover:bg-neon/20 transition-all">
          重新连接神殿
        </button>
      </div>
    </div>
    <!-- ── 主内容区（全屏，无顶部 padding） ── -->
    <main id="main-content" class="flex-1 relative overflow-hidden" role="main" aria-label="知识图谱视图">
      <GraphView v-show="viewMode === 'graph'" ref="graphViewRef" class="view-layer absolute inset-0"
        :class="{ 'view-active': viewMode === 'graph' }" />
      <ListView v-show="viewMode === 'list'" class="view-layer absolute inset-0"
        :class="{ 'view-active': viewMode === 'list' }" />
    </main>

    <!-- ── 左侧指挥中枢（导航 + 控制面板） ── -->
    <LeftDock @fit-view="handleFitView" />

    <!-- ── 阅读抽屉 ── -->
    <DetailDrawer />

    <!-- ── 全屏禅模式阅读 ── -->
    <ZenReader />

    <!-- ── Cmd+K 全局搜索 ── -->
    <SearchBar />

    <!-- ── 入场动画（2.4s 后自动消失） ── -->
    <EntranceAnimation v-if="showEntrance" @done="showEntrance = false" />
  </div>
</template>

<style>
/* ── 视图切换纵深感过渡：scale + blur + opacity ── */
.view-layer {
  opacity: 0;
  transform: scale(0.97);
  filter: blur(4px);
  transition:
    opacity 400ms cubic-bezier(0.16, 1, 0.3, 1),
    transform 400ms cubic-bezier(0.16, 1, 0.3, 1),
    filter 400ms cubic-bezier(0.16, 1, 0.3, 1);
  pointer-events: none;
}

.view-layer.view-active {
  opacity: 1;
  transform: scale(1);
  filter: blur(0px);
  pointer-events: auto;
}
</style>
