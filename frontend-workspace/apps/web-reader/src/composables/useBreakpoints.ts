/**
 * 🌟 useBreakpoints — 响应式断点检测
 *
 * 基于 matchMedia 的响应式断点，返回当前设备类型。
 * - mobile：<640px
 * - tablet：640px–1023px
 * - desktop：≥1024px
 */

import { ref, computed, onMounted, onUnmounted } from "vue";

type Breakpoint = "mobile" | "tablet" | "desktop";

export function useBreakpoints() {
  const current = ref<Breakpoint>("desktop");

  const mobileQuery = window.matchMedia("(max-width: 639px)");
  const tabletQuery = window.matchMedia(
    "(min-width: 640px) and (max-width: 1023px)",
  );

  function update() {
    if (mobileQuery.matches) {
      current.value = "mobile";
    } else if (tabletQuery.matches) {
      current.value = "tablet";
    } else {
      current.value = "desktop";
    }
  }

  onMounted(() => {
    update();
    mobileQuery.addEventListener("change", update);
    tabletQuery.addEventListener("change", update);
  });

  onUnmounted(() => {
    mobileQuery.removeEventListener("change", update);
    tabletQuery.removeEventListener("change", update);
  });

  const isMobile = computed(() => current.value === "mobile");
  const isTablet = computed(() => current.value === "tablet");
  const isDesktop = computed(() => current.value === "desktop");

  return { current, isMobile, isTablet, isDesktop };
}
