<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
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
import HudEditorOverlay from '../components/HudEditorOverlay.vue'
import HudEditorPanel from '../components/HudEditorPanel.vue'
import InputDiagnosticsPanel from '../components/InputDiagnosticsPanel.vue'
import RmBottomShortcutBar from '../components/RmBottomShortcutBar.vue'
import RmCrosshairHud from '../components/RmCrosshairHud.vue'
import RmHudPanel from '../components/RmHudPanel.vue'
import RmLeftInfoRail from '../components/RmLeftInfoRail.vue'
import RmLinkMonitor from '../components/RmLinkMonitor.vue'
import RmQuickActionPanel from '../components/RmQuickActionPanel.vue'
import RmTopStatusBar from '../components/RmTopStatusBar.vue'
import VideoCanvas from '../components/VideoCanvas.vue'
import { useModeSync } from '../composables/useModeSync'
import { useLowLatencyInput } from '../composables/useLowLatencyInput'
import { useUiPersistence } from '../composables/useUiPersistence'
import { useVideoStream } from '../composables/useVideoStream'
import { useModeStore } from '../stores/mode'
import { useHudEditorStore } from '../stores/hudEditor'
import { useInputControlStore } from '../stores/inputControl'
import { useUiStore } from '../stores/ui'
import { useVideoStore } from '../stores/video'

type DrawerKey = 'params' | 'debug' | 'comm' | 'mode' | 'hud' | 'input' | null
type SwitchingMode = 'hero_lob' | 'normal' | null

useUiPersistence()

const uiStore = useUiStore()
const {
  crosshairOffsetX,
  crosshairOffsetY,
  crosshairWidth,
  displayScale,
  showCrosshair,
  showCenterDot,
} = storeToRefs(uiStore)

const modeStore = useModeStore()
const videoStore = useVideoStore()
const hudStore = useHudEditorStore()
const inputStore = useInputControlStore()
const shellRef = ref<HTMLElement | null>(null)
const { enterFpsMode, exitFpsMode } = useLowLatencyInput(() => shellRef.value)
const {
  host,
  port,
  clientId,
  robotOptions,
  commandMessage,
  handleConnect,
  handleDisconnect,
  handleMockToggle,
  useLocalMqttEndpoint,
  useOfficialMqttEndpoint,
} = useModeSync()
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
const helpVisible = ref(false)
const switching = ref<SwitchingMode>(null)
const lastError = ref('')
const successMessage = ref('')
const runtimeSeconds = ref(0)
const systemMessages = ref<string[]>([])
const rightPanelCollapsed = ref(false)
const isCompetitionFullscreen = ref(false)

let runtimeTimer: number | null = null
let toastTimer: number | null = null
let lastRuntimeWarning = ''

function handleBrowserFullscreenChange() {
  if (!isTauriRuntime()) isCompetitionFullscreen.value = Boolean(document.fullscreenElement)
}

const parserOptions = [
  { label: 'raw_annexb_stream', value: 'raw_annexb_stream' },
  { label: 'packetized_frame', value: 'packetized_frame' },
]

const runtimeText = computed(() => {
  const minutes = Math.floor(runtimeSeconds.value / 60)
  const seconds = runtimeSeconds.value % 60
  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
})

const activePresetName = computed(() => {
  return uiStore.crosshairPresets.find((preset) => preset.id === uiStore.activePresetId)?.name ?? '默认'
})
const visibleCrosshairColor = computed(() => uiStore.crosshairColor ?? '#19f7ff')
const selectedRobotLabel = computed(() => {
  return robotOptions.find((option) => option.value === clientId.value)?.label ?? `Client ID ${clientId.value}`
})

function openDrawer(key: Exclude<DrawerKey, null>) {
  if (activeDrawer.value === key && drawerVisible.value) {
    handleDrawerVisible(false)
    return
  }
  helpVisible.value = false
  activeDrawer.value = key
  drawerVisible.value = true
}

function handleDrawerVisible(value: boolean) {
  drawerVisible.value = value
  if (!value) {
    activeDrawer.value = null
  }
}

function showToast(message: string, level: 'ok' | 'error' | 'info' = 'ok') {
  successMessage.value = message
  pushMessage(level === 'error' ? `ERROR ${message}` : message)
  if (level === 'error') lastError.value = message
  if (toastTimer !== null) {
    window.clearTimeout(toastTimer)
  }
  toastTimer = window.setTimeout(() => {
    successMessage.value = ''
    toastTimer = null
  }, 1400)
}

