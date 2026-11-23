<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import { api } from '@/lib/api'
import {
  asArray, formatAnyTs, formatCompactNumber, formatDurationShort, formatLatencyMs, formatMoneyCent, formatRatePerSecond,
  formatRelative, formatTrafficKb,
} from '@/lib/format'
import { useAsync } from '@/lib/useAsync'

const { data, loading, error, reload } = useAsync(() => api<any>('/api/v1/admin/command-center'))
const overview = computed(() => data.value?.overview || {})
const system = computed(() => data.value?.system || {})
const trend = computed(() => asArray(data.value?.traffic_trend))
const trendMax = computed(() => Math.max(1, ...trend.value.map((day: any) => Number(day.total_kb) || 0)))
const nodes = computed(() => asArray(data.value?.node_watchlist))
const LANE_LABELS: Record<string, string> = { push_traffic: '流量上报', alive_session: '在线会话', legacy_submit: '兼容上报' }
const queueLanes = computed(() =>
  Object.entries((system.value.rust_async_queues || {}) as Record<string, any>)
    .filter(([, lane]) => lane && typeof lane === 'object')
    .map(([key, lane]) => ({
      key, label: LANE_LABELS[key] || key, queued: Number(lane.queued_jobs) || 0, capacity: Number(lane.capacity) || 0,
      failures: Number(lane.flush_failures_total) || 0,
    })),
)

// Background refresh keeps the last good data on screen when a poll fails.
const auto = ref(true)
let timer: number | undefined
function schedule() {
  window.clearTimeout(timer)
  const seconds = Math.max(10, Number(data.value?.refresh_interval_seconds) || 20)
  timer = window.setTimeout(async () => {
    if (auto.value && !document.hidden) await reload(true).catch(() => null)
    schedule()
  }, seconds * 1000)
}
onMounted(schedule)
onBeforeUnmount(() => window.clearTimeout(timer))

const tcpingText = (node: any) =>
  node.tcping_status === 'unsupported' ? '不支持探测' : node.tcping_status === 'offline' ? '探测失败' : node.tcping_status === 'online' ? '探测正常' : '等待探测'
const tcpingClass = (node: any) => (node.tcping_status === 'offline' ? 'danger' : node.tcping_status === 'online' ? 'ok' : '')
const actionClass = (value: unknown) => (value === 'blocked' ? 'danger' : value === 'allowed' ? 'ok' : '')
const actionText = (value: unknown) => (value === 'blocked' ? '已拦截' : value === 'allowed' ? '已放行' : '已记录')
const percent = (part: unknown, total: unknown) => (Number(total) > 0 ? Math.round((Number(part) / Number(total)) * 100) : 0)
</script>

