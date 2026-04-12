/**
 * useLongPress — Long-press gesture handler with circular progress
 *
 * Tracks pointer down/up/leave events and calculates progress for a
 * long-press action with visual feedback.
 */

import { ref, computed } from "vue";

const HOLD_DURATION_MS = 3000;
const CIRCUMFERENCE = 2 * Math.PI * 16;

export function useLongPress(callback: () => void | Promise<void>, canExecute: () => boolean) {
  const isHolding = ref(false);
  const holdProgress = ref(0);
  const isExecuting = ref(false);

  let holdTimer: ReturnType<typeof setInterval> | null = null;
  let holdStartTime = 0;

  function handlePointerDown(e: PointerEvent) {
    if (e.button !== 0) return; // Left click only
    if (!canExecute() || isExecuting.value) return;

    isHolding.value = true;
    holdStartTime = Date.now();
    holdProgress.value = 0;

    holdTimer = setInterval(() => {
      const elapsed = Date.now() - holdStartTime;
      holdProgress.value = Math.min(100, (elapsed / HOLD_DURATION_MS) * 100);

      if (elapsed >= HOLD_DURATION_MS) {
        clearHoldTimer();
        executeCallback();
      }
    }, 50);
  }

  function handlePointerUp() {
    if (!isHolding.value) return;
    clearHoldTimer();
    isHolding.value = false;
    holdProgress.value = 0;
  }

  function handlePointerLeave() {
    if (isHolding.value) {
      clearHoldTimer();
      isHolding.value = false;
      holdProgress.value = 0;
    }
  }

  function clearHoldTimer() {
    if (holdTimer) {
      clearInterval(holdTimer);
      holdTimer = null;
    }
  }

  async function executeCallback() {
    isExecuting.value = true;
    try {
      await callback();
    } finally {
      isExecuting.value = false;
      isHolding.value = false;
      holdProgress.value = 0;
    }
  }

  const progressOffset = computed(() => {
    return CIRCUMFERENCE - (holdProgress.value / 100) * CIRCUMFERENCE;
  });

  return {
    isHolding,
    holdProgress,
    isExecuting,
    progressOffset,
    handlePointerDown,
    handlePointerUp,
    handlePointerLeave,
  };
}

export type UseLongPressReturn = ReturnType<typeof useLongPress>;
