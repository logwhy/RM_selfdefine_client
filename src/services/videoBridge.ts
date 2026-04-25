import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type {
  DecoderModeResult,
  LatestFramePayload,
  VideoPipelineConfigInput,
  VideoCommandResult,
  VideoStatsPayload,
} from '../types/video'

const VIDEO_STATS_EVENT = 'video_stats'
const NOOP_UNLISTEN: UnlistenFn = async () => {}

function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

export async function startVideo(port: number, codecMode: 'auto' | 'hevc' | 'h264' = 'hevc'): Promise<VideoCommandResult> {
  if (!isTauriRuntime()) {
    return { success: false, message: 'Tauri backend not available in browser mode' }
  }
  return invoke<VideoCommandResult>('start_video', { port, codecMode })
}

export async function stopVideo(): Promise<VideoCommandResult> {
  if (!isTauriRuntime()) {
    return { success: false, message: 'Tauri backend not available in browser mode' }
  }
  return invoke<VideoCommandResult>('stop_video')
}

export async function startMockVideoSource(port: number): Promise<VideoCommandResult> {
  if (!isTauriRuntime()) {
    return { success: false, message: 'Tauri backend not available in browser mode' }
  }
  return invoke<VideoCommandResult>('start_mock_video', { port })
}

export async function stopMockVideoSource(): Promise<VideoCommandResult> {
  if (!isTauriRuntime()) {
    return { success: false, message: 'Tauri backend not available in browser mode' }
  }
  return invoke<VideoCommandResult>('stop_mock_video')
}

export async function setVideoPipelineConfig(
  config: VideoPipelineConfigInput,
): Promise<VideoCommandResult> {
  if (!isTauriRuntime()) {
    return { success: false, message: 'Tauri backend not available in browser mode' }
  }
  return invoke<VideoCommandResult>('set_video_pipeline_config', { config })
}

export async function startHeroLobVideo(): Promise<VideoCommandResult> {
  if (!isTauriRuntime()) {
    return { success: false, message: 'Tauri backend not available in browser mode' }
  }
  return invoke<VideoCommandResult>('start_hero_lob_video')
}

export async function startMockHeroLobH264(): Promise<VideoCommandResult> {
  if (!isTauriRuntime()) {
    return { success: false, message: 'Tauri backend not available in browser mode' }
  }
  return invoke<VideoCommandResult>('start_mock_hero_lob_h264')
}

export async function stopMockHeroLobH264(): Promise<VideoCommandResult> {
  if (!isTauriRuntime()) {
    return { success: false, message: 'Tauri backend not available in browser mode' }
  }
  return invoke<VideoCommandResult>('stop_mock_hero_lob_h264')
}

export async function resetVideoStats(): Promise<VideoCommandResult> {
  if (!isTauriRuntime()) {
    return { success: false, message: 'Tauri backend not available in browser mode' }
  }
  return invoke<VideoCommandResult>('reset_video_stats')
}

export async function getLatestFrame(
  sinceVersion?: number,
): Promise<LatestFramePayload | null> {
  if (!isTauriRuntime()) {
    return null
  }
  return invoke<LatestFramePayload | null>('get_latest_frame', {
    sinceVersion,
  })
}

export async function getDecoderMode(): Promise<DecoderModeResult> {
  if (!isTauriRuntime()) {
    return { realDecoderEnabled: false, stubDecoderEnabled: true }
  }
  return invoke<DecoderModeResult>('get_decoder_mode')
}

export async function subscribeVideoStats(
  handler: (payload: VideoStatsPayload) => void,
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return NOOP_UNLISTEN
  }
  return listen<VideoStatsPayload>(VIDEO_STATS_EVENT, (event) => {
    handler(event.payload)
  })
}
