import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { readFromStorage, writeToStorage } from '../utils/storage'

export interface KeyboardMouseCommand {
  mouseX: number
  mouseY: number
  mouseZ: number
  leftButtonDown: boolean
  rightButtonDown: boolean
  midButtonDown: boolean
  keyboardValue: number
  dryRun: boolean
  disabledFire: boolean
  producedAtMs: number
}

export interface InputDiagnostics {
  inputSendHz: number
  inputLatencyMs: number
  droppedInputFrames: number
  cmdX: number
  cmdY: number
  keyboardValue: number
  dryRun: boolean
}

const STORAGE_KEY = 'hero-deploy-input-control'

const keyBits: Record<string, number> = {
  KeyW: 0,
  KeyS: 1,
  KeyA: 2,
  KeyD: 3,
  ShiftLeft: 4,
  ShiftRight: 4,
  ControlLeft: 5,
  ControlRight: 5,
  KeyQ: 6,
  KeyE: 7,
  KeyR: 8,
  KeyF: 9,
  KeyG: 10,
  KeyZ: 11,
  KeyX: 12,
  KeyC: 13,
  KeyV: 14,
  KeyB: 15,
}

export const useInputControlStore = defineStore('inputControl', () => {
  const fpsMode = ref(false)
  const pointerLocked = ref(false)
  const dryRun = ref(true)
  const disabledFire = ref(true)
  const sensitivity = ref(1)
  const feedforwardEnabled = ref(true)
  const feedforwardGain = ref(0.018)
  const feedforwardDecayMs = ref(45)
  const maxMouseSpeed = ref(120)
  const pressedKeys = ref<Set<string>>(new Set())
  const leftButtonDown = ref(false)
  const rightButtonDown = ref(false)
  const midButtonDown = ref(false)
  const diagnostics = ref<InputDiagnostics>({
    inputSendHz: 0,
    inputLatencyMs: 0,
    droppedInputFrames: 0,
    cmdX: 0,
    cmdY: 0,
    keyboardValue: 0,
    dryRun: true,
  })

  const keyboardValue = computed(() => {
    let value = 0
    pressedKeys.value.forEach((code) => {
      const bit = keyBits[code]
      if (bit !== undefined) value |= 1 << bit
    })
    return value >>> 0
  })

  const keyboardValueBinary = computed(() => keyboardValue.value.toString(2).padStart(16, '0'))

  function setKey(code: string, down: boolean) {
    if (!(code in keyBits)) return
    const next = new Set(pressedKeys.value)
    if (down) next.add(code)
    else next.delete(code)
    pressedKeys.value = next
  }

  function resetButtonsAndKeys() {
    pressedKeys.value = new Set()
    leftButtonDown.value = false
    rightButtonDown.value = false
    midButtonDown.value = false
  }

  function clampCommand(value: number) {
    return Math.max(-maxMouseSpeed.value, Math.min(maxMouseSpeed.value, value))
  }

  function computeAxisCommand(delta: number, dtMs: number) {
    const velocity = dtMs > 0 ? delta / (dtMs / 1000) : 0
    const ff = feedforwardEnabled.value ? feedforwardGain.value * velocity : 0
    return clampCommand(sensitivity.value * delta + ff)
  }

  function save() {
    writeToStorage(STORAGE_KEY, {
      dryRun: dryRun.value,
      disabledFire: disabledFire.value,
      sensitivity: sensitivity.value,
      feedforwardEnabled: feedforwardEnabled.value,
      feedforwardGain: feedforwardGain.value,
      feedforwardDecayMs: feedforwardDecayMs.value,
      maxMouseSpeed: maxMouseSpeed.value,
    })
  }

  function restore() {
    const saved = readFromStorage<{
      dryRun?: boolean
      disabledFire?: boolean
      sensitivity?: number
      feedforwardEnabled?: boolean
      feedforwardGain?: number
      feedforwardDecayMs?: number
      maxMouseSpeed?: number
    }>(STORAGE_KEY)
    if (!saved) return
    dryRun.value = saved.dryRun ?? true
    disabledFire.value = saved.disabledFire ?? true
    sensitivity.value = saved.sensitivity ?? 1
    feedforwardEnabled.value = saved.feedforwardEnabled ?? true
    feedforwardGain.value = saved.feedforwardGain ?? 0.018
    feedforwardDecayMs.value = saved.feedforwardDecayMs ?? 45
    maxMouseSpeed.value = saved.maxMouseSpeed ?? 120
  }

  return {
    fpsMode,
    pointerLocked,
    dryRun,
    disabledFire,
    sensitivity,
    feedforwardEnabled,
    feedforwardGain,
    feedforwardDecayMs,
    maxMouseSpeed,
    pressedKeys,
    leftButtonDown,
    rightButtonDown,
    midButtonDown,
    diagnostics,
    keyboardValue,
    keyboardValueBinary,
    setKey,
    resetButtonsAndKeys,
    computeAxisCommand,
    save,
    restore,
  }
})
