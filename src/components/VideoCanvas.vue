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
  }>(),
  {
    offsetX: 0,
    offsetY: 0,
    lineWidth: 2,
    displayScale: 1,
    showCenterDot: true,
  },
)

const canvasRef = ref<HTMLCanvasElement | null>(null)
const videoStore = useVideoStore()
const hasFrame = ref(false)
let pollingTimer: number | null = null
const crosshairLength = computed(() => 24 * props.displayScale)
const crosshairGap = computed(() => 8 * props.displayScale)
const statusText = computed(() => {
  if (!videoStore.streamAlive) {
    return '视频流未连接'
  }
  if (!hasFrame.value) {
    return '等待解码首帧...'
  }
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
  ctx.fillStyle = '#000000'
  ctx.fillRect(0, 0, width, height)
  drawOverlay(ctx, width, height)

  if (statusText.value) {
    ctx.fillStyle = '#7c8596'
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

  ctx.strokeStyle = '#34d399'
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

  if (props.showCenterDot) {
    ctx.fillStyle = '#34d399'
    ctx.beginPath()
    ctx.arc(centerX, centerY, Math.max(2, props.lineWidth), 0, Math.PI * 2)
    ctx.fill()
  }
}

async function pollLatestFrame() {
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
      return
    }
  } catch (error) {
    console.warn('poll latest frame failed', error)
  }

  drawPlaceholder()
}

function startPolling() {
  if (pollingTimer !== null) {
    window.clearInterval(pollingTimer)
  }
  pollingTimer = window.setInterval(() => {
    void pollLatestFrame()
  }, 33)
}

onMounted(() => {
  drawPlaceholder()
  startPolling()
  window.addEventListener('resize', drawPlaceholder)
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', drawPlaceholder)
  if (pollingTimer !== null) {
    window.clearInterval(pollingTimer)
    pollingTimer = null
  }
})

watch(
  () => [props.offsetX, props.offsetY, props.lineWidth, props.displayScale, props.showCenterDot],
  () => {
    void pollLatestFrame()
  },
)

watch(
  () => videoStore.streamAlive,
  () => {
    if (!videoStore.streamAlive) {
      hasFrame.value = false
    }
    void pollLatestFrame()
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
  min-height: 480px;
  border-radius: 10px;
  border: 1px solid #2d333d;
  overflow: hidden;
}

.video-canvas {
  display: block;
  width: 100%;
  height: 100%;
  background: #000;
}
</style>
