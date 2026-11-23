<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import { api, request } from '@/lib/api'
import { normalizeHttpUrl, orderStatus, periodLabel, submitGatewayForm } from '@/lib/domain'
import { asArray, formatMoneyCent, formatTs } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useSession } from '@/stores/session'
import { useUi } from '@/stores/ui'

const route = useRoute()
const router = useRouter()
const session = useSession()
const ui = useUi()
const tradeNo = computed(() => String(route.params.tradeNo || ''))
const method = ref<number | null>(null)
const payError = ref('')
const payUrl = ref('')
const payInfo = ref('')
const payRaw = ref('')
const pay = useAction()

const { data, loading, error, reload } = useAsync(async () => {
  const [detail, methods] = await Promise.all([
    api<any>('/api/v1/user/order/detail', { query: { trade_no: tradeNo.value } }),
    api<any>('/api/v1/user/order/getPaymentMethod'),
  ])
  const list = asArray(methods)
  if (method.value === null && list.length) method.value = Number(list[0].id)
  return { detail: detail || {}, methods: list }
})
const pending = computed(() => Number(data.value?.detail.status) === 0)
// A zero-total order is completed by checkout without a payment channel.
const free = computed(() => Number(data.value?.detail.total_amount) <= 0)

async function checkout() {
  payError.value = ''
  payUrl.value = ''
  payInfo.value = ''
  payRaw.value = ''
  await pay.run(async () => {
    const out = await request<any>('/api/v1/user/order/checkout', {
      method: 'POST',
      body: free.value ? { trade_no: tradeNo.value } : { trade_no: tradeNo.value, method: Number(method.value) },
    })
    if (out?.type === -1) {
      payInfo.value = '0 元订单已完成。'
      await reload(true)
      return
    }
    const url = typeof out?.data === 'string' ? normalizeHttpUrl(out.data) : ''
    if (url) {
      payUrl.value = url
      return
    }
    if (typeof out?.data === 'string' && out.data.includes('<form')) {
      payInfo.value = '收到支付表单，正在跳转…'
      submitGatewayForm(out.data)
      return
    }
    payRaw.value = JSON.stringify(out, null, 2)
  }, (message) => { payError.value = message || '支付失败' })
}

async function cancel() {
  if (!(await ui.confirm('取消该订单？', { message: '取消后需要重新下单。', confirmText: '取消订单', danger: true }))) return
  await pay.run(async () => {
    await api('/api/v1/user/order/cancel', { method: 'POST', body: { trade_no: tradeNo.value } })
    ui.ok('订单已取消')
    router.push('/orders')
  }, ui.error)
}
</script>

<template>
  <div class="page narrow">
    <div class="page-head">
      <div><h2>订单详情</h2><p class="mono">{{ tradeNo }}</p></div>
      <div class="actions"><RouterLink class="btn" to="/orders"><ArrowLeft />返回订单</RouterLink></div>
    </div>

    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <template v-if="data">
        <div class="card">
          <div class="card-head">
            <h3>{{ data.detail.plan?.name || '套餐' }}</h3>
            <span class="sp" />
            <span class="badge" :class="orderStatus(data.detail.status).tone">{{ orderStatus(data.detail.status).label }}</span>
          </div>
          <div class="card-body">
            <dl class="kv">
              <dt>周期</dt><dd>{{ periodLabel(data.detail.period) }}</dd>
              <dt>金额</dt><dd class="strong">{{ formatMoneyCent(data.detail.total_amount) }}</dd>
              <dt>创建时间</dt><dd>{{ formatTs(data.detail.created_at) }}</dd>
            </dl>
          </div>
        </div>

        <div v-if="pending" class="card mt">
          <div class="card-head"><h3>支付</h3><span class="sub">选择支付方式后将打开支付页面</span></div>
          <div class="card-body col">
            <div v-if="payError" class="notice error" role="alert">{{ payError }}</div>
            <div v-if="payInfo" class="notice ok">{{ payInfo }}</div>
            <div v-if="payUrl" class="notice accent row wrap">
              <a class="link break sp" :href="payUrl" target="_blank" rel="noopener noreferrer">{{ payUrl }}</a>
              <a class="btn primary sm" :href="payUrl">前往支付</a>
            </div>
            <pre v-if="payRaw" class="codebox">{{ payRaw }}</pre>
            <div v-if="free" class="muted">这是 0 元订单，无需支付，确认后立即生效。</div>
            <div v-else-if="!data.methods.length" class="notice warn">当前没有可用的支付方式，请联系管理员。</div>
            <div v-else class="field">
              <label for="pay-method">支付方式</label>
              <select id="pay-method" v-model="method" class="select">
                <option v-for="m in data.methods" :key="m.id" :value="Number(m.id)">{{ m.name }}</option>
              </select>
            </div>
          </div>
          <div class="card-foot">
            <button type="button" class="btn primary" :disabled="pay.busy.value || (!free && !data.methods.length)" @click="checkout">{{ free ? '确认订单' : '发起支付' }}</button>
            <button type="button" class="btn danger" :disabled="pay.busy.value" @click="cancel">取消订单</button>
          </div>
        </div>

        <div class="card mt">
          <div class="card-head"><h3>退款</h3></div>
          <div class="card-body muted">
            {{ session.disputeEnabled ? '仅支持节点套餐退款；如进入争议将开放投票。' : '仅支持节点套餐退款；争议投票已关闭。' }}
          </div>
          <div class="card-foot">
            <RouterLink class="btn" :to="{ path: '/refunds', query: { trade_no: tradeNo } }">申请退款</RouterLink>
            <RouterLink class="btn ghost" to="/refunds">查看退款记录</RouterLink>
          </div>
        </div>
      </template>
    </AsyncState>
  </div>
</template>
