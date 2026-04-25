<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useModeStore } from '../stores/mode'
import { useVideoStore } from '../stores/video'
import RmStatusPill from './RmStatusPill.vue'

const props = defineProps<{
  lastError: string
  runtimeText: string
}>()

const modeStore = useModeStore()
const videoStore = useVideoStore()
const bitrateKbps = ref(0)
let bitrateTimer: number | null = null
let lastBytes = 0

const modeLabel = computed(() => (videoStore.currentMode === 'hero_lob' ? 'HERO LOB' : 'NORMAL'))
const sourceLabel = computed(() =>
  videoStore.currentVideoSource === 'custombyteblock_h264' ? '0x0310 CustomBlock' : 'UDP 3334',
)
const codecLabel = computed(() => videoStore.currentCodecMode.toUpperCase())
const bitrateLabel = computed(() => {
  if (bitrateKbps.value <= 0) return '--'
  return `${bitrateKbps.value.toFixed(1)} KB/s`
})

onMounted(() => {
  bitrateTimer = window.setInterval(() => {
    const bytes = videoStore.currentVideoSource === 'custombyteblock_h264'
      ? videoStore.customBlockBytesReceived
      : videoStore.packetsReceived * 1200
    bitrateKbps.value = Math.max(0, bytes - lastBytes) / 1024
    lastBytes = bytes
  }, 1000)
})

onBeforeUnmount(() => {
  if (bitrateTimer !== null) window.clearInterval(bitrateTimer)
})
</script>

<template>
  <header class="tactical-topbar rm-glass rm-corners">
    <div class="brand-block">
      <span class="brand-main">RM HERO CLIENT</span>
      <span class="brand-sub">HERO DEPLOY / TACTICAL HUD</span>
    </div>

    <div class="status-cluster center-cluster">
      <RmStatusPill label="Mode" :value="modeLabel" :tone="videoStore.currentMode === 'hero_lob' ? 'orange' : 'cyan'" />
      <RmStatusPill label="Source" :value="sourceLabel" tone="blue" />
      <RmStatusPill label="Codec" :value="codecLabel" tone="cyan" />
      <RmStatusPill label="MQTT" :value="modeStore.mqttConnected ? 'ONLINE' : 'OFFLINE'" :tone="modeStore.mqttConnected ? 'green' : 'red'" />
      <RmStatusPill label="Video" :value="videoStore.streamAlive ? 'LIVE' : 'STANDBY'" :tone="videoStore.streamAlive ? 'green' : 'muted'" />
    </div>

    <div class="status-cluster right-cluster">
      <RmStatusPill label="FPS" :value="videoStore.fps || '--'" tone="green" />
      <RmStatusPill label="Bitrate" :value="bitrateLabel" tone="blue" />
      <RmStatusPill label="Latency" :value="`${videoStore.lastDecodeCostMs || 0} ms`" tone="cyan" />
      <RmStatusPill label="Alive" :value="videoStore.streamAlive ? 'YES' : 'NO'" :tone="videoStore.streamAlive ? 'green' : 'red'" />
      <RmStatusPill label="Run" :value="props.runtimeText" tone="muted" />
    </div>

    <div v-if="lastError" class="top-error">{{ lastError }}</div>
  </header>
</template>

<style scoped>
.tactical-topbar {
  position: relative;
  display: grid;
  grid-template-columns: 230px minmax(0, 1fr) auto;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  overflow: hidden;
}

.brand-block {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.brand-main {
  color: var(--rm-text);
  font-size: 16px;
  font-weight: 900;
  letter-spacing: 0.08em;
}

.brand-sub {
  color: var(--rm-cyan);
  font-size: 10px;
  letter-spacing: 0.14em;
}

.status-cluster {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 7px;
  overflow: hidden;
}

.center-cluster {
  justify-content: center;
}

.right-cluster {
  justify-content: flex-end;
}

.top-error {
  position: absolute;
  right: 12px;
  bottom: 2px;
  max-width: 42vw;
  color: var(--rm-red);
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 1180px) {
  .tactical-topbar {
    grid-template-columns: 210px minmax(0, 1fr);
  }

  .right-cluster {
    display: none;
  }
}
</style>
