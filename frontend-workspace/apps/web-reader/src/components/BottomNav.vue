<script setup lang="ts">
/**
 * BottomNav — 移动端悬浮经文签 (血肉神殿)
 *
 * 浮动胶囊设计，不占满宽度：
 * - 居中悬浮，避开左下角 VueFlow Controls 和右下角 StatsWidget
 * - 金缮顶线 + 香炉形体
 * - 三段式：列表 / 搜索 / 图谱
 * - 血珀选中态 + 余烬过渡
 */
import { storeToRefs } from "pinia";
import { useRouter } from "vue-router";
import { List, Network, Search } from "lucide-vue-next";
import { useGraphStore } from "../store/useGraphStore";
import { useBreakpoints } from "../composables/useBreakpoints";

const store = useGraphStore();
const router = useRouter();
const { viewMode } = storeToRefs(store);
const { isMobile } = useBreakpoints();
</script>

<template>
    <div v-if="isMobile" class="bottom-nav">
        <div class="bottom-nav__capsule">
            <!-- 金缮顶线 -->
            <div class="bottom-nav__topline" />

            <div class="bottom-nav__inner">
                <!-- 列表 -->
                <button class="bottom-nav__tab" :class="{ 'bottom-nav__tab--active': viewMode === 'list' }"
                    @click="router.push('/list')">
                    <List :size="16" />
                    <span>列表</span>
                </button>

                <!-- 搜索 -->
                <button class="bottom-nav__tab bottom-nav__tab--center"
                    @click="store.toggleCommandPalette()">
                    <div class="bottom-nav__search-orb">
                        <Search :size="14" />
                    </div>
                </button>

                <!-- 图谱 -->
                <button class="bottom-nav__tab" :class="{ 'bottom-nav__tab--active': viewMode === 'graph' }"
                    @click="router.push('/graph')">
                    <Network :size="16" />
                    <span>图谱</span>
                </button>
            </div>
        </div>
    </div>
</template>

<style scoped>
.bottom-nav {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 20;
    display: flex;
    justify-content: center;
    padding: 0 24px calc(8px + env(safe-area-inset-bottom, 0px));
    pointer-events: none;
}

.bottom-nav__capsule {
    pointer-events: auto;
    display: flex;
    flex-direction: column;
    background: #0e0d0a;
    border: 1px solid rgba(58, 50, 40, 0.5);
    border-radius: 20px;
    box-shadow:
        0 -2px 16px rgba(0, 0, 0, 0.5),
        0 0 0 1px rgba(0, 0, 0, 0.3),
        inset 0 1px 0 rgba(232, 223, 208, 0.03);
    overflow: hidden;
    max-width: 220px;
    width: 100%;
}

/* 金缮顶线 */
.bottom-nav__topline {
    height: 1px;
    background: linear-gradient(90deg, transparent 10%, rgba(201, 168, 76, 0.2) 30%, rgba(201, 168, 76, 0.25) 50%, rgba(201, 168, 76, 0.2) 70%, transparent 90%);
}

.bottom-nav__inner {
    display: flex;
    align-items: center;
    justify-content: space-around;
    height: 44px;
    padding: 0 4px;
}

.bottom-nav__tab {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    background: transparent;
    border: none;
    color: rgba(90, 79, 62, 0.5);
    font-size: 9px;
    font-family: 'JetBrains Mono', monospace;
    cursor: pointer;
    transition: color 200ms ease;
    -webkit-tap-highlight-color: transparent;
    border-radius: 12px;
}

.bottom-nav__tab--active {
    color: #c9a84c;
    background: rgba(201, 168, 76, 0.06);
}

.bottom-nav__tab:active {
    color: #c8bfa8;
    background: rgba(58, 50, 40, 0.15);
}

/* 搜索按钮 */
.bottom-nav__tab--center {
    flex: 0 0 44px;
}

.bottom-nav__search-orb {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: rgba(58, 50, 40, 0.15);
    border: 1px solid rgba(58, 50, 40, 0.35);
    color: rgba(90, 79, 62, 0.5);
    transition: all 200ms ease;
}

.bottom-nav__tab--center:active .bottom-nav__search-orb {
    background: rgba(58, 50, 40, 0.3);
    border-color: rgba(201, 168, 76, 0.3);
    color: #c9a84c;
}
</style>
