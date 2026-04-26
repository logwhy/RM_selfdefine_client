import { invoke } from '@tauri-apps/api/core'
import type { InputDiagnostics, KeyboardMouseCommand } from '../stores/inputControl'
import type { MqttCommandResult } from '../types/mode'

function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

export async function submitKeyboardMouseControl(command: KeyboardMouseCommand): Promise<MqttCommandResult> {
  if (!isTauriRuntime()) {
    return { success: true, message: 'browser dry-run input accepted' }
  }
  return invoke<MqttCommandResult>('submit_keyboard_mouse_control', { command })
}

export async function sendZeroKeyboardMouseControl(): Promise<MqttCommandResult> {
  if (!isTauriRuntime()) {
    return { success: true, message: 'browser zero input accepted' }
  }
  return invoke<MqttCommandResult>('send_zero_keyboard_mouse_control')
}

export async function getInputDiagnostics(): Promise<InputDiagnostics | null> {
  if (!isTauriRuntime()) return null
  return invoke<InputDiagnostics>('get_input_diagnostics')
}
