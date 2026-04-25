import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { VideoStatsPayload } from '../types/video'

export const useVideoStore = defineStore('video', () => {
  const frameVersion = ref(0)
  const fps = ref(0)
  const decoderLatencyMs = ref(0)
  const packetLossCount = ref(0)
  const streamAlive = ref(false)
  const lastFrameAt = ref<string | null>(null)
  const packetsReceived = ref(0)
  const activeFrames = ref(0)
  const timeoutDroppedFrames = ref(0)
  const incompleteFrames = ref(0)
  const readyFrames = ref(0)
  const decoderResetCount = ref(0)
  const lastDecodeCostMs = ref(0)
  const latestFrameAgeMs = ref<number | null>(null)
  const isRenderingRealFrame = ref(false)
  const realDecoderEnabled = ref(false)
  const mockDecoderEnabled = ref(true)
  const latestFrameVersion = ref(0)

  function applyVideoStats(payload: VideoStatsPayload) {
    streamAlive.value = payload.streamAlive
    packetLossCount.value = payload.packetLossCount
    lastFrameAt.value = payload.lastFrameAt
    fps.value = Number(payload.fps.toFixed(2))
    packetsReceived.value = payload.packetsReceived
    activeFrames.value = payload.activeFrames
    timeoutDroppedFrames.value = payload.timeoutDroppedFrames
    incompleteFrames.value = payload.incompleteFrames
    readyFrames.value = payload.readyFrames
    decoderResetCount.value = payload.decoderResetCount
    lastDecodeCostMs.value = Number(payload.lastDecodeCostMs.toFixed(2))
    latestFrameAgeMs.value = payload.latestFrameAgeMs
    isRenderingRealFrame.value = payload.isRenderingRealFrame
    realDecoderEnabled.value = payload.realDecoderEnabled
    mockDecoderEnabled.value = payload.mockDecoderEnabled
    frameVersion.value += 1
  }

  return {
    frameVersion,
    fps,
    decoderLatencyMs,
    packetLossCount,
    streamAlive,
    lastFrameAt,
    packetsReceived,
    activeFrames,
    timeoutDroppedFrames,
    incompleteFrames,
    readyFrames,
    decoderResetCount,
    lastDecodeCostMs,
    latestFrameAgeMs,
    isRenderingRealFrame,
    realDecoderEnabled,
    mockDecoderEnabled,
    latestFrameVersion,
    applyVideoStats,
  }
})
