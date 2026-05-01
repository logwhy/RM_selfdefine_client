<script setup lang="ts">
import { computed } from 'vue'
import { useVideoStore } from '../stores/video'

withDefaults(
  defineProps<{
    successMessage: string
    showCrosshair?: boolean
  }>(),
  {
    showCrosshair: true,
  },
)

const videoStore = useVideoStore()
const modeLabel = computed(() =>
  videoStore.currentMode === 'hero_lob' ? 'HERO LOB / 0x0310 / H264' : 'NORMAL / UDP / HEVC',
)
</script>

<template>
  <div class="rm-crosshair-hud">
    <template v-if="showCrosshair">
      <div class="ring outer" />
      <div class="ring inner" />
      <div class="tick top" />
      <div class="tick right" />
      <div class="tick bottom" />
      <div class="tick left" />
      <div v-if="videoStore.currentMode === 'hero_lob'" class="mode-tag">{{ modeLabel }}</div>
    </template>
    <div v-if="successMessage" class="center-toast">{{ successMessage }}</div>
    <div v-if="!videoStore.streamAlive" class="connect-hint">
      按 P 打开面板，选择视频源后连接；或点击右侧一键英雄吊射
    </div>
  </div>
</template>

<style scoped>
.rm-crosshair-hud {
  position: absolute;
  inset: 0;
  z-index: 11;
  pointer-events: none;
}

.ring,
.tick {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
}

.ring {
  border: 2px solid rgba(0, 229, 255, 0.78);
  border-radius: 50%;
  box-shadow: 0 0 18px rgba(0, 229, 255, 0.3);
}

.outer {
  width: 86px;
  height: 86px;
  border-left-color: transparent;
  border-right-color: transparent;
}

.inner {
  width: 48px;
  height: 48px;
  border-top-color: rgba(138, 77, 255, 0.9);
  border-bottom-color: rgba(138, 77, 255, 0.9);
}

.tick {
  background: rgba(0, 229, 255, 0.9);
  box-shadow: 0 0 10px rgba(0, 229, 255, 0.42);
}

.tick.top,
.tick.bottom {
  width: 2px;
  height: 16px;
}

.tick.left,
.tick.right {
  width: 16px;
  height: 2px;
}

.tick.top {
  transform: translate(-50%, -72px);
}

.tick.bottom {
  transform: translate(-50%, 56px);
}

.tick.left {
  transform: translate(-72px, -50%);
}

.tick.right {
  transform: translate(56px, -50%);
}

.mode-tag {
  position: absolute;
  left: 50%;
  top: calc(50% + 68px);
  transform: translateX(-50%);
  padding: 4px 10px;
  border: 1px solid rgba(255, 201, 58, 0.5);
  border-radius: 999px;
  background: rgba(15, 10, 6, 0.68);
  color: var(--rm-op-yellow);
  font-size: 11px;
  font-weight: 900;
  letter-spacing: 0.12em;
}

.center-toast,
.connect-hint {
  position: absolute;
  left: 50%;
  top: calc(50% - 102px);
  transform: translateX(-50%);
  color: var(--rm-op-green);
  font-size: 18px;
  font-weight: 900;
  text-shadow: 0 0 18px rgba(44, 255, 140, 0.35);
}

.connect-hint {
  top: calc(50% + 112px);
  color: rgba(234, 247, 255, 0.62);
  font-size: 18px;
  font-weight: 500;
  text-shadow: 0 0 18px rgba(0, 229, 255, 0.26);
}
</style>
