<script setup lang="ts">
import AsyncState from '@/components/AsyncState.vue'
import { api } from '@/lib/api'
import { orderStatus, periodLabel } from '@/lib/domain'
import { asArray, formatMoneyCent, formatTs } from '@/lib/format'
import { useAsync } from '@/lib/useAsync'

const { data: orders, loading, error, reload } = useAsync(async () => asArray(await api('/api/v1/user/order/fetch')))
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div><h2>订单</h2><p>购买记录与支付状态</p></div>
      <div class="actions"><RouterLink class="btn primary" to="/plans">购买套餐</RouterLink></div>
    </div>
    <div class="card">
      <AsyncState :loading="loading" :error="error" :empty="!orders?.length" empty-title="还没有订单" empty-text="购买套餐后，订单会显示在这里。" @retry="reload()">
        <div class="table-wrap">
          <table class="table">
            <thead><tr><th>订单号</th><th>套餐</th><th>周期</th><th>金额</th><th>状态</th><th class="hide-m">创建时间</th><th /></tr></thead>
            <tbody>
              <tr v-for="order in orders" :key="order.trade_no">
                <td class="mono muted">{{ order.trade_no }}</td>
                <td class="strong">{{ order.plan?.name || order.plan_id }}</td>
                <td class="muted">{{ periodLabel(order.period) }}</td>
                <td class="nowrap">{{ order.total_amount == null ? '-' : formatMoneyCent(order.total_amount) }}</td>
                <td><span class="badge" :class="orderStatus(order.status).tone">{{ orderStatus(order.status).label }}</span></td>
                <td class="muted nowrap hide-m">{{ formatTs(order.created_at) }}</td>
                <td><div class="actions"><RouterLink class="btn sm" :to="`/order/${order.trade_no}`">查看</RouterLink></div></td>
              </tr>
            </tbody>
          </table>
        </div>
      </AsyncState>
    </div>
  </div>
</template>
