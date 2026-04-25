import { defineStore } from 'pinia'
import { reactive, toRefs } from 'vue'
import type { CrosshairConfig } from '../types/crosshair'
import { readFromStorage, writeToStorage } from '../utils/storage'

const STORAGE_KEY = 'hero-deploy-ui-config'

const defaultCrosshairConfig: CrosshairConfig = {
  crosshairOffsetX: 0,
  crosshairOffsetY: 0,
  crosshairWidth: 2,
  displayScale: 1,
  showCenterDot: true,
}

export const useUiStore = defineStore('ui', () => {
  const state = reactive({
    ...defaultCrosshairConfig,
    showDebug: false,
  })

  function restore() {
    const saved = readFromStorage<CrosshairConfig>(STORAGE_KEY)
    if (!saved) {
      return
    }
    state.crosshairOffsetX = saved.crosshairOffsetX ?? defaultCrosshairConfig.crosshairOffsetX
    state.crosshairOffsetY = saved.crosshairOffsetY ?? defaultCrosshairConfig.crosshairOffsetY
    state.crosshairWidth = saved.crosshairWidth ?? defaultCrosshairConfig.crosshairWidth
    state.displayScale = saved.displayScale ?? defaultCrosshairConfig.displayScale
    state.showCenterDot = saved.showCenterDot ?? defaultCrosshairConfig.showCenterDot
  }

  function save() {
    return writeToStorage<CrosshairConfig>(STORAGE_KEY, {
      crosshairOffsetX: state.crosshairOffsetX,
      crosshairOffsetY: state.crosshairOffsetY,
      crosshairWidth: state.crosshairWidth,
      displayScale: state.displayScale,
      showCenterDot: state.showCenterDot,
    })
  }

  function resetDefaults() {
    Object.assign(state, defaultCrosshairConfig)
  }

  function toggleDebug() {
    state.showDebug = !state.showDebug
  }

  return {
    ...toRefs(state),
    save,
    restore,
    resetDefaults,
    toggleDebug,
  }
})
