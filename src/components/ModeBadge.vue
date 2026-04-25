<script setup lang="ts">
import { computed } from 'vue'
import { NCard, NSpace, NTag } from 'naive-ui'
import { useModeStore } from '../stores/mode'
import { useVideoStore } from '../stores/video'

const modeStore = useModeStore()
const videoStore = useVideoStore()

const mqttText = computed(() => (modeStore.mqttConnected ? '已连接' : '未连接'))
const videoText = computed(() => (videoStore.streamAlive ? '已启动' : '未启动'))
const deployText = computed(() => {
  if (modeStore.deployModeState === 'active') {
    return '部署模式中'
  }
  if (modeStore.deployModeState === 'inactive') {
    return '非部署模式'
  }
  return '未知'
})
</script>

<template>
  <n-card size="small" title="模式状态" class="mode-badge">
    <n-space vertical size="small">
      <n-tag :type="modeStore.mqttConnected ? 'success' : 'warning'" size="small">MQTT：{{ mqttText }}</n-tag>
      <n-tag :type="videoStore.streamAlive ? 'success' : 'default'" size="small">视频：{{ videoText }}</n-tag>
      <n-tag :type="modeStore.deployModeState === 'active' ? 'info' : 'default'" size="small">
        部署模式：{{ deployText }}
      </n-tag>
    </n-space>
  </n-card>
</template>

<style scoped>
.mode-badge {
  width: 220px;
  background: rgba(9, 18, 29, 0.92);
  border: 1px solid #28415f;
}
</style>
