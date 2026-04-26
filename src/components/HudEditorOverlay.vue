<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useHudEditorStore, type HudElement } from '../stores/hudEditor'
import { useInputControlStore } from '../stores/inputControl'
import { useModeStore } from '../stores/mode'
import { useVideoStore } from '../stores/video'

type DragMode = 'move' | 'resize' | 'line-start' | 'line-end' | 'draw'

const hudStore = useHudEditorStore()
const modeStore = useModeStore()
const videoStore = useVideoStore()
const inputStore = useInputControlStore()
const canvasRef = ref<HTMLCanvasElement | null>(null)

let dragState: {
  id: string
  mode: DragMode
  startX: number
  startY: number
  originX: number
  originY: number
  element: HudElement
} | null = null

const interactive = computed(() => hudStore.editMode && !hudStore.locked)

function bindingValue(element: HudElement) {
  if (element.binding === 'GameStatus.stage_countdown_sec') return `${modeStore.gameStatus.stageCountdownSec ?? 420}s`
  if (element.binding === 'RobotDynamicStatus.hp') return modeStore.robotDynamicStatus.currentHealth ?? element.value ?? 85
  if (element.binding === 'RobotDynamicStatus.heat') return Math.round(modeStore.robotDynamicStatus.currentHeat ?? 36)
  if (element.binding === 'RobotDynamicStatus.chassis_energy') return modeStore.robotDynamicStatus.currentChassisEnergy ?? element.value ?? 70
  if (element.binding === 'RobotDynamicStatus.buffer_energy') return modeStore.robotDynamicStatus.currentBufferEnergy ?? element.value ?? 70
  if (element.binding === 'RobotStaticStatus.robot_id') return modeStore.robotStaticStatus.robotId ?? '-'
  if (element.binding === 'mqttConnected') return modeStore.mqttConnected
  if (element.binding === 'fps') return videoStore.fps
  return element.value ?? element.text ?? ''
}

function bindingRatio(element: HudElement) {
  if (element.binding === 'RobotDynamicStatus.hp') {
    const current = modeStore.robotDynamicStatus.currentHealth ?? element.value ?? 85
    const max = modeStore.robotStaticStatus.maxHealth ?? 100
    return max > 0 ? current / max : 0
  }
  if (element.binding === 'RobotDynamicStatus.heat') {
    const current = modeStore.robotDynamicStatus.currentHeat ?? element.value ?? 36
    const max = modeStore.robotStaticStatus.maxHeat ?? 100
    return max > 0 ? current / max : 0
  }
  if (element.binding === 'RobotDynamicStatus.chassis_energy') {
    const current = modeStore.robotDynamicStatus.currentChassisEnergy ?? element.value ?? 70
    const max = modeStore.robotStaticStatus.maxChassisEnergy ?? 100
    return max > 0 ? current / max : 0
  }
  if (element.binding === 'RobotDynamicStatus.buffer_energy') {
    const current = modeStore.robotDynamicStatus.currentBufferEnergy ?? element.value ?? 70
    const max = modeStore.robotStaticStatus.maxBufferEnergy ?? 100
    return max > 0 ? current / max : 0
  }
  const raw = Number(bindingValue(element))
  return raw / 100
}

function dynamicElementColor(element: HudElement, percent?: number) {
  if (element.binding === 'mqttConnected') return modeStore.mqttConnected ? '#2cff8c' : '#ff3045'
  if (!element.autoColor || percent === undefined) return element.color
  const warn = element.warnThreshold ?? (element.binding === 'RobotDynamicStatus.heat' ? 0.7 : 0.5)
  const danger = element.dangerThreshold ?? (element.binding === 'RobotDynamicStatus.heat' ? 0.9 : 0.25)
  if (element.binding === 'RobotDynamicStatus.hp') {
    if (percent <= danger) return element.dangerColor ?? '#ff3045'
    if (percent <= warn) return element.warnColor ?? '#ffc93a'
    return element.color
  }
  if (element.binding === 'RobotDynamicStatus.heat') {
    if (percent >= danger) return element.dangerColor ?? '#ff3045'
    if (percent >= warn) return element.warnColor ?? '#ffc93a'
    return element.color
  }
  if (percent <= danger) return element.dangerColor ?? '#ff3045'
  if (percent <= warn) return element.warnColor ?? '#ffc93a'
  return element.color
}

function resizeCanvas() {
  const canvas = canvasRef.value
  if (!canvas) return
  const ratio = window.devicePixelRatio || 1
  const width = canvas.clientWidth
  const height = canvas.clientHeight
  canvas.width = Math.floor(width * ratio)
  canvas.height = Math.floor(height * ratio)
  const ctx = canvas.getContext('2d')
  if (ctx) ctx.setTransform(ratio, 0, 0, ratio, 0, 0)
  draw()
}

