<script setup lang="ts">
import { reactive, ref } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import Modal from '@/components/Modal.vue'
import { adminV2, errorMessage } from '@/lib/api'
import { adminList, adminPost } from '@/lib/admin/list'
import { isOn } from '@/lib/admin/configSchema'
import { formatMoneyCent } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useSession } from '@/stores/session'
import { useUi } from '@/stores/ui'

interface GatewayField { key: string; type: string; label: string; placeholder: string; description: string }

const ui = useUi()
const session = useSession()
const action = useAction()
const disputeEnabled = ref(false)
const { data, loading, error, reload } = useAsync(async () => {
  const [config, payments, methods] = await Promise.all([
    // Site config is super-admin only; plain admins manage channels without the policy card.
    session.isSuperAdmin ? adminV2<any>('config/fetch') : Promise.resolve(null),
    adminList('payment/fetch'), adminList<string>('payment/getPaymentMethods'),
  ])
  disputeEnabled.value = isOn(config?.system?.refund_dispute_enable)
  return { payments: payments.items, methods: methods.items.map(String) }
})

async function savePolicy() {
  await action.run(async () => {
    await adminPost('config/save', { refund_dispute_enable: disputeEnabled.value ? 1 : 0 })
    ui.ok('退款策略已保存')
    await session.loadMe(true).catch(() => null)
  }, ui.error)
}

const blank = () => ({
  open: false, id: 0, name: '', payment: '', icon: '', notifyDomain: '', feeFixed: '', feePercent: '',
  config: {} as Record<string, any>, fields: [] as GatewayField[], loadingFields: false, error: '', saving: false,
})
const form = reactive(blank())
const isSecret = (key: string) => /key|secret|token|private|password/i.test(key)

