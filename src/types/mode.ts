export interface ModeSyncPayload {
  mqttConnected: boolean
  deployModeActive: boolean | null
  lastModeSyncAt: string | null
}

export interface MqttConnectParams {
  host: string
  port: number
}

export interface MqttCommandResult {
  success: boolean
  message: string
}

export interface GameStatusPayload {
  stageCountdownSec: number | null
  currentStage: number | null
  currentRound: number | null
  totalRounds: number | null
}

export interface RobotDynamicStatusPayload {
  currentHealth: number | null
  currentHeat: number | null
  currentChassisEnergy: number | null
  currentBufferEnergy: number | null
  remainingAmmo: number | null
}

export interface RefereeMessagePayload {
  topic: string
  bytes: number
  receivedAt: string
  gameStatus: GameStatusPayload | null
  robotDynamicStatus: RobotDynamicStatusPayload | null
}
