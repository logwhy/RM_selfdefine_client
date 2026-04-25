export interface CrosshairConfig {
  crosshairOffsetX: number
  crosshairOffsetY: number
  crosshairWidth: number
  displayScale: number
  showCenterDot: boolean
  crosshairColor?: string
}

export interface CrosshairPreset extends CrosshairConfig {
  id: 1 | 2 | 3
  name: string
}
