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
