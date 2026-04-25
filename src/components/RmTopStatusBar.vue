<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useModeStore } from '../stores/mode'
import { useVideoStore } from '../stores/video'

defineProps<{
  runtimeText: string
  lastError: string
}>()

const modeStore = useModeStore()
const videoStore = useVideoStore()
const bitrateKbps = ref(0)
let timer: number | null = null
let lastBytes = 0

const modeText = computed(() => (videoStore.currentMode === 'hero_lob' ? 'HERO LOB' : 'NORMAL'))
const sourceText = computed(() => (videoStore.currentVideoSource === 'custombyteblock_h264' ? '0x0310' : 'UDP 3334'))
const deployText = computed(() => {
  if (modeStore.deployModeState === 'active') return '已部署'
  if (modeStore.deployModeState === 'inactive') return '未部署'
  return '未知'
})

onMounted(() => {
  timer = window.setInterval(() => {
    const bytes = videoStore.currentVideoSource === 'custombyteblock_h264'
      ? videoStore.customBlockBytesReceived
      : videoStore.packetsReceived * 1200
    bitrateKbps.value = Math.max(0, bytes - lastBytes) / 1024
    lastBytes = bytes
  }, 1000)
})

onBeforeUnmount(() => {
  if (timer !== null) window.clearInterval(timer)
})
</script>

<template>
  <header class="rm-top-status" :class="{ lob: videoStore.currentMode === 'hero_lob' }">
    <div class="team-dot red"><span /></div>
    <div class="team-dot blue"><span /></div>

    <div class="mode-tabs">
      <b>{{ modeText }}</b>
      <span>{{ sourceText }}</span>
      <span>{{ videoStore.currentCodecMode.toUpperCase() }}</span>
      <span>{{ deployText }}</span>
      <span :class="videoStore.streamAlive ? 'online' : 'offline'">
        {{ videoStore.streamAlive ? '视频在线' : '视频断流' }}
      </span>
    </div>

    <div class="energy-meter">
      <i :style="{ width: videoStore.streamAlive ? '72%' : '18%' }" />
    </div>

    <div class="right-metrics">
      <span>FPS <b>{{ videoStore.fps || '--' }}</b></span>
      <span>RATE <b>{{ bitrateKbps.toFixed(1) }}</b></span>
      <span>LAT <b>{{ videoStore.lastDecodeCostMs || 0 }}ms</b></span>
      <span>RUN <b>{{ runtimeText }}</b></span>
      <em>9000</em>
    </div>

    <div v-if="lastError" class="top-error">{{ lastError }}</div>
  </header>
</template>

<style scoped>
.rm-top-status {
  position: absolute;
  top: 10px;
  left: 50%;
  z-index: 20;
  width: min(920px, calc(100vw - 42px));
  min-height: 58px;
  display: grid;
  grid-template-columns: 34px 34px minmax(0, 1fr) 118px auto;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  border: 1px solid rgba(0, 229, 255, 0.28);
  border-radius: 12px;
  background: rgba(5, 8, 13, 0.78);
  box-shadow: 0 0 28px rgba(0, 229, 255, 0.12), inset 0 0 18px rgba(255, 255, 255, 0.03);
  transform: translateX(-50%);
  backdrop-filter: blur(14px);
}

.rm-top-status.lob {
  border-color: rgba(255, 201, 58, 0.45);
  box-shadow: 0 0 28px rgba(255, 201, 58, 0.14), inset 0 0 18px rgba(0, 229, 255, 0.04);
}

.team-dot {
  width: 28px;
  height: 28px;
  display: grid;
  place-items: center;
  border: 1px solid currentColor;
  border-radius: 7px;
}

.team-dot span {
  width: 11px;
  height: 11px;
  border: 2px solid currentColor;
}

.team-dot.red {
  color: var(--rm-op-red);
  box-shadow: 0 0 16px rgba(255, 48, 69, 0.35);
}

.team-dot.blue {
  color: var(--rm-op-blue);
  box-shadow: 0 0 16px rgba(43, 124, 255, 0.35);
}

.mode-tabs {
  display: flex;
  min-width: 0;
  align-items: center;
  justify-content: center;
  gap: 7px;
}

.mode-tabs b,
.mode-tabs span {
  position: relative;
  padding: 4px 11px;
  color: var(--rm-op-muted);
  font-size: 12px;
  font-weight: 900;
  white-space: nowrap;
}

.mode-tabs b {
  color: var(--rm-op-text);
}

.mode-tabs b::before,
.mode-tabs span::before {
  position: absolute;
  inset: 2px 0;
  z-index: -1;
  background: rgba(255, 255, 255, 0.07);
  content: '';
  transform: skewX(-22deg);
}

.mode-tabs .online {
  color: var(--rm-op-green);
}

.mode-tabs .offline {
  color: var(--rm-op-red);
}

.energy-meter {
  height: 12px;
  padding: 2px;
  background: rgba(255, 255, 255, 0.1);
  transform: skewX(-18deg);
}

.energy-meter i {
  display: block;
  height: 100%;
  background: linear-gradient(90deg, var(--rm-op-cyan), var(--rm-op-blue));
  box-shadow: 0 0 12px rgba(0, 229, 255, 0.55);
}

.right-metrics {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--rm-op-muted);
  font-size: 11px;
  white-space: nowrap;
}

.right-metrics b {
  color: var(--rm-op-cyan);
}

.right-metrics em {
  padding: 6px 11px;
  border: 1px solid rgba(255, 201, 58, 0.42);
  border-radius: 8px;
  color: var(--rm-op-yellow);
  font-style: normal;
  font-weight: 900;
  letter-spacing: 0.14em;
  box-shadow: inset 0 0 14px rgba(255, 201, 58, 0.08);
}

.top-error {
  position: absolute;
  right: 14px;
  bottom: -18px;
  max-width: 46vw;
  color: var(--rm-op-red);
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 1180px) {
  .rm-top-status {
    grid-template-columns: 34px 34px minmax(0, 1fr);
  }

  .energy-meter,
  .right-metrics {
    display: none;
  }
}
</style>
