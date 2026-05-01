export interface ModeSyncPayload {
  mqttConnected: boolean
  mqttHost: string | null
  mqttPort: number | null
  mqttClientId: string | null
  deployModeActive: boolean | null
  lastModeSyncAt: string | null
}

export interface MqttConnectParams {
  host: string
  port: number
  clientId: string
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

export interface RobotStaticStatusPayload {
  connectionState: number | null
  fieldState: number | null
  aliveState: number | null
  robotId: number | null
  robotType: number | null
  level: number | null
  maxHealth: number | null
  maxHeat: number | null
  heatCooldownRate: number | null
  maxPower: number | null
  maxBufferEnergy: number | null
  maxChassisEnergy: number | null
}

export interface RefereeEventPayload {
  eventId: number | null
  param: string | null
}

export interface RefereeMessagePayload {
  topic: string
  bytes: number
  receivedAt: string
  gameStatus: GameStatusPayload | null
  robotDynamicStatus: RobotDynamicStatusPayload | null
  robotStaticStatus: RobotStaticStatusPayload | null
  event: RefereeEventPayload | null
}