async function loadFields() {
  form.fields = []
  if (!form.payment) return
  form.loadingFields = true
  form.error = ''
  try {
    const schema = await adminPost<Record<string, any>>('payment/getPaymentForm', form.id ? { payment: form.payment, id: form.id } : { payment: form.payment })
    const config: Record<string, any> = {}
    form.fields = Object.entries(schema || {}).map(([key, field]) => {
      config[key] = field?.type === 'boolean' ? isOn(field?.value) : String(field?.value ?? '')
      return { key, type: String(field?.type || 'string'), label: String(field?.label || key), placeholder: String(field?.placeholder || ''), description: String(field?.description || '') }
    })
    form.config = config
  } catch (e) {
    form.error = errorMessage(e, '读取支付方式配置项失败')
  } finally {
    form.loadingFields = false
  }
}
async function edit(payment?: any) {
  Object.assign(form, blank(), { open: true, payment: data.value?.methods[0] || '' })
  if (payment) {
    Object.assign(form, {
      id: Number(payment.id), name: payment.name || '', payment: payment.payment || '', icon: payment.icon || '',
      notifyDomain: payment.notify_domain || '',
      feeFixed: payment.handling_fee_fixed ? String(Number(payment.handling_fee_fixed) / 100) : '',
      feePercent: payment.handling_fee_percent ? String(payment.handling_fee_percent) : '',
    })
  }
  await loadFields()
}
async function save() {
  form.error = ''
  const payload: Record<string, any> = {
    name: form.name.trim(), payment: form.payment, icon: form.icon.trim(), config: { ...form.config },
    notify_domain: form.notifyDomain.trim() || null,
    handling_fee_fixed: Math.max(0, Math.round((Number(form.feeFixed) || 0) * 100)),
    handling_fee_percent: Math.min(100, Math.max(0, Number(form.feePercent) || 0)),
  }
  if (!payload.name || !payload.payment) {
    form.error = '名称和支付方式不能为空'
    return
  }
  if (form.id) payload.id = form.id
  form.saving = true
  try {
    await adminPost('payment/save', payload)
    form.open = false
    ui.ok('支付渠道已保存')
    await reload(true)
  } catch (e) {
    form.error = errorMessage(e, '保存失败')
  } finally {
    form.saving = false
  }
}
async function toggle(payment: any) {
  await action.run(async () => {
    await adminPost('payment/show', { id: payment.id })
    await reload(true)
  }, ui.error)
}
async function remove(payment: any) {
  if (!(await ui.confirm(`删除支付渠道「${payment.name || payment.id}」？`, { message: '删除后用户将无法再使用它付款。', confirmText: '删除', danger: true }))) return
  await action.run(async () => {
    await adminPost('payment/drop', { id: payment.id })
    ui.ok('支付渠道已删除')
    await reload(true)
  }, ui.error)
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div><h2>支付渠道</h2><p>平台收款方式、手续费和退款策略。</p></div>
      <div class="actions"><button type="button" class="btn primary" @click="edit()">新建支付渠道</button></div>
    </div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <div v-if="session.isSuperAdmin" class="card mb">
        <div class="card-head"><h3>退款与争议</h3></div>
        <div class="card-body row wrap">
          <label class="switch"><input v-model="disputeEnabled" type="checkbox" role="switch"><span>启用争议退款投票</span></label>
          <span class="sp" />
          <button type="button" class="btn" :disabled="action.busy.value" @click="savePolicy">保存策略</button>
        </div>
      </div>
      <div class="card">
        <div class="card-head"><h3>支付方式</h3></div>
        <div v-if="!data?.payments.length" class="empty">还没有支付渠道。</div>
        <div v-else class="table-wrap">
          <table class="table">
            <thead><tr><th>编号</th><th>名称</th><th>支付方式</th><th>手续费</th><th>状态</th><th class="right">操作</th></tr></thead>
            <tbody>
              <tr v-for="payment in data.payments" :key="payment.id">
                <td class="tnum muted">{{ payment.id }}</td>
                <td>{{ payment.name || '-' }}</td>
                <td class="mono">{{ payment.payment || '-' }}</td>
                <td class="tnum">{{ formatMoneyCent(payment.handling_fee_fixed || 0) }} + {{ payment.handling_fee_percent || 0 }}%</td>
                <td><span class="badge" :class="isOn(payment.enable) ? 'ok' : 'warn'">{{ isOn(payment.enable) ? '启用' : '停用' }}</span></td>
                <td class="right nowrap">
                  <button type="button" class="btn sm" @click="edit(payment)">编辑</button>
                  <button type="button" class="btn sm" :disabled="action.busy.value" @click="toggle(payment)">{{ isOn(payment.enable) ? '停用' : '启用' }}</button>
                  <button type="button" class="btn sm danger" :disabled="action.busy.value" @click="remove(payment)">删除</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </AsyncState>

    <Modal v-if="form.open" :title="form.id ? `编辑支付渠道 #${form.id}` : '新建支付渠道'" wide @close="form.open = false">
      <form id="admin-payment-form" class="form-grid" @submit.prevent="save">
        <div v-if="form.error" class="notice error full" role="alert">{{ form.error }}</div>
        <div class="field"><label for="apay-name">显示名称</label><input id="apay-name" v-model="form.name" class="input" placeholder="例如：支付宝" required></div>
        <div class="field"><label for="apay-method">支付方式</label>
          <select id="apay-method" v-model="form.payment" class="select" required @change="loadFields">
            <option v-if="!data?.methods.length" value="">暂无可用支付方式</option>
            <option v-for="method in data?.methods || []" :key="method" :value="method">{{ method }}</option>
          </select>
        </div>
        <div class="field"><label for="apay-icon">图标（可空）</label><input id="apay-icon" v-model="form.icon" class="input" placeholder="alipay"></div>
        <div class="field"><label for="apay-notify">回调域名（可空）</label><input id="apay-notify" v-model="form.notifyDomain" class="input" placeholder="https://example.com"></div>
        <div class="field"><label for="apay-fee-fixed">固定手续费（元）</label><input id="apay-fee-fixed" v-model="form.feeFixed" class="input" type="number" min="0" step="0.01"></div>
        <div class="field"><label for="apay-fee-percent">百分比手续费（%）</label><input id="apay-fee-percent" v-model="form.feePercent" class="input" type="number" min="0" max="100" step="0.01"></div>
        <p v-if="form.loadingFields" class="muted small full">正在读取配置项…</p>
        <template v-for="field in form.fields" :key="field.key">
          <div class="field" :class="{ full: field.type === 'text' }">
            <label v-if="field.type === 'boolean'" class="switch"><input v-model="form.config[field.key]" type="checkbox" role="switch"><span>{{ field.label }}</span></label>
            <template v-else>
              <label :for="`apay-cfg-${field.key}`">{{ field.label }}</label>
              <textarea v-if="field.type === 'text'" :id="`apay-cfg-${field.key}`" v-model="form.config[field.key]" class="textarea mono" rows="4" :placeholder="field.placeholder" />
              <input v-else :id="`apay-cfg-${field.key}`" v-model="form.config[field.key]" class="input" :type="isSecret(field.key) ? 'password' : 'text'" :placeholder="field.placeholder" autocomplete="off">
            </template>
            <span v-if="field.description" class="hint">{{ field.description }}</span>
          </div>
        </template>
      </form>
      <template #footer>
        <button type="button" class="btn" @click="form.open = false">取消</button>
        <button type="submit" form="admin-payment-form" class="btn primary" :disabled="form.saving || form.loadingFields">{{ form.saving ? '保存中…' : '保存' }}</button>
      </template>
    </Modal>
  </div>
</template>
