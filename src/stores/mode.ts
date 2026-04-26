import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import type { GameStatusPayload, RefereeMessagePayload, RobotDynamicStatusPayload } from '../types/mode'

export type DeployModeState = 'unknown' | 'active' | 'inactive'

export const useModeStore = defineStore('mode', () => {
  const mqttConnected = ref(false)
  const deployModeActive = ref<boolean | null>(null)
  const lastModeSyncAt = ref<string | null>(null)
  const gameStage = ref('unknown')
  const lastRefereeMessageAt = ref<string | null>(null)
  const refereeTopicCounts = ref<Record<string, number>>({})
  const gameStatus = ref<GameStatusPayload>({
    stageCountdownSec: null,
    currentStage: null,
    currentRound: null,
    totalRounds: null,
  })
  const robotDynamicStatus = ref<RobotDynamicStatusPayload>({
    currentHealth: null,
    currentHeat: null,
    currentChassisEnergy: null,
    currentBufferEnergy: null,
    remainingAmmo: null,
  })

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
    lastRefereeMessageAt.value = null
    refereeTopicCounts.value = {}
  }

  function applyRefereeMessage(payload: RefereeMessagePayload) {
    lastRefereeMessageAt.value = payload.receivedAt
    refereeTopicCounts.value = {
      ...refereeTopicCounts.value,
      [payload.topic]: (refereeTopicCounts.value[payload.topic] ?? 0) + 1,
    }
    if (payload.gameStatus) {
      gameStatus.value = { ...gameStatus.value, ...payload.gameStatus }
      gameStage.value = String(payload.gameStatus.currentStage ?? gameStage.value)
    }
    if (payload.robotDynamicStatus) {
      robotDynamicStatus.value = { ...robotDynamicStatus.value, ...payload.robotDynamicStatus }
    }
  }

  return {
    mqttConnected,
    deployModeActive,
    lastModeSyncAt,
    gameStage,
    lastRefereeMessageAt,
    refereeTopicCounts,
    gameStatus,
    robotDynamicStatus,
    deployModeState,
    applyModeSync,
    applyRefereeMessage,
    resetModeSync,
  }
})
