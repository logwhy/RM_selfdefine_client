export interface VideoStatsPayload {
  currentMode: ClientMode
  currentVideoSource: VideoSource
  currentCodecMode: CodecMode
  currentDecoderName: string
  decoderInitSuccess: boolean
  streamAlive: boolean
  packetLossCount: number
  lastFrameAt: string | null
  fps: number
  packetsReceived: number
  activeFrames: number
  timeoutDroppedFrames: number
  incompleteFrames: number
  readyFrames: number
  decoderResetCount: number
  lastDecodeCostMs: number
  latestFrameAgeMs: number | null
  isRenderingRealFrame: boolean
  realDecoderEnabled: boolean
  stubDecoderEnabled: boolean
  customBlockPacketsReceived: number
  customBlockBytesReceived: number
  customBlockReadyFrames: number
  customBlockInvalidPackets: number
  customBlockParserMode: CustomBlockParserMode
  customBlockMockActive: boolean
}

export interface VideoCommandResult {
  success: boolean
  message: string
}

export interface LatestFramePayload {
  version: number
  width: number
  height: number
  rgba: number[]
  producedAtMs: number
}

export interface DecoderModeResult {
  realDecoderEnabled: boolean
  stubDecoderEnabled: boolean
}

export type ClientMode = 'normal' | 'hero_lob'
export type VideoSource = 'udp_hevc' | 'custombyteblock_h264' | 'mock'
export type CodecMode = 'auto' | 'hevc' | 'h264'
export type CustomBlockParserMode = 'raw_annexb_stream' | 'packetized_frame'

export interface VideoPipelineConfigInput {
  currentMode: ClientMode
  currentVideoSource: VideoSource
  currentCodecMode: CodecMode
  customBlockParserMode: CustomBlockParserMode
}
