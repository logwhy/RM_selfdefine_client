<script setup lang="ts">
import { NButton, NDescriptions, NDescriptionsItem, NInputNumber, NSpace, NSwitch, NTag } from 'naive-ui'
import { useInputControlStore } from '../stores/inputControl'

const inputStore = useInputControlStore()
</script>

<template>
  <div class="input-diagnostics-panel">
    <n-space vertical>
      <label class="panel-row">
        dry-run
        <n-switch v-model:value="inputStore.dryRun" @update:value="inputStore.save" />
      </label>
      <label class="panel-row">
        禁用发射
        <n-switch v-model:value="inputStore.disabledFire" @update:value="inputStore.save" />
      </label>
      <label class="panel-row">
        前馈
        <n-switch v-model:value="inputStore.feedforwardEnabled" @update:value="inputStore.save" />
      </label>
      <div class="numeric-grid">
        <span>灵敏度</span>
        <n-input-number v-model:value="inputStore.sensitivity" size="small" :step="0.1" @update:value="inputStore.save" />
        <span>前馈增益</span>
        <n-input-number v-model:value="inputStore.feedforwardGain" size="small" :step="0.001" @update:value="inputStore.save" />
        <span>衰减 ms</span>
        <n-input-number v-model:value="inputStore.feedforwardDecayMs" size="small" :min="0" @update:value="inputStore.save" />
        <span>最大速度</span>
        <n-input-number v-model:value="inputStore.maxMouseSpeed" size="small" :min="1" @update:value="inputStore.save" />
      </div>
      <n-space>
        <n-tag :type="inputStore.fpsMode ? 'success' : 'default'">FPS {{ inputStore.fpsMode ? 'ON' : 'OFF' }}</n-tag>
        <n-tag :type="inputStore.pointerLocked ? 'success' : 'warning'">Pointer {{ inputStore.pointerLocked ? 'LOCKED' : 'FREE' }}</n-tag>
      </n-space>
      <n-descriptions label-placement="left" :column="1" bordered size="small">
        <n-descriptions-item label="inputSendHz">{{ inputStore.diagnostics.inputSendHz }}</n-descriptions-item>
        <n-descriptions-item label="inputLatencyMs">{{ inputStore.diagnostics.inputLatencyMs }}</n-descriptions-item>
        <n-descriptions-item label="droppedInputFrames">{{ inputStore.diagnostics.droppedInputFrames }}</n-descriptions-item>
        <n-descriptions-item label="cmdX/Y">{{ inputStore.diagnostics.cmdX }} / {{ inputStore.diagnostics.cmdY }}</n-descriptions-item>
        <n-descriptions-item label="keyboard_value">{{ inputStore.keyboardValueBinary }}</n-descriptions-item>
      </n-descriptions>
      <n-button size="small" type="primary" @click="inputStore.save">保存控制参数</n-button>
    </n-space>
  </div>
</template>

<style scoped>
.input-diagnostics-panel {
  color: var(--rm-op-text);
}

.panel-row,
.numeric-grid {
  color: rgba(234, 247, 255, 0.78);
  font-size: 12px;
}

.panel-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.numeric-grid {
  display: grid;
  grid-template-columns: 72px 1fr;
  gap: 8px;
  align-items: center;
}
</style>
