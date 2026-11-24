<script setup lang="ts">
import { computed } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import { adminV2 } from '@/lib/api'
import { formatBytes, formatCompactNumber, formatMoneyCent } from '@/lib/format'
import { NAV } from '@/lib/nav'
import { useAsync } from '@/lib/useAsync'
import { useSession } from '@/stores/session'

const session = useSession()
const { data, loading, error, reload } = useAsync(() => adminV2<any>('stat/getStats'))
const growth = (value: unknown) => {
  const n = Number(value)
  if (!Number.isFinite(n) || n === 0) return '与上期持平'
  return `较上期 ${n > 0 ? '+' : ''}${n.toFixed(1)}%`
}
const shortcuts = computed(() =>
  NAV.filter((group) => group.role)
    .flatMap((group) => group.items)
    .filter((item) => item.to !== '/admin' && (item.role !== 'super' || session.isSuperAdmin)),
)
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div><h2>工作台</h2><p>平台收入、用户和节点的整体情况。</p></div>
      <div class="actions"><button type="button" class="btn" @click="reload(true)">刷新</button></div>
    </div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <div class="grid g4 mb">
        <div class="card stat"><div class="k">今日收入</div><div class="v tnum">{{ formatMoneyCent(data?.today_income) }}</div><div class="d">{{ growth(data?.day_income_growth) }}</div></div>
        <div class="card stat"><div class="k">本月收入</div><div class="v tnum">{{ formatMoneyCent(data?.current_month_income) }}</div><div class="d">{{ growth(data?.month_income_growth) }}，上月 {{ formatMoneyCent(data?.last_month_income) }}</div></div>
        <div class="card stat"><div class="k">本月新增用户</div><div class="v tnum">{{ formatCompactNumber(data?.current_month_register_total) }}</div><div class="d">{{ growth(data?.month_register_growth) }}</div></div>
        <div class="card stat"><div class="k">在线用户</div><div class="v tnum">{{ formatCompactNumber(data?.online_users) }}</div><div class="d">{{ formatCompactNumber(data?.online_devices) }} 台设备，{{ formatCompactNumber(data?.online_nodes) }} 个节点在线</div></div>
      </div>
      <div class="grid g2 mb">
        <div class="card">
          <div class="card-head"><h3>待处理</h3></div>
          <div class="card-body todo">
            <RouterLink to="/admin/tickets" class="todo-row"><span>待回复工单</span><b class="tnum" :class="{ hot: Number(data?.ticket_pending_total) > 0 }">{{ data?.ticket_pending_total ?? 0 }}</b></RouterLink>
            <div class="todo-row"><span>待确认佣金</span><b class="tnum">{{ data?.commission_pending_total ?? 0 }}</b></div>
          </div>
        </div>
        <div class="card">
          <div class="card-head"><h3>流量</h3></div>
          <div class="card-body">
            <dl class="kv">
              <dt>今日</dt><dd class="tnum">{{ formatBytes(data?.today_traffic?.total) }}<span class="muted small"> ↑ {{ formatBytes(data?.today_traffic?.upload) }} ↓ {{ formatBytes(data?.today_traffic?.download) }}</span></dd>
              <dt>本月</dt><dd class="tnum">{{ formatBytes(data?.month_traffic?.total) }}<span class="muted small"> ↑ {{ formatBytes(data?.month_traffic?.upload) }} ↓ {{ formatBytes(data?.month_traffic?.download) }}</span></dd>
              <dt>累计</dt><dd class="tnum">{{ formatBytes(data?.total_traffic?.total) }}</dd>
            </dl>
          </div>
        </div>
      </div>
    </AsyncState>
    <div class="card">
      <div class="card-head"><h3>管理入口</h3></div>
      <div class="card-body shortcuts">
        <RouterLink v-for="item in shortcuts" :key="item.to" :to="item.to" class="shortcut"><component :is="item.icon" :size="16" /><span>{{ item.label }}</span></RouterLink>
      </div>
    </div>
  </div>
</template>

<style scoped>
.todo { display: grid; gap: 4px; }
.todo-row { display: flex; align-items: center; justify-content: space-between; padding: 8px 10px; border-radius: var(--radius-sm); color: var(--text); text-decoration: none; }
a.todo-row:hover { background: var(--surface-2); }
.todo-row b.hot { color: var(--danger); }
.shortcuts { display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); gap: 8px; }
.shortcut { display: flex; align-items: center; gap: 8px; padding: 10px 12px; border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text); text-decoration: none; }
.shortcut:hover { border-color: var(--accent); color: var(--accent); }
</style>
