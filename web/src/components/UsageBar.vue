<script setup lang="ts">
import { computed } from 'vue'
const props = defineProps<{ percent: number; label?: string }>()
const value = computed(() => Math.max(0, Math.min(100, Number(props.percent) || 0)))
const tone = computed(() => (value.value >= 95 ? 'danger' : value.value >= 80 ? 'warn' : ''))
</script>

<template>
  <div class="usage">
    <div class="bar" :class="tone" role="progressbar" :aria-label="label || '使用进度'" aria-valuemin="0" aria-valuemax="100" :aria-valuenow="value.toFixed(1)">
      <i :style="{ width: `${value.toFixed(2)}%` }" />
    </div>
    <span class="small muted">{{ value.toFixed(1) }}%</span>
  </div>
</template>

<style scoped>
.usage { display: flex; align-items: center; gap: 8px; min-width: 120px; }
.usage .bar { flex: 1; }
</style>
