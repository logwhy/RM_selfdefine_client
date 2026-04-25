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
  customBlockPacketsPerSecond: number
  customBlockBytesPerSecond: number
  customBlockBitrateKbps: number
  customBlockDroppedBlocks: number
  customBlockBufferedBytes: number
  customBlockLastReceiveAt: string | null
  customBlockNoDataDurationMs: number | null
  customBlockParserMode: CustomBlockParserMode
  customBlockMockActive: boolean
  h264SeenSps: boolean
  h264SeenPps: boolean
  h264SeenIdr: boolean
  h264LastNalType: number | null
  h264BufferedBytes: number
  h264NalUnitsParsed: number
  h264FramesSubmittedToDecoder: number
  h264FramesDecoded: number
  h264FramesDropped: number
  h264DecoderErrors: number
  h264ConsecutiveDecodeErrors: number
  droppedOldFrames: number
  droppedByBackpressure: number
  decodeInputQueueLen: number
  frameRenderQueueLen: number
  avgDecodeCostMs: number
  maxDecodeCostMs: number
  lastRenderCostMs: number
  avgEndToEndLatencyMs: number
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
