<script setup lang="ts">
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
  <aside class="rm-quick-panel rm-glass-panel rm-angular">
    <button class="primary-lob" :disabled="switching !== null" @click="$emit('heroLob')">
      <span>{{ switching === 'hero_lob' ? '切换中...' : '一键英雄吊射' }}</span>
      <b>0x0310 / H264</b>
    </button>
    <button class="normal" :disabled="switching !== null" @click="$emit('normal')">
      <span>{{ switching === 'normal' ? '切换中...' : '普通图传' }}</span>
      <b>UDP / HEVC</b>
    </button>
    <div class="tool-grid">
      <button @click="$emit('open', 'params')">参数</button>
      <button @click="$emit('open', 'debug')">调试</button>
      <button @click="$emit('open', 'comm')">通信</button>
      <button @click="$emit('open', 'mode')">模式</button>
      <button @click="$emit('fullscreen')">全屏</button>
    </div>
    <div v-if="successMessage" class="switch-ok">{{ successMessage }}</div>
  </aside>
</template>

<style scoped>
.rm-quick-panel {
  position: absolute;
  right: 24px;
  top: 50%;
  z-index: 14;
  width: 246px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  transform: translateY(-50%);
  border-color: rgba(138, 77, 255, 0.46);
  box-shadow: 0 0 28px rgba(138, 77, 255, 0.18), inset 0 0 18px rgba(138, 77, 255, 0.08);
}

button {
  border: 1px solid rgba(234, 247, 255, 0.14);
  background: rgba(234, 247, 255, 0.07);
  color: var(--rm-op-text);
  cursor: pointer;
  font-family: inherit;
}

button:disabled {
  cursor: wait;
  opacity: 0.72;
}

.primary-lob,
.normal {
  min-height: 58px;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  gap: 3px;
  padding: 10px 13px;
  border-radius: 7px;
}

.primary-lob {
  border-color: rgba(255, 201, 58, 0.65);
  background:
    linear-gradient(135deg, rgba(255, 48, 69, 0.28), transparent 60%),
    rgba(20, 12, 12, 0.86);
  box-shadow: 0 0 22px rgba(255, 48, 69, 0.22), inset 0 0 20px rgba(255, 201, 58, 0.08);
}

.normal {
  border-color: rgba(0, 229, 255, 0.46);
  background: rgba(5, 18, 26, 0.78);
}

.primary-lob span,
.normal span {
  font-size: 15px;
  font-weight: 900;
}

.primary-lob b {
  color: var(--rm-op-yellow);
}

.normal b {
  color: var(--rm-op-cyan);
}

.tool-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 7px;
}

.tool-grid button {
  height: 32px;
  border-radius: 5px;
  color: var(--rm-op-muted);
}

.tool-grid button:hover {
  border-color: var(--rm-op-cyan);
  color: var(--rm-op-cyan);
}

.switch-ok {
  padding: 7px 9px;
  border: 1px solid rgba(44, 255, 140, 0.42);
  color: var(--rm-op-green);
  font-size: 12px;
  text-align: center;
}

@media (max-width: 1180px) {
  .rm-quick-panel {
    right: 16px;
    width: 210px;
  }
}
</style>
