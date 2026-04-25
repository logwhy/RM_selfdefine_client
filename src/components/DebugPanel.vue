<script setup lang="ts">
import { NCard, NDescriptions, NDescriptionsItem } from 'naive-ui'
import { useVideoStore } from '../stores/video'

const videoStore = useVideoStore()
</script>

<template>
  <n-card title="链路调试信息" size="small" class="debug-card">
    <n-descriptions label-placement="left" :column="1" bordered size="small">
      <n-descriptions-item label="fps">{{ videoStore.fps }}</n-descriptions-item>
      <n-descriptions-item label="decoderLatencyMs">{{ videoStore.decoderLatencyMs }}</n-descriptions-item>
      <n-descriptions-item label="packetLossCount">{{ videoStore.packetLossCount }}</n-descriptions-item>
      <n-descriptions-item label="streamAlive">{{ videoStore.streamAlive }}</n-descriptions-item>
      <n-descriptions-item label="lastFrameAt">{{ videoStore.lastFrameAt ?? '-' }}</n-descriptions-item>
      <n-descriptions-item label="packetsReceived">{{ videoStore.packetsReceived }}</n-descriptions-item>
      <n-descriptions-item label="activeFrames">{{ videoStore.activeFrames }}</n-descriptions-item>
      <n-descriptions-item label="timeoutDroppedFrames">{{ videoStore.timeoutDroppedFrames }}</n-descriptions-item>
      <n-descriptions-item label="incompleteFrames">{{ videoStore.incompleteFrames }}</n-descriptions-item>
      <n-descriptions-item label="readyFrames">{{ videoStore.readyFrames }}</n-descriptions-item>
      <n-descriptions-item label="decoderResetCount">{{ videoStore.decoderResetCount }}</n-descriptions-item>
      <n-descriptions-item label="lastDecodeCostMs">{{ videoStore.lastDecodeCostMs }}</n-descriptions-item>
      <n-descriptions-item label="latestFrameAgeMs">{{ videoStore.latestFrameAgeMs ?? '-' }}</n-descriptions-item>
      <n-descriptions-item label="isRenderingRealFrame">{{ videoStore.isRenderingRealFrame }}</n-descriptions-item>
      <n-descriptions-item label="realDecoderEnabled">{{ videoStore.realDecoderEnabled }}</n-descriptions-item>
      <n-descriptions-item label="mockDecoderEnabled">{{ videoStore.mockDecoderEnabled }}</n-descriptions-item>
    </n-descriptions>
  </n-card>
</template>

<style scoped>
.debug-card {
  border: 1px solid #28415f;
  background: rgba(9, 18, 29, 0.92);
}
</style>
