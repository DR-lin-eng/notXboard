<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import { api } from '@/lib/api'
import { orderStatus } from '@/lib/domain'
import { formatAnyTs, formatMoneyCent } from '@/lib/format'
import { useAsync } from '@/lib/useAsync'

const route = useRoute()
const tradeNo = computed(() => String(route.params.tradeNo))
const { data, loading, error, reload } = useAsync(() => api<any>(`/api/v1/user/sponsor/${encodeURIComponent(tradeNo.value)}`))
</script>

<template>
  <div class="page narrow">
    <div class="page-head">
      <div><h2>赞助订单</h2><p class="mono">{{ tradeNo }}</p></div>
      <div class="actions"><RouterLink class="btn" to="/sponsor"><ArrowLeft />返回</RouterLink></div>
    </div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <div v-if="data" class="card">
        <div class="card-body">
          <dl class="kv">
            <dt>订单号</dt><dd class="mono">{{ data.trade_no }}</dd>
            <dt>金额</dt><dd>{{ formatMoneyCent(data.total_amount) }}</dd>
            <dt>状态</dt><dd><span class="badge" :class="orderStatus(data.status).tone">{{ orderStatus(data.status).label }}</span></dd>
            <dt>支付时间</dt><dd>{{ formatAnyTs(data.paid_at) }}</dd>
          </dl>
          <p class="muted small mt-sm">支付完成后网关会回调，状态随后更新为已完成。</p>
        </div>
      </div>
    </AsyncState>
  </div>
</template>
