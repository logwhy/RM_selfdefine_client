export interface StreamStatus {
  connected: boolean
  startedAt: string | null
}

export function getStreamStatusPlaceholder(): StreamStatus {
  return {
    connected: false,
    startedAt: null,
  }
}