function normalizeBox(element: HudElement) {
  return {
    x: Math.min(element.x, element.x + element.w),
    y: Math.min(element.y, element.y + element.h),
    w: Math.abs(element.w),
    h: Math.abs(element.h),
  }
}

function drawHandles(ctx: CanvasRenderingContext2D, element: HudElement) {
  ctx.save()
  ctx.fillStyle = '#ffffff'
  ctx.strokeStyle = '#02070d'
  ctx.lineWidth = 1
  if (element.type === 'line') {
    const points = [
      [element.x, element.y],
      [element.x + element.w, element.y + element.h],
    ]
    points.forEach(([x, y]) => {
      ctx.beginPath()
      ctx.arc(x, y, 6, 0, Math.PI * 2)
      ctx.fill()
      ctx.stroke()
    })
  } else {
    const box = normalizeBox(element)
    ctx.fillRect(box.x + box.w - 5, box.y + box.h - 5, 10, 10)
    ctx.strokeRect(box.x + box.w - 5, box.y + box.h - 5, 10, 10)
  }
  ctx.restore()
}

function drawElement(ctx: CanvasRenderingContext2D, element: HudElement) {
  const selected = element.id === hudStore.selectedId
  const box = normalizeBox(element)
  ctx.save()
  ctx.strokeStyle = element.color
  ctx.fillStyle = element.color
  ctx.lineWidth = selected ? 3 : 2
  ctx.shadowColor = element.color
  ctx.shadowBlur = 8

  if (element.type === 'line') {
    ctx.beginPath()
    ctx.moveTo(element.x, element.y)
    ctx.lineTo(element.x + element.w, element.y + element.h)
    ctx.stroke()
  } else if (element.type === 'rect') {
    ctx.strokeRect(box.x, box.y, box.w, box.h)
  } else if (element.type === 'circle') {
    ctx.beginPath()
    ctx.ellipse(box.x + box.w / 2, box.y + box.h / 2, Math.max(4, box.w / 2), Math.max(4, box.h / 2), 0, 0, Math.PI * 2)
    ctx.stroke()
  } else if (element.type === 'text') {
    ctx.font = '700 18px Segoe UI'
    ctx.textBaseline = 'top'
    ctx.fillText(String(bindingValue(element) || element.text || 'TEXT'), element.x, element.y)
  } else if (element.type === 'bar') {
    const percent = Math.max(0, Math.min(1, bindingRatio(element)))
    const fillColor = dynamicElementColor(element, percent)
    ctx.strokeStyle = fillColor
    ctx.fillStyle = fillColor
    ctx.shadowColor = fillColor
    ctx.strokeRect(box.x, box.y, box.w, Math.max(10, box.h))
    ctx.globalAlpha = 0.65
    ctx.fillRect(box.x + 2, box.y + 2, Math.max(0, box.w - 4) * percent, Math.max(0, box.h - 4))
  } else if (element.type === 'statusLight') {
    ctx.fillStyle = dynamicElementColor(element)
    ctx.shadowColor = ctx.fillStyle
    ctx.beginPath()
    ctx.arc(box.x + box.w / 2, box.y + box.h / 2, Math.max(6, Math.min(box.w, box.h) / 2), 0, Math.PI * 2)
    ctx.fill()
  }

  ctx.shadowBlur = 0
  if (selected && interactive.value) {
    ctx.setLineDash([5, 4])
    ctx.strokeStyle = '#ffffff'
    if (element.type === 'line') {
      ctx.strokeRect(box.x - 8, box.y - 8, box.w + 16, box.h + 16)
    } else {
      ctx.strokeRect(box.x - 5, box.y - 5, box.w + 10, box.h + 10)
    }
    ctx.setLineDash([])
    drawHandles(ctx, element)
  }
  ctx.restore()
}

function draw() {
  const canvas = canvasRef.value
  const ctx = canvas?.getContext('2d')
  if (!canvas || !ctx) return
  ctx.clearRect(0, 0, canvas.clientWidth, canvas.clientHeight)
  hudStore.elements.forEach((element) => drawElement(ctx, element))
  if (hudStore.editMode) {
    ctx.save()
    ctx.fillStyle = 'rgba(0, 229, 255, 0.85)'
    ctx.font = '700 12px Segoe UI'
    ctx.fillText(hudStore.locked ? 'HUD LOCKED' : `HUD ${hudStore.activeTool.toUpperCase()} / ${hudStore.templateName}`, 18, canvas.clientHeight - 26)
    ctx.restore()
  }
}

function distanceToSegment(px: number, py: number, ax: number, ay: number, bx: number, by: number) {
  const dx = bx - ax
  const dy = by - ay
  const lenSq = dx * dx + dy * dy
  const t = lenSq === 0 ? 0 : Math.max(0, Math.min(1, ((px - ax) * dx + (py - ay) * dy) / lenSq))
  const x = ax + t * dx
  const y = ay + t * dy
  return Math.hypot(px - x, py - y)
}