function pushMessage(message: string) {
  systemMessages.value = [`${runtimeText.value} ${message}`, ...systemMessages.value].slice(0, 5)
}

function modeStatusWarning() {
  if (videoStore.currentMode === 'hero_lob') {
    if (!modeStore.mqttConnected) return 'MQTT 未连接'
    if (videoStore.customBlockPacketsReceived === 0 && !videoStore.customBlockMockActive) return '未收到 CustomByteBlock / 0x0310'
    if (!videoStore.h264SeenSps || !videoStore.h264SeenPps) return 'H264 等待 SPS/PPS'
    if (!videoStore.decoderInitSuccess && videoStore.realDecoderEnabled) return 'H264 decoder 未就绪'
    if (videoStore.h264ConsecutiveDecodeErrors > 0) return 'decoder 异常'
  }
  if (!videoStore.streamAlive) return '视频断流'
  return ''
}

async function runSwitch(mode: Exclude<SwitchingMode, null>, action: () => Promise<void>) {
  if (switching.value !== null) return
  switching.value = mode
  lastError.value = ''
  try {
    await action()
    const warning = modeStatusWarning()
    if (warning && mode === 'hero_lob') {
      lastError.value = warning
      showToast(warning, 'info')
    } else {
      showToast(mode === 'hero_lob' ? '英雄吊射模式已就绪' : '普通图传模式已就绪')
    }
  } catch (error) {
    const message = String(error)
    lastError.value = message
    showToast(message, 'error')
  } finally {
    switching.value = null
  }
}

async function quickHeroLob() {
  await runSwitch('hero_lob', async () => {
    await handleStopVideo()
    if (!modeStore.mqttConnected) {
      try {
        await handleConnect()
      } catch (error) {
        throw new Error(`MQTT 未连接: ${String(error)}`)
      }
    }
    await handleUseHeroLobMode(videoStore.customBlockParserMode)
  })
}

async function quickNormal() {
  await runSwitch('normal', async () => {
    await handleStopMockHeroLobH264()
    await handleUseNormalMode()
    await handleStartVideo()
  })
}

function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

async function setCompetitionFullscreen(next: boolean) {
  drawerVisible.value = false
  activeDrawer.value = null
  helpVisible.value = false
  if (isTauriRuntime()) {
    const appWindow = getCurrentWindow()
    await appWindow.setFullscreen(next)
    isCompetitionFullscreen.value = await appWindow.isFullscreen()
  } else if (next && !document.fullscreenElement) {
    await document.documentElement.requestFullscreen()
    isCompetitionFullscreen.value = true
  } else if (!next && document.fullscreenElement) {
    await document.exitFullscreen()
    isCompetitionFullscreen.value = false
  } else {
    isCompetitionFullscreen.value = next
  }
  showToast(isCompetitionFullscreen.value ? '已进入全屏比赛模式' : '已退出全屏比赛模式', 'info')
}

async function toggleFullscreen() {
  await setCompetitionFullscreen(!isCompetitionFullscreen.value)
}

function toggleCrosshair() {
  uiStore.toggleCrosshair()
  showToast(uiStore.showCrosshair ? '准星已显示' : '准星已隐藏', 'info')
}

function isTypingTarget(target: EventTarget | null) {
  const element = target as HTMLElement | null
  if (!element) return false
  const tag = element.tagName.toLowerCase()
  return tag === 'input' || tag === 'textarea' || tag === 'select' || element.isContentEditable
}

function adjustCrosshair(event: KeyboardEvent) {
  const step = event.shiftKey ? 8 : event.ctrlKey ? 0.2 : 1
  if (event.key === 'ArrowUp') uiStore.crosshairOffsetY -= step
  if (event.key === 'ArrowDown') uiStore.crosshairOffsetY += step
  if (event.key === 'ArrowLeft') uiStore.crosshairOffsetX -= step
  if (event.key === 'ArrowRight') uiStore.crosshairOffsetX += step
}

