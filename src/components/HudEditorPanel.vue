<script setup lang="ts">
import { NButton, NColorPicker, NInput, NInputNumber, NSelect, NSpace, NSwitch } from 'naive-ui'
import { useHudEditorStore, type HudDataBinding, type HudTool } from '../stores/hudEditor'

const hudStore = useHudEditorStore()

const toolOptions: { label: string; value: HudTool }[] = [
  { label: '选择', value: 'select' },
  { label: '直线', value: 'line' },
  { label: '矩形', value: 'rect' },
  { label: '圆形', value: 'circle' },
  { label: '文本', value: 'text' },
  { label: '能量条', value: 'bar' },
  { label: '状态灯', value: 'statusLight' },
]

const swatches = ['#00e5ff', '#2cff8c', '#ffc93a', '#ff3045', '#ffffff', '#8a4dff']

const bindingOptions: { label: string; value: HudDataBinding }[] = [
  { label: '无绑定', value: 'none' },
  { label: '比赛倒计时', value: 'GameStatus.stage_countdown_sec' },
  { label: '机器人血量', value: 'RobotDynamicStatus.hp' },
  { label: '枪口热量', value: 'RobotDynamicStatus.heat' },
  { label: 'MQTT 连接', value: 'mqttConnected' },
  { label: '视频 FPS', value: 'fps' },
]
</script>

<template>
  <div class="hud-editor-panel">
    <n-space vertical size="medium">
      <div class="tool-grid">
        <button
          v-for="option in toolOptions"
          :key="option.value"
          class="tool-button"
          :class="{ active: hudStore.activeTool === option.value }"
          @click="hudStore.activeTool = option.value"
        >
          {{ option.label }}
        </button>
      </div>

      <div class="color-row">
        <n-color-picker v-model:value="hudStore.currentColor" :show-alpha="false" size="small" />
        <button
          v-for="color in swatches"
          :key="color"
          class="swatch"
          :class="{ active: hudStore.currentColor.toLowerCase() === color }"
          :style="{ background: color }"
          @click="hudStore.currentColor = color"
        />
      </div>

      <div class="hint-line">在画面上按住拖动即可绘制；选中元素后拖动移动，拖白色手柄改变长度/大小。</div>

      <n-space>
        <n-button size="small" @click="hudStore.undo">撤销</n-button>
        <n-button size="small" @click="hudStore.redo">重做</n-button>
        <n-button size="small" type="error" @click="hudStore.deleteSelected">删除选中</n-button>
      </n-space>
      <n-space>
        <n-button size="small" @click="hudStore.setTemplate(hudStore.defaultHeroTemplate)">默认英雄</n-button>
        <n-button size="small" @click="hudStore.setTemplate(hudStore.minimalTemplate)">极简比赛</n-button>
        <n-button size="small" type="primary" @click="hudStore.save">保存</n-button>
      </n-space>
      <label class="panel-row">
        比赛锁定
        <n-switch v-model:value="hudStore.locked" />
      </label>
      <template v-if="hudStore.selectedElement">
        <div class="selected-title">选中：{{ hudStore.selectedElement.type }}</div>
        <n-input
          v-if="hudStore.selectedElement.type === 'text'"
          :value="hudStore.selectedElement.text"
          @update:value="(text) => hudStore.selectedElement && hudStore.updateElement(hudStore.selectedElement.id, { text })"
        />
        <n-select
          :value="hudStore.selectedElement.binding"
          :options="bindingOptions"
          @update:value="(binding) => hudStore.selectedElement && hudStore.updateElement(hudStore.selectedElement.id, { binding })"
        />
        <n-color-picker
          :value="hudStore.selectedElement.color"
          :show-alpha="false"
          @update:value="(color) => hudStore.selectedElement && hudStore.updateElement(hudStore.selectedElement.id, { color })"
        />
        <div class="numeric-grid">
          <span>X</span>
          <n-input-number
            :value="hudStore.selectedElement.x"
            size="small"
            @update:value="(x) => hudStore.selectedElement && hudStore.updateElement(hudStore.selectedElement.id, { x: x ?? 0 })"
          />
          <span>Y</span>
          <n-input-number
            :value="hudStore.selectedElement.y"
            size="small"
            @update:value="(y) => hudStore.selectedElement && hudStore.updateElement(hudStore.selectedElement.id, { y: y ?? 0 })"
          />
          <span>W</span>
          <n-input-number
            :value="hudStore.selectedElement.w"
            size="small"
            @update:value="(w) => hudStore.selectedElement && hudStore.updateElement(hudStore.selectedElement.id, { w: w ?? 0 })"
          />
          <span>H</span>
          <n-input-number
            :value="hudStore.selectedElement.h"
            size="small"
            @update:value="(h) => hudStore.selectedElement && hudStore.updateElement(hudStore.selectedElement.id, { h: h ?? 0 })"
          />
        </div>
      </template>
    </n-space>
  </div>
</template>

<style scoped>
.hud-editor-panel {
  color: var(--rm-op-text);
}

.tool-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
}

.tool-button {
  height: 30px;
  border: 1px solid rgba(234, 247, 255, 0.18);
  border-radius: 6px;
  background: rgba(234, 247, 255, 0.06);
  color: rgba(234, 247, 255, 0.78);
  cursor: pointer;
  font-size: 12px;
}

.tool-button.active {
  border-color: var(--rm-op-cyan);
  color: var(--rm-op-cyan);
  box-shadow: inset 0 0 14px rgba(0, 229, 255, 0.14);
}

.color-row {
  display: grid;
  grid-template-columns: 1fr repeat(6, 24px);
  gap: 8px;
  align-items: center;
}

.swatch {
  width: 24px;
  height: 24px;
  border: 2px solid rgba(255, 255, 255, 0.28);
  border-radius: 6px;
  cursor: pointer;
}

.swatch.active {
  border-color: #ffffff;
}

.hint-line,
.panel-row,
.numeric-grid {
  color: rgba(234, 247, 255, 0.72);
  font-size: 12px;
}

.panel-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.numeric-grid {
  display: grid;
  grid-template-columns: 24px 1fr 24px 1fr;
  gap: 8px;
  align-items: center;
}

.selected-title {
  color: var(--rm-op-cyan);
  font-size: 12px;
  font-weight: 800;
}
</style>
