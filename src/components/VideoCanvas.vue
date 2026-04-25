<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { getLatestFrame } from '../services/videoBridge'
import { useVideoStore } from '../stores/video'

const props = withDefaults(
  defineProps<{
    offsetX: number
    offsetY: number
    lineWidth: number
    displayScale: number
    showCenterDot: boolean
    crosshairColor?: string
  }>(),
  {
    offsetX: 0,
    offsetY: 0,
    lineWidth: 2,
    displayScale: 1,
    showCenterDot: true,
    crosshairColor: '#19f7ff',
  },
)

const canvasRef = ref<HTMLCanvasElement | null>(null)
const videoStore = useVideoStore()
const hasFrame = ref(false)
let pollingTimer: number | null = null
let rafId: number | null = null
let renderSamples = 0

const crosshairLength = computed(() => 24 * props.displayScale)
const crosshairGap = computed(() => 8 * props.displayScale)
const statusText = computed(() => {
  if (!videoStore.streamAlive) return '视频流未连接'
  if (!hasFrame.value) return '等待解码首帧...'
  return ''
})

function ensureCanvasContext() {
  const canvas = canvasRef.value
  if (!canvas) return null
  const ctx = canvas.getContext('2d')
  if (!ctx) return null

  const width = canvas.clientWidth
  const height = canvas.clientHeight
  const ratio = window.devicePixelRatio || 1
  canvas.width = Math.floor(width * ratio)
  canvas.height = Math.floor(height * ratio)
  ctx.setTransform(ratio, 0, 0, ratio, 0, 0)
  return { ctx, width, height }
}

function drawPlaceholder() {
  const context = ensureCanvasContext()
  if (!context) return
  const { ctx, width, height } = context

  ctx.clearRect(0, 0, width, height)
  ctx.fillStyle = '#02070d'
  ctx.fillRect(0, 0, width, height)
  drawOverlay(ctx, width, height)

  if (statusText.value) {
    ctx.fillStyle = '#8ca6b8'
    ctx.font = '15px Segoe UI'
    ctx.textAlign = 'center'
    ctx.fillText(statusText.value, width / 2, height - 24)
  }
}

function drawOverlay(ctx: CanvasRenderingContext2D, width: number, height: number) {
  const centerX = width / 2 + props.offsetX
  const centerY = height / 2 + props.offsetY
  const length = crosshairLength.value
  const gap = crosshairGap.value

  ctx.strokeStyle = props.crosshairColor
  ctx.shadowColor = props.crosshairColor
  ctx.shadowBlur = 8
  ctx.lineWidth = props.lineWidth
  ctx.beginPath()
  ctx.moveTo(centerX - gap - length, centerY)
  ctx.lineTo(centerX - gap, centerY)
  ctx.moveTo(centerX + gap, centerY)
  ctx.lineTo(centerX + gap + length, centerY)
  ctx.moveTo(centerX, centerY - gap - length)
  ctx.lineTo(centerX, centerY - gap)
  ctx.moveTo(centerX, centerY + gap)
  ctx.lineTo(centerX, centerY + gap + length)
  ctx.stroke()
  ctx.shadowBlur = 0

  if (props.showCenterDot) {
    ctx.fillStyle = '#35ff9b'
    ctx.beginPath()
    ctx.arc(centerX, centerY, Math.max(2, props.lineWidth), 0, Math.PI * 2)
    ctx.fill()
  }
}

async function renderLatestFrame() {
  const renderStart = performance.now()
  try {
    const frame = await getLatestFrame(videoStore.latestFrameVersion || undefined)
    if (frame) {
      const context = ensureCanvasContext()
      if (!context) return
      const { ctx, width, height } = context

      const rgba = Uint8ClampedArray.from(frame.rgba)
      const imageData = new ImageData(rgba, frame.width, frame.height)
      const bitmap = await createImageBitmap(imageData)

      ctx.clearRect(0, 0, width, height)
      ctx.drawImage(bitmap, 0, 0, width, height)
      bitmap.close()
      drawOverlay(ctx, width, height)

      hasFrame.value = true
      videoStore.latestFrameVersion = frame.version
      const renderCost = performance.now() - renderStart
      renderSamples += 1
      videoStore.lastRenderCostMs = Number(renderCost.toFixed(2))
      videoStore.avgEndToEndLatencyMs = Number(
        (((videoStore.avgEndToEndLatencyMs * (renderSamples - 1)) +
          (Date.now() - Number(frame.producedAtMs))) /
          renderSamples
        ).toFixed(2),
      )
      return
    }
  } catch (error) {
    console.warn('poll latest frame failed', error)
  }

  if (!hasFrame.value || !videoStore.streamAlive) {
    drawPlaceholder()
  }
}

function pollLatestFrame() {
  if (rafId !== null) return
  rafId = window.requestAnimationFrame(() => {
    rafId = null
    void renderLatestFrame()
  })
}

function startPolling() {
  if (pollingTimer !== null) window.clearInterval(pollingTimer)
  pollingTimer = window.setInterval(pollLatestFrame, 33)
}

onMounted(() => {
  drawPlaceholder()
  startPolling()
  window.addEventListener('resize', drawPlaceholder)
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', drawPlaceholder)
  if (rafId !== null) window.cancelAnimationFrame(rafId)
  if (pollingTimer !== null) {
    window.clearInterval(pollingTimer)
    pollingTimer = null
  }
})

watch(
  () => [props.offsetX, props.offsetY, props.lineWidth, props.displayScale, props.showCenterDot],
  pollLatestFrame,
)

watch(
  () => videoStore.streamAlive,
  () => {
    if (!videoStore.streamAlive) hasFrame.value = false
    pollLatestFrame()
  },
)
</script>

<template>
  <div class="video-canvas-wrapper">
    <canvas ref="canvasRef" class="video-canvas" />
  </div>
</template>

<style scoped>
.video-canvas-wrapper {
  width: 100%;
  height: 100%;
  min-height: 0;
  border: 1px solid #26384f;
  border-radius: 4px;
  overflow: hidden;
  background:
    radial-gradient(circle at center, rgba(25, 247, 255, 0.08), transparent 34%),
    #02070d;
  box-shadow: inset 0 0 42px rgba(25, 247, 255, 0.08);
}

.video-canvas {
  display: block;
  width: 100%;
  height: 100%;
  background: #02070d;
}
</style>