function handlePresetShortcut(id: 1 | 2 | 3, savePreset: boolean) {
  if (savePreset) {
    const preset = uiStore.saveCurrentToPreset(id)
    if (preset) showToast(`已保存预设：${preset.name}`, 'info')
    return
  }
  const preset = uiStore.applyPreset(id)
  if (preset) showToast(`已切换：${preset.name}`, 'info')
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'F11') {
    event.preventDefault()
    void toggleFullscreen()
    return
  }
  if (isTypingTarget(event.target)) return
  if (inputStore.fpsMode && !['Escape', 'Enter', 'Tab'].includes(event.key)) return
  if (['Tab', 'F11', 'Escape', 'Enter', 'Delete', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(event.key)) {
    event.preventDefault()
  }
  if (event.key === 'Enter') {
    if (inputStore.fpsMode) void exitFpsMode()
    else void enterFpsMode()
    return
  }
  if (event.key === 'Tab') {
    helpVisible.value = !helpVisible.value
    if (helpVisible.value) handleDrawerVisible(false)
    return
  }
  if (event.key === 'Escape') {
    if (inputStore.fpsMode) {
      void exitFpsMode()
      return
    }
    if (helpVisible.value) {
      helpVisible.value = false
      return
    }
    if (drawerVisible.value) {
      handleDrawerVisible(false)
      return
    }
    if (isCompetitionFullscreen.value) void setCompetitionFullscreen(false)
    return
  }
  if (event.key.startsWith('Arrow')) {
    adjustCrosshair(event)
    return
  }
  if (event.key === 'Delete') {
    hudStore.deleteSelected()
    return
  }
  if (event.ctrlKey && event.key.toLowerCase() === 'z') {
    hudStore.undo()
    return
  }
  if (event.ctrlKey && event.key.toLowerCase() === 'y') {
    hudStore.redo()
    return
  }
  if (['1', '2', '3'].includes(event.key)) {
    handlePresetShortcut(Number(event.key) as 1 | 2 | 3, event.ctrlKey)
    return
  }
  const key = event.key.toLowerCase()
  if (key === 'h') {
    void quickHeroLob()
  } else if (key === 'n') {
    void quickNormal()
  } else if (key === 'p') {
    openDrawer('params')
  } else if (key === 'd') {
    openDrawer('debug')
  } else if (key === 'c') {
    openDrawer('comm')
  } else if (key === 'u') {
    hudStore.editMode = !hudStore.editMode
    if (hudStore.editMode) {
      hudStore.locked = false
      openDrawer('hud')
    } else {
      handleDrawerVisible(false)
    }
    showToast(hudStore.editMode ? 'HUD 编辑已开启' : 'HUD 编辑已关闭', 'info')
  } else if (key === 'v') {
    toggleCrosshair()
  } else if (key === 'i') {
    openDrawer('input')
  } else if (key === 'r') {
    uiStore.resetDefaults()
    showToast('准星已恢复默认', 'info')
  } else if (key === 's') {
    uiStore.save()
    showToast('准星配置已保存', 'info')
  }
}

onMounted(() => {
  runtimeTimer = window.setInterval(() => {
    runtimeSeconds.value += 1
  }, 1000)
  window.addEventListener('keydown', handleKeydown)
  document.addEventListener('fullscreenchange', handleBrowserFullscreenChange)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('fullscreenchange', handleBrowserFullscreenChange)
  if (runtimeTimer !== null) window.clearInterval(runtimeTimer)
  if (toastTimer !== null) window.clearTimeout(toastTimer)
})

watch(
  () => [
    videoStore.streamAlive,
    videoStore.customBlockPacketsReceived,
    videoStore.h264SeenSps,
    videoStore.h264SeenPps,
    videoStore.h264ConsecutiveDecodeErrors,
    modeStore.mqttConnected,
  ],
  () => {
    const warning = modeStatusWarning()
    if (warning && warning !== lastRuntimeWarning) {
      lastRuntimeWarning = warning
      lastError.value = warning
      pushMessage(`WARN ${warning}`)
    }
    if (!warning && lastRuntimeWarning) {
      lastRuntimeWarning = ''
      lastError.value = ''
    }
  },
)
</script>

