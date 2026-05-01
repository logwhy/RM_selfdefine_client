<script setup lang="ts">
import { computed, ref } from 'vue'
import { useModeStore } from '../stores/mode'
import { useUiStore } from '../stores/ui'
import { useVideoStore } from '../stores/video'

const props = defineProps<{
  lastError: string
  messages: string[]
  selectedClientId: string
  selectedRobotLabel: string
}>()

const modeStore = useModeStore()
const uiStore = useUiStore()
const videoStore = useVideoStore()
const collapsed = ref(false)

const deployText = computed(() => {
  if (modeStore.deployModeState === 'active') return '已部署'
  if (modeStore.deployModeState === 'inactive') return '未部署'
  return '未知'
})

const activePresetName = computed(() => {
  return uiStore.crosshairPresets.find((preset) => preset.id === uiStore.activePresetId)?.name ?? '默认'
})

const latestEvent = computed(() => modeStore.refereeEvents[0] ?? null)
</script>

<template>
  <aside class="rm-left-rail" :class="{ collapsed }">
    <div v-if="collapsed && latestEvent" class="event-toast rm-glass-panel">
      <span>裁判事件</span>
      <strong>{{ latestEvent.text }}</strong>
    </div>

    <div class="left-rail-content rm-glass-panel">
      <div class="hero-block">
        <span>当前机器人</span>
        <strong>{{ props.selectedRobotLabel }}</strong>
      </div>

      <div class="status-list">
        <p><span>部署模式</span><b>{{ deployText }}</b></p>
        <p><span>MQTT</span><b :class="modeStore.mqttConnected ? 'ok' : 'bad'">{{ modeStore.mqttConnected ? 'ONLINE' : 'OFFLINE' }}</b></p>
        <p><span>MQTT Endpoint</span><b>{{ modeStore.mqttHost ? `${modeStore.mqttHost}:${modeStore.mqttPort ?? '-'}` : '-' }}</b></p>
        <p><span>MQTT Client ID</span><b>{{ modeStore.mqttClientId ?? props.selectedClientId }}</b></p>
        <p><span>CustomByteBlock</span><b :class="videoStore.customBlockPacketsReceived > 0 ? 'ok' : ''">{{ videoStore.customBlockPacketsReceived }}</b></p>
        <p><span>裁判消息</span><b :class="modeStore.lastRefereeMessageAt ? 'ok' : ''">{{ modeStore.lastRefereeMessageAt ? 'RX' : '-' }}</b></p>
        <p><span>裁判机器人 ID</span><b>{{ modeStore.robotStaticStatus.robotId ?? '-' }}</b></p>
        <p><span>准星预设</span><b>{{ activePresetName }}</b></p>
      </div>

      <div class="event-list">
        <h4>裁判事件</h4>
        <p v-for="event in modeStore.refereeEvents.slice(0, 4)" :key="`${event.receivedAt}-${event.eventId}-${event.param}`">
          {{ event.text }}
        </p>
        <p v-if="modeStore.refereeEvents.length === 0">等待事件...</p>
      </div>

      <div class="message-list">
        <h4>系统消息</h4>
        <p v-if="lastError" class="error">{{ lastError }}</p>
        <p v-for="message in props.messages.slice(0, 5)" :key="message">{{ message }}</p>
        <p v-if="props.messages.length === 0 && !lastError">等待链路状态...</p>
      </div>
    </div>

    <button class="left-rail-toggle" :title="collapsed ? '展开左侧信息' : '隐藏左侧信息'" @click="collapsed = !collapsed">
      {{ collapsed ? '>' : '<' }}
    </button>
  </aside>
</template>

<style scoped>
.rm-left-rail {
  position: absolute;
  left: 24px;
  top: 96px;
  z-index: 40;
  width: 326px;
  height: auto;
  color: var(--rm-op-text);
  pointer-events: none;
}

.left-rail-content {
  width: 286px;
  max-height: calc(100vh - 150px);
  overflow: auto;
  padding: 14px;
  border-color: rgba(0, 229, 255, 0.5);
  background:
    linear-gradient(135deg, rgba(0, 229, 255, 0.1), transparent 42%),
    rgba(5, 10, 16, 0.92);
  opacity: 1;
  pointer-events: auto;
  transform: translateX(0);
  transition: transform 180ms ease, opacity 140ms ease;
}

.rm-left-rail.collapsed .left-rail-content {
  opacity: 0;
  transform: translateX(-326px);
}

.left-rail-toggle {
  position: absolute;
  left: 294px;
  top: 0;
  width: 28px;
  height: 76px;
  border: 1px solid rgba(0, 229, 255, 0.48);
  border-radius: 0 8px 8px 0;
  background: rgba(3, 8, 14, 0.98);
  color: var(--rm-op-cyan);
  cursor: pointer;
  font-size: 18px;
  line-height: 1;
  pointer-events: auto;
  box-shadow: 0 0 18px rgba(0, 229, 255, 0.18);
  transition: left 180ms ease;
}

.rm-left-rail.collapsed .left-rail-toggle {
  left: 0;
}

.left-rail-toggle:hover {
  border-color: var(--rm-op-cyan);
}

.event-toast {
  position: absolute;
  left: 42px;
  top: 0;
  width: 330px;
  padding: 10px 12px;
  border-color: rgba(255, 201, 58, 0.5);
  background: rgba(28, 18, 4, 0.88);
  pointer-events: none;
}

.event-toast span {
  display: block;
  color: var(--rm-op-yellow);
  font-size: 11px;
  font-weight: 800;
}

.event-toast strong {
  display: block;
  margin-top: 4px;
  color: var(--rm-op-text);
  font-size: 13px;
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
.message-list p,
.event-list p {
  display: flex;
  justify-content: space-between;
  margin: 0;
  padding: 7px 0;
  border-bottom: 1px solid rgba(234, 247, 255, 0.07);
  color: rgba(234, 247, 255, 0.72);
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

.message-list,
.event-list {
  margin-top: 20px;
}

.message-list h4,
.event-list h4 {
  margin: 0 0 8px;
  color: rgba(234, 247, 255, 0.78);
  font-size: 12px;
}

.message-list p,
.event-list p {
  display: block;
}

@media (max-width: 1180px) {
  .rm-left-rail {
    left: 12px;
    top: 86px;
  }
}
</style>
