import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { ModeSyncPayload, MqttCommandResult, MqttConnectParams } from '../types/mode'

export const MODE_SYNC_EVENT = 'mode_sync'
const NOOP_UNLISTEN: UnlistenFn = async () => {}

function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

export async function connectMqtt(params: MqttConnectParams): Promise<MqttCommandResult> {
  if (!isTauriRuntime()) {
    return { success: false, message: 'Tauri backend not available in browser mode' }
  }
  return invoke<MqttCommandResult>('connect_mqtt', {
    host: params.host,
    port: params.port,
  })
}

export async function disconnectMqtt(): Promise<MqttCommandResult> {
  if (!isTauriRuntime()) {
    return { success: false, message: 'Tauri backend not available in browser mode' }
  }
  return invoke<MqttCommandResult>('disconnect_mqtt')
}

export async function emitMockModeSync(): Promise<MqttCommandResult> {
  if (!isTauriRuntime()) {
    return { success: false, message: 'Tauri backend not available in browser mode' }
  }
  return invoke<MqttCommandResult>('emit_mock_mode_sync')
}

export async function subscribeModeSync(
  handler: (payload: ModeSyncPayload) => void,
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return NOOP_UNLISTEN
  }
  return listen<ModeSyncPayload>(MODE_SYNC_EVENT, (event) => {
    handler(event.payload)
  })
}
