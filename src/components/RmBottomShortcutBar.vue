<script setup lang="ts">
const shortcuts = [
  ['H', '一键英雄吊射'],
  ['N', '普通图传'],
  ['P', '参数'],
  ['D', '调试'],
  ['C', '通信'],
  ['F11', '全屏'],
  ['↑↓←→', '微调准星'],
  ['S', '保存配置'],
]
</script>

<template>
  <footer class="rm-shortcut-bar">
    <span v-for="[key, text] in shortcuts" :key="key">
      <kbd class="rm-key">{{ key }}</kbd>
      {{ text }}
    </span>
  </footer>
</template>

<style scoped>
.rm-shortcut-bar {
  position: absolute;
  left: 50%;
  bottom: 14px;
  z-index: 16;
  display: flex;
  max-width: calc(100vw - 42px);
  align-items: center;
  justify-content: center;
  gap: 14px;
  padding: 8px 14px;
  border: 1px solid rgba(138, 77, 255, 0.38);
  border-radius: 999px;
  background: rgba(8, 10, 16, 0.68);
  color: rgba(234, 247, 255, 0.68);
  font-size: 12px;
  transform: translateX(-50%);
  backdrop-filter: blur(12px);
  box-shadow: 0 0 22px rgba(138, 77, 255, 0.16);
}

.rm-shortcut-bar span {
  white-space: nowrap;
}

@media (max-width: 980px) {
  .rm-shortcut-bar {
    display: none;
  }
}
</style>