<template>
  <div
    ref="shellRef"
    class="rm-operator-shell"
    :class="{ 'competition-fullscreen': isCompetitionFullscreen, 'fps-control-active': inputStore.fpsMode }"
  >
    <div class="rm-operator-video">
      <VideoCanvas
        :offset-x="crosshairOffsetX"
        :offset-y="crosshairOffsetY"
        :line-width="crosshairWidth"
        :display-scale="displayScale"
        :show-crosshair="showCrosshair"
        :show-center-dot="showCenterDot"
        :crosshair-color="visibleCrosshairColor"
      />
      <HudEditorOverlay />
    </div>

    <RmTopStatusBar :last-error="lastError" :runtime-text="runtimeText" />
    <RmLeftInfoRail
      :last-error="lastError"
      :messages="systemMessages"
      :selected-client-id="clientId"
      :selected-robot-label="selectedRobotLabel"
    />
    <RmCrosshairHud :success-message="successMessage" :show-crosshair="showCrosshair" />

    <div v-if="!isCompetitionFullscreen" class="right-hud-stack" :class="{ collapsed: rightPanelCollapsed }">
      <button class="right-hud-toggle" @click="rightPanelCollapsed = !rightPanelCollapsed">
        {{ rightPanelCollapsed ? '<' : '>' }}
      </button>
      <div class="right-hud-content">
        <RmQuickActionPanel
          :switching="switching"
          :success-message="successMessage"
          :show-crosshair="showCrosshair"
          @hero-lob="quickHeroLob"
          @normal="quickNormal"
          @open="openDrawer"
          @fullscreen="toggleFullscreen"
          @toggle-crosshair="toggleCrosshair"
        />
        <RmLinkMonitor />
      </div>
    </div>

    <div v-else class="fullscreen-actions">
      <RmQuickActionPanel
        compact
        :switching="switching"
        :success-message="successMessage"
        :show-crosshair="showCrosshair"
        @hero-lob="quickHeroLob"
        @normal="quickNormal"
        @open="openDrawer"
        @fullscreen="toggleFullscreen"
        @toggle-crosshair="toggleCrosshair"
      />
    </div>

    <RmBottomShortcutBar />

    <section v-if="helpVisible" class="shortcut-help rm-glass-panel rm-angular">
      <div>
        <h3>模式</h3>
        <p><kbd class="rm-key">H</kbd> 英雄吊射</p>
        <p><kbd class="rm-key">N</kbd> 普通图传</p>
      </div>
      <div>
        <h3>面板</h3>
        <p><kbd class="rm-key">P</kbd> 参数</p>
        <p><kbd class="rm-key">D</kbd> 调试</p>
        <p><kbd class="rm-key">C</kbd> 通信</p>
        <p><kbd class="rm-key">Tab</kbd> 帮助</p>
      </div>
      <div>
        <h3>准星</h3>
        <p><kbd class="rm-key">方向键</kbd> 微调</p>
        <p><kbd class="rm-key">Shift</kbd> + 方向键 大步长</p>
        <p><kbd class="rm-key">Ctrl</kbd> + 方向键 小步长</p>
        <p><kbd class="rm-key">V</kbd> 显示/隐藏</p>
        <p><kbd class="rm-key">R</kbd> 恢复默认</p>
        <p><kbd class="rm-key">S</kbd> 保存配置</p>
      </div>
      <div>
        <h3>预设</h3>
        <p><kbd class="rm-key">1/2/3</kbd> 切换预设</p>
        <p><kbd class="rm-key">Ctrl</kbd> + 1/2/3 保存预设</p>
      </div>
      <div>
        <h3>全屏</h3>
        <p><kbd class="rm-key">F11</kbd> 全屏</p>
        <p><kbd class="rm-key">Esc</kbd> 关闭/退出</p>
      </div>
      <div>
        <h3>HUD</h3>
        <p><kbd class="rm-key">U</kbd> 编辑器</p>
        <p><kbd class="rm-key">Del</kbd> 删除元素</p>
        <p><kbd class="rm-key">Ctrl</kbd> + Z/Y 撤销/重做</p>
      </div>
      <div>
        <h3>键鼠</h3>
        <p><kbd class="rm-key">Enter</kbd> FPS 模式</p>
        <p><kbd class="rm-key">WASD</kbd> 移动位</p>
        <p><kbd class="rm-key">I</kbd> 诊断面板</p>
        <p><kbd class="rm-key">Left</kbd> 默认禁用发射</p>
      </div>
    </section>

    <div v-if="lastError" class="hud-last-error">{{ lastError }}</div>
    <div class="preset-chip">PRESET {{ uiStore.activePresetId }} / {{ activePresetName }}</div>

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

      <n-drawer-content v-if="activeDrawer === 'hud'" title="HUD EDITOR" closable>
        <RmHudPanel title="Custom HUD">
          <HudEditorPanel />
        </RmHudPanel>
      </n-drawer-content>

      <n-drawer-content v-if="activeDrawer === 'input'" title="LOW LATENCY INPUT" closable>
        <RmHudPanel title="Keyboard / Mouse">
          <InputDiagnosticsPanel />
        </RmHudPanel>
      </n-drawer-content>

      <n-drawer-content v-if="activeDrawer === 'comm'" title="COMMUNICATION" closable>
        <n-space vertical size="large">
          <RmHudPanel title="MQTT Link">
            <n-input v-model:value="host" placeholder="MQTT Host" />
            <n-input-number v-model:value="port" :min="1" :max="65535" style="width: 100%" />
            <n-select
              v-model:value="clientId"
              :options="robotOptions"
              placeholder="选择机器人 Client ID"
            />
            <n-space size="small">
              <n-button size="small" secondary @click="useLocalMqttEndpoint">127.0.0.1</n-button>
              <n-button size="small" secondary @click="useOfficialMqttEndpoint">192.168.12.1</n-button>
            </n-space>
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
              <n-tag>CLIENT {{ modeStore.mqttClientId ?? clientId }}</n-tag>
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
                普通图传 UDP 3334 / HEVC
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

          <RmHudPanel title="CustomByteBlock H264">
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
.right-hud-stack {
  position: absolute;
  right: 24px;
  top: 50%;
  z-index: 14;
  display: flex;
  align-items: center;
  gap: 8px;
  transform: translateY(-50%);
  transition: transform 180ms ease;
}

