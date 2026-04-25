<script setup lang="ts">
import { computed } from 'vue'
import { NSpace, NTag } from 'naive-ui'
import { useModeStore } from '../stores/mode'
import { useVideoStore } from '../stores/video'

const modeStore = useModeStore()
const videoStore = useVideoStore()

const modeText = computed(() =>
  videoStore.currentMode === 'hero_lob' ? '英雄吊射模式' : '普通图传模式',
)
const videoText = computed(() => (videoStore.streamAlive ? '视频在线' : '视频离线'))
const mqttText = computed(() => (modeStore.mqttConnected ? 'MQTT 已连接' : 'MQTT 未连接'))
const sourceText = computed(() => {
  if (videoStore.currentVideoSource === 'custombyteblock_h264') return 'CustomByteBlock / 0x0310'
  if (videoStore.currentVideoSource === 'udp_hevc') return 'UDP 3334'
  return 'Mock'
})
</script>

<template>
  <div class="mode-badge">
    <n-space size="small" align="center" wrap>
      <n-tag :type="videoStore.currentMode === 'hero_lob' ? 'warning' : 'info'" size="small">
        {{ modeText }}
      </n-tag>
      <n-tag :type="videoStore.streamAlive ? 'success' : 'default'" size="small">
        {{ videoText }}
      </n-tag>
      <n-tag :type="modeStore.mqttConnected ? 'success' : 'default'" size="small">
        {{ mqttText }}
      </n-tag>
      <n-tag size="small">{{ videoStore.currentCodecMode.toUpperCase() }}</n-tag>
      <n-tag size="small">{{ sourceText }}</n-tag>
    </n-space>
  </div>
</template>

<style scoped>
.mode-badge {
  padding: 8px 10px;
  border: 1px solid rgba(80, 108, 144, 0.7);
  border-radius: 8px;
  background: rgba(8, 14, 24, 0.86);
  backdrop-filter: blur(10px);
}
</style>
