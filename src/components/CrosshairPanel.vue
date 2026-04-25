<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { NButton, NCard, NForm, NFormItem, NInputNumber, NSpace, NSwitch } from 'naive-ui'
import { useUiStore } from '../stores/ui'

const uiStore = useUiStore()
const { crosshairOffsetX, crosshairOffsetY, crosshairWidth, displayScale, showCenterDot } =
  storeToRefs(uiStore)

function handleReset() {
  uiStore.resetDefaults()
}

function handleSave() {
  uiStore.save()
}
</script>

<template>
  <n-card size="small" title="准星参数">
    <n-form label-placement="left" label-width="120">
      <n-form-item label="offsetX">
        <n-input-number v-model:value="crosshairOffsetX" :step="1" />
      </n-form-item>
      <n-form-item label="offsetY">
        <n-input-number v-model:value="crosshairOffsetY" :step="1" />
      </n-form-item>
      <n-form-item label="width">
        <n-input-number v-model:value="crosshairWidth" :min="1" :max="8" :step="1" />
      </n-form-item>
      <n-form-item label="displayScale">
        <n-input-number v-model:value="displayScale" :min="0.3" :max="3" :step="0.1" />
      </n-form-item>
      <n-form-item label="showCenterDot">
        <n-switch v-model:value="showCenterDot" />
      </n-form-item>
    </n-form>
    <n-space justify="end">
      <n-button secondary @click="handleReset">恢复默认值</n-button>
      <n-button type="primary" @click="handleSave">保存配置</n-button>
    </n-space>
  </n-card>
</template>
