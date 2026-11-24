<script setup lang="ts">
// TCPing latency over time, one line per probe region; unreachable samples are drawn as red dots on the baseline.
import { computed } from 'vue'

export interface LatencySeries {
  key: string
  label: string
  color: string
  samples: any[]
}
const props = defineProps<{ series: LatencySeries[] }>()
const W = 960
const H = 240
const LEFT = 44
const RIGHT = 16
const TOP = 14
const BOTTOM = 26

const model = computed(() => {
  const rows = props.series.flatMap((item) => item.samples)
  if (!rows.length) return null
  const reachable = rows.filter((item) => item?.is_reachable && Number.isFinite(Number(item?.latency_ms)))
  const maxLatency = Math.max(50, ...reachable.map((item) => Number(item.latency_ms || 0)))
  const times = rows.map((item) => Number(item?.sampled_at || 0)).filter((ts) => ts > 0)
  if (!times.length) return null
  const minTime = Math.min(...times)
  const span = Math.max(1, Math.max(...times) - minTime)
  const x = (ts: unknown) => LEFT + ((Number(ts || 0) - minTime) / span) * (W - LEFT - RIGHT)
  const y = (latency: unknown) => TOP + (1 - Number(latency || 0) / maxLatency) * (H - TOP - BOTTOM)
  return {
    ticks: [0.25, 0.5, 0.75, 1].map((ratio) => ({ latency: Math.round(maxLatency * ratio), y: y(maxLatency * ratio) })),
    lines: props.series.map((item) => ({
      key: item.key,
      color: item.color,
      points: item.samples
        .filter((s) => s?.is_reachable && Number.isFinite(Number(s?.latency_ms)))
        .map((s) => `${x(s.sampled_at).toFixed(1)},${y(s.latency_ms).toFixed(1)}`)
        .join(' '),
    })).filter((line) => line.points),
    dots: props.series.flatMap((item, index) =>
      item.samples.filter((s) => !s?.is_reachable).map((s, i) => ({
        key: `${item.key}-${i}`,
        cx: x(s.sampled_at),
        cy: H - BOTTOM - Math.min(18, index * 6),
        stroke: item.color,
      }))),
  }
})
</script>

<template>
  <div v-if="!model" class="empty">当前时间范围内还没有 TCPing 采样。</div>
  <div v-else>
    <svg :viewBox="`0 0 ${W} ${H}`" class="chart" role="img" aria-label="TCPing 延迟曲线">
      <g v-for="tick in model.ticks" :key="tick.latency">
        <line :x1="LEFT" :y1="tick.y" :x2="W - RIGHT" :y2="tick.y" class="grid-line" />
        <text :x="LEFT - 6" :y="tick.y + 4" text-anchor="end" class="tick">{{ tick.latency }}ms</text>
      </g>
      <line :x1="LEFT" :y1="H - BOTTOM" :x2="W - RIGHT" :y2="H - BOTTOM" class="axis" />
      <polyline v-for="line in model.lines" :key="line.key" :points="line.points" fill="none" :stroke="line.color" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
      <circle v-for="dot in model.dots" :key="dot.key" :cx="dot.cx" :cy="dot.cy" r="4" class="down" :stroke="dot.stroke" />
    </svg>
    <div class="row wrap muted small">
      <span>线条：可达延迟（颜色为探针地区）</span>
      <span class="row"><i class="legend-dot" />不可连通</span>
    </div>
  </div>
</template>

<style scoped>
.chart { width: 100%; height: auto; display: block; }
.grid-line { stroke: var(--border); stroke-dasharray: 3 5; }
.axis { stroke: var(--border); }
.tick { font-size: 11px; fill: var(--muted); }
.down { fill: var(--danger); stroke-width: 2; }
.legend-dot { width: 8px; height: 8px; border-radius: 99px; background: var(--danger); display: inline-block; }
</style>
