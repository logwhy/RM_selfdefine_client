import { onMounted } from 'vue'
import { useUiStore } from '../stores/ui'

export function useUiPersistence() {
  const uiStore = useUiStore()

  onMounted(() => {
    uiStore.restore()
  })

  return {
    saveUiConfig: uiStore.save,
    resetUiConfig: uiStore.resetDefaults,
  }
}
