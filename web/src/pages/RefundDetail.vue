<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import EvidenceList from '@/components/EvidenceList.vue'
import { api } from '@/lib/api'
import { refundStatus } from '@/lib/domain'
import { asArray, formatMoneyCent, formatTrafficKb } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useSession } from '@/stores/session'
import { useUi } from '@/stores/ui'

const route = useRoute()
const session = useSession()
const ui = useUi()
const id = computed(() => String(route.params.id))
const { data: r, loading, error, reload } = useAsync(() => api<any>(`/api/v1/user/refunds/${encodeURIComponent(id.value)}`))
const action = useAction()

async function addEvidence() {
  const content = await ui.prompt('追加证据', { multiline: true, placeholder: '描述情况，可粘贴截图链接', confirmText: '提交' })
  if (!content?.trim()) return
  await action.run(async () => {
    await api(`/api/v1/user/refunds/${encodeURIComponent(id.value)}/evidence`, { method: 'POST', body: { content } })
    ui.ok('证据已提交')
    await reload(true)
  }, ui.error)
}
</script>

<template>
  <div class="page narrow">
    <div class="page-head">
      <div><h2>退款 #{{ id }}</h2></div>
      <div class="actions"><RouterLink class="btn" to="/refunds"><ArrowLeft />返回</RouterLink></div>
    </div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <template v-if="r">
        <div class="card">
          <div class="card-head">
            <h3>{{ r.plan?.name || r.plan_id }}</h3>
            <span class="sp" />
            <span class="badge" :class="refundStatus(r.status).tone">{{ refundStatus(r.status).label }}</span>
          </div>
          <div class="card-body">
            <dl class="kv">
              <dt>订单</dt><dd class="mono">{{ r.trade_no }}</dd>
              <dt>退回</dt><dd class="strong">{{ r.refund_amount == null ? '-' : formatMoneyCent(r.refund_amount) }}</dd>
              <dt>收费</dt><dd>{{ r.charged_amount == null ? '-' : formatMoneyCent(r.charged_amount) }}</dd>
              <dt>用量</dt><dd>{{ r.used_kb == null ? '-' : formatTrafficKb(r.used_kb) }} / {{ r.allowance_kb == null ? '-' : formatTrafficKb(r.allowance_kb) }}</dd>
              <template v-if="session.disputeEnabled">
                <dt>投票</dt>
                <dd class="row wrap">
                  <span>同意 {{ r.vote_counts?.approve ?? 0 }} · 拒绝 {{ r.vote_counts?.deny ?? 0 }}</span>
                  <RouterLink v-if="r.status === 'voting'" class="btn sm" :to="`/refund-vote/${r.id}`">去投票</RouterLink>
                </dd>
              </template>
            </dl>
          </div>
        </div>
        <div class="card mt">
          <div class="card-head">
            <h3>证据</h3><span class="sp" />
            <button type="button" class="btn sm" :disabled="action.busy.value" @click="addEvidence">追加证据</button>
          </div>
          <div class="card-body"><EvidenceList :items="asArray(r.evidences)" /></div>
        </div>
      </template>
    </AsyncState>
  </div>
</template>
