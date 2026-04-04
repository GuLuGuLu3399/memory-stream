/**
 * 🌟 useSwipeClose — 右滑关闭手势（移动端）
 *
 * 监听 touch 事件，当用户在抽屉左边缘向右滑动超过阈值时触发关闭。
 * 同时提供实时偏移量，可绑定到 transform 实现跟手动画。
 */

import { ref, onMounted, onUnmounted } from "vue";

interface SwipeOptions {
  /** 触发关闭的最小滑动距离（px），默认 80 */
  threshold?: number;
  /** 只在距左边缘 N px 内开始触摸才生效，默认 40 */
  edgeWidth?: number;
  /** 关闭回调 */
  onClose: () => void;
}

export function useSwipeClose(options: SwipeOptions) {
  const { threshold = 80, edgeWidth = 40, onClose } = options;

  const offsetX = ref(0);
  let startX = 0;
  let startY = 0;
  let tracking = false;

  function onTouchStart(e: TouchEvent) {
    const touch = e.touches[0];
    // 只在左边缘区域激活
    if (touch.clientX > edgeWidth) {
      tracking = false;
      return;
    }
    startX = touch.clientX;
    startY = touch.clientY;
    tracking = true;
  }

  function onTouchMove(e: TouchEvent) {
    if (!tracking) return;
    const touch = e.touches[0];
    const dx = touch.clientX - startX;
    const dy = Math.abs(touch.clientY - startY);

    // 如果垂直滑动大于水平，取消追踪
    if (dy > Math.abs(dx) && dy > 10) {
      tracking = false;
      offsetX.value = 0;
      return;
    }

    // 只响应右滑（dx > 0）
    offsetX.value = Math.max(0, dx);
  }

  function onTouchEnd() {
    if (!tracking) return;
    tracking = false;

    if (offsetX.value > threshold) {
      onClose();
    }
    offsetX.value = 0;
  }

  onMounted(() => {
    window.addEventListener("touchstart", onTouchStart, { passive: true });
    window.addEventListener("touchmove", onTouchMove, { passive: true });
    window.addEventListener("touchend", onTouchEnd, { passive: true });
  });

  onUnmounted(() => {
    window.removeEventListener("touchstart", onTouchStart);
    window.removeEventListener("touchmove", onTouchMove);
    window.removeEventListener("touchend", onTouchEnd);
  });

  return { offsetX };
}
