<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { NButton, NInput, NInputNumber, NSpace, NTag } from 'naive-ui'
import CrosshairPanel from '../components/CrosshairPanel.vue'
import DebugPanel from '../components/DebugPanel.vue'
import ModeBadge from '../components/ModeBadge.vue'
import VideoCanvas from '../components/VideoCanvas.vue'
import { useModeSync } from '../composables/useModeSync'
import { useUiPersistence } from '../composables/useUiPersistence'
import { useVideoStream } from '../composables/useVideoStream'
import { useUiStore } from '../stores/ui'
import { useVideoStore } from '../stores/video'

useUiPersistence()

const uiStore = useUiStore()
const {
  crosshairOffsetX,
  crosshairOffsetY,
  crosshairWidth,
  displayScale,
  showCenterDot,
  showDebug,
} = storeToRefs(uiStore)

const { host, port, commandMessage, handleConnect, handleDisconnect, handleMockToggle } = useModeSync()
const {
  port: videoPort,
  videoCommandMessage,
  handleStartVideo,
  handleStopVideo,
  handleStartMockVideo,
  handleStopMockVideo,
} = useVideoStream()
const videoStore = useVideoStore()
</script>

<template>
  <div class="hero-deploy-page">
    <div class="top-strip">
      <div class="title-block">
        <span class="title-main">RoboMaster Hero Deploy Client</span>
        <span class="title-sub">Low-Latency Tactical Overlay Console</span>
      </div>
      <n-space>
        <n-tag :type="videoStore.realDecoderEnabled ? 'success' : 'warning'">
          {{ videoStore.realDecoderEnabled ? 'REAL DECODER' : 'STUB DECODER' }}
        </n-tag>
        <n-tag :type="videoStore.streamAlive ? 'success' : 'default'">
          {{ videoStore.streamAlive ? 'STREAM ONLINE' : 'STREAM OFFLINE' }}
        </n-tag>
      </n-space>
    </div>

    <div class="video-stage">
      <VideoCanvas
        :offset-x="crosshairOffsetX"
        :offset-y="crosshairOffsetY"
        :line-width="crosshairWidth"
        :display-scale="displayScale"
        :show-center-dot="showCenterDot"
      />
      <div class="mode-badge">
        <ModeBadge />
      </div>
    </div>

    <div class="right-panel">
      <n-space vertical>
        <n-input-number v-model:value="videoPort" :min="1" :max="65535" style="width: 100%" />
        <n-space>
          <n-button type="primary" @click="handleStartVideo">启动视频接收</n-button>
          <n-button secondary @click="handleStopVideo">停止视频接收</n-button>
        </n-space>
        <n-space>
          <n-button tertiary @click="handleStartMockVideo">启动Mock视频源</n-button>
          <n-button tertiary @click="handleStopMockVideo">停止Mock视频源</n-button>
        </n-space>
        <span class="command-message">{{ videoCommandMessage }}</span>
        <n-input v-model:value="host" placeholder="MQTT Host" />
        <n-input-number v-model:value="port" :min="1" :max="65535" style="width: 100%" />
        <n-space>
          <n-button type="primary" @click="handleConnect">连接 MQTT</n-button>
          <n-button secondary @click="handleDisconnect">断开 MQTT</n-button>
          <n-button tertiary @click="handleMockToggle">Mock 切换部署</n-button>
        </n-space>
        <span class="command-message">{{ commandMessage }}</span>
      </n-space>
      <CrosshairPanel />
      <DebugPanel v-if="showDebug" />
    </div>

    <div class="bottom-bar">
      <div class="stats">
        <span>offsetX: {{ crosshairOffsetX }}</span>
        <span>offsetY: {{ crosshairOffsetY }}</span>
        <span>width: {{ crosshairWidth }}</span>
        <span>scale: {{ displayScale }}</span>
      </div>
      <n-button size="small" tertiary @click="uiStore.toggleDebug">
        {{ showDebug ? '隐藏调试面板' : '显示调试面板' }}
      </n-button>
    </div>
  </div>
</template>

<style scoped>
.hero-deploy-page {
  height: 100vh;
  display: grid;
  grid-template-columns: minmax(0, 1fr) 360px;
  grid-template-rows: 58px minmax(0, 1fr) 64px;
  gap: 12px;
  padding: 12px;
  background:
    radial-gradient(circle at 15% 15%, rgba(0, 160, 255, 0.12), transparent 35%),
    radial-gradient(circle at 85% 10%, rgba(255, 40, 40, 0.1), transparent 30%),
    #0b1018;
}

.top-strip {
  grid-column: 1 / span 2;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  border-radius: 10px;
  border: 1px solid #253244;
  background: linear-gradient(90deg, rgba(13, 23, 38, 0.95), rgba(16, 25, 33, 0.95));
}

.title-block {
  display: flex;
  flex-direction: column;
}

.title-main {
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 0.3px;
  color: #dce6f4;
}

.title-sub {
  font-size: 11px;
  color: #6d8098;
}

.video-stage {
  position: relative;
  min-width: 0;
}

.mode-badge {
  position: absolute;
  top: 12px;
  right: 12px;
  z-index: 2;
}

.right-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}

.bottom-bar {
  grid-column: 1 / span 2;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 0 12px;
  border-radius: 10px;
  border: 1px solid #253244;
  background: #0f1622;
}

.stats {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: #8f9bb3;
}

.command-message {
  color: #8f9bb3;
  font-size: 12px;
}

@media (max-width: 1280px) {
  .hero-deploy-page {
    grid-template-columns: 1fr;
    grid-template-rows: 58px minmax(0, 1fr) auto 64px;
  }

  .top-strip,
  .bottom-bar {
    grid-column: 1;
  }
}
</style>
