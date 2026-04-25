<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string
    value: string | number | boolean
    tone?: 'cyan' | 'blue' | 'red' | 'orange' | 'green' | 'muted'
  }>(),
  {
    tone: 'cyan',
  },
)
</script>

<template>
  <div class="rm-status-pill" :class="`tone-${tone}`">
    <span class="pill-label">{{ label }}</span>
    <span class="pill-value">{{ value }}</span>
  </div>
</template>

<style scoped>
.rm-status-pill {
  display: inline-flex;
  min-height: 28px;
  align-items: center;
  gap: 7px;
  padding: 4px 9px;
  border: 1px solid currentColor;
  background: rgba(4, 11, 18, 0.64);
  box-shadow: inset 0 0 12px rgba(255, 255, 255, 0.03);
  clip-path: polygon(8px 0, 100% 0, 100% calc(100% - 8px), calc(100% - 8px) 100%, 0 100%, 0 8px);
  color: var(--rm-cyan);
  font-size: 11px;
  white-space: nowrap;
}

.pill-label {
  color: var(--rm-muted);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.pill-value {
  color: currentColor;
  font-weight: 800;
}

.tone-blue {
  color: var(--rm-blue);
}

.tone-red {
  color: var(--rm-red);
}

.tone-orange {
  color: var(--rm-orange);
}

.tone-green {
  color: var(--rm-green);
}

.tone-muted {
  color: var(--rm-muted);
}
</style>
