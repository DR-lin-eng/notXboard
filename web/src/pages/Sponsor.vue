<script setup lang="ts">
import { ref, watch } from 'vue'
import { HeartHandshake } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import { api, request } from '@/lib/api'
import { normalizeHttpUrl, submitGatewayForm } from '@/lib/domain'
import { asArray } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'

const amount = ref(10)
const method = ref<number | null>(null)
const payError = ref('')
const payUrl = ref('')
const payInfo = ref('')
const payRaw = ref('')
const tradeNo = ref('')
const pay = useAction()

const { data: methods, loading, error, reload } = useAsync(async () => asArray(await api('/api/v1/user/sponsor/methods').catch(() => [])))
watch(methods, (list) => {
  if (method.value === null && list?.length) method.value = Number(list[0].id)
})

async function create() {
  payError.value = payUrl.value = payInfo.value = payRaw.value = ''
  const value = Number(amount.value)
  if (!Number.isFinite(value) || value <= 0) {
    payError.value = '请输入有效的金额'
    return
  }
  if (method.value === null) {
    payError.value = '请选择支付方式'
    return
  }
  await pay.run(async () => {
    const created = await api<any>('/api/v1/user/sponsor', { method: 'POST', body: { amount: value } })
    tradeNo.value = String(created?.trade_no || '')
    const out = await request<any>('/api/v1/user/sponsor/checkout', { method: 'POST', body: { trade_no: tradeNo.value, method: method.value } })
    if (out?.type === 1 && typeof out?.data === 'string' && out.data.includes('<form')) {
      payInfo.value = '已生成支付表单，正在跳转…'
      submitGatewayForm(out.data)
      return
    }
    if (out?.type === 1 && typeof out?.data === 'string') {
      const url = normalizeHttpUrl(out.data)
      if (!url) throw new Error('支付网关返回了无效链接')
      payUrl.value = url
      window.location.href = url
      return
    }
    payRaw.value = JSON.stringify(out, null, 2)
  }, (message) => { payError.value = message || '发起失败' })
}
</script>

<template>
  <div class="page narrow">
    <div class="page-head"><div><h2>赞助</h2><p>支持站点运营，独立于套餐购买。</p></div></div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <form class="card" @submit.prevent="create">
        <div class="card-body col">
          <div v-if="payError" class="notice error" role="alert">{{ payError }}</div>
          <div v-if="!methods?.length" class="notice warn">站长还没有配置赞助收款方式。</div>
          <div class="form-grid">
            <div class="field"><label for="s-amount">金额（元）</label><input id="s-amount" v-model.number="amount" class="input" type="number" min="0.01" step="0.01"></div>
            <div class="field">
              <label for="s-method">支付方式</label>
              <select id="s-method" v-model="method" class="select" :disabled="!methods?.length">
                <option v-for="m in methods" :key="m.id" :value="Number(m.id)">{{ m.name }}</option>
              </select>
            </div>
          </div>
          <div v-if="payInfo" class="notice">{{ payInfo }}</div>
          <div v-if="payUrl" class="notice">正在跳转到支付页面。如果没有自动跳转，<a class="link" :href="payUrl" target="_blank" rel="noopener noreferrer">点击这里打开</a>。</div>
          <pre v-if="payRaw" class="codebox">{{ payRaw }}</pre>
          <div v-if="tradeNo" class="muted small">赞助订单 <RouterLink class="link mono" :to="`/sponsor/${tradeNo}`">{{ tradeNo }}</RouterLink></div>
        </div>
        <div class="card-foot"><button type="submit" class="btn primary" :disabled="pay.busy.value || !methods?.length"><HeartHandshake />{{ pay.busy.value ? '处理中…' : '发起赞助' }}</button></div>
      </form>
    </AsyncState>
  </div>
</template>