function hitHandle(element: HudElement, x: number, y: number): DragMode | null {
  if (element.type === 'line') {
    if (Math.hypot(x - element.x, y - element.y) <= 10) return 'line-start'
    if (Math.hypot(x - (element.x + element.w), y - (element.y + element.h)) <= 10) return 'line-end'
    return null
  }
  const box = normalizeBox(element)
  if (Math.abs(x - (box.x + box.w)) <= 12 && Math.abs(y - (box.y + box.h)) <= 12) return 'resize'
  return null
}

function hitElement(element: HudElement, x: number, y: number) {
  if (element.type === 'line') {
    return distanceToSegment(x, y, element.x, element.y, element.x + element.w, element.y + element.h) <= 10
  }
  const box = normalizeBox(element)
  return x >= box.x - 8 && x <= box.x + box.w + 8 && y >= box.y - 8 && y <= box.y + box.h + 8
}

function hitTest(x: number, y: number) {
  for (const element of [...hudStore.elements].reverse()) {
    const handle = hitHandle(element, x, y)
    if (handle) return { element, handle }
    if (hitElement(element, x, y)) return { element, handle: 'move' as DragMode }
  }
  return null
}

function canvasPoint(event: MouseEvent) {
  const rect = canvasRef.value?.getBoundingClientRect()
  return {
    x: event.clientX - (rect?.left ?? 0),
    y: event.clientY - (rect?.top ?? 0),
  }
}

function handleMouseDown(event: MouseEvent) {
  if (!interactive.value || inputStore.fpsMode) return
  const point = canvasPoint(event)
  const hit = hitTest(point.x, point.y)

  if (hit) {
    hudStore.activeTool = 'select'
    hudStore.selectedId = hit.element.id
    hudStore.captureHistory()
    dragState = {
      id: hit.element.id,
      mode: hit.handle,
      startX: point.x,
      startY: point.y,
      originX: point.x,
      originY: point.y,
      element: { ...hit.element },
    }
    draw()
    return
  }

  if (hudStore.activeTool === 'select') {
    hudStore.selectedId = null
    draw()
    return
  }

  const created = hudStore.addElementAt(hudStore.activeTool, point.x, point.y, hudStore.currentColor)
  if (!created) return
  const patch = created.type === 'line' ? { w: 1, h: 1 } : { w: 1, h: created.type === 'bar' ? 18 : 1 }
  hudStore.updateElement(created.id, patch, false)
  dragState = {
    id: created.id,
    mode: 'draw',
    startX: point.x,
    startY: point.y,
    originX: point.x,
    originY: point.y,
    element: { ...created, ...patch },
  }
  draw()
}

function handleMouseMove(event: MouseEvent) {
  if (!dragState || !interactive.value) return
  const point = canvasPoint(event)
  const dx = point.x - dragState.startX
  const dy = point.y - dragState.startY
  const element = dragState.element

  if (dragState.mode === 'move') {
    hudStore.updateElement(dragState.id, { x: element.x + dx, y: element.y + dy }, false)
  } else if (dragState.mode === 'resize') {
    hudStore.updateElement(dragState.id, { w: element.w + dx, h: element.h + dy }, false)
  } else if (dragState.mode === 'line-start') {
    const endX = element.x + element.w
    const endY = element.y + element.h
    hudStore.updateElement(dragState.id, { x: point.x, y: point.y, w: endX - point.x, h: endY - point.y }, false)
  } else if (dragState.mode === 'line-end' || dragState.mode === 'draw') {
    hudStore.updateElement(dragState.id, { w: point.x - dragState.originX, h: point.y - dragState.originY }, false)
  }
  draw()
}

function handleMouseUp() {
  if (dragState) hudStore.save()
  dragState = null
}

onMounted(() => {
  hudStore.restore()
  resizeCanvas()
  window.addEventListener('resize', resizeCanvas)
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', resizeCanvas)
})

watch(
  () => [hudStore.elements, hudStore.selectedId, modeStore.mqttConnected, videoStore.fps, hudStore.editMode, hudStore.activeTool],
  draw,
  { deep: true },
)
</script>

<template>
  <canvas
    ref="canvasRef"
    class="hud-editor-overlay"
    :class="{ editing: hudStore.editMode, locked: hudStore.locked }"
    @mousedown="handleMouseDown"
    @mousemove="handleMouseMove"
    @mouseup="handleMouseUp"
    @mouseleave="handleMouseUp"
  />
</template>

<style scoped>
.hud-editor-overlay {
  position: absolute;
  inset: 0;
  z-index: 8;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.hud-editor-overlay.editing {
  cursor: crosshair;
  pointer-events: auto;
}

.hud-editor-overlay.locked {
  cursor: default;
  pointer-events: none;
}
</style>
