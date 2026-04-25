<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useModeStore } from '../stores/mode'
import { useUiStore } from '../stores/ui'
import { useVideoStore } from '../stores/video'
import RmStatusPill from './RmStatusPill.vue'

defineProps<{
  lastError: string
}>()

const modeStore = useModeStore()
const uiStore = useUiStore()
const videoStore = useVideoStore()
const customBlockKbps = ref(0)
let bitrateTimer: number | null = null
let lastCustomBlockBytes = 0

const modeText = computed(() => (videoStore.currentMode === 'hero_lob' ? 'HERO LOB' : 'NORMAL'))
const bitrateText = computed(() => `${customBlockKbps.value.toFixed(1)} KB/s`)
const droppedText = computed(() => videoStore.timeoutDroppedFrames + videoStore.incompleteFrames)

onMounted(() => {
  bitrateTimer = window.setInterval(() => {
    const bytes = videoStore.customBlockBytesReceived
    customBlockKbps.value = Math.max(0, bytes - lastCustomBlockBytes) / 1024
    lastCustomBlockBytes = bytes
  }, 1000)
})

onBeforeUnmount(() => {
  if (bitrateTimer !== null) window.clearInterval(bitrateTimer)
})
</script>

<template>
  <footer class="tactical-bottom rm-glass rm-corners">
    <RmStatusPill label="Mode" :value="modeText" :tone="videoStore.currentMode === 'hero_lob' ? 'orange' : 'cyan'" />
    <RmStatusPill label="Deploy" :value="modeStore.deployModeState.toUpperCase()" :tone="modeStore.deployModeState === 'active' ? 'green' : 'muted'" />
    <RmStatusPill label="Custom RX" :value="bitrateText" tone="blue" />
    <RmStatusPill label="Dropped" :value="droppedText" :tone="droppedText > 0 ? 'orange' : 'green'" />
    <RmStatusPill label="Decoder Reset" :value="videoStore.decoderResetCount" :tone="videoStore.decoderResetCount > 0 ? 'orange' : 'green'" />
    <RmStatusPill label="Crosshair" value="DEFAULT" tone="cyan" />
    <RmStatusPill label="Offset" :value="`${uiStore.crosshairOffsetX}, ${uiStore.crosshairOffsetY}`" tone="muted" />
    <div class="last-error">
      <span>LAST ERROR</span>
      <strong>{{ lastError || 'CLEAR' }}</strong>
    </div>
  </footer>
</template>

<style scoped>
.tactical-bottom {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  overflow: hidden;
}

.last-error {
  display: flex;
  min-width: 160px;
  flex: 1;
  justify-content: flex-end;
  gap: 8px;
  color: var(--rm-muted);
  font-size: 11px;
  overflow: hidden;
}

.last-error strong {
  max-width: 42vw;
  color: var(--rm-red);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
