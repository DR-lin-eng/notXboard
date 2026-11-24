<script setup lang="ts">
import { computed, reactive } from 'vue'
import { useRoute } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import EvidenceList from '@/components/EvidenceList.vue'
import Modal from '@/components/Modal.vue'
import { api, errorMessage } from '@/lib/api'
import { refundStatus } from '@/lib/domain'
import { asArray, formatMoneyCent } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useSession } from '@/stores/session'
import { useUi } from '@/stores/ui'

const route = useRoute()
const session = useSession()
const ui = useUi()
const id = computed(() => String(route.params.id))
const base = computed(() => `/api/v1/user/node-admin/refunds/${encodeURIComponent(id.value)}`)
const action = useAction()
const { data: refund, loading, error, reload } = useAsync(() => api<any>(base.value))

async function post(path: string, body: Record<string, unknown>, done: string) {
  await action.run(async () => {
    await api(`${base.value}/${path}`, { method: 'POST', body })
    ui.ok(done)
    await reload(true)
  }, ui.error)
}
async function approve() {
  if (!(await ui.confirm('同意退款？', { message: '退款金额会退回给用户，操作不可撤销。', confirmText: '同意退款' }))) return
  await post('approve', {}, '已同意退款')
}
async function deny() {
  const reason = await ui.prompt('拒绝退款', { message: '可以填写拒绝原因（可选）。', multiline: true, confirmText: '拒绝退款', danger: true })
  if (reason === null) return
  await post('deny', { reason }, '已拒绝退款')
}
async function evidence() {
  const content = await ui.prompt('提交证据', { multiline: true, confirmText: '提交' })
  if (!content?.trim()) return
  await post('evidence', { content: content.trim() }, '证据已提交')
}

const dispute = reactive({ open: false, minutes: 1440, evidence: '', error: '', saving: false })
async function startDispute() {
  dispute.saving = true
  dispute.error = ''
  try {
    await api(`${base.value}/dispute`, { method: 'POST', body: { minutes: Number(dispute.minutes) || 1440, evidence: dispute.evidence } })
    dispute.open = false
    ui.ok('已发起争议投票')
    await reload(true)
  } catch (e) {
    dispute.error = errorMessage(e, '操作失败')
  } finally {
    dispute.saving = false
  }
}
</script>

<template>
  <div class="page narrow">
    <div class="page-head">
      <div><h2>处理退款 #{{ id }}</h2></div>
      <div class="actions"><RouterLink class="btn" to="/node-admin/refunds"><ArrowLeft />返回</RouterLink></div>
    </div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <template v-if="refund">
        <div class="card">
          <div class="card-body">
            <dl class="kv">
              <dt>订单</dt><dd class="mono">{{ refund.trade_no }}</dd>
              <dt>用户</dt><dd>#{{ refund.user_id }}</dd>
              <dt>状态</dt><dd><span class="badge" :class="refundStatus(refund.status).tone">{{ refundStatus(refund.status).label }}</span></dd>
              <dt>退回金额</dt><dd>{{ refund.refund_amount == null ? '-' : formatMoneyCent(refund.refund_amount) }}</dd>
              <dt>收费金额</dt><dd>{{ refund.charged_amount == null ? '-' : formatMoneyCent(refund.charged_amount) }}</dd>
              <template v-if="session.disputeEnabled">
                <dt>投票</dt><dd>同意 {{ refund.vote_counts?.approve ?? 0 }} · 反对 {{ refund.vote_counts?.deny ?? 0 }}</dd>
              </template>
            </dl>
          </div>
          <div class="card-foot">
            <button type="button" class="btn primary" :disabled="action.busy.value" @click="approve">同意退款</button>
            <button type="button" class="btn danger" :disabled="action.busy.value" @click="deny">拒绝退款</button>
            <button v-if="session.disputeEnabled" type="button" class="btn" :disabled="action.busy.value" @click="dispute.open = true">发起争议投票</button>
            <button type="button" class="btn" :disabled="action.busy.value" @click="evidence">提交证据</button>
          </div>
        </div>
        <div class="card mt">
          <div class="card-head"><h3>证据</h3></div>
          <div class="card-body"><EvidenceList :items="asArray(refund.evidences)" /></div>
        </div>
      </template>
    </AsyncState>

    <Modal v-if="dispute.open" title="发起争议投票" @close="dispute.open = false">
      <form id="dispute-form" class="col" @submit.prevent="startDispute">
        <div v-if="dispute.error" class="notice error" role="alert">{{ dispute.error }}</div>
        <div class="field"><label for="d-min">投票时长（分钟）</label><input id="d-min" v-model.number="dispute.minutes" class="input" type="number" min="1"></div>
        <div class="field"><label for="d-ev">证据（可选）</label><textarea id="d-ev" v-model="dispute.evidence" class="textarea" rows="4" /></div>
      </form>
      <template #footer>
        <button type="button" class="btn" @click="dispute.open = false">取消</button>
        <button type="submit" form="dispute-form" class="btn primary" :disabled="dispute.saving">{{ dispute.saving ? '提交中…' : '发起投票' }}</button>
      </template>
    </Modal>
  </div>
</template>
