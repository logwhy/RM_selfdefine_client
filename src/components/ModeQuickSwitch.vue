<script setup lang="ts">
defineProps<{
  switching: 'hero_lob' | 'normal' | null
  successMessage: string
}>()

defineEmits<{
  heroLob: []
  normal: []
}>()
</script>

<template>
  <div class="mode-quick-switch">
    <button
      class="quick-mode hero"
      :disabled="switching !== null"
      @click="$emit('heroLob')"
    >
      <span class="quick-title">{{ switching === 'hero_lob' ? '切换中...' : '英雄吊射' }}</span>
      <span class="quick-sub">0x0310 / H264</span>
    </button>
    <button
      class="quick-mode normal"
      :disabled="switching !== null"
      @click="$emit('normal')"
    >
      <span class="quick-title">{{ switching === 'normal' ? '切换中...' : '普通图传' }}</span>
      <span class="quick-sub">UDP / HEVC</span>
    </button>
    <div v-if="successMessage" class="switch-toast">{{ successMessage }}</div>
  </div>
</template>

<style scoped>
.mode-quick-switch {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.quick-mode {
  width: 164px;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 3px;
  padding: 11px 13px;
  border: 1px solid currentColor;
  background: rgba(5, 12, 20, 0.82);
  color: var(--rm-cyan);
  cursor: pointer;
  clip-path: polygon(10px 0, 100% 0, 100% calc(100% - 12px), calc(100% - 12px) 100%, 0 100%, 0 10px);
  box-shadow: 0 0 18px rgba(25, 247, 255, 0.18), inset 0 0 20px rgba(25, 247, 255, 0.08);
}

.quick-mode:disabled {
  cursor: wait;
  opacity: 0.7;
}

.quick-mode.hero {
  color: var(--rm-orange);
  background:
    linear-gradient(135deg, rgba(255, 59, 59, 0.24), transparent 55%),
    rgba(8, 14, 22, 0.9);
  box-shadow: 0 0 24px rgba(255, 59, 59, 0.18), inset 0 0 18px rgba(25, 247, 255, 0.1);
}

.quick-mode.normal {
  color: var(--rm-cyan);
}

.quick-title {
  color: var(--rm-text);
  font-size: 15px;
  font-weight: 900;
}

.quick-sub {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
}

.switch-toast {
  position: absolute;
  right: 0;
  bottom: 100%;
  margin-bottom: 8px;
  padding: 6px 10px;
  border: 1px solid var(--rm-green);
  background: rgba(5, 20, 14, 0.9);
  color: var(--rm-green);
  font-size: 12px;
  white-space: nowrap;
}
</style>
