import { defineStore } from 'pinia'
import { reactive, toRefs } from 'vue'
import type { CrosshairConfig, CrosshairPreset } from '../types/crosshair'
import { readFromStorage, writeToStorage } from '../utils/storage'

const STORAGE_KEY = 'hero-deploy-ui-config'
const PRESETS_STORAGE_KEY = 'hero-deploy-crosshair-presets'

const defaultCrosshairConfig: CrosshairConfig = {
  crosshairOffsetX: 0,
  crosshairOffsetY: 0,
  crosshairWidth: 2,
  displayScale: 1,
  showCenterDot: true,
  crosshairColor: '#19f7ff',
}

const defaultPresets: CrosshairPreset[] = [
  { id: 1, name: '默认', ...defaultCrosshairConfig },
  {
    id: 2,
    name: '近点吊射',
    crosshairOffsetX: 0,
    crosshairOffsetY: 14,
    crosshairWidth: 2,
    displayScale: 1.05,
    showCenterDot: true,
    crosshairColor: '#2cff8c',
  },
  {
    id: 3,
    name: '远点吊射',
    crosshairOffsetX: 0,
    crosshairOffsetY: -18,
    crosshairWidth: 2,
    displayScale: 0.95,
    showCenterDot: true,
    crosshairColor: '#ffc93a',
  },
]

function normalizePreset(preset: Partial<CrosshairPreset>, fallback: CrosshairPreset): CrosshairPreset {
  return {
    id: fallback.id,
    name: preset.name ?? fallback.name,
    crosshairOffsetX: preset.crosshairOffsetX ?? fallback.crosshairOffsetX,
    crosshairOffsetY: preset.crosshairOffsetY ?? fallback.crosshairOffsetY,
    crosshairWidth: preset.crosshairWidth ?? fallback.crosshairWidth,
    displayScale: preset.displayScale ?? fallback.displayScale,
    showCenterDot: preset.showCenterDot ?? fallback.showCenterDot,
    crosshairColor: preset.crosshairColor ?? fallback.crosshairColor,
  }
}

export const useUiStore = defineStore('ui', () => {
  const state = reactive({
    ...defaultCrosshairConfig,
    activePresetId: 1 as 1 | 2 | 3,
    crosshairPresets: defaultPresets.map((preset) => ({ ...preset })) as CrosshairPreset[],
    showDebug: false,
  })

  function applyConfig(config: CrosshairConfig) {
    state.crosshairOffsetX = config.crosshairOffsetX ?? defaultCrosshairConfig.crosshairOffsetX
    state.crosshairOffsetY = config.crosshairOffsetY ?? defaultCrosshairConfig.crosshairOffsetY
    state.crosshairWidth = config.crosshairWidth ?? defaultCrosshairConfig.crosshairWidth
    state.displayScale = config.displayScale ?? defaultCrosshairConfig.displayScale
    state.showCenterDot = config.showCenterDot ?? defaultCrosshairConfig.showCenterDot
    state.crosshairColor = config.crosshairColor ?? defaultCrosshairConfig.crosshairColor
  }

  function currentConfig(): CrosshairConfig {
    return {
      crosshairOffsetX: state.crosshairOffsetX,
      crosshairOffsetY: state.crosshairOffsetY,
      crosshairWidth: state.crosshairWidth,
      displayScale: state.displayScale,
      showCenterDot: state.showCenterDot,
      crosshairColor: state.crosshairColor,
    }
  }

  function restore() {
    const savedPresets = readFromStorage<CrosshairPreset[]>(PRESETS_STORAGE_KEY)
    if (savedPresets?.length) {
      state.crosshairPresets = defaultPresets.map((fallback) => {
        const saved = savedPresets.find((preset) => preset.id === fallback.id)
        return normalizePreset(saved ?? {}, fallback)
      })
    }

    const saved = readFromStorage<CrosshairConfig & { activePresetId?: 1 | 2 | 3 }>(STORAGE_KEY)
    if (!saved) {
      applyPreset(1, false)
      return
    }
    state.activePresetId = saved.activePresetId ?? 1
    applyConfig(saved)
  }

  function save() {
    return writeToStorage<CrosshairConfig & { activePresetId: 1 | 2 | 3 }>(STORAGE_KEY, {
      ...currentConfig(),
      activePresetId: state.activePresetId,
    })
  }

  function savePresets() {
    return writeToStorage<CrosshairPreset[]>(PRESETS_STORAGE_KEY, state.crosshairPresets)
  }

  function resetDefaults() {
    applyConfig(defaultCrosshairConfig)
    state.activePresetId = 1
    save()
  }

  function applyPreset(id: 1 | 2 | 3, persist = true) {
    const preset = state.crosshairPresets.find((item) => item.id === id)
    if (!preset) return null
    state.activePresetId = id
    applyConfig(preset)
    if (persist) save()
    return preset
  }

  function saveCurrentToPreset(id: 1 | 2 | 3) {
    const index = state.crosshairPresets.findIndex((item) => item.id === id)
    if (index < 0) return null
    const nextPreset: CrosshairPreset = {
      ...state.crosshairPresets[index],
      ...currentConfig(),
    }
    state.crosshairPresets[index] = nextPreset
    state.activePresetId = id
    savePresets()
    save()
    return nextPreset
  }

  function toggleDebug() {
    state.showDebug = !state.showDebug
  }

  return {
    ...toRefs(state),
    save,
    restore,
    resetDefaults,
    applyPreset,
    saveCurrentToPreset,
    toggleDebug,
  }
})
