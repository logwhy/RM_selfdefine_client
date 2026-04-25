<script setup lang="ts">
import { NCard, NDescriptions, NDescriptionsItem } from 'naive-ui'
import { useVideoStore } from '../stores/video'

const videoStore = useVideoStore()
</script>

<template>
  <n-card title="链路调试信息" size="small" class="debug-card" :bordered="false">
    <n-descriptions label-placement="left" :column="1" bordered size="small">
      <n-descriptions-item label="currentMode">{{ videoStore.currentMode }}</n-descriptions-item>
      <n-descriptions-item label="currentVideoSource">{{ videoStore.currentVideoSource }}</n-descriptions-item>
      <n-descriptions-item label="currentCodecMode">{{ videoStore.currentCodecMode }}</n-descriptions-item>
      <n-descriptions-item label="currentDecoderName">{{ videoStore.currentDecoderName }}</n-descriptions-item>
      <n-descriptions-item label="decoderInitSuccess">{{ videoStore.decoderInitSuccess }}</n-descriptions-item>
      <n-descriptions-item label="realDecoderEnabled">{{ videoStore.realDecoderEnabled }}</n-descriptions-item>
      <n-descriptions-item label="stubDecoderEnabled">{{ videoStore.stubDecoderEnabled }}</n-descriptions-item>
      <n-descriptions-item label="customBlockParserMode">{{ videoStore.customBlockParserMode }}</n-descriptions-item>
      <n-descriptions-item label="customBlockPacketsReceived">{{ videoStore.customBlockPacketsReceived }}</n-descriptions-item>
      <n-descriptions-item label="customBlockBytesReceived">{{ videoStore.customBlockBytesReceived }}</n-descriptions-item>
      <n-descriptions-item label="customBlockReadyFrames">{{ videoStore.customBlockReadyFrames }}</n-descriptions-item>
      <n-descriptions-item label="customBlockInvalidPackets">{{ videoStore.customBlockInvalidPackets }}</n-descriptions-item>
      <n-descriptions-item label="customBlockMockActive">{{ videoStore.customBlockMockActive }}</n-descriptions-item>
      <n-descriptions-item label="fps">{{ videoStore.fps }}</n-descriptions-item>
      <n-descriptions-item label="streamAlive">{{ videoStore.streamAlive }}</n-descriptions-item>
      <n-descriptions-item label="packetsReceived">{{ videoStore.packetsReceived }}</n-descriptions-item>
      <n-descriptions-item label="readyFrames">{{ videoStore.readyFrames }}</n-descriptions-item>
      <n-descriptions-item label="activeFrames">{{ videoStore.activeFrames }}</n-descriptions-item>
      <n-descriptions-item label="incompleteFrames">{{ videoStore.incompleteFrames }}</n-descriptions-item>
      <n-descriptions-item label="timeoutDroppedFrames">{{ videoStore.timeoutDroppedFrames }}</n-descriptions-item>
      <n-descriptions-item label="packetLossCount">{{ videoStore.packetLossCount }}</n-descriptions-item>
      <n-descriptions-item label="decoderResetCount">{{ videoStore.decoderResetCount }}</n-descriptions-item>
      <n-descriptions-item label="lastDecodeCostMs">{{ videoStore.lastDecodeCostMs }}</n-descriptions-item>
      <n-descriptions-item label="lastFrameAt">{{ videoStore.lastFrameAt ?? '-' }}</n-descriptions-item>
      <n-descriptions-item label="latestFrameAgeMs">{{ videoStore.latestFrameAgeMs ?? '-' }}</n-descriptions-item>
      <n-descriptions-item label="isRenderingRealFrame">{{ videoStore.isRenderingRealFrame }}</n-descriptions-item>
    </n-descriptions>
  </n-card>
</template>

<style scoped>
.debug-card {
  background: transparent;
}
</style>
