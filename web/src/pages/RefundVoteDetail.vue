<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import EvidenceList from '@/components/EvidenceList.vue'
import { api } from '@/lib/api'
import { voteLabel } from '@/lib/domain'
import { asArray } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const route = useRoute()
const ui = useUi()
const id = computed(() => String(route.params.id))
const base = computed(() => `/api/v1/user/refund-votes/${encodeURIComponent(id.value)}`)
const { data: r, loading, error, reload } = useAsync(() => api<any>(base.value))
const action = useAction()

async function cast(vote: 'approve' | 'deny') {
  await action.run(async () => {
    await api(base.value, { method: 'POST', body: { vote } })
    ui.ok('投票已记录')
    await reload(true)
  }, ui.error)
}
async function addEvidence() {
  const content = await ui.prompt('提交证据', { multiline: true, confirmText: '提交' })
  if (!content?.trim()) return
  await action.run(async () => {
    await api(`${base.value}/evidence`, { method: 'POST', body: { content } })
    ui.ok('证据已提交')
    await reload(true)
  }, ui.error)
}
</script>

<template>
  <div class="page narrow">
    <div class="page-head">
      <div><h2>投票：退款 #{{ id }}</h2></div>
      <div class="actions"><RouterLink class="btn" to="/refund-votes"><ArrowLeft />返回</RouterLink></div>
    </div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <template v-if="r">
        <div class="card">
          <div class="card-body">
            <dl class="kv">
              <dt>套餐</dt><dd class="strong">{{ r.plan?.name || r.plan_id }}</dd>
              <dt>投票结束</dt><dd>{{ r.voting_ends_at || '-' }}</dd>
              <dt>票数</dt><dd>同意 {{ r.vote_counts?.approve ?? 0 }} · 拒绝 {{ r.vote_counts?.deny ?? 0 }}</dd>
              <dt>我的投票</dt><dd>{{ voteLabel(r.my_vote) }}</dd>
            </dl>
          </div>
          <div class="card-foot">
            <button type="button" class="btn primary" :disabled="action.busy.value" @click="cast('approve')">同意退款</button>
            <button type="button" class="btn danger" :disabled="action.busy.value" @click="cast('deny')">拒绝退款</button>
            <button type="button" class="btn" :disabled="action.busy.value" @click="addEvidence">提交证据</button>
            <RouterLink class="btn ghost" :to="`/refund/${r.id}`">退款详情</RouterLink>
          </div>
        </div>
        <div class="card mt">
          <div class="card-head"><h3>证据</h3></div>
          <div class="card-body"><EvidenceList :items="asArray(r.evidences)" /></div>
        </div>
      </template>
    </AsyncState>
  </div>
</template>
