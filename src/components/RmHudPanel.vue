<script setup lang="ts">
defineProps<{
  title?: string
}>()
</script>

<template>
  <section class="rm-hud-panel rm-corners">
    <div v-if="title" class="rm-panel-title">{{ title }}</div>
    <slot />
  </section>
</template>

<style scoped>
.rm-hud-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px;
  border: 1px solid rgba(25, 247, 255, 0.28);
  background: rgba(6, 13, 22, 0.72);
  box-shadow: inset 0 0 18px rgba(46, 123, 255, 0.08);
  clip-path: polygon(12px 0, 100% 0, 100% calc(100% - 12px), calc(100% - 12px) 100%, 0 100%, 0 12px);
}
</style>
