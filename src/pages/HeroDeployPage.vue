<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import {
  NButton,
  NDivider,
  NDrawer,
  NDrawerContent,
  NInput,
  NInputNumber,
  NRadioButton,
  NRadioGroup,
  NSelect,
  NSpace,
  NTag,
} from 'naive-ui'
import CrosshairPanel from '../components/CrosshairPanel.vue'
import DebugPanel from '../components/DebugPanel.vue'
import FloatingActionDock from '../components/FloatingActionDock.vue'
import RmHudPanel from '../components/RmHudPanel.vue'
import TacticalBottomBar from '../components/TacticalBottomBar.vue'
import TacticalTopBar from '../components/TacticalTopBar.vue'
import VideoCanvas from '../components/VideoCanvas.vue'
import { useModeSync } from '../composables/useModeSync'
import { useUiPersistence } from '../composables/useUiPersistence'
import { useVideoStream } from '../composables/useVideoStream'
import { useModeStore } from '../stores/mode'
import { useUiStore } from '../stores/ui'
import { useVideoStore } from '../stores/video'

type DrawerKey = 'params' | 'debug' | 'comm' | 'mode' | null
type SwitchingMode = 'hero_lob' | 'normal' | null

useUiPersistence()

const uiStore = useUiStore()
const {
  crosshairOffsetX,
  crosshairOffsetY,
  crosshairWidth,
  displayScale,
  showCenterDot,
} = storeToRefs(uiStore)

const modeStore = useModeStore()
const videoStore = useVideoStore()
const { host, port, commandMessage, handleConnect, handleDisconnect, handleMockToggle } = useModeSync()
const {
  port: videoPort,
  videoCommandMessage,
  handleStartVideo,
  handleStopVideo,
  handleStartMockVideo,
  handleStopMockVideo,
  handleUseNormalMode,
  handleUseHeroLobMode,
  handleStartMockHeroLobH264,
  handleStopMockHeroLobH264,
} = useVideoStream()

const activeDrawer = ref<DrawerKey>(null)
const drawerVisible = ref(false)
const switching = ref<SwitchingMode>(null)
const lastError = ref('')
const successMessage = ref('')
const runtimeSeconds = ref(0)

let runtimeTimer: number | null = null
let toastTimer: number | null = null

const parserOptions = [
  { label: 'raw_annexb_stream', value: 'raw_annexb_stream' },
  { label: 'packetized_frame', value: 'packetized_frame' },
]

const runtimeText = computed(() => {
  const minutes = Math.floor(runtimeSeconds.value / 60)
  const seconds = runtimeSeconds.value % 60
  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
})

function openDrawer(key: Exclude<DrawerKey, null>) {
  activeDrawer.value = key
  drawerVisible.value = true
}

function handleDrawerVisible(value: boolean) {
  drawerVisible.value = value
  if (!value) {
    activeDrawer.value = null
  }
}

function showSuccess(message: string) {
  successMessage.value = message
  if (toastTimer !== null) {
    window.clearTimeout(toastTimer)
  }
  toastTimer = window.setTimeout(() => {
    successMessage.value = ''
    toastTimer = null
  }, 1000)
}

async function runSwitch(mode: Exclude<SwitchingMode, null>, action: () => Promise<void>) {
  if (switching.value !== null) return
  switching.value = mode
  lastError.value = ''
  try {
    await action()
    showSuccess(mode === 'hero_lob' ? 'HERO LOB READY' : 'NORMAL VIDEO READY')
  } catch (error) {
    lastError.value = String(error)
  } finally {
    switching.value = null
  }
}

async function quickHeroLob() {
  await runSwitch('hero_lob', async () => {
    await handleStopVideo()
    if (!modeStore.mqttConnected) {
      await handleConnect()
    }
    await handleUseHeroLobMode(videoStore.customBlockParserMode)
    if (!modeStore.mqttConnected) {
      lastError.value = 'MQTT is connecting or unavailable; CustomByteBlock will start after connection.'
    }
  })
}

async function quickNormal() {
  await runSwitch('normal', async () => {
    await handleStopMockHeroLobH264()
    await handleUseNormalMode()
    await handleStartVideo()
  })
}

function toggleFullscreen() {
  drawerVisible.value = false
  activeDrawer.value = null
  if (!document.fullscreenElement) {
    void document.documentElement.requestFullscreen()
  } else {
    void document.exitFullscreen()
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'F11') {
    event.preventDefault()
    toggleFullscreen()
  }
}

onMounted(() => {
  runtimeTimer = window.setInterval(() => {
    runtimeSeconds.value += 1
  }, 1000)
  window.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
  if (runtimeTimer !== null) window.clearInterval(runtimeTimer)
  if (toastTimer !== null) window.clearTimeout(toastTimer)
})
</script>

