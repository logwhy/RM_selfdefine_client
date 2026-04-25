import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export type DeployModeState = 'unknown' | 'active' | 'inactive'

export const useModeStore = defineStore('mode', () => {
  const mqttConnected = ref(false)
  const deployModeActive = ref<boolean | null>(null)
  const lastModeSyncAt = ref<string | null>(null)
  const gameStage = ref('unknown')

  const deployModeState = computed<DeployModeState>(() => {
    if (deployModeActive.value === null) {
      return 'unknown'
    }
    return deployModeActive.value ? 'active' : 'inactive'
  })

  function applyModeSync(payload: {
    mqttConnected: boolean
    deployModeActive: boolean | null
    lastModeSyncAt: string | null
  }) {
    mqttConnected.value = payload.mqttConnected
    deployModeActive.value = payload.deployModeActive
    lastModeSyncAt.value = payload.lastModeSyncAt
  }

  function resetModeSync() {
    mqttConnected.value = false
    deployModeActive.value = null
    lastModeSyncAt.value = null
  }

  return {
    mqttConnected,
    deployModeActive,
    lastModeSyncAt,
    gameStage,
    deployModeState,
    applyModeSync,
    resetModeSync,
  }
})
