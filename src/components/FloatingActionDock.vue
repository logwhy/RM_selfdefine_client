<script setup lang="ts">
import ModeQuickSwitch from './ModeQuickSwitch.vue'

defineProps<{
  switching: 'hero_lob' | 'normal' | null
  successMessage: string
}>()

defineEmits<{
  heroLob: []
  normal: []
  open: [panel: 'params' | 'debug' | 'comm' | 'mode']
  fullscreen: []
}>()
</script>

<template>
  <aside class="floating-action-dock">
    <ModeQuickSwitch
      :switching="switching"
      :success-message="successMessage"
      @hero-lob="$emit('heroLob')"
      @normal="$emit('normal')"
    />
    <div class="dock-buttons rm-glass">
      <button @click="$emit('open', 'params')">参数</button>
      <button @click="$emit('open', 'debug')">调试</button>
      <button @click="$emit('open', 'comm')">通信</button>
      <button @click="$emit('open', 'mode')">模式</button>
      <button @click="$emit('fullscreen')">全屏</button>
    </div>
  </aside>
</template>

<style scoped>
.floating-action-dock {
  position: absolute;
  right: 18px;
  bottom: 66px;
  z-index: 8;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 10px;
}

.dock-buttons {
  display: grid;
  grid-template-columns: repeat(5, auto);
  gap: 6px;
  padding: 8px;
  border-radius: 6px;
}

.dock-buttons button {
  min-width: 44px;
  height: 30px;
  border: 1px solid rgba(25, 247, 255, 0.36);
  background: rgba(7, 15, 24, 0.84);
  color: var(--rm-text);
  cursor: pointer;
  font-size: 12px;
}

.dock-buttons button:hover {
  border-color: var(--rm-cyan);
  color: var(--rm-cyan);
}
</style>
