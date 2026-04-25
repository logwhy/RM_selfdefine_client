export interface VideoStatsPayload {
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
  mockDecoderEnabled: boolean
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
  mockDecoderEnabled: boolean
}
