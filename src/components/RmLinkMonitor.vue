<script setup lang="ts">
import { computed } from 'vue'
import { useVideoStore } from '../stores/video'

const videoStore = useVideoStore()

const showMonitor = computed(() => videoStore.currentMode === 'hero_lob')
const parameterStatus = computed(() =>
  videoStore.h264SeenSps && videoStore.h264SeenPps
    ? videoStore.h264SeenIdr
      ? 'SPS/PPS/IDR'
      : 'SPS/PPS'
    : 'WAIT',
)
</script>

<template>
  <aside v-if="showMonitor" class="rm-link-monitor rm-glass-panel">
    <h3>链路监控</h3>
    <div class="monitor-grid">
      <span>CustomByteBlock</span><b>{{ videoStore.customBlockPacketsPerSecond }}/s</b>
      <span>Bitrate</span><b>{{ videoStore.customBlockBitrateKbps }} kbps</b>
      <span>Decoder Reset</span><b>{{ videoStore.decoderResetCount }}</b>
      <span>SPS/PPS/IDR</span><b>{{ parameterStatus }}</b>
      <span>Dropped</span><b>{{ videoStore.droppedByBackpressure + videoStore.droppedOldFrames }}</b>
      <span>Stream</span><b :class="videoStore.streamAlive ? 'ok' : 'bad'">{{ videoStore.streamAlive ? 'ALIVE' : 'WAIT' }}</b>
      <span>Codec</span><b>{{ videoStore.currentCodecMode.toUpperCase() }}</b>
    </div>
    <div class="wave">
      <i
        v-for="i in 18"
        :key="i"
        :style="{ height: `${18 + ((i * 7 + Math.round(videoStore.customBlockPacketsPerSecond)) % 28)}px` }"
      />
    </div>
  </aside>
</template>

<style scoped>
.rm-link-monitor {
  width: 246px;
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

</style>
