<script setup lang="ts">
import AsyncState from '@/components/AsyncState.vue'
import { api } from '@/lib/api'
import { asArray, formatAnyTs } from '@/lib/format'
import { useAsync } from '@/lib/useAsync'

const ACTIONS: Record<string, { label: string; tone: string }> = {
  block: { label: '拦截', tone: 'danger' }, allow: { label: '放行', tone: 'ok' }, log: { label: '记录', tone: '' },
}
const { data: items, loading, error, reload } = useAsync(async () => asArray(await api('/api/v1/user/audit-logs', { query: { limit: 100 } })))
</script>

<template>
  <div class="page">
    <div class="page-head"><div><h2>审计日志</h2><p>你的节点审计规则命中的最近 100 条记录。</p></div></div>
    <div class="card">
      <AsyncState :loading="loading" :error="error" :empty="!items?.length" empty-title="还没有审计记录" @retry="reload()">
        <div class="table-wrap">
          <table class="table">
            <thead><tr><th>ID</th><th>节点</th><th>动作</th><th>目标</th><th>IP</th><th>时间</th></tr></thead>
            <tbody>
              <tr v-for="l in items" :key="l.id">
                <td>{{ l.id }}</td>
                <td class="muted">{{ l.node?.name || (l.node_id ? `#${l.node_id}` : '-') }}</td>
                <td><span class="badge" :class="ACTIONS[l.action_taken]?.tone">{{ ACTIONS[l.action_taken]?.label || l.action_taken }}</span></td>
                <td class="mono small">{{ l.target_domain || l.target_protocol || '-' }}</td>
                <td class="mono small">{{ l.ip_address || '-' }}</td>
                <td class="muted nowrap">{{ formatAnyTs(l.created_at) }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </AsyncState>
    </div>
  </div>
</template>