<template>
  <div class="page wide">
    <div class="page-head">
      <div><h2>指挥中心</h2><p>全站节点、流量、告警和待办的实时情况。更新于 {{ formatAnyTs(data?.generated_at) }}</p></div>
      <div class="actions">
        <label class="switch"><input v-model="auto" type="checkbox" role="switch"><span>自动刷新</span></label>
        <button type="button" class="btn" @click="reload(true)">刷新</button>
      </div>
    </div>
    <div v-if="data && error" class="notice error mb">刷新失败，显示的是上一次的数据：{{ error }}</div>
    <AsyncState :loading="loading" :error="data ? '' : error" @retry="reload()">
      <div class="grid g4 mb">
        <div class="card stat"><div class="k">在线用户</div><div class="v tnum">{{ formatCompactNumber(overview.live_users) }}<small> / {{ formatCompactNumber(overview.total_users) }}</small></div><div class="d">{{ formatCompactNumber(overview.active_subscriptions) }} 个有效订阅</div></div>
        <div class="card stat"><div class="k">在线节点</div><div class="v tnum">{{ overview.online_nodes ?? 0 }}<small> / {{ overview.total_nodes ?? 0 }}</small></div><div class="d">{{ overview.maintenance_nodes ?? 0 }} 个维护中，{{ overview.traffic_hot_nodes ?? 0 }} 个流量告急</div></div>
        <div class="card stat"><div class="k">实时速率</div><div class="v tnum">↓ {{ formatRatePerSecond(overview.throughput_download_bps) }}</div><div class="d">↑ {{ formatRatePerSecond(overview.throughput_upload_bps) }}</div></div>
        <div class="card stat"><div class="k">今日流量</div><div class="v tnum">{{ formatTrafficKb(overview.traffic_today_kb) }}</div><div class="d">{{ formatCompactNumber(overview.traffic_today_unique_users) }} 位用户使用</div></div>
        <div class="card stat"><div class="k">24 小时收入</div><div class="v tnum">{{ formatMoneyCent(overview.revenue_24h_amount) }}</div><div class="d">{{ overview.completed_orders_24h ?? 0 }} 笔已完成订单</div></div>
        <div class="card stat"><div class="k">活动告警</div><div class="v tnum" :class="{ hot: Number(overview.tcping_alerts_active) > 0 }">{{ overview.tcping_alerts_active ?? 0 }}</div><div class="d">{{ overview.tcping_enabled_nodes ?? 0 }} 个节点开启探测</div></div>
        <div class="card stat"><div class="k">探针</div><div class="v tnum">{{ overview.tcping_agents_online ?? 0 }}<small> / {{ overview.tcping_agents_total ?? 0 }} 在线</small></div></div>
        <div class="card stat"><div class="k">待办</div><div class="v tnum">{{ overview.open_tickets ?? 0 }}<small> 工单</small></div><div class="d">{{ overview.pending_refunds ?? 0 }} 笔待处理退款</div></div>
      </div>

      <div class="grid g2 mb">
        <div class="card">
          <div class="card-head"><h3>最近 7 天流量</h3></div>
          <div class="card-body">
            <div v-if="!trend.length" class="muted">暂无流量数据。</div>
            <div v-else class="bars" role="img" :aria-label="`最近 ${trend.length} 天流量走势`">
              <div v-for="day in trend" :key="day.date" class="bar-col" :title="`${day.date}：${formatTrafficKb(day.total_kb)}，${day.unique_users} 位用户`">
                <span class="bar-v tnum">{{ formatTrafficKb(day.total_kb) }}</span>
                <div class="bar" :style="{ height: `${Math.max(2, (Number(day.total_kb) / trendMax) * 100)}%` }" />
                <span class="bar-k">{{ String(day.date).slice(5) }}</span>
              </div>
            </div>
          </div>
        </div>
        <div class="card">
          <div class="card-head"><h3>系统状态</h3></div>
          <div class="card-body">
            <dl class="kv">
              <dt>定时调度</dt><dd><span class="badge" :class="system.schedule_ok == null ? '' : system.schedule_ok ? 'ok' : 'danger'">{{ system.schedule_ok == null ? '未知' : system.schedule_ok ? '运行中' : '未运行' }}</span><span class="muted small"> 上次运行 {{ formatAnyTs(system.schedule_last_runtime) }}</span></dd>
              <dt>日志（info / warning / error）</dt><dd class="tnum">{{ system.logs?.info ?? 0 }} / {{ system.logs?.warning ?? 0 }} / <b :class="{ hot: Number(system.logs?.error) > 0 }">{{ system.logs?.error ?? 0 }}</b></dd>
              <dt>后台队列</dt><dd>
                <div v-for="lane in queueLanes" :key="lane.key" class="small">{{ lane.label }}：排队 {{ lane.queued }} / 容量 {{ lane.capacity }}<b v-if="lane.failures > 0" class="hot">，失败 {{ lane.failures }} 次</b></div>
                <span v-if="!queueLanes.length" class="muted">没有队列数据</span>
              </dd>
            </dl>
            <RouterLink to="/admin/system" class="small">查看系统日志</RouterLink>
          </div>
        </div>
      </div>

      <div class="card mb">
        <div class="card-head"><h3>需要关注的节点</h3><span class="muted small">按风险从高到低</span></div>
        <div v-if="!nodes.length" class="empty">没有节点。</div>
        <div v-else class="table-wrap">
          <table class="table">
            <thead><tr><th>节点</th><th>地区 / 协议</th><th>状态</th><th>探测</th><th>流量使用</th><th>在线</th><th>近 7 天</th><th>所有者</th></tr></thead>
            <tbody>
              <tr v-for="node in nodes" :key="node.id">
                <td><b>{{ node.name || '-' }}</b><div class="muted small mono">{{ node.host }}:{{ node.port }}</div></td>
                <td>{{ node.location_name || '-' }}<div class="muted small">{{ node.protocol_label || node.protocol }}</div></td>
                <td><span class="badge" :class="node.online_status === 'online' ? 'ok' : 'danger'">{{ node.online_status === 'online' ? '在线' : '离线' }}</span><span v-if="node.status === 'maintenance'" class="badge warn" style="margin-left: 4px">维护中</span></td>
                <td><span class="badge" :class="tcpingClass(node)">{{ tcpingText(node) }}</span>
                  <div v-if="node.tcping_status === 'online'" class="muted small">{{ formatLatencyMs(node.tcping_last_latency_ms) }}</div>
                  <div v-else-if="node.active_alert?.latest_error || node.tcping_last_error" class="muted small err">{{ node.active_alert?.latest_error || node.tcping_last_error }}</div>
                </td>
                <td class="tnum nowrap">{{ formatTrafficKb(node.traffic_used_kb) }}<span v-if="Number(node.traffic_limit_kb) > 0" class="muted"> / {{ formatTrafficKb(node.traffic_limit_kb) }}</span>
                  <div v-if="Number(node.traffic_limit_kb) > 0" class="small" :class="Number(node.traffic_usage_percentage) >= 80 ? 'hot' : 'muted'">{{ Number(node.traffic_usage_percentage || 0).toFixed(0) }}%</div>
                </td>
                <td class="tnum">{{ node.online_users ?? 0 }} 人<div class="muted small">{{ node.active_connections ?? 0 }} 连接</div></td>
                <td class="tnum nowrap">{{ formatTrafficKb(node.weekly_traffic_kb) }}<div class="muted small">{{ node.weekly_unique_users ?? 0 }} 位用户</div></td>
                <td class="small">{{ node.owner_name || node.owner_email || '-' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="grid g2 mb">
        <div class="card">
          <div class="card-head"><h3>活动告警</h3></div>
          <div v-if="!asArray(data?.tcping_alerts).length" class="empty">当前没有告警。</div>
          <ul v-else class="feed">
            <li v-for="alert in asArray(data?.tcping_alerts)" :key="alert.id">
              <div><b>{{ alert.node_name || '-' }}</b><span class="muted small"> {{ alert.node_location_name }} · {{ alert.node_protocol }}</span></div>
              <div class="muted small">已持续 {{ formatDurationShort(alert.duration_seconds) }}，所有者 {{ alert.user_name || alert.user_email || '-' }}</div>
              <div v-if="alert.latest_error" class="small err">{{ alert.latest_error }}</div>
            </li>
          </ul>
        </div>
        <div class="card">
          <div class="card-head"><h3>探针</h3></div>
          <div v-if="!asArray(data?.tcping_agents).length" class="empty">还没有探针。</div>
          <ul v-else class="feed">
            <li v-for="agent in asArray(data?.tcping_agents)" :key="agent.id" class="row">
              <div><b>{{ agent.name || `探针 #${agent.id}` }}</b><div class="muted small">{{ agent.owner_name || agent.owner_email || '-' }} · 心跳 {{ formatRelative(agent.last_heartbeat_at) }}</div></div>
              <span class="sp" /><span class="badge" :class="agent.is_online ? 'ok' : 'danger'">{{ agent.is_online ? '在线' : '离线' }}</span>
            </li>
          </ul>
        </div>
      </div>

      <div class="grid g2 mb">
        <div class="card">
          <div class="card-head"><h3>协议分布</h3></div>
          <div class="card-body dist">
            <div v-for="item in asArray(data?.protocol_distribution)" :key="item.protocol" class="dist-row">
              <span>{{ item.label || item.protocol }}</span><div class="meter"><i :style="{ width: `${percent(item.online, item.total)}%` }" /></div><span class="tnum muted small">{{ item.online }} / {{ item.total }} 在线</span>
            </div>
            <div v-if="!asArray(data?.protocol_distribution).length" class="muted">暂无数据。</div>
          </div>
        </div>
        <div class="card">
          <div class="card-head"><h3>地区分布</h3></div>
          <div class="card-body dist">
            <div v-for="item in asArray(data?.region_distribution)" :key="item.location_name" class="dist-row">
              <span>{{ item.location_name }}</span><div class="meter"><i :style="{ width: `${percent(item.online, item.total)}%` }" /></div><span class="tnum muted small">{{ item.online }} / {{ item.total }} 在线</span>
            </div>
            <div v-if="!asArray(data?.region_distribution).length" class="muted">暂无数据。</div>
          </div>
        </div>
      </div>

      <div class="grid g2 mb">
        <div class="card">
          <div class="card-head"><h3>待处理工单</h3><RouterLink to="/admin/tickets" class="small">全部工单</RouterLink></div>
          <div v-if="!asArray(data?.tickets).length" class="empty">没有待处理工单。</div>
          <ul v-else class="feed">
            <li v-for="ticket in asArray(data?.tickets)" :key="ticket.id">
              <div><b>#{{ ticket.id }} {{ ticket.subject || '-' }}</b></div>
              <div class="muted small">{{ ticket.user_name || ticket.user_email || '-' }}<template v-if="ticket.node_name"> · {{ ticket.node_name }}</template> · 更新于 {{ formatRelative(ticket.updated_at) }}</div>
            </li>
          </ul>
        </div>
        <div class="card">
          <div class="card-head"><h3>待处理退款</h3></div>
          <div v-if="!asArray(data?.refunds).length" class="empty">没有待处理退款。</div>
          <ul v-else class="feed">
            <li v-for="refund in asArray(data?.refunds)" :key="refund.id">
              <div><b class="mono">{{ refund.trade_no }}</b> <span class="badge warn">{{ refund.status === 'voting' ? '投票中' : '待处理' }}</span></div>
              <div class="muted small">{{ refund.user_name || refund.user_email || '-' }} · {{ refund.plan_name || '-' }} · 申请 {{ formatMoneyCent(refund.refund_amount) }} / 原金额 {{ formatMoneyCent(refund.gateway_amount) }}</div>
            </li>
          </ul>
        </div>
      </div>

      <div class="grid g2">
        <div class="card">
          <div class="card-head"><h3>流量最高的用户（近 7 天）</h3></div>
          <div v-if="!asArray(data?.top_users).length" class="empty">暂无数据。</div>
          <div v-else class="table-wrap">
            <table class="table">
              <thead><tr><th>用户</th><th>套餐</th><th>流量</th><th>最近登录</th></tr></thead>
              <tbody>
                <tr v-for="user in asArray(data?.top_users)" :key="user.id">
                  <td>{{ user.display_name || user.email }}<div class="muted small">#{{ user.id }} · {{ user.email }}</div></td>
                  <td>{{ user.plan_name || '-' }}</td>
                  <td class="tnum nowrap">{{ formatTrafficKb(user.traffic_kb) }}</td>
                  <td class="muted nowrap">{{ formatRelative(user.last_login_at) }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
        <div class="card">
          <div class="card-head"><h3>审计事件</h3><RouterLink to="/audit" class="small">审计日志</RouterLink></div>
          <div v-if="!asArray(data?.audit_stream).length" class="empty">暂无审计事件。</div>
          <ul v-else class="feed">
            <li v-for="log in asArray(data?.audit_stream)" :key="log.id">
              <div><span class="badge" :class="actionClass(log.action_taken)">{{ actionText(log.action_taken) }}</span> <b class="mono">{{ log.target_domain || '-' }}</b></div>
              <div class="muted small">{{ log.user_name || log.user_email || '-' }} · {{ log.node_name || '-' }} · {{ log.ip_address || '-' }} · {{ formatRelative(log.created_at) }}</div>
            </li>
          </ul>
        </div>
      </div>
    </AsyncState>
  </div>
</template>

<style scoped>
.hot { color: var(--danger); }
.err { color: var(--danger); word-break: break-word; }
.bars { display: flex; align-items: flex-end; gap: 10px; height: 180px; }
.bar-col { flex: 1; min-width: 0; height: 100%; display: flex; flex-direction: column; justify-content: flex-end; align-items: center; gap: 4px; }
.bar { width: 100%; max-width: 44px; background: var(--accent); border-radius: 4px 4px 0 0; }
.bar-v { font-size: 11px; color: var(--muted); white-space: nowrap; }
.bar-k { font-size: 11.5px; color: var(--muted); }
.feed { list-style: none; margin: 0; padding: 0; }
.feed li { padding: 10px 16px; border-top: 1px solid var(--border); }
.feed li:first-child { border-top: 0; }
.feed li.row { display: flex; align-items: center; gap: 8px; }
.dist { display: grid; gap: 10px; }
.dist-row { display: grid; grid-template-columns: minmax(80px, 1fr) 2fr max-content; align-items: center; gap: 10px; }
.meter { height: 6px; border-radius: 99px; background: var(--surface-2); overflow: hidden; }
.meter i { display: block; height: 100%; background: var(--accent); border-radius: 99px; }
@media (max-width: 640px) { .bar-v { display: none; } }
</style>
