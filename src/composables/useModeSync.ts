import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import {
  DEFAULT_MQTT_CLIENT_ID,
  ROBOT_CLIENT_OPTIONS,
  normalizeRobotClientId,
} from '../constants/robotClients'
import { useModeStore } from '../stores/mode'
import { connectMqtt, disconnectMqtt, emitMockModeSync, subscribeModeSync, subscribeRefereeMessages } from '../services/mqttBridge'
import { readFromStorage, writeToStorage } from '../utils/storage'

const OFFICIAL_HOST = '192.168.12.1'
const LOCAL_HOST = '127.0.0.1'
const DEFAULT_HOST = OFFICIAL_HOST
const DEFAULT_PORT = 3333
const STORAGE_KEY = 'hero-deploy-mqtt-endpoint'

export function useModeSync() {
  const modeStore = useModeStore()
  const savedEndpoint = readFromStorage<{ host?: string; port?: number; clientId?: string }>(STORAGE_KEY)
  const host = ref(savedEndpoint?.host ?? DEFAULT_HOST)
  const port = ref(savedEndpoint?.port ?? DEFAULT_PORT)
  const clientId = ref(normalizeRobotClientId(savedEndpoint?.clientId ?? DEFAULT_MQTT_CLIENT_ID))
  const robotOptions = ROBOT_CLIENT_OPTIONS
  const commandMessage = ref('')

  let unlisten: (() => void) | null = null
  let unlistenReferee: (() => void) | null = null

  onMounted(async () => {
    unlisten = await subscribeModeSync((payload) => {
      modeStore.applyModeSync(payload)
    })
    unlistenReferee = await subscribeRefereeMessages((payload) => {
      modeStore.applyRefereeMessage(payload)
    })
  })

  onBeforeUnmount(() => {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
    if (unlistenReferee) {
      unlistenReferee()
      unlistenReferee = null
    }
  })

  watch(clientId, () => {
    writeToStorage(STORAGE_KEY, { host: host.value, port: port.value, clientId: clientId.value })
    if (modeStore.mqttConnected && modeStore.mqttClientId !== clientId.value) {
      commandMessage.value = 'Client ID 已变更，重新连接 MQTT 后生效'
    }
  })

  async function handleConnect() {
    try {
      const normalizedClientId = normalizeRobotClientId(clientId.value)
      clientId.value = normalizedClientId
      writeToStorage(STORAGE_KEY, { host: host.value, port: port.value, clientId: normalizedClientId })
      const result = await connectMqtt({ host: host.value, port: port.value, clientId: normalizedClientId })
      commandMessage.value = result.message
    } catch (error) {
      commandMessage.value = `连接失败: ${String(error)}`
      throw error
    }
  }

  function setMqttEndpoint(nextHost: string, nextPort = DEFAULT_PORT) {
    host.value = nextHost
    port.value = nextPort
    writeToStorage(STORAGE_KEY, { host: host.value, port: port.value, clientId: clientId.value })
    commandMessage.value = `MQTT endpoint set: ${host.value}:${port.value}`
  }

  function useLocalMqttEndpoint() {
    setMqttEndpoint(LOCAL_HOST)
  }

  function useOfficialMqttEndpoint() {
    setMqttEndpoint(OFFICIAL_HOST)
  }

  async function handleDisconnect() {
    try {
      const result = await disconnectMqtt()
      commandMessage.value = result.message
      modeStore.resetModeSync()
    } catch (error) {
      commandMessage.value = `断开失败: ${String(error)}`
      throw error
    }
  }

  async function handleMockToggle() {
    try {
      const result = await emitMockModeSync()
      commandMessage.value = result.message
    } catch (error) {
      commandMessage.value = `Mock 发送失败: ${String(error)}`
      throw error
    }
  }

  return {
    host,
    port,
    clientId,
    robotOptions,
    commandMessage,
    handleConnect,
    handleDisconnect,
    handleMockToggle,
    useLocalMqttEndpoint,
    useOfficialMqttEndpoint,
  }
}
