<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import {
  NButton,
  NCard,
  NColorPicker,
  NForm,
  NFormItem,
  NInputNumber,
  NSelect,
  NSpace,
  NSwitch,
} from 'naive-ui'
import { useUiStore } from '../stores/ui'

const uiStore = useUiStore()
const {
  activePresetId,
  crosshairOffsetX,
  crosshairOffsetY,
  crosshairPresets,
  crosshairWidth,
  displayScale,
  showCenterDot,
} = storeToRefs(uiStore)

const editableCrosshairColor = computed({
  get: () => uiStore.crosshairColor ?? '#19f7ff',
  set: (value: string) => {
    uiStore.crosshairColor = value
  },
})

const presetOptions = computed(() =>
  crosshairPresets.value.map((preset) => ({
    label: `${preset.id}. ${preset.name}`,
    value: preset.id,
  })),
)

function handlePresetChange(value: 1 | 2 | 3) {
  uiStore.applyPreset(value)
}
</script>

<template>
  <n-card size="small" title="准星参数" :bordered="false" class="panel-card">
    <n-form label-placement="left" label-width="120">
      <n-form-item label="preset">
        <n-select
          :value="activePresetId"
          :options="presetOptions"
          @update:value="handlePresetChange"
        />
      </n-form-item>
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
      <n-form-item label="color">
        <n-color-picker v-model:value="editableCrosshairColor" :show-alpha="false" />
      </n-form-item>
    </n-form>
    <n-space justify="end">
      <n-button secondary @click="uiStore.resetDefaults">恢复默认</n-button>
      <n-button tertiary @click="uiStore.saveCurrentToPreset(activePresetId)">保存到预设</n-button>
      <n-button type="primary" @click="uiStore.save">保存配置</n-button>
    </n-space>
  </n-card>
</template>

<style scoped>
.panel-card {
  background: transparent;
}
</style>