<template>
  <div class="tactical-page">
    <TacticalTopBar :last-error="lastError" :runtime-text="runtimeText" />

    <main class="tactical-stage rm-corners">
      <VideoCanvas
        :offset-x="crosshairOffsetX"
        :offset-y="crosshairOffsetY"
        :line-width="crosshairWidth"
        :display-scale="displayScale"
        :show-center-dot="showCenterDot"
      />
      <div class="stage-label left">
        <span>HERO OPTICAL FEED</span>
        <strong>{{ videoStore.currentMode === 'hero_lob' ? '0x0310 / H264' : 'UDP 3334 / HEVC' }}</strong>
      </div>
      <div class="stage-label right">
        <span>DECODER</span>
        <strong>{{ videoStore.currentDecoderName }}</strong>
      </div>
    </main>

    <TacticalBottomBar :last-error="lastError" />

    <FloatingActionDock
      :switching="switching"
      :success-message="successMessage"
      @hero-lob="quickHeroLob"
      @normal="quickNormal"
      @open="openDrawer"
      @fullscreen="toggleFullscreen"
    />

    <n-drawer
      :show="drawerVisible"
      :width="430"
      placement="right"
      display-directive="show"
      class="rm-hud-drawer"
      @update:show="handleDrawerVisible"
    >
      <n-drawer-content v-if="activeDrawer === 'params'" title="CROSSHAIR PARAMETERS" closable>
        <RmHudPanel title="Crosshair">
          <CrosshairPanel />
        </RmHudPanel>
      </n-drawer-content>

      <n-drawer-content v-if="activeDrawer === 'debug'" title="LINK DEBUG" closable>
        <RmHudPanel title="Decoder / Transport">
          <DebugPanel />
        </RmHudPanel>
      </n-drawer-content>

      <n-drawer-content v-if="activeDrawer === 'comm'" title="COMMUNICATION" closable>
        <n-space vertical size="large">
          <RmHudPanel title="MQTT Link">
            <n-input v-model:value="host" placeholder="MQTT Host" />
            <n-input-number v-model:value="port" :min="1" :max="65535" style="width: 100%" />
            <n-space>
              <n-button type="primary" @click="handleConnect">连接 MQTT</n-button>
              <n-button secondary @click="handleDisconnect">断开 MQTT</n-button>
            </n-space>
            <n-button tertiary @click="handleMockToggle">Mock 切换部署</n-button>
            <p class="tactical-command-message">{{ commandMessage }}</p>
          </RmHudPanel>
          <RmHudPanel title="Comm Status">
            <n-space>
              <n-tag :type="modeStore.mqttConnected ? 'success' : 'error'">
                {{ modeStore.mqttConnected ? 'MQTT ONLINE' : 'MQTT OFFLINE' }}
              </n-tag>
              <n-tag :type="modeStore.deployModeState === 'active' ? 'success' : 'default'">
                DEPLOY {{ modeStore.deployModeState.toUpperCase() }}
              </n-tag>
            </n-space>
          </RmHudPanel>
        </n-space>
      </n-drawer-content>

      <n-drawer-content v-if="activeDrawer === 'mode'" title="MODE CONTROL" closable>
        <n-space vertical size="large">
          <RmHudPanel title="One-click Switch">
            <n-space>
              <n-button type="warning" :loading="switching === 'hero_lob'" @click="quickHeroLob">
                英雄吊射 0x0310 / H264
              </n-button>
              <n-button type="primary" :loading="switching === 'normal'" @click="quickNormal">
                普通图传 UDP / HEVC
              </n-button>
            </n-space>
            <n-space>
              <n-tag>{{ videoStore.currentMode }}</n-tag>
              <n-tag>{{ videoStore.currentVideoSource }}</n-tag>
              <n-tag>{{ videoStore.currentCodecMode.toUpperCase() }}</n-tag>
            </n-space>
          </RmHudPanel>

          <RmHudPanel title="Manual Mode">
            <n-radio-group :value="videoStore.currentMode">
              <n-radio-button value="normal" @click="quickNormal">普通图传模式</n-radio-button>
              <n-radio-button value="hero_lob" @click="quickHeroLob">英雄吊射模式</n-radio-button>
            </n-radio-group>
          </RmHudPanel>

          <n-divider />

          <RmHudPanel title="UDP HEVC">
            <n-input-number v-model:value="videoPort" :min="1" :max="65535" style="width: 100%" />
            <n-space>
              <n-button type="primary" @click="handleStartVideo">启动 UDP 接收</n-button>
              <n-button secondary @click="handleStopVideo">停止 UDP 接收</n-button>
            </n-space>
            <n-space>
              <n-button tertiary @click="handleStartMockVideo">启动 Mock 视频源</n-button>
              <n-button tertiary @click="handleStopMockVideo">停止 Mock 视频源</n-button>
            </n-space>
          </RmHudPanel>

          <RmHudPanel title="CustomBlock H264">
            <n-select
              v-model:value="videoStore.customBlockParserMode"
              :options="parserOptions"
              @update:value="handleUseHeroLobMode"
            />
            <n-space>
              <n-button type="warning" @click="handleUseHeroLobMode(videoStore.customBlockParserMode)">
                启用 CustomByteBlock H264
              </n-button>
              <n-button tertiary @click="handleStartMockHeroLobH264">启动 Mock H264</n-button>
              <n-button tertiary @click="handleStopMockHeroLobH264">停止 Mock H264</n-button>
            </n-space>
          </RmHudPanel>

          <p class="tactical-command-message">{{ videoCommandMessage }}</p>
        </n-space>
      </n-drawer-content>
    </n-drawer>
  </div>
</template>

<style scoped>
.stage-label {
  position: absolute;
  z-index: 4;
  top: 18px;
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 8px 10px;
  border: 1px solid rgba(25, 247, 255, 0.28);
  background: rgba(5, 12, 20, 0.62);
  color: var(--rm-muted);
  font-size: 10px;
  letter-spacing: 0.12em;
}

.stage-label strong {
  color: var(--rm-cyan);
  font-size: 12px;
}

.stage-label.left {
  left: 18px;
}

.stage-label.right {
  right: 18px;
  text-align: right;
}
</style>
