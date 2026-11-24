<script setup lang="ts">
import AsyncState from '@/components/AsyncState.vue'
import EpayProfileForm from '@/components/EpayProfileForm.vue'
import { api, errorMessage } from '@/lib/api'
import { useAsync } from '@/lib/useAsync'
import { useSession } from '@/stores/session'

const session = useSession()
const { data, loading, error, reload } = useAsync(async () => {
  const mine = await api<any>('/api/v1/user/payment-profiles/epay').catch(() => null)
  let sponsor: any = null
  let sponsorError = ''
  if (session.isSuperAdmin) {
    try {
      sponsor = await api<any>('/api/v1/admin/sponsor-epay')
    } catch (e) {
      sponsorError = errorMessage(e, '加载失败')
    }
  }
  return { mine, sponsor, sponsorError }
})
</script>

<template>
  <div class="page">
    <div class="page-head"><div><h2>收款设置</h2><p>配置 EPay 协议的商户信息，用于收取你的节点套餐收入。</p></div></div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <div v-if="data" class="grid" :class="{ g2: session.isSuperAdmin }">
        <div class="card">
          <div class="card-head"><h3>我的收款</h3></div>
          <div class="card-body col">
            <EpayProfileForm endpoint="/api/v1/user/payment-profiles/epay" id-prefix="pp" :profile="data.mine" />
            <p class="muted small">用户购买你发布的节点套餐时，会优先使用你的商户号和密钥支付，收入直接进入你的账户。</p>
          </div>
        </div>
        <div v-if="session.isSuperAdmin" class="card">
          <div class="card-head"><h3>赞助收款</h3><span class="badge info">站长</span></div>
          <div class="card-body col">
            <div v-if="data.sponsorError" class="notice error">{{ data.sponsorError }}</div>
            <EpayProfileForm endpoint="/api/v1/admin/sponsor-epay" id-prefix="sp" :profile="data.sponsor" />
            <p class="muted small">用户发起赞助，或在支付节点套餐订单时选择“赞助站长”，会使用这里的商户信息收款。</p>
          </div>
        </div>
      </div>
    </AsyncState>
  </div>
</template>
