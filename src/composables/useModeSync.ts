import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useModeStore } from '../stores/mode'
import { connectMqtt, disconnectMqtt, emitMockModeSync, subscribeModeSync, subscribeRefereeMessages } from '../services/mqttBridge'

const DEFAULT_HOST = '192.168.12.1'
const DEFAULT_PORT = 3333

export function useModeSync() {
  const modeStore = useModeStore()
  const host = ref(DEFAULT_HOST)
  const port = ref(DEFAULT_PORT)
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

  async function handleConnect() {
    try {
      const result = await connectMqtt({ host: host.value, port: port.value })
      commandMessage.value = result.message
    } catch (error) {
      commandMessage.value = `连接失败: ${String(error)}`
      throw error
    }
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
    commandMessage,
    handleConnect,
    handleDisconnect,
    handleMockToggle,
  }
}
