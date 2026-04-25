import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useVideoStore } from '../stores/video'
import {
  getDecoderMode,
  resetVideoStats,
  startMockVideoSource,
  startVideo,
  stopMockVideoSource,
  stopVideo,
  subscribeVideoStats,
} from '../services/videoBridge'

const DEFAULT_VIDEO_PORT = 3334

export function useVideoStream() {
  const videoStore = useVideoStore()
  const port = ref(DEFAULT_VIDEO_PORT)
  const videoCommandMessage = ref('')

  let unlisten: (() => void) | null = null

  onMounted(async () => {
    const mode = await getDecoderMode()
    videoStore.realDecoderEnabled = mode.realDecoderEnabled
    videoStore.mockDecoderEnabled = mode.mockDecoderEnabled
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
      const result = await startVideo(port.value)
      videoCommandMessage.value = result.message
    } catch (error) {
      videoCommandMessage.value = `启动视频失败: ${String(error)}`
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
    }
  }

  async function handleStartMockVideo() {
    try {
      const result = await startMockVideoSource(port.value)
      videoCommandMessage.value = result.message
    } catch (error) {
      videoCommandMessage.value = `启动Mock视频源失败: ${String(error)}`
    }
  }

  async function handleStopMockVideo() {
    try {
      const result = await stopMockVideoSource()
      videoCommandMessage.value = result.message
    } catch (error) {
      videoCommandMessage.value = `停止Mock视频源失败: ${String(error)}`
    }
  }

  return {
    port,
    videoCommandMessage,
    handleStartVideo,
    handleStopVideo,
    handleStartMockVideo,
    handleStopMockVideo,
  }
}
