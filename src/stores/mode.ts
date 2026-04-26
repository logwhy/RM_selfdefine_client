import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import type {
  GameStatusPayload,
  RefereeMessagePayload,
  RefereeEventPayload,
  RobotDynamicStatusPayload,
  RobotStaticStatusPayload,
} from '../types/mode'

export type DeployModeState = 'unknown' | 'active' | 'inactive'

export const useModeStore = defineStore('mode', () => {
  const mqttConnected = ref(false)
  const mqttHost = ref<string | null>(null)
  const mqttPort = ref<number | null>(null)
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
  const robotStaticStatus = ref<RobotStaticStatusPayload>({
    connectionState: null,
    fieldState: null,
    aliveState: null,
    robotId: null,
    robotType: null,
    level: null,
    maxHealth: null,
    maxHeat: null,
    heatCooldownRate: null,
    maxPower: null,
    maxBufferEnergy: null,
    maxChassisEnergy: null,
  })
  const refereeEvents = ref<Array<RefereeEventPayload & { receivedAt: string; text: string }>>([])

  const deployModeState = computed<DeployModeState>(() => {
    if (deployModeActive.value === null) {
      return 'unknown'
    }
    return deployModeActive.value ? 'active' : 'inactive'
  })

  function applyModeSync(payload: {
    mqttConnected: boolean
    mqttHost?: string | null
    mqttPort?: number | null
    deployModeActive: boolean | null
    lastModeSyncAt: string | null
  }) {
    mqttConnected.value = payload.mqttConnected
    mqttHost.value = payload.mqttHost ?? mqttHost.value
    mqttPort.value = payload.mqttPort ?? mqttPort.value
    deployModeActive.value = payload.deployModeActive
    lastModeSyncAt.value = payload.lastModeSyncAt
  }

  function resetModeSync() {
    mqttConnected.value = false
    mqttHost.value = null
    mqttPort.value = null
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
    if (payload.robotStaticStatus) {
      robotStaticStatus.value = { ...robotStaticStatus.value, ...payload.robotStaticStatus }
    }
    if (payload.event) {
      refereeEvents.value = [
        {
          ...payload.event,
          receivedAt: payload.receivedAt,
          text: describeRefereeEvent(payload.event),
        },
        ...refereeEvents.value,
      ].slice(0, 8)
    }
  }

  function describeRefereeEvent(event: RefereeEventPayload) {
    const suffix = event.param ? ` / ${event.param}` : ''
    switch (event.eventId) {
      case 1:
        return `己方机器人战亡${suffix}`
      case 2:
        return `对方机器人战亡${suffix}`
      case 3:
        return `大能量机关激活结果${suffix}`
      case 4:
        return `能量机关已激活${suffix}`
      case 5:
        return `己方英雄狙击伤害${suffix}`
      case 6:
        return `对方英雄狙击伤害${suffix}`
      case 7:
        return '对方呼叫空中支援'
      case 8:
        return `对方空中支援被反制${suffix}`
      case 9:
        return `飞镖命中${suffix}`
      case 10:
        return '对方飞镖闸门开启'
      case 11:
        return '基地遭到攻击'
      case 12:
        return '对方前哨站停转'
      case 13:
        return '对方基地护甲展开'
      case 14:
        return '对方请求四级装配，进入强制退出缓冲期'
      case 15:
        return `装配结果事件${suffix}`
      default:
        return `裁判事件 ${event.eventId ?? '-'}${suffix}`
    }
  }

  return {
    mqttConnected,
    mqttHost,
    mqttPort,
    deployModeActive,
    lastModeSyncAt,
    gameStage,
    lastRefereeMessageAt,
    refereeTopicCounts,
    gameStatus,
    robotDynamicStatus,
    robotStaticStatus,
    refereeEvents,
    deployModeState,
    applyModeSync,
    applyRefereeMessage,
    resetModeSync,
  }
})