.right-hud-stack.collapsed {
  transform: translate(calc(100% - 18px), -50%);
}

.right-hud-content {
  display: flex;
  width: 286px;
  max-height: calc(100vh - 168px);
  flex-direction: column;
  gap: 12px;
  overflow: auto;
  padding-right: 2px;
}

.right-hud-toggle {
  width: 26px;
  height: 76px;
  border: 1px solid rgba(0, 229, 255, 0.4);
  border-radius: 8px 0 0 8px;
  background: rgba(8, 12, 18, 0.82);
  color: var(--rm-op-cyan);
  cursor: pointer;
  font-size: 18px;
  line-height: 1;
  box-shadow: 0 0 18px rgba(0, 229, 255, 0.14);
}

.right-hud-toggle:hover {
  border-color: var(--rm-op-cyan);
}

.fullscreen-actions {
  position: absolute;
  right: 22px;
  bottom: 58px;
  z-index: 18;
}

.shortcut-help {
  position: absolute;
  left: 50%;
  top: 105px;
  z-index: 30;
  display: grid;
  width: min(980px, calc(100vw - 48px));
  grid-template-columns: repeat(7, minmax(0, 1fr));
  gap: 16px;
  padding: 18px;
  border-color: rgba(0, 229, 255, 0.52);
  background: rgba(2, 6, 10, 0.86);
  transform: translateX(-50%);
}

.shortcut-help h3 {
  margin: 0 0 10px;
  color: var(--rm-op-cyan);
  font-size: 13px;
}

.shortcut-help p {
  display: flex;
  align-items: center;
  gap: 7px;
  margin: 7px 0;
  color: rgba(234, 247, 255, 0.78);
  font-size: 12px;
}

.hud-last-error {
  position: absolute;
  left: 50%;
  top: 78px;
  z-index: 19;
  max-width: min(620px, calc(100vw - 48px));
  padding: 6px 12px;
  border: 1px solid rgba(255, 48, 69, 0.5);
  border-radius: 5px;
  background: rgba(26, 6, 8, 0.72);
  color: var(--rm-op-red);
  font-size: 12px;
  transform: translateX(-50%);
}

.preset-chip {
  position: absolute;
  left: 24px;
  bottom: 18px;
  z-index: 16;
  padding: 6px 10px;
  border: 1px solid rgba(0, 229, 255, 0.28);
  border-radius: 5px;
  background: rgba(8, 12, 18, 0.64);
  color: var(--rm-op-cyan);
  font-size: 11px;
  font-weight: 800;
}

.competition-fullscreen .preset-chip {
  left: 22px;
  bottom: 18px;
}

.fps-control-active {
  cursor: none;
}

@media (max-width: 1180px) {
  .right-hud-stack {
    right: 14px;
  }

  .right-hud-content {
    width: 230px;
  }

  .shortcut-help {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
