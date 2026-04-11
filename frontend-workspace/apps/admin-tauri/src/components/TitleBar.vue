<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useLayoutStore } from "../stores/layout";
import { useKnowledgeStore } from "../stores/knowledge";
import { storeToRefs } from "pinia";
import { Settings, Plus, Merge, FolderCog, PanelLeftClose, PanelLeftOpen } from "lucide-vue-next";

const appWindow = getCurrentWindow();
const layoutStore = useLayoutStore();
const knowledgeStore = useKnowledgeStore();
const { isLeftDrawerOpen, isCategoryPanelOpen, isMergeConsoleOpen } = storeToRefs(layoutStore);

async function minimize() {
    await appWindow.minimize();
}

async function toggleMaximize() {
    await appWindow.toggleMaximize();
}

async function close() {
    await appWindow.close();
}
</script>

<template>
    <div data-tauri-drag-region
        class="h-[36px] bg-ms-void border-b border-ms-border flex items-center justify-between select-none shrink-0 z-chrome rivet-top relative">
        <!-- Brass gradient engrave line at bottom -->
        <div class="absolute bottom-0 left-0 right-0 h-[1px]"
            style="background: linear-gradient(90deg, transparent 0%, rgb(199, 145, 86, 0.3) 20%, rgb(199, 145, 86, 0.5) 50%, rgb(199, 145, 86, 0.3) 80%, transparent 100%);">
        </div>

        <!-- Left: Global controls -->
        <div data-tauri-drag-region class="flex items-center pl-[68px] gap-0.5">
            <span class="text-[9px] text-brass font-mono tracking-[0.2em] uppercase mr-2">Memory Stream</span>

            <button @click="layoutStore.toggleLeftDrawer()" title="切换侧栏 (Ctrl+B)"
                class="w-7 h-7 flex items-center justify-center text-slate-600 hover:text-neon transition-colors">
                <PanelLeftClose v-if="isLeftDrawerOpen" :size="14" />
                <PanelLeftOpen v-else :size="14" />
            </button>
            <button @click="knowledgeStore.newCard()" title="锻造新卡片"
                class="w-7 h-7 flex items-center justify-center text-slate-600 hover:text-neon transition-colors neon-glow-hover">
                <Plus :size="14" />
            </button>
            <button @click="layoutStore.openMergeConsole()" title="概念坍缩引擎"
                class="w-7 h-7 flex items-center justify-center transition-all"
                :class="isMergeConsoleOpen ? 'text-amber-400 shadow-[1px_1px_0_0_rgba(0,0,0,0.4)]' : 'text-slate-600 hover:text-amber-400'">
                <Merge :size="14" />
            </button>
            <button @click="layoutStore.openCategoryPanel()" title="分类档案库"
                class="w-7 h-7 flex items-center justify-center transition-all"
                :class="isCategoryPanelOpen ? 'text-neon shadow-[1px_1px_0_0_rgba(0,0,0,0.4)]' : 'text-slate-600 hover:text-neon'">
                <FolderCog :size="14" />
            </button>
            <div class="w-px h-4 bg-ms-border mx-1"></div>
            <button @click="layoutStore.openSettings()" title="系统配置"
                class="w-7 h-7 flex items-center justify-center text-slate-600 hover:text-neon transition-colors">
                <Settings :size="14" />
            </button>
        </div>

        <!-- Right: Window controls -->
        <div class="flex items-center h-full">
            <button @click="minimize"
                class="h-full w-11 flex items-center justify-center text-slate-600 hover:bg-white/5 hover:text-brass-light transition-colors"
                title="最小化">
                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14" />
                </svg>
            </button>
            <button @click="toggleMaximize"
                class="h-full w-11 flex items-center justify-center text-slate-600 hover:bg-white/5 hover:text-neon transition-colors"
                title="最大化/还原">
                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
                        d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
                </svg>
            </button>
            <button @click="close"
                class="h-full w-11 flex items-center justify-center text-slate-600 hover:bg-red-500/60 hover:text-white transition-colors"
                title="关闭">
                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
            </button>
        </div>
    </div>
</template>
