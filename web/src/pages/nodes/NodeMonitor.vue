<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import LatencyChart, { type LatencySeries } from '@/components/LatencyChart.vue'
import { api } from '@/lib/api'
import { asArray, formatLatencyMs, formatTs } from '@/lib/format'
import { TCPING_MONITOR_RANGE_OPTIONS } from '@/lib/nodeConstants'
import { protocolDisplayName } from '@/lib/nodeProtocol'
import { tcpingStatus } from '@/lib/nodes'
import { useAsync } from '@/lib/useAsync'

const route = useRoute()
const nodeId = computed(() => String(route.params.id))
const hours = ref(Math.max(1, Math.min(720, Number(route.query.hours) || 24)))
const hidden = ref(new Set<string>())

const { data, loading, error, reload } = useAsync(() =>
  api<any>(`/api/v1/user/server-nodes/${encodeURIComponent(nodeId.value)}/tcping`, { query: { hours: hours.value } }))
watch(hours, () => reload())

const node = computed(() => data.value?.node || {})
const stats = computed(() => data.value?.stats || {})
const alerts = computed(() => asArray(data.value?.alerts))
const unsupported = computed(() => String(node.value.tcping_status || '').toLowerCase() === 'unsupported')

const series = computed<LatencySeries[]>(() => {
  const agents = new Map<number, any>(asArray(data.value?.agents).map((agent: any) => [Number(agent?.id), agent]))
  const buckets = new Map<string, any[]>()
  for (const sample of asArray(data.value?.samples)) {
    const agent = agents.get(Number(sample?.agent_id))
    const label = String(agent?.location_display || agent?.location_name || '未知地区').trim() || '未知地区'
    if (!buckets.has(label)) buckets.set(label, [])
    buckets.get(label)!.push(sample)
  }
  return Array.from(buckets.entries())
    .sort((a, b) => a[0].localeCompare(b[0], 'zh-CN'))
    .map(([label, samples], index) => ({ key: label, label, color: `var(--chart-${(index % 8) + 1})`, samples }))
})
const visible = computed(() => series.value.filter((item) => !hidden.value.has(item.key)))
function toggle(key: string) {
  const next = new Set(hidden.value)
  if (next.has(key)) next.delete(key)
  else next.add(key)
  hidden.value = next
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div>
        <h2>节点监控：{{ node.name || `#${nodeId}` }}</h2>
        <p>{{ node.location_name || '未设置地区' }} · {{ protocolDisplayName(node.protocol) }}</p>
      </div>
      <div class="actions">
        <div class="seg" role="group" aria-label="时间范围">
          <button v-for="item in TCPING_MONITOR_RANGE_OPTIONS" :key="item.hours" type="button" :class="{ active: hours === item.hours }" @click="hours = item.hours">{{ item.label }}</button>
        </div>
        <RouterLink class="btn" to="/nodes"><ArrowLeft />返回</RouterLink>
      </div>
    </div>

    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <div class="grid g3">
        <div class="card stat">
          <div class="k">当前状态</div>
          <div><span class="badge" :class="tcpingStatus(node.tcping_status, Boolean(node.tcping_enabled)).tone">{{ tcpingStatus(node.tcping_status, Boolean(node.tcping_enabled)).label }}</span></div>
          <div class="s">
            <template v-if="unsupported">未配置 TCPing 探测端口（TCP）</template>
            <template v-else>最近延迟 {{ formatLatencyMs(node.tcping_last_latency_ms) }} · {{ formatTs(node.tcping_last_sampled_at) }}</template>
          </div>
        </div>
        <div class="card stat">
          <div class="k">可达率</div>
          <div class="v">{{ stats.reachability_rate == null ? '-' : `${Number(stats.reachability_rate).toFixed(2)}%` }}</div>
          <div class="s">{{ stats.reachable_samples || 0 }} / {{ stats.total_samples || 0 }} 次成功</div>
        </div>
        <div class="card stat">
          <div class="k">活动告警</div>
          <div><span class="badge" :class="node.tcping_alert_active ? 'danger' : 'ok'">{{ node.tcping_alert_active ? '存在活动告警' : '当前无活动告警' }}</span></div>
          <div class="s">{{ node.tcping_active_alert?.latest_error || '暂无告警详情' }}</div>
        </div>
      </div>

      <div class="card mt">
        <div class="card-head"><h3>延迟变化曲线</h3></div>
        <div class="card-body col">
          <div v-if="unsupported" class="notice">TCPing 不支持 UDP 探测。请在节点高级设置中配置 TCPing 探测端口（TCP）。</div>
          <template v-else>
            <div v-if="series.length > 1" class="row wrap">
              <label v-for="item in series" :key="item.key" class="check small">
                <input type="checkbox" :checked="!hidden.has(item.key)" @change="toggle(item.key)">
                <i class="swatch" :style="{ background: item.color }" />{{ item.label }}
              </label>
            </div>
            <LatencyChart :series="visible" />
          </template>
        </div>
      </div>

      <div class="card mt">
        <div class="card-head"><h3>告警记录</h3></div>
        <div v-if="unsupported || !alerts.length" class="empty">{{ unsupported ? '该节点当前未开启 TCPing 探测，暂无告警。' : '当前没有告警记录。' }}</div>
        <div v-else class="table-wrap">
          <table class="table">
            <thead><tr><th>ID</th><th>状态</th><th>开始时间</th><th>触发时间</th><th>恢复时间</th><th>错误</th></tr></thead>
            <tbody>
              <tr v-for="item in alerts" :key="item.id">
                <td>{{ item.id }}</td>
                <td><span class="badge" :class="item.status === 'active' ? 'danger' : 'ok'">{{ item.status === 'active' ? '活动中' : '已恢复' }}</span></td>
                <td class="muted nowrap">{{ formatTs(item.started_at) }}</td>
                <td class="muted nowrap">{{ formatTs(item.triggered_at) }}</td>
                <td class="muted nowrap">{{ formatTs(item.recovered_at) }}</td>
                <td class="muted">{{ item.latest_error || '-' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </AsyncState>
  </div>
</template>

<style scoped>
.swatch { width: 10px; height: 10px; border-radius: 99px; display: inline-block; }
</style>
