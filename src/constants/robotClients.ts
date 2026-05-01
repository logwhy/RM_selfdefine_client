export interface RobotClientOption {
  [key: string]: unknown
  label: string
  value: string
}

export const DEFAULT_MQTT_CLIENT_ID = '1'

export const ROBOT_CLIENT_OPTIONS: RobotClientOption[] = [
  { label: '1：红方英雄机器人', value: '1' },
  { label: '2：红方工程机器人', value: '2' },
  { label: '3：红方步兵机器人 3', value: '3' },
  { label: '4：红方步兵机器人 4', value: '4' },
  { label: '5：红方步兵机器人 5', value: '5' },
  { label: '6：红方空中机器人', value: '6' },
  { label: '7：红方哨兵机器人', value: '7' },
  { label: '8：红方飞镖', value: '8' },
  { label: '9：红方雷达', value: '9' },
  { label: '10：红方前哨站', value: '10' },
  { label: '11：红方基地', value: '11' },
  { label: '101：蓝方英雄机器人', value: '101' },
  { label: '102：蓝方工程机器人', value: '102' },
  { label: '103：蓝方步兵机器人 3', value: '103' },
  { label: '104：蓝方步兵机器人 4', value: '104' },
  { label: '105：蓝方步兵机器人 5', value: '105' },
  { label: '106：蓝方空中机器人', value: '106' },
  { label: '107：蓝方哨兵机器人', value: '107' },
  { label: '108：蓝方飞镖', value: '108' },
  { label: '109：蓝方雷达', value: '109' },
  { label: '110：蓝方前哨站', value: '110' },
  { label: '111：蓝方基地', value: '111' },
]

export function normalizeRobotClientId(value: unknown): string {
  const stringValue = typeof value === 'string' ? value : String(value ?? '')
  return ROBOT_CLIENT_OPTIONS.some((option) => option.value === stringValue)
    ? stringValue
    : DEFAULT_MQTT_CLIENT_ID
}

export function getRobotClientLabel(clientId: string | null | undefined): string {
  const normalized = normalizeRobotClientId(clientId)
  return ROBOT_CLIENT_OPTIONS.find((option) => option.value === normalized)?.label ?? normalized
}
