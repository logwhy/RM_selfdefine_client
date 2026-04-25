<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useVideoStore } from '../stores/video'

const videoStore = useVideoStore()
const packetsPerSecond = ref(0)
const bitrateKbps = ref(0)
let timer: number | null = null
let lastPackets = 0
let lastBytes = 0

const showMonitor = computed(() => videoStore.currentMode === 'hero_lob')
const parameterStatus = computed(() => (videoStore.decoderInitSuccess ? 'READY' : 'WAIT'))

onMounted(() => {
  timer = window.setInterval(() => {
    const packets = videoStore.customBlockPacketsReceived
    const bytes = videoStore.customBlockBytesReceived
    packetsPerSecond.value = Math.max(0, packets - lastPackets)
    bitrateKbps.value = Math.max(0, bytes - lastBytes) / 1024
    lastPackets = packets
    lastBytes = bytes
  }, 1000)
})

onBeforeUnmount(() => {
  if (timer !== null) window.clearInterval(timer)
})
</script>

<template>
  <aside v-if="showMonitor" class="rm-link-monitor rm-glass-panel">
    <h3>链路监控</h3>
    <div class="monitor-grid">
      <span>CustomBlock</span><b>{{ packetsPerSecond }}/s</b>
      <span>Bitrate</span><b>{{ bitrateKbps.toFixed(1) }} KB/s</b>
      <span>Decoder Reset</span><b>{{ videoStore.decoderResetCount }}</b>
      <span>SPS/PPS/IDR</span><b>{{ parameterStatus }}</b>
      <span>Stream</span><b :class="videoStore.streamAlive ? 'ok' : 'bad'">{{ videoStore.streamAlive ? 'ALIVE' : 'WAIT' }}</b>
      <span>Codec</span><b>{{ videoStore.currentCodecMode.toUpperCase() }}</b>
    </div>
    <div class="wave">
      <i v-for="i in 18" :key="i" :style="{ height: `${18 + ((i * 7 + packetsPerSecond) % 28)}px` }" />
    </div>
  </aside>
</template>

<style scoped>
.rm-link-monitor {
  position: absolute;
  right: 24px;
  bottom: 72px;
  z-index: 13;
  width: 286px;
  padding: 13px;
  border-color: rgba(0, 229, 255, 0.22);
}

h3 {
  margin: 0 0 10px;
  color: var(--rm-op-cyan);
  font-size: 13px;
  letter-spacing: 0.12em;
}

.monitor-grid {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 7px 12px;
  font-size: 12px;
}

.monitor-grid span {
  color: var(--rm-op-muted);
}

.monitor-grid b {
  color: var(--rm-op-text);
}

.monitor-grid .ok {
  color: var(--rm-op-green);
}

.monitor-grid .bad {
  color: var(--rm-op-red);
}

.wave {
  height: 48px;
  display: flex;
  align-items: end;
  gap: 4px;
  margin-top: 12px;
}

.wave i {
  width: 6px;
  background: linear-gradient(180deg, var(--rm-op-cyan), rgba(43, 124, 255, 0.32));
  box-shadow: 0 0 10px rgba(0, 229, 255, 0.24);
}

@media (max-width: 1180px) {
  .rm-link-monitor {
    display: none;
  }
}
</style>
