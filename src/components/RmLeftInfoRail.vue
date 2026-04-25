<script setup lang="ts">
import { computed } from 'vue'
import { useModeStore } from '../stores/mode'
import { useVideoStore } from '../stores/video'

const props = defineProps<{
  lastError: string
  messages: string[]
}>()

const modeStore = useModeStore()
const videoStore = useVideoStore()
const deployText = computed(() => {
  if (modeStore.deployModeState === 'active') return '已部署'
  if (modeStore.deployModeState === 'inactive') return '未部署'
  return '未知'
})
</script>

<template>
  <aside class="rm-left-rail">
    <div class="hero-block">
      <span>当前机器人</span>
      <strong>英雄</strong>
    </div>
    <div class="status-list">
      <p><span>部署模式</span><b>{{ deployText }}</b></p>
      <p><span>MQTT</span><b :class="modeStore.mqttConnected ? 'ok' : 'bad'">{{ modeStore.mqttConnected ? 'ONLINE' : 'OFFLINE' }}</b></p>
      <p><span>CustomBlock</span><b :class="videoStore.customBlockPacketsReceived > 0 ? 'ok' : ''">{{ videoStore.customBlockPacketsReceived }}</b></p>
      <p><span>准星预设</span><b>DEFAULT</b></p>
    </div>
    <div class="message-list">
      <h4>系统消息</h4>
      <p v-if="lastError" class="error">{{ lastError }}</p>
      <p v-for="message in props.messages.slice(0, 5)" :key="message">{{ message }}</p>
      <p v-if="props.messages.length === 0 && !lastError">等待链路状态...</p>
    </div>
  </aside>
</template>

<style scoped>
.rm-left-rail {
  position: absolute;
  left: 24px;
  top: 96px;
  z-index: 12;
  width: 286px;
  color: var(--rm-op-text);
}

.hero-block {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 18px;
}

.hero-block span {
  color: var(--rm-op-muted);
  font-size: 11px;
}

.hero-block strong {
  width: fit-content;
  padding: 5px 13px;
  border: 1px solid var(--rm-op-yellow);
  border-radius: 7px;
  color: var(--rm-op-yellow);
  font-size: 16px;
  box-shadow: 0 0 16px rgba(255, 201, 58, 0.22);
}

.status-list {
  padding: 12px 0;
  border-top: 1px solid rgba(234, 247, 255, 0.1);
  border-bottom: 1px solid rgba(234, 247, 255, 0.1);
}

.status-list p,
.message-list p {
  display: flex;
  justify-content: space-between;
  margin: 0;
  padding: 7px 0;
  border-bottom: 1px solid rgba(234, 247, 255, 0.07);
  color: rgba(234, 247, 255, 0.66);
  font-size: 12px;
}

.status-list b {
  color: var(--rm-op-cyan);
}

.status-list b.ok {
  color: var(--rm-op-green);
}

.status-list b.bad,
.message-list .error {
  color: var(--rm-op-red);
}

.message-list {
  margin-top: 20px;
}

.message-list h4 {
  margin: 0 0 8px;
  color: rgba(234, 247, 255, 0.72);
  font-size: 12px;
}

.message-list p {
  display: block;
}

@media (max-width: 1180px) {
  .rm-left-rail {
    display: none;
  }
}
</style>
