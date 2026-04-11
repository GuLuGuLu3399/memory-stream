<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed, onErrorCaptured } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ScanResult } from "@memory-stream/types/ipc";
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

// Error boundary state
const hasError = ref(false);
const errorMessage = ref("");

// Capture render errors
onErrorCaptured((err) => {
  console.error("[App] render error captured:", err);
  hasError.value = true;
  errorMessage.value = err instanceof Error ? err.message : String(err);
  return false;
});

// Reload page method
function reloadPage() {
  window.location.reload();
}

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
    const result = await invoke<ScanResult>("scan_config");
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
    const result = await invoke<ScanResult>("scan_config");
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
  <!-- Error Boundary -->
  <div v-if="hasError" class="h-screen w-screen bg-ms-deep/95 flex items-center justify-center">
    <div class="flex flex-col items-center gap-4 text-center max-w-md px-6">
      <!-- Brass Gear Icon with Neon Glow -->
      <svg class="w-16 h-16 text-brass animate-spin" style="animation-duration: 3s;" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="2"/>
        <path d="M12 2V4M12 20V22M2 12H4M20 12H22M4.929 4.929L6.343 6.343M17.657 17.657L19.071 19.071M4.929 19.071L6.343 17.657M17.657 6.343L19.071 4.929" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      </svg>

      <h1 class="text-xl font-mono font-bold text-white">祭坛能量中断</h1>

      <p class="text-sm text-slate-500 font-mono break-all">
        {{ errorMessage }}
      </p>

      <button
        @click="reloadPage"
        class="mt-4 px-6 py-2 text-sm font-mono border border-brass/30 text-brass hover:bg-brass/10 transition-all"
      >
        重新连接
      </button>
    </div>
  </div>

  <!-- Loading State -->
  <div v-else-if="!isReady" class="h-screen w-screen bg-ms-deep flex items-center justify-center">
    <div class="flex flex-col items-center gap-3">
      <!-- Large Brass Gear Spinner -->
      <svg class="w-12 h-12 text-brass animate-spin" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="2"/>
        <path d="M12 2V4M12 20V22M2 12H4M20 12H22M4.929 4.929L6.343 6.343M17.657 17.657L19.071 19.071M4.929 19.071L6.343 17.657M17.657 6.343L19.071 4.929" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      </svg>
      <span class="text-slate-500 text-xs font-mono tracking-wider">CONNECTING...</span>
    </div>
  </div>

  <!-- Main Interface: Ready -->
  <div v-else class="h-screen w-screen bg-ms-deep overflow-hidden font-body flex flex-col">

    <!-- First-run configuration banner (fixed at top) -->
    <div v-if="showConfigBanner" class="fixed top-0 left-0 right-0 z-chrome bg-amber-500/10 border-b border-amber-500/30 px-4 py-2 flex items-center justify-between font-mono text-xs">
      <span class="text-amber-400">⚠ 首次使用 — 请完成系统配置</span>
      <div class="flex gap-2">
        <button @click="openSettings" class="text-amber-400 hover:text-amber-300 underline">前往配置</button>
        <button @click="dismissBanner" class="text-slate-500 hover:text-slate-400">✕</button>
      </div>
    </div>

    <!-- Top Title Bar -->
    <TitleBar />

    <!-- Main Area: Left card library, center editing area, right graph -->
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

    <!-- Bottom Status Bar -->
    <footer class="h-6 bg-ms-void border-t border-ms-border flex items-center px-3 justify-between select-none shrink-0 engrave">
      <div class="flex items-center gap-3">
        <span class="text-2xs text-brass font-mono font-medium">MS-ADMIN</span>
        <span class="text-2xs text-slate-700 font-mono">·</span>
        <span class="text-2xs text-slate-700 font-mono">v0.1.0</span>
      </div>
      <div class="flex items-center gap-3">
        <span class="w-1.5 h-1.5 rounded-full bg-ms-success/70" />
        <span class="text-2xs text-slate-700 font-mono">CONNECTED</span>
        <span class="text-2xs text-slate-700 font-mono">·</span>
        <span class="text-2xs text-brass-dim font-mono">READY</span>
      </div>
    </footer>

    <!-- Floating Components -->
    <CommandPalette />
    <ConfirmDialog />
    <CategoryPanel />
    <Settings />

    <!-- Toast Notifications -->
    <div class="fixed bottom-8 right-4 z-toast flex flex-col-reverse gap-1.5 max-w-[220px]">
      <TransitionGroup name="toast">
        <div v-for="toast in toasts" :key="toast.id"
          class="px-3 py-1.5 rounded-sm text-xs font-mono backdrop-blur-md border flex items-center gap-1.5"
          :class="{
            'bg-ms-void/90 text-ms-success/80 border-brass/30 shadow-brass-glow-sm': toast.type === 'success',
            'bg-ms-void/90 text-ms-danger/80 border-ms-danger/20': toast.type === 'error',
            'bg-ms-void/90 text-neon/80 border-brass/30': toast.type === 'info',
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
