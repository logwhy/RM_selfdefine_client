import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useModeStore } from '../stores/mode'
import { connectMqtt, disconnectMqtt, emitMockModeSync, subscribeModeSync } from '../services/mqttBridge'

const DEFAULT_HOST = '127.0.0.1'
const DEFAULT_PORT = 1883

export function useModeSync() {
  const modeStore = useModeStore()
  const host = ref(DEFAULT_HOST)
  const port = ref(DEFAULT_PORT)
  const commandMessage = ref('')

  let unlisten: (() => void) | null = null

  onMounted(async () => {
    unlisten = await subscribeModeSync((payload) => {
      modeStore.applyModeSync(payload)
    })
  })

  onBeforeUnmount(() => {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  })

  async function handleConnect() {
    try {
      const result = await connectMqtt({ host: host.value, port: port.value })
      commandMessage.value = result.message
    } catch (error) {
      commandMessage.value = `连接失败: ${String(error)}`
    }
  }

  async function handleDisconnect() {
    try {
      const result = await disconnectMqtt()
      commandMessage.value = result.message
      modeStore.resetModeSync()
    } catch (error) {
      commandMessage.value = `断开失败: ${String(error)}`
    }
  }

  async function handleMockToggle() {
    try {
      const result = await emitMockModeSync()
      commandMessage.value = result.message
    } catch (error) {
      commandMessage.value = `Mock 发送失败: ${String(error)}`
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
