import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useVideoStore } from '../stores/video'
import {
  getDecoderMode,
  resetVideoStats,
  setVideoPipelineConfig,
  startHeroLobVideo,
  startMockHeroLobH264,
  startMockVideoSource,
  startVideo,
  stopMockHeroLobH264,
  stopMockVideoSource,
  stopVideo,
  subscribeVideoStats,
} from '../services/videoBridge'
import type { CustomBlockParserMode } from '../types/video'

const DEFAULT_VIDEO_PORT = 3334

export function useVideoStream() {
  const videoStore = useVideoStore()
  const port = ref(DEFAULT_VIDEO_PORT)
  const videoCommandMessage = ref('')

  let unlisten: (() => void) | null = null

  onMounted(async () => {
    videoStore.restorePipelineConfig()
    const mode = await getDecoderMode()
    videoStore.realDecoderEnabled = mode.realDecoderEnabled
    videoStore.stubDecoderEnabled = mode.stubDecoderEnabled
    unlisten = await subscribeVideoStats((payload) => {
      videoStore.applyVideoStats(payload)
    })
  })

  onBeforeUnmount(() => {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  })

  async function handleStartVideo() {
    try {
      const config = {
        currentMode: 'normal' as const,
        currentVideoSource: 'udp_hevc' as const,
        currentCodecMode: 'hevc' as const,
        customBlockParserMode: videoStore.customBlockParserMode,
      }
      videoStore.applyPipelineConfig(config)
      videoStore.savePipelineConfig()
      await setVideoPipelineConfig(config)
      const result = await startVideo(port.value, 'hevc')
      videoCommandMessage.value = result.message
    } catch (error) {
      videoCommandMessage.value = `启动视频失败: ${String(error)}`
      throw error
    }
  }

  async function handleStopVideo() {
    try {
      const result = await stopVideo()
      await resetVideoStats()
      videoStore.latestFrameVersion = 0
      videoCommandMessage.value = result.message
    } catch (error) {
      videoCommandMessage.value = `停止视频失败: ${String(error)}`
      throw error
    }
  }

  async function handleStartMockVideo() {
    try {
      const result = await startMockVideoSource(port.value)
      videoCommandMessage.value = result.message
    } catch (error) {
      videoCommandMessage.value = `启动 Mock 视频源失败: ${String(error)}`
      throw error
    }
  }

  async function handleStopMockVideo() {
    try {
      const result = await stopMockVideoSource()
      videoCommandMessage.value = result.message
    } catch (error) {
      videoCommandMessage.value = `停止 Mock 视频源失败: ${String(error)}`
      throw error
    }
  }

  async function handleUseNormalMode() {
    try {
      const config = {
        currentMode: 'normal' as const,
        currentVideoSource: 'udp_hevc' as const,
        currentCodecMode: 'hevc' as const,
        customBlockParserMode: videoStore.customBlockParserMode,
      }
      const result = await setVideoPipelineConfig(config)
      videoStore.applyPipelineConfig(config)
      videoStore.savePipelineConfig()
      videoCommandMessage.value = result.message
    } catch (error) {
      videoCommandMessage.value = `切换普通图传模式失败: ${String(error)}`
      throw error
    }
  }

  async function handleUseHeroLobMode(parserMode?: CustomBlockParserMode) {
    try {
      const nextParserMode = parserMode ?? videoStore.customBlockParserMode
      const config = {
        currentMode: 'hero_lob' as const,
        currentVideoSource: 'custombyteblock_h264' as const,
        currentCodecMode: 'h264' as const,
        customBlockParserMode: nextParserMode,
      }
      const configResult = await setVideoPipelineConfig(config)
      const armResult = await startHeroLobVideo()
      videoStore.applyPipelineConfig(config)
      videoStore.savePipelineConfig()
      videoCommandMessage.value = `${configResult.message}; ${armResult.message}`
    } catch (error) {
      videoCommandMessage.value = `切换英雄吊射模式失败: ${String(error)}`
      throw error
    }
  }

  async function handleStartMockHeroLobH264() {
    try {
      const result = await startMockHeroLobH264()
      videoCommandMessage.value = result.message
    } catch (error) {
      videoCommandMessage.value = `启动英雄吊射 Mock H264 失败: ${String(error)}`
      throw error
    }
  }

  async function handleStopMockHeroLobH264() {
    try {
      const result = await stopMockHeroLobH264()
      videoCommandMessage.value = result.message
    } catch (error) {
      videoCommandMessage.value = `停止英雄吊射 Mock H264 失败: ${String(error)}`
      throw error
    }
  }

  return {
    port,
    videoCommandMessage,
    handleStartVideo,
    handleStopVideo,
    handleStartMockVideo,
    handleStopMockVideo,
    handleUseNormalMode,
    handleUseHeroLobMode,
    handleStartMockHeroLobH264,
    handleStopMockHeroLobH264,
  }
}
