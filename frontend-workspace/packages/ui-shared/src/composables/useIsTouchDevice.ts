import { ref, readonly } from 'vue'

const isTouchDevice = ref(false)
const isPrimaryTouch = ref(false)
let initialized = false

function detect() {
  const hoverNone = window.matchMedia('(hover: none)')
  const pointerCoarse = window.matchMedia('(pointer: coarse)')
  isPrimaryTouch.value = hoverNone.matches && pointerCoarse.matches
  isTouchDevice.value = isPrimaryTouch.value || ('ontouchstart' in window)
}

export function useIsTouchDevice() {
  if (!initialized) {
    detect()
    window.matchMedia('(hover: none)').addEventListener('change', detect)
    window.matchMedia('(pointer: coarse)').addEventListener('change', detect)
    initialized = true
  }
  return {
    isTouchDevice: readonly(isTouchDevice),
    isPrimaryTouch: readonly(isPrimaryTouch),
  }
}
