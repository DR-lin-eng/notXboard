<script setup lang="ts">
// Node owner workspace: per-node user traffic, ticket inbox and refund inbox.
import AsyncState from '@/components/AsyncState.vue'
import { api } from '@/lib/api'
import { refundStatus } from '@/lib/domain'
import { asArray } from '@/lib/format'
import { nodeStatus } from '@/lib/nodes'
import { useAsync } from '@/lib/useAsync'
import { useSession } from '@/stores/session'

const session = useSession()
const { data, loading, error, reload } = useAsync(async () => {
  const [nodes, tickets, refunds] = await Promise.all([
    api('/api/v1/user/server-nodes'),
    api('/api/v1/user/node-admin/tickets'),
    api('/api/v1/user/node-admin/refunds').catch(() => []),
  ])
  return { nodes: asArray(nodes), tickets: asArray(tickets), refunds: asArray(refunds) }
})
</script>

<template>
  <div class="page">
    <div class="page-head"><div><h2>节点用户</h2><p>查看你的节点上的用户流量，处理指派给你的工单和退款。</p></div></div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <template v-if="data">
        <div class="grid g2">
          <div class="card">
            <div class="card-head"><h3>节点统计</h3></div>
            <div v-if="!data.nodes.length" class="empty">你还没有节点。</div>
            <div v-else class="table-wrap">
              <table class="table">
                <thead><tr><th>节点</th><th>状态</th><th /></tr></thead>
                <tbody>
                  <tr v-for="n in data.nodes" :key="n.id">
                    <td><span class="strong">{{ n.name }}</span> <span class="muted small">#{{ n.id }}</span></td>
                    <td><span class="badge" :class="nodeStatus(n).tone">{{ nodeStatus(n).label }}</span></td>
                    <td><div class="actions"><RouterLink class="btn sm" :to="`/node-admin/node/${n.id}`">用户流量</RouterLink></div></td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
          <div class="card">
            <div class="card-head"><h3>工单收件箱</h3><span class="sub">指派给你负责的节点</span></div>
            <div v-if="!data.tickets.length" class="empty">没有待处理的工单。</div>
            <div v-else class="table-wrap">
              <table class="table">
                <thead><tr><th>ID</th><th>节点</th><th>主题</th><th /></tr></thead>
                <tbody>
                  <tr v-for="t in data.tickets.slice(0, 20)" :key="t.id">
                    <td>#{{ t.id }}</td>
                    <td class="muted">{{ t.node_id ? `#${t.node_id}` : '-' }}</td>
                    <td>{{ t.subject }}</td>
                    <td><div class="actions"><RouterLink class="btn sm" :to="`/node-admin/ticket/${t.id}`">处理</RouterLink></div></td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>
        <div class="card mt">
          <div class="card-head">
            <h3>退款收件箱</h3>
            <span class="sub">{{ session.disputeEnabled ? '你可以同意退款，或发起争议投票' : '你可以同意或拒绝退款' }}</span>
            <span class="sp" /><RouterLink class="btn sm" to="/node-admin/refunds">全部</RouterLink>
          </div>
          <div v-if="!data.refunds.length" class="empty">没有待处理的退款。</div>
          <div v-else class="table-wrap">
            <table class="table">
              <thead><tr><th>ID</th><th>订单</th><th>用户</th><th>状态</th><th /></tr></thead>
              <tbody>
                <tr v-for="r in data.refunds.slice(0, 10)" :key="r.id">
                  <td>#{{ r.id }}</td>
                  <td class="mono small">{{ r.trade_no }}</td>
                  <td class="muted">#{{ r.user_id }}</td>
                  <td><span class="badge" :class="refundStatus(r.status).tone">{{ refundStatus(r.status).label }}</span></td>
                  <td><div class="actions"><RouterLink class="btn sm" :to="`/node-admin/refund/${r.id}`">处理</RouterLink></div></td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </template>
    </AsyncState>
  </div>
</template>
