<script setup lang="ts">
import { NCard, NDescriptions, NDescriptionsItem } from 'naive-ui'
import { computed } from 'vue'
import { useModeStore } from '../stores/mode'
import { useVideoStore } from '../stores/video'

const videoStore = useVideoStore()
const modeStore = useModeStore()
const refereeTopicText = computed(() => {
  const entries = Object.entries(modeStore.refereeTopicCounts)
  if (entries.length === 0) return '-'
  return entries.map(([topic, count]) => `${topic}:${count}`).join(' / ')
})
</script>

<template>
  <n-card title="链路调试信息" size="small" class="debug-card" :bordered="false">
    <n-descriptions label-placement="left" :column="1" bordered size="small">
      <n-descriptions-item label="currentMode">{{ videoStore.currentMode }}</n-descriptions-item>
      <n-descriptions-item label="currentVideoSource">{{ videoStore.currentVideoSource }}</n-descriptions-item>
      <n-descriptions-item label="currentCodecMode">{{ videoStore.currentCodecMode }}</n-descriptions-item>
      <n-descriptions-item label="currentDecoderName">{{ videoStore.currentDecoderName }}</n-descriptions-item>
      <n-descriptions-item label="decoderInitSuccess">{{ videoStore.decoderInitSuccess }}</n-descriptions-item>
      <n-descriptions-item label="customBlockPacketsReceived">{{ videoStore.customBlockPacketsReceived }}</n-descriptions-item>
      <n-descriptions-item label="customBlockPacketsPerSecond">{{ videoStore.customBlockPacketsPerSecond }}</n-descriptions-item>
      <n-descriptions-item label="customBlockBitrateKbps">{{ videoStore.customBlockBitrateKbps }}</n-descriptions-item>
      <n-descriptions-item label="customBlockDroppedBlocks">{{ videoStore.customBlockDroppedBlocks }}</n-descriptions-item>
      <n-descriptions-item label="customBlockBufferedBytes">{{ videoStore.customBlockBufferedBytes }}</n-descriptions-item>
      <n-descriptions-item label="customBlockNoDataDurationMs">{{ videoStore.customBlockNoDataDurationMs ?? '-' }}</n-descriptions-item>
      <n-descriptions-item label="h264SeenSps">{{ videoStore.h264SeenSps }}</n-descriptions-item>
      <n-descriptions-item label="h264SeenPps">{{ videoStore.h264SeenPps }}</n-descriptions-item>
      <n-descriptions-item label="h264SeenIdr">{{ videoStore.h264SeenIdr }}</n-descriptions-item>
      <n-descriptions-item label="h264LastNalType">{{ videoStore.h264LastNalType ?? '-' }}</n-descriptions-item>
      <n-descriptions-item label="h264BufferedBytes">{{ videoStore.h264BufferedBytes }}</n-descriptions-item>
      <n-descriptions-item label="h264NalUnitsParsed">{{ videoStore.h264NalUnitsParsed }}</n-descriptions-item>
      <n-descriptions-item label="h264FramesSubmittedToDecoder">{{ videoStore.h264FramesSubmittedToDecoder }}</n-descriptions-item>
      <n-descriptions-item label="h264FramesDecoded">{{ videoStore.h264FramesDecoded }}</n-descriptions-item>
      <n-descriptions-item label="h264FramesDropped">{{ videoStore.h264FramesDropped }}</n-descriptions-item>
      <n-descriptions-item label="h264DecoderErrors">{{ videoStore.h264DecoderErrors }}</n-descriptions-item>
      <n-descriptions-item label="h264ConsecutiveDecodeErrors">{{ videoStore.h264ConsecutiveDecodeErrors }}</n-descriptions-item>
      <n-descriptions-item label="droppedOldFrames">{{ videoStore.droppedOldFrames }}</n-descriptions-item>
      <n-descriptions-item label="droppedByBackpressure">{{ videoStore.droppedByBackpressure }}</n-descriptions-item>
      <n-descriptions-item label="decodeInputQueueLen">{{ videoStore.decodeInputQueueLen }}</n-descriptions-item>
      <n-descriptions-item label="lastDecodeCostMs">{{ videoStore.lastDecodeCostMs }}</n-descriptions-item>
      <n-descriptions-item label="avgDecodeCostMs">{{ videoStore.avgDecodeCostMs }}</n-descriptions-item>
      <n-descriptions-item label="maxDecodeCostMs">{{ videoStore.maxDecodeCostMs }}</n-descriptions-item>
      <n-descriptions-item label="lastRenderCostMs">{{ videoStore.lastRenderCostMs }}</n-descriptions-item>
      <n-descriptions-item label="avgEndToEndLatencyMs">{{ videoStore.avgEndToEndLatencyMs }}</n-descriptions-item>
      <n-descriptions-item label="fps">{{ videoStore.fps }}</n-descriptions-item>
      <n-descriptions-item label="streamAlive">{{ videoStore.streamAlive }}</n-descriptions-item>
      <n-descriptions-item label="refereeLastMessageAt">{{ modeStore.lastRefereeMessageAt ?? '-' }}</n-descriptions-item>
      <n-descriptions-item label="refereeTopicCounts">{{ refereeTopicText }}</n-descriptions-item>
      <n-descriptions-item label="gameStageCountdown">{{ modeStore.gameStatus.stageCountdownSec ?? '-' }}</n-descriptions-item>
      <n-descriptions-item label="robotHpHeat">
        {{ modeStore.robotDynamicStatus.currentHealth ?? '-' }} / {{ modeStore.robotDynamicStatus.currentHeat ?? '-' }}
      </n-descriptions-item>
      <n-descriptions-item label="packetsReceived">{{ videoStore.packetsReceived }}</n-descriptions-item>
      <n-descriptions-item label="readyFrames">{{ videoStore.readyFrames }}</n-descriptions-item>
      <n-descriptions-item label="decoderResetCount">{{ videoStore.decoderResetCount }}</n-descriptions-item>
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
