<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useKnowledgeStore } from "./stores/knowledge";
import { useLayoutStore } from "./stores/layout";
import { useWSListener } from "./composables/useWSListener";
import { useGlobalShortcuts } from "./composables/useGlobalShortcuts";
import { useAuth } from "./composables/useAuth";
import { storeToRefs } from "pinia";
import TitleBar from "./components/TitleBar.vue";
import LeftSidebar from "./components/LeftSidebar.vue";
import TheForge from "./components/TheForge.vue";
import RightAstrolabe from "./components/RightAstrolabe.vue";
import CommandPalette from "./components/CommandPalette.vue";
import ConfirmDialog from "./components/ConfirmDialog.vue";
import CategoryPanel from "./components/CategoryPanel.vue";
import Settings from "./views/Settings.vue";
import MergePanel from "./components/MergePanel.vue";

const store = useKnowledgeStore();
const layoutStore = useLayoutStore();
const { toasts, recentCards, orphanCards } = storeToRefs(store);
const { isRightPanelOpen, isMergeConsoleOpen } = storeToRefs(layoutStore);
const { isReady, silentLogin } = useAuth();

useWSListener();
useGlobalShortcuts();

const allCardsForMerge = computed(() => {
  const seen = new Set<string>();
  const result = [...recentCards.value, ...orphanCards.value];
  return result.filter((c) => {
    if (seen.has(c.id)) return false;
    seen.add(c.id);
    return true;
  }).map((c) => ({ id: c.id, title: c.title }));
});

// First-run banner state
const showConfigBanner = ref(false);
const bannerPermanentDismissKey = "ms_config_banner_permanent_dismissed";

function openSettings() {
  layoutStore.openSettings();
}

async function checkInitialConfig() {
  try {
    const dismissed = localStorage.getItem(bannerPermanentDismissKey);
    if (dismissed === "true") {
      showConfigBanner.value = false;
      return;
    }
    const result: any = await invoke("scan_config");
    if (result && result.is_healthy === false) {
      showConfigBanner.value = true;
    } else {
      showConfigBanner.value = false;
    }
  } catch {
    // ignore if scan_config not available yet
  }
}

function dismissBanner() {
  showConfigBanner.value = false;
}

function handleMergeCompleted(_payload: { survivorId: string; victimIds: string[] }) {
  // Reload card lists after merge
  store.loadOrphans();
  store.loadRecent();
}

async function handleConfigSaved() {
  try {
    const result: any = await invoke("scan_config");
    if (result && result.is_healthy) {
      showConfigBanner.value = false;
      localStorage.setItem(bannerPermanentDismissKey, "true");
    } else {
      showConfigBanner.value = true;
    }
  } catch {
    // ignore
  }
}

let _configSavedListener: (() => void) | undefined;

onMounted(async () => {
  await silentLogin();
  await checkInitialConfig();
  const onConfigSaved = () => handleConfigSaved();
  window.addEventListener("config_saved", onConfigSaved);
  _configSavedListener = () => window.removeEventListener("config_saved", onConfigSaved);
});

onUnmounted(() => {
  if (_configSavedListener) _configSavedListener();
});
</script>

<template>
  <!-- 未就绪时的加载状态 -->
  <div v-if="!isReady" class="h-screen w-screen bg-ms-deep flex items-center justify-center">
    <div class="flex flex-col items-center gap-3">
      <div class="w-6 h-6 border-2 border-neon/30 border-t-neon rounded-full animate-spin" />
      <span class="text-slate-500 text-xs font-mono tracking-wider">CONNECTING...</span>
    </div>
  </div>

  <!-- 主要界面：已就绪 -->
  <div v-else class="h-screen w-screen bg-ms-deep overflow-hidden font-body flex flex-col">

    <!-- 首次使用配置提示横幅（固定在顶部） -->
    <div v-if="showConfigBanner" class="fixed top-0 left-0 right-0 z-chrome bg-amber-500/10 border-b border-amber-500/30 px-4 py-2 flex items-center justify-between font-mono text-xs">
      <span class="text-amber-400">⚠ 首次使用 — 请完成系统配置</span>
      <div class="flex gap-2">
        <button @click="openSettings" class="text-amber-400 hover:text-amber-300 underline">前往配置</button>
        <button @click="dismissBanner" class="text-slate-500 hover:text-slate-400">✕</button>
      </div>
    </div>

    <!-- 顶部标题栏 -->
    <TitleBar />

    <!-- 主区域：左侧卡片库、中心编辑区、右侧图谱 -->
    <div class="flex-1 min-h-0 flex">
            <LeftSidebar />
            <main class="flex-1 min-w-0 relative">
                <TheForge :class="{ 'opacity-10 pointer-events-none': isMergeConsoleOpen }" />
                <Transition name="merge-console">
                    <MergePanel
                        v-if="isMergeConsoleOpen"
                        :cards="allCardsForMerge"
                        @merge-completed="handleMergeCompleted"
                    />
                </Transition>
            </main>
            <RightAstrolabe v-if="isRightPanelOpen" />
        </div>

    <!-- 底部状态栏 -->
    <footer class="h-6 bg-ms-void border-t border-ms-border flex items-center px-3 justify-between select-none shrink-0">
      <div class="flex items-center gap-3">
        <span class="text-[10px] text-slate-700 font-mono">MS-ADMIN</span>
        <span class="text-[10px] text-slate-700 font-mono">v0.1.0</span>
      </div>
      <div class="flex items-center gap-3">
        <span class="text-[10px] text-slate-700 font-mono">WS ●</span>
        <span class="text-[10px] text-slate-700 font-mono">READY</span>
      </div>
    </footer>

    <!-- 浮层组件 -->
    <CommandPalette />
    <ConfirmDialog />
    <CategoryPanel />
    <Settings />

    <!-- 吐司通知 -->
    <div class="fixed bottom-8 right-4 z-toast flex flex-col-reverse gap-1.5 max-w-[220px]">
      <TransitionGroup name="toast">
        <div v-for="toast in toasts" :key="toast.id"
          class="px-3 py-1.5 rounded-sm text-xs font-mono backdrop-blur-md border flex items-center gap-1.5"
          :class="{
            'bg-ms-void/90 text-ms-success/80 border-ms-success/20': toast.type === 'success',
            'bg-ms-void/90 text-ms-danger/80 border-ms-danger/20': toast.type === 'error',
            'bg-ms-void/90 text-neon/80 border-neon/20': toast.type === 'info',
          }">
          <span class="w-1 h-1 rounded-full shrink-0" :class="{
            'bg-ms-success': toast.type === 'success',
            'bg-ms-danger': toast.type === 'error',
            'bg-neon': toast.type === 'info',
          }" />
          {{ toast.text }}
        </div>
      </TransitionGroup>
    </div>
  </div>
</template>

<style>
.toast-enter-active {
  transition: all 0.2s ease-out;
}
.toast-leave-active {
  transition: opacity 0.15s ease-in;
}
.toast-enter-from {
  opacity: 0;
  transform: translateX(12px);
}
.toast-leave-to {
  opacity: 0;
}

.fade-enter-active {
  transition: opacity 0.2s ease-out;
}
.fade-leave-active {
  transition: opacity 0.15s ease-in;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.merge-console-enter-active {
  transition: all 0.2s ease-out;
}

.merge-console-leave-active {
  transition: all 0.15s ease-in;
}

.merge-console-enter-from,
.merge-console-leave-to {
  opacity: 0;
}
</style>
