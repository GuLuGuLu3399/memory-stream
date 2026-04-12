<script setup lang="ts">
/**
 * App.vue — 血肉神殿沉浸式入口
 *
 * 架构：
 * - vue-router 驱动视图切换（/list, /graph）
 * - KeepAlive 保留 VueFlow 图谱状态
 * - ?card=<id> query param 同步 DetailDrawer
 * - LeftDock：殿门旌旗（导航 + 控制面板）
 * - DetailDrawer：经文卷轴（右侧阅读抽屉）
 * - ZenReader：内殿祭坛（全屏禅模式）
 * - SearchBar：铜镜搜索（Cmd+K）
 * - EntranceAnimation：入殿动画
 */

import { ref, watch, onErrorCaptured, onUnmounted } from "vue";
import { storeToRefs } from "pinia";
import { useRouter, useRoute } from "vue-router";
import DetailDrawer from "./components/DetailDrawer.vue";
import LeftDock from "./components/LeftDock.vue";
import BottomNav from "./components/BottomNav.vue";
import SearchBar from "./components/SearchBar.vue";
import ZenReader from "./components/ZenReader.vue";
import EntranceAnimation from "./components/EntranceAnimation.vue";
import { useGraphStore } from "./store/useGraphStore";
import { useKeyboardNav } from "./composables/useKeyboardNav";
import { useBreakpoints } from "./composables/useBreakpoints";

const store = useGraphStore();
const { viewMode, selectedId } = storeToRefs(store);
const router = useRouter();
const route = useRoute();
const { isMobile } = useBreakpoints();

// ── 全局键盘导航（含 Cmd+K） ──
useKeyboardNav();

// ── Route ↔ Store 双向同步 ──
// route → store：URL 变化时更新 store
watch(
  () => route.name,
  (name) => {
    if (name === "list" || name === "graph") {
      viewMode.value = name;
    }
  },
);

// route.query.card → selectedId
watch(
  () => route.query.card as string | undefined,
  (cardId) => {
    const currentId = selectedId.value;
    if (cardId && cardId !== currentId) {
      selectedId.value = cardId;
    } else if (!cardId && currentId) {
      selectedId.value = null;
    }
  },
);

// selectedId → route.query.card
watch(selectedId, (id) => {
  const current = route.query.card as string | undefined;
  if (id && id !== current) {
    router.replace({ query: { ...route.query, card: id } });
  } else if (!id && current) {
    const { card, ...rest } = route.query;
    router.replace({ query: rest });
  }
});

// ── 全局错误边界 ──
const hasError = ref(false);
const errorMessage = ref("");
const countdown = ref(10);
const countdownInterval = ref<number | null>(null);
const copyButtonText = ref("复制");
const copyButtonTimeout = ref<number | null>(null);

onErrorCaptured((err) => {
  console.error("[App] render error captured:", err);
  hasError.value = true;
  errorMessage.value = err instanceof Error ? err.message : String(err);
  startCountdown();
  return false;
});

function startCountdown() {
  countdown.value = 10;
  countdownInterval.value = window.setInterval(() => {
    countdown.value--;
    if (countdown.value <= 0) {
      clearInterval(countdownInterval.value!);
      handleRecover();
    }
  }, 1000);
}

onUnmounted(() => {
  if (countdownInterval.value) {
    clearInterval(countdownInterval.value);
  }
  if (copyButtonTimeout.value) {
    clearTimeout(copyButtonTimeout.value);
  }
});

function handleRecover() {
  if (countdownInterval.value) {
    clearInterval(countdownInterval.value);
  }
  hasError.value = false;
  errorMessage.value = "";
  countdown.value = 10;
  location.reload();
}

async function copyErrorMessage() {
  try {
    await navigator.clipboard.writeText(errorMessage.value);
    copyButtonText.value = "已复制";
    if (copyButtonTimeout.value) {
      clearTimeout(copyButtonTimeout.value);
    }
    copyButtonTimeout.value = window.setTimeout(() => {
      copyButtonText.value = "复制";
    }, 2000);
  } catch (err) {
    console.error("Failed to copy error message:", err);
  }
}

// ── 入场动画（首次加载后自动消失） ──
const showEntrance = ref(true);
</script>

<template>
  <div class="h-screen w-screen bg-ms-xuan flex flex-col overflow-hidden font-serif" role="application"
    aria-label="Memory Stream 血肉神殿">
    <!-- ── Skip to content（无障碍快捷跳转） ── -->
    <a href="#main-content"
      class="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-error focus:px-4 focus:py-2 focus:bg-xuepo focus:text-ms-xuan focus:text-sm focus:font-bold">
      跳转到主内容
    </a>
    <!-- ── 全局错误边界 ── -->
    <div v-if="hasError"
      class="fixed inset-0 z-error bg-ms-xuan/95 flex flex-col items-center justify-center">
      <div class="text-center max-w-md px-6">
        <div
          class="w-16 h-16 mx-auto mb-6 bg-xuepo/10 border border-xuepo/20 flex items-center justify-center animate-pulse">
          <svg class="w-8 h-8 text-xuepo" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
              d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
          </svg>
        </div>
        <h2 class="text-xl font-semibold text-ms-ivory mb-2">神殿能量中断</h2>
        <p class="text-ms-smoke text-sm mb-1">渲染层发生了未预期的异常</p>
        <div class="flex items-center justify-center gap-2 mb-6">
          <p class="text-ms-ash text-xs font-mono break-all flex-1 text-left">{{ errorMessage }}</p>
          <button
            @click="copyErrorMessage"
            class="px-2 py-1 text-xs bg-ms-xiang/50 text-ms-smoke hover:bg-ms-xiang border border-ms-copper/30 transition-all shrink-0">
            {{ copyButtonText }}
          </button>
        </div>
        <div class="flex items-center justify-center gap-3">
          <button @click="handleRecover"
            class="px-6 py-2.5 bg-xuepo/10 text-xuepo text-sm border border-xuepo/20 hover:bg-xuepo/15 transition-all">
            重新连接神殿
          </button>
          <span v-if="countdown > 0" class="text-ms-ash text-xs font-mono">
            自动重连 {{ countdown }}s...
          </span>
        </div>
      </div>
    </div>

    <!-- ── 主内容区：router-view + KeepAlive + Transition ── -->
    <main id="main-content" class="flex-1 relative overflow-hidden" role="main" aria-label="知识图谱视图">
      <router-view v-slot="{ Component, route: currentRoute }">
        <Transition name="ms-page" mode="out-in">
          <KeepAlive>
            <component :is="Component" :key="currentRoute.path" />
          </KeepAlive>
        </Transition>
      </router-view>
    </main>

    <!-- ── 殿门旌旗（仅桌面端） ── -->
    <LeftDock v-if="!isMobile" />

    <!-- ── 经文卷轴 ── -->
    <DetailDrawer />

    <!-- ── 内殿祭坛（全屏禅模式阅读） ── -->
    <ZenReader />

    <!-- ── 铜镜搜索 ── -->
    <SearchBar />

    <!-- ── 移动端底部导航 ── -->
    <BottomNav />

    <!-- ── 入场动画 ── -->
    <EntranceAnimation v-if="showEntrance" @done="showEntrance = false" />
  </div>
</template>
