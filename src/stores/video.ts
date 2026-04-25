import { defineStore } from 'pinia'
import { ref } from 'vue'
import type {
  ClientMode,
  CodecMode,
  CustomBlockParserMode,
  VideoSource,
  VideoStatsPayload,
} from '../types/video'
import { readFromStorage, writeToStorage } from '../utils/storage'

const STORAGE_KEY = 'hero-deploy-video-config'

export const useVideoStore = defineStore('video', () => {
  const frameVersion = ref(0)
  const currentMode = ref<ClientMode>('normal')
  const currentVideoSource = ref<VideoSource>('udp_hevc')
  const currentCodecMode = ref<CodecMode>('hevc')
  const currentDecoderName = ref('-')
  const decoderInitSuccess = ref(false)
  const customBlockParserMode = ref<CustomBlockParserMode>('raw_annexb_stream')
  const heroLobModeEnabled = ref(false)
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
  const stubDecoderEnabled = ref(true)
  const customBlockPacketsReceived = ref(0)
  const customBlockBytesReceived = ref(0)
  const customBlockReadyFrames = ref(0)
  const customBlockInvalidPackets = ref(0)
  const customBlockPacketsPerSecond = ref(0)
  const customBlockBytesPerSecond = ref(0)
  const customBlockBitrateKbps = ref(0)
  const customBlockDroppedBlocks = ref(0)
  const customBlockBufferedBytes = ref(0)
  const customBlockLastReceiveAt = ref<string | null>(null)
  const customBlockNoDataDurationMs = ref<number | null>(null)
  const customBlockMockActive = ref(false)
  const h264SeenSps = ref(false)
  const h264SeenPps = ref(false)
  const h264SeenIdr = ref(false)
  const h264LastNalType = ref<number | null>(null)
  const h264BufferedBytes = ref(0)
  const h264NalUnitsParsed = ref(0)
  const h264FramesSubmittedToDecoder = ref(0)
  const h264FramesDecoded = ref(0)
  const h264FramesDropped = ref(0)
  const h264DecoderErrors = ref(0)
  const h264ConsecutiveDecodeErrors = ref(0)
  const droppedOldFrames = ref(0)
  const droppedByBackpressure = ref(0)
  const decodeInputQueueLen = ref(0)
  const frameRenderQueueLen = ref(0)
  const avgDecodeCostMs = ref(0)
  const maxDecodeCostMs = ref(0)
  const lastRenderCostMs = ref(0)
  const avgEndToEndLatencyMs = ref(0)
  const latestFrameVersion = ref(0)

  function applyVideoStats(payload: VideoStatsPayload) {
    currentMode.value = payload.currentMode
    currentVideoSource.value = payload.currentVideoSource
    currentCodecMode.value = payload.currentCodecMode
    currentDecoderName.value = payload.currentDecoderName || '-'
    decoderInitSuccess.value = payload.decoderInitSuccess
    customBlockParserMode.value = payload.customBlockParserMode
    heroLobModeEnabled.value = payload.currentMode === 'hero_lob'
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
    stubDecoderEnabled.value = payload.stubDecoderEnabled
    customBlockPacketsReceived.value = payload.customBlockPacketsReceived
    customBlockBytesReceived.value = payload.customBlockBytesReceived
    customBlockReadyFrames.value = payload.customBlockReadyFrames
    customBlockInvalidPackets.value = payload.customBlockInvalidPackets
    customBlockPacketsPerSecond.value = Number(payload.customBlockPacketsPerSecond.toFixed(2))
    customBlockBytesPerSecond.value = Number(payload.customBlockBytesPerSecond.toFixed(2))
    customBlockBitrateKbps.value = Number(payload.customBlockBitrateKbps.toFixed(2))
    customBlockDroppedBlocks.value = payload.customBlockDroppedBlocks
    customBlockBufferedBytes.value = payload.customBlockBufferedBytes
    customBlockLastReceiveAt.value = payload.customBlockLastReceiveAt
    customBlockNoDataDurationMs.value = payload.customBlockNoDataDurationMs
    customBlockMockActive.value = payload.customBlockMockActive
    h264SeenSps.value = payload.h264SeenSps
    h264SeenPps.value = payload.h264SeenPps
    h264SeenIdr.value = payload.h264SeenIdr
    h264LastNalType.value = payload.h264LastNalType
    h264BufferedBytes.value = payload.h264BufferedBytes
    h264NalUnitsParsed.value = payload.h264NalUnitsParsed
    h264FramesSubmittedToDecoder.value = payload.h264FramesSubmittedToDecoder
    h264FramesDecoded.value = payload.h264FramesDecoded
    h264FramesDropped.value = payload.h264FramesDropped
    h264DecoderErrors.value = payload.h264DecoderErrors
    h264ConsecutiveDecodeErrors.value = payload.h264ConsecutiveDecodeErrors
    droppedOldFrames.value = payload.droppedOldFrames
    droppedByBackpressure.value = payload.droppedByBackpressure
    decodeInputQueueLen.value = payload.decodeInputQueueLen
    frameRenderQueueLen.value = payload.frameRenderQueueLen
    avgDecodeCostMs.value = Number(payload.avgDecodeCostMs.toFixed(2))
    maxDecodeCostMs.value = Number(payload.maxDecodeCostMs.toFixed(2))
    lastRenderCostMs.value = Number(payload.lastRenderCostMs.toFixed(2))
    avgEndToEndLatencyMs.value = Number(payload.avgEndToEndLatencyMs.toFixed(2))
    frameVersion.value += 1
  }

  function applyPipelineConfig(config: {
    currentMode: ClientMode
    currentVideoSource: VideoSource
    currentCodecMode: CodecMode
    customBlockParserMode: CustomBlockParserMode
  }) {
    currentMode.value = config.currentMode
    currentVideoSource.value = config.currentVideoSource
    currentCodecMode.value = config.currentCodecMode
    customBlockParserMode.value = config.customBlockParserMode
    heroLobModeEnabled.value = config.currentMode === 'hero_lob'
  }

  function savePipelineConfig() {
    return writeToStorage(STORAGE_KEY, {
      currentMode: currentMode.value,
      currentVideoSource: currentVideoSource.value,
      currentCodecMode: currentCodecMode.value,
      customBlockParserMode: customBlockParserMode.value,
    })
  }

  function restorePipelineConfig() {
    const saved = readFromStorage<{
      currentMode?: ClientMode
      currentVideoSource?: VideoSource
      currentCodecMode?: CodecMode
      customBlockParserMode?: CustomBlockParserMode
    }>(STORAGE_KEY)
    if (!saved) return
    applyPipelineConfig({
      currentMode: saved.currentMode ?? 'normal',
      currentVideoSource: saved.currentVideoSource ?? 'udp_hevc',
      currentCodecMode: saved.currentCodecMode ?? 'hevc',
      customBlockParserMode: saved.customBlockParserMode ?? 'raw_annexb_stream',
    })
  }

  return {
    frameVersion,
    currentMode,
    currentVideoSource,
    currentCodecMode,
    currentDecoderName,
    decoderInitSuccess,
    customBlockParserMode,
    heroLobModeEnabled,
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
    stubDecoderEnabled,
    customBlockPacketsReceived,
    customBlockBytesReceived,
    customBlockReadyFrames,
    customBlockInvalidPackets,
    customBlockPacketsPerSecond,
    customBlockBytesPerSecond,
    customBlockBitrateKbps,
    customBlockDroppedBlocks,
    customBlockBufferedBytes,
    customBlockLastReceiveAt,
    customBlockNoDataDurationMs,
    customBlockMockActive,
    h264SeenSps,
    h264SeenPps,
    h264SeenIdr,
    h264LastNalType,
    h264BufferedBytes,
    h264NalUnitsParsed,
    h264FramesSubmittedToDecoder,
    h264FramesDecoded,
    h264FramesDropped,
    h264DecoderErrors,
    h264ConsecutiveDecodeErrors,
    droppedOldFrames,
    droppedByBackpressure,
    decodeInputQueueLen,
    frameRenderQueueLen,
    avgDecodeCostMs,
    maxDecodeCostMs,
    lastRenderCostMs,
    avgEndToEndLatencyMs,
    latestFrameVersion,
    applyVideoStats,
    applyPipelineConfig,
    savePipelineConfig,
    restorePipelineConfig,
  }
})
