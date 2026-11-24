<script setup lang="ts">
import { ArrowLeft } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import { api } from '@/lib/api'
import { refundStatus } from '@/lib/domain'
import { asArray, formatMoneyCent } from '@/lib/format'
import { useAsync } from '@/lib/useAsync'

const { data: items, loading, error, reload } = useAsync(async () => asArray(await api('/api/v1/user/node-admin/refunds')))
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div><h2>节点退款收件箱</h2></div>
      <div class="actions"><RouterLink class="btn" to="/node-admin"><ArrowLeft />返回</RouterLink></div>
    </div>
    <div class="card">
      <AsyncState :loading="loading" :error="error" :empty="!items?.length" empty-title="没有退款申请" @retry="reload()">
        <div class="table-wrap">
          <table class="table">
            <thead><tr><th>ID</th><th>订单</th><th>用户</th><th>状态</th><th>退回 / 收费</th><th /></tr></thead>
            <tbody>
              <tr v-for="r in items" :key="r.id">
                <td>#{{ r.id }}</td>
                <td class="mono small">{{ r.trade_no }}</td>
                <td class="muted">#{{ r.user_id }}</td>
                <td><span class="badge" :class="refundStatus(r.status).tone">{{ refundStatus(r.status).label }}</span></td>
                <td class="muted nowrap">{{ r.refund_amount == null ? '-' : formatMoneyCent(r.refund_amount) }} / {{ r.charged_amount == null ? '-' : formatMoneyCent(r.charged_amount) }}</td>
                <td><div class="actions"><RouterLink class="btn sm" :to="`/node-admin/refund/${r.id}`">处理</RouterLink></div></td>
              </tr>
            </tbody>
          </table>
        </div>
      </AsyncState>
    </div>
  </div>
</template>
