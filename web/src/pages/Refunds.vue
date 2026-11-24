<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AsyncState from '@/components/AsyncState.vue'
import Modal from '@/components/Modal.vue'
import { api, errorMessage } from '@/lib/api'
import { refundStatus } from '@/lib/domain'
import { asArray, formatMoneyCent } from '@/lib/format'
import { useAsync } from '@/lib/useAsync'
import { useSession } from '@/stores/session'

const route = useRoute()
const router = useRouter()
const session = useSession()
const { data: items, loading, error, reload } = useAsync(async () => asArray(await api('/api/v1/user/refunds')))

const open = ref(false)
const form = reactive({ trade_no: '', reason: '', evidence: '' })
const formError = ref('')
const saving = ref(false)

onMounted(() => {
  const tradeNo = String(route.query.trade_no || '')
  if (tradeNo) {
    form.trade_no = tradeNo
    open.value = true
  }
})

async function submit() {
  formError.value = ''
  if (!form.trade_no.trim()) {
    formError.value = '请填写订单号'
    return
  }
  saving.value = true
  try {
    const res = await api<any>('/api/v1/user/refunds', {
      method: 'POST',
      body: { trade_no: form.trade_no.trim(), reason: form.reason, evidence: form.evidence },
    })
    open.value = false
    if (res?.id) router.push(`/refund/${res.id}`)
    else await reload(true)
  } catch (e) {
    formError.value = errorMessage(e, '申请失败')
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div>
        <h2>退款</h2>
        <p>
          退款按节点套餐内流量使用比例计算；{{ session.disputeEnabled ? '投票结束会自动裁定，站长可最终判定。' : '当前已关闭争议投票。' }}
        </p>
      </div>
      <div class="actions"><button type="button" class="btn primary" @click="open = true">申请退款</button></div>
    </div>
    <div class="card">
      <AsyncState :loading="loading" :error="error" :empty="!items?.length" empty-title="没有退款记录" @retry="reload()">
        <div class="table-wrap">
          <table class="table">
            <thead><tr><th>编号</th><th>订单</th><th>套餐</th><th>状态</th><th>退回</th><th>收费</th><th /></tr></thead>
            <tbody>
              <tr v-for="r in items" :key="r.id">
                <td>#{{ r.id }}</td>
                <td class="mono muted">{{ r.trade_no }}</td>
                <td class="strong">{{ r.plan?.name || r.plan_id }}</td>
                <td><span class="badge" :class="refundStatus(r.status).tone">{{ refundStatus(r.status).label }}</span></td>
                <td class="nowrap">{{ r.refund_amount == null ? '-' : formatMoneyCent(r.refund_amount) }}</td>
                <td class="nowrap muted">{{ r.charged_amount == null ? '-' : formatMoneyCent(r.charged_amount) }}</td>
                <td><div class="actions"><RouterLink class="btn sm" :to="`/refund/${r.id}`">查看</RouterLink></div></td>
              </tr>
            </tbody>
          </table>
        </div>
      </AsyncState>
    </div>

    <Modal v-if="open" title="申请退款" @close="open = false">
      <form id="refund-form" class="col" @submit.prevent="submit">
        <div v-if="formError" class="notice error" role="alert">{{ formError }}</div>
        <div class="field">
          <label for="rf-trade">订单号</label>
          <input id="rf-trade" v-model="form.trade_no" class="input mono" placeholder="订单 trade_no" required>
          <span class="hint">仅支持节点套餐订单。</span>
        </div>
        <div class="field">
          <label for="rf-reason">退款原因（可选）</label>
          <textarea id="rf-reason" v-model="form.reason" class="textarea" />
        </div>
        <div class="field">
          <label for="rf-evidence">证据（可选）</label>
          <textarea id="rf-evidence" v-model="form.evidence" class="textarea" />
        </div>
      </form>
      <template #footer>
        <button type="button" class="btn" @click="open = false">取消</button>
        <button type="submit" form="refund-form" class="btn primary" :disabled="saving">{{ saving ? '提交中…' : '提交申请' }}</button>
      </template>
    </Modal>
  </div>
</template>
