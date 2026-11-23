<script setup lang="ts">
import { computed, reactive } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AsyncState from '@/components/AsyncState.vue'
import { api } from '@/lib/api'
import { ownerName, planPeriodOptions } from '@/lib/domain'
import { asArray, formatMoneyCent } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const route = useRoute()
const router = useRouter()
const ui = useUi()
const token = computed(() => String(route.params.token || '').trim())
const selected = reactive<Record<string, string>>({})
const buy = useAction()

const { data: plans, loading, error, reload } = useAsync(async () => {
  const res = await api<any>('/api/v1/user/plan/fetch', { query: { token: token.value || undefined } })
  const list = token.value ? [res].filter(Boolean) : asArray(res)
  for (const plan of list) {
    const first = planPeriodOptions(plan)[0]
    if (first && !selected[plan.id]) selected[plan.id] = first.key
  }
  return list
})

async function purchase(plan: any) {
  const period = selected[plan.id]
  if (!period) return
  await buy.run(async () => {
    const tradeNo = await api<string>('/api/v1/user/order/save', {
      method: 'POST',
      body: { plan_id: Number(plan.id), period, purchase_token: token.value || undefined },
    })
    router.push(`/order/${encodeURIComponent(String(tradeNo))}`)
  }, ui.error)
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div>
        <h2>{{ token ? '专属套餐购买' : '套餐' }}</h2>
        <p>{{ token ? '你正在通过专属链接购买，该套餐不会在公开列表展示。' : '购买会创建订单；0 元订单将直接完成。' }}</p>
      </div>
      <div class="actions"><RouterLink class="btn" to="/orders">我的订单</RouterLink></div>
    </div>

    <AsyncState :loading="loading" :error="error" :empty="!plans?.length" empty-title="暂无可购买的套餐" @retry="reload()">
      <div class="grid g3">
        <article v-for="plan in plans" :key="plan.id" class="card plan">
          <div class="card-body col">
            <div class="row">
              <h3 class="truncate">{{ plan.name }}</h3>
              <span class="sp" />
              <span class="badge">{{ plan.scope || 'legacy' }}</span>
            </div>
            <div class="muted small">发布者：{{ ownerName(plan) }} · #{{ plan.id }}</div>
            <p class="muted desc">{{ String(plan.content || '').slice(0, 160) || '暂无描述' }}</p>
            <span class="sp" />
            <template v-if="planPeriodOptions(plan).length">
              <div class="field">
                <label :for="`period-${plan.id}`">购买周期</label>
                <select :id="`period-${plan.id}`" v-model="selected[plan.id]" class="select">
                  <option v-for="opt in planPeriodOptions(plan)" :key="opt.key" :value="opt.key">
                    {{ opt.label }}（{{ formatMoneyCent(opt.price) }}）
                  </option>
                </select>
              </div>
              <button type="button" class="btn primary" :disabled="buy.busy.value" @click="purchase(plan)">购买</button>
            </template>
            <div v-else class="notice">暂无可购买周期</div>
          </div>
        </article>
      </div>
    </AsyncState>
  </div>
</template>

<style scoped>
.plan .card-body { height: 100%; }
.plan h3 { font-size: 15.5px; font-weight: 650; }
.desc { white-space: pre-wrap; word-break: break-word; }
</style>
