import { onBeforeUnmount, onMounted } from 'vue'
import { getInputDiagnostics, sendZeroKeyboardMouseControl, submitKeyboardMouseControl } from '../services/inputBridge'
import { useInputControlStore } from '../stores/inputControl'

const TICK_MS = 1000 / 75
const IDLE_HEARTBEAT_MS = 180

export function useLowLatencyInput(target: () => HTMLElement | null) {
  const inputStore = useInputControlStore()
  let accumulatedX = 0
  let accumulatedY = 0
  let accumulatedWheel = 0
  let lastTickAt = performance.now()
  let lastNonZeroSentAt = 0
  let tickTimer: number | null = null
  let diagnosticsTimer: number | null = null
  let ticks = 0
  let lastHzSampleAt = performance.now()
  let pendingMouseEvents = 0

  function isTypingTarget(eventTarget: EventTarget | null) {
    const element = eventTarget as HTMLElement | null
    if (!element) return false
    const tag = element.tagName.toLowerCase()
    return tag === 'input' || tag === 'textarea' || tag === 'select' || element.isContentEditable
  }

  async function enterFpsMode() {
    inputStore.fpsMode = true
    const element = target() ?? document.body
    if (document.pointerLockElement !== element) {
      await element.requestPointerLock()
    }
  }

  async function exitFpsMode() {
    inputStore.fpsMode = false
    inputStore.pointerLocked = false
    inputStore.resetButtonsAndKeys()
    accumulatedX = 0
    accumulatedY = 0
    accumulatedWheel = 0
    if (document.pointerLockElement) document.exitPointerLock()
    await sendZeroKeyboardMouseControl()
  }

  function handlePointerLockChange() {
    inputStore.pointerLocked = Boolean(document.pointerLockElement)
    if (!document.pointerLockElement && inputStore.fpsMode) {
      void exitFpsMode()
    }
  }

  function handleMouseMove(event: MouseEvent) {
    if (!inputStore.fpsMode || !inputStore.pointerLocked) return
    accumulatedX += event.movementX
    accumulatedY += event.movementY
    pendingMouseEvents += 1
  }

  function handleWheel(event: WheelEvent) {
    if (!inputStore.fpsMode || !inputStore.pointerLocked) return
    event.preventDefault()
    accumulatedWheel += Math.sign(event.deltaY)
  }

  function handleMouseButton(event: MouseEvent, down: boolean) {
    if (!inputStore.fpsMode) return
    if (event.button === 0) inputStore.leftButtonDown = down && !inputStore.disabledFire
    if (event.button === 2) inputStore.rightButtonDown = down
    if (event.button === 1) inputStore.midButtonDown = down
  }

  function handleKey(event: KeyboardEvent, down: boolean) {
    if (!inputStore.fpsMode || isTypingTarget(event.target)) return
    if (event.key === 'Escape') {
      event.preventDefault()
      void exitFpsMode()
      return
    }
    inputStore.setKey(event.code, down)
  }

  function commandHasActivity(mouseX: number, mouseY: number, mouseZ: number) {
    return (
      mouseX !== 0 ||
      mouseY !== 0 ||
      mouseZ !== 0 ||
      inputStore.keyboardValue !== 0 ||
      inputStore.leftButtonDown ||
      inputStore.rightButtonDown ||
      inputStore.midButtonDown
    )
  }

  async function sendTick() {
    const now = performance.now()
    const dtMs = Math.max(1, now - lastTickAt)
    lastTickAt = now

    const rawX = accumulatedX
    const rawY = accumulatedY
    const rawZ = accumulatedWheel
    accumulatedX = 0
    accumulatedY = 0
    accumulatedWheel = 0

    const cmdX = Math.round(inputStore.computeAxisCommand(rawX, dtMs))
    const cmdY = Math.round(inputStore.computeAxisCommand(rawY, dtMs))
    const cmdZ = Math.round(rawZ)
    const active = commandHasActivity(cmdX, cmdY, cmdZ)
    const idleDue = now - lastNonZeroSentAt >= IDLE_HEARTBEAT_MS
    if (!active && !idleDue) return
    if (active) lastNonZeroSentAt = now

    if (pendingMouseEvents > 1) {
      inputStore.diagnostics.droppedInputFrames += pendingMouseEvents - 1
    }
    pendingMouseEvents = 0

    const producedAtMs = Date.now()
    await submitKeyboardMouseControl({
      mouseX: cmdX,
      mouseY: cmdY,
      mouseZ: cmdZ,
      leftButtonDown: inputStore.leftButtonDown,
      rightButtonDown: inputStore.rightButtonDown,
      midButtonDown: inputStore.midButtonDown,
      keyboardValue: inputStore.keyboardValue,
      dryRun: inputStore.dryRun,
      disabledFire: inputStore.disabledFire,
      producedAtMs,
    })

    ticks += 1
    inputStore.diagnostics.cmdX = cmdX
    inputStore.diagnostics.cmdY = cmdY
    inputStore.diagnostics.keyboardValue = inputStore.keyboardValue
    inputStore.diagnostics.dryRun = inputStore.dryRun
    inputStore.diagnostics.inputLatencyMs = Number((Date.now() - producedAtMs).toFixed(2))
    if (now - lastHzSampleAt >= 500) {
      inputStore.diagnostics.inputSendHz = Number(((ticks * 1000) / (now - lastHzSampleAt)).toFixed(1))
      ticks = 0
      lastHzSampleAt = now
    }
  }

  function startSchedulers() {
    if (tickTimer === null) tickTimer = window.setInterval(() => void sendTick(), TICK_MS)
    if (diagnosticsTimer === null) {
      diagnosticsTimer = window.setInterval(async () => {
        const diagnostics = await getInputDiagnostics()
        if (!diagnostics) return
        inputStore.diagnostics = {
          ...inputStore.diagnostics,
          ...diagnostics,
          droppedInputFrames: Math.max(inputStore.diagnostics.droppedInputFrames, diagnostics.droppedInputFrames),
        }
      }, 500)
    }
  }

  function stopSchedulers() {
    if (tickTimer !== null) window.clearInterval(tickTimer)
    if (diagnosticsTimer !== null) window.clearInterval(diagnosticsTimer)
    tickTimer = null
    diagnosticsTimer = null
  }

  onMounted(() => {
    inputStore.restore()
    startSchedulers()
    document.addEventListener('pointerlockchange', handlePointerLockChange)
    window.addEventListener('mousemove', handleMouseMove)
    window.addEventListener('wheel', handleWheel, { passive: false })
    window.addEventListener('mousedown', (event) => handleMouseButton(event, true))
    window.addEventListener('mouseup', (event) => handleMouseButton(event, false))
    window.addEventListener('keydown', (event) => handleKey(event, true))
    window.addEventListener('keyup', (event) => handleKey(event, false))
    window.addEventListener('blur', () => void exitFpsMode())
  })

  onBeforeUnmount(() => {
    stopSchedulers()
    document.removeEventListener('pointerlockchange', handlePointerLockChange)
    void sendZeroKeyboardMouseControl()
  })

  return {
    enterFpsMode,
    exitFpsMode,
  }
}
