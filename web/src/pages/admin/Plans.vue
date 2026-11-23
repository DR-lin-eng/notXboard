<script setup lang="ts">
import { computed, reactive } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import Modal from '@/components/Modal.vue'
import { errorMessage } from '@/lib/api'
import { adminList, adminPost } from '@/lib/admin/list'
import { isOn } from '@/lib/admin/configSchema'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

// Platform plans store prices in yuan, keyed by billing period.
const PRICE_PERIODS = [
  { key: 'monthly', label: '月付' }, { key: 'quarterly', label: '季付' }, { key: 'half_yearly', label: '半年付' },
  { key: 'yearly', label: '年付' }, { key: 'two_yearly', label: '两年付' }, { key: 'three_yearly', label: '三年付' },
  { key: 'onetime', label: '一次性' }, { key: 'reset_traffic', label: '重置流量' },
] as const
const RESET_METHODS = [
  { value: '', label: '跟随系统设置' }, { value: '0', label: '每月 1 日' }, { value: '1', label: '按订购日每月' },
  { value: '2', label: '不重置' }, { value: '3', label: '每年 1 月 1 日' }, { value: '4', label: '按订购日每年' },
]

const ui = useUi()
const action = useAction()
const { data, loading, error, reload } = useAsync(async () => {
  const [plans, groups] = await Promise.all([adminList('plan/fetch'), adminList('server/group/fetch')])
  return { plans: plans.items, groups: groups.items }
})
const stats = computed(() => {
  const plans = data.value?.plans || []
  return { total: plans.length, shown: plans.filter((p) => isOn(p.show)).length, selling: plans.filter((p) => isOn(p.sell)).length }
})
const priceSummary = (plan: any) => PRICE_PERIODS
  .filter((period) => Number(plan.prices?.[period.key]) > 0)
  .map((period) => `${period.label} ¥${Number(plan.prices[period.key]).toFixed(2)}`).join(' · ') || '未定价'
const resetLabel = (plan: any) => RESET_METHODS.find((item) => item.value === String(plan.reset_traffic_method ?? ''))?.label || '-'

const blank = () => ({
  open: false, id: 0, name: '', group: '', transfer: 100, speed: 0, device: 0, capacity: 0, reset: '',
  tags: '', content: '', prices: {} as Record<string, string>, error: '', saving: false,
})
const form = reactive(blank())
function edit(plan?: any) {
  Object.assign(form, blank(), { open: true })
  if (!plan) return
  Object.assign(form, {
    id: Number(plan.id), name: plan.name || '', group: plan.group_id ? String(plan.group_id) : '',
    transfer: Number(plan.transfer_enable) || 0, speed: Number(plan.speed_limit) || 0, device: Number(plan.device_limit) || 0,
    capacity: Number(plan.capacity_limit) || 0, reset: plan.reset_traffic_method == null ? '' : String(plan.reset_traffic_method),
    tags: Array.isArray(plan.tags) ? plan.tags.join(', ') : '', content: plan.content || '',
  })
  for (const period of PRICE_PERIODS) {
    const value = Number(plan.prices?.[period.key])
    if (value > 0) form.prices[period.key] = String(value)
  }
}
const int = (value: unknown) => Math.max(0, Math.floor(Number(value) || 0))
async function save() {
  form.error = ''
  const prices: Record<string, number> = {}
  for (const period of PRICE_PERIODS) {
    const value = Number(form.prices[period.key])
    if (Number.isFinite(value) && value > 0) prices[period.key] = Math.round(value * 100) / 100
  }
  const payload: Record<string, any> = {
    name: form.name.trim(), transfer_enable: int(form.transfer), group_id: form.group ? Number(form.group) : null,
    speed_limit: int(form.speed), device_limit: int(form.device), capacity_limit: int(form.capacity),
    reset_traffic_method: form.reset === '' ? null : Number(form.reset),
    tags: form.tags.split(/[,，]/).map((item) => item.trim()).filter(Boolean), content: form.content, prices,
  }
  if (!payload.name) {
    form.error = '套餐名称不能为空'
    return
  }
  if (form.id) payload.id = form.id
  form.saving = true
  try {
    await adminPost('plan/save', payload)
    form.open = false
    ui.ok('套餐已保存')
    await reload(true)
  } catch (e) {
    form.error = errorMessage(e, '保存失败')
  } finally {
    form.saving = false
  }
}
async function toggle(plan: any, key: 'show' | 'sell' | 'renew') {
  const payload = { id: plan.id, show: isOn(plan.show) ? 1 : 0, sell: isOn(plan.sell) ? 1 : 0, renew: isOn(plan.renew) ? 1 : 0 }
  payload[key] = payload[key] ? 0 : 1
  await action.run(async () => {
    await adminPost('plan/update', payload)
    await reload(true)
  }, ui.error)
}
async function remove(plan: any) {
  if (!(await ui.confirm(`删除套餐「${plan.name || plan.id}」？`, { message: '删除后无法恢复，已有订阅的用户不受影响。', confirmText: '删除', danger: true }))) return
  await action.run(async () => {
    await adminPost('plan/drop', { id: plan.id })
    ui.ok('套餐已删除')
    await reload(true)
  }, ui.error)
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div><h2>套餐管理</h2><p>平台套餐的价格、展示和上下架。节点主自己的套餐在“节点套餐”里管理。</p></div>
      <div class="actions"><button type="button" class="btn primary" @click="edit()">新建套餐</button></div>
    </div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <div class="grid g3 mb">
        <div class="card stat"><span class="k">套餐总数</span><span class="v tnum">{{ stats.total }}</span></div>
        <div class="card stat"><span class="k">展示中</span><span class="v tnum">{{ stats.shown }}</span></div>
        <div class="card stat"><span class="k">在售</span><span class="v tnum">{{ stats.selling }}</span></div>
      </div>
      <div class="card">
        <div v-if="!data?.plans.length" class="empty">还没有平台套餐。</div>
        <div v-else class="table-wrap">
          <table class="table">
            <thead><tr><th>编号</th><th>名称</th><th>分组</th><th>流量</th><th>限制</th><th>价格</th><th>状态</th><th class="right">操作</th></tr></thead>
            <tbody>
              <tr v-for="plan in data.plans" :key="plan.id">
                <td class="tnum muted">{{ plan.id }}</td>
                <td><b>{{ plan.name || '-' }}</b><div v-if="plan.tags?.length" class="muted small">{{ plan.tags.join(', ') }}</div></td>
                <td>{{ plan.group?.name || '-' }}</td>
                <td class="tnum">{{ plan.transfer_enable ?? '-' }} GB<div class="muted small">{{ resetLabel(plan) }}</div></td>
                <td class="small">限速 {{ plan.speed_limit || '不限' }}{{ plan.speed_limit ? ' Mbps' : '' }}<br>设备 {{ plan.device_limit || '不限' }} · 容量 {{ plan.capacity_limit || '不限' }}</td>
                <td class="small">{{ priceSummary(plan) }}</td>
                <td>
                  <span class="badge" :class="isOn(plan.show) ? 'ok' : ''">{{ isOn(plan.show) ? '展示' : '隐藏' }}</span>
                  <span class="badge" :class="isOn(plan.sell) ? 'ok' : 'warn'">{{ isOn(plan.sell) ? '在售' : '停售' }}</span>
                  <span class="badge" :class="isOn(plan.renew) ? 'ok' : 'warn'">{{ isOn(plan.renew) ? '可续费' : '停止续费' }}</span>
                </td>
                <td class="right nowrap">
                  <button type="button" class="btn sm" @click="edit(plan)">编辑</button>
                  <button type="button" class="btn sm" :disabled="action.busy.value" @click="toggle(plan, 'show')">{{ isOn(plan.show) ? '隐藏' : '展示' }}</button>
                  <button type="button" class="btn sm" :disabled="action.busy.value" @click="toggle(plan, 'sell')">{{ isOn(plan.sell) ? '停售' : '上架' }}</button>
                  <button type="button" class="btn sm" :disabled="action.busy.value" @click="toggle(plan, 'renew')">{{ isOn(plan.renew) ? '停止续费' : '允许续费' }}</button>
                  <button type="button" class="btn sm danger" :disabled="action.busy.value" @click="remove(plan)">删除</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </AsyncState>

    <Modal v-if="form.open" :title="form.id ? `编辑套餐 #${form.id}` : '新建套餐'" wide @close="form.open = false">
      <form id="admin-plan-form" class="form-grid" @submit.prevent="save">
        <div v-if="form.error" class="notice error full" role="alert">{{ form.error }}</div>
        <div class="field"><label for="ap-name">套餐名称</label><input id="ap-name" v-model="form.name" class="input" required></div>
        <div class="field"><label for="ap-group">节点分组</label>
          <select id="ap-group" v-model="form.group" class="select">
            <option value="">未指定</option>
            <option v-for="group in data?.groups || []" :key="group.id" :value="String(group.id)">{{ group.name || `Group-${group.id}` }}</option>
          </select>
        </div>
        <div class="field"><label for="ap-transfer">流量配额（GB）</label><input id="ap-transfer" v-model="form.transfer" class="input" type="number" min="1"></div>
        <div class="field"><label for="ap-reset">流量重置</label>
          <select id="ap-reset" v-model="form.reset" class="select"><option v-for="item in RESET_METHODS" :key="item.value" :value="item.value">{{ item.label }}</option></select>
        </div>
        <div class="field"><label for="ap-speed">速度限制（Mbps，0 = 不限）</label><input id="ap-speed" v-model="form.speed" class="input" type="number" min="0"></div>
        <div class="field"><label for="ap-device">设备限制（0 = 不限）</label><input id="ap-device" v-model="form.device" class="input" type="number" min="0"></div>
        <div class="field"><label for="ap-capacity">容量限制（人数，0 = 不限）</label><input id="ap-capacity" v-model="form.capacity" class="input" type="number" min="0"></div>
        <div class="field"><label for="ap-tags">标签（逗号分隔）</label><input id="ap-tags" v-model="form.tags" class="input"></div>
        <fieldset class="field full">
          <legend>价格（元，留空表示不提供该周期）</legend>
          <div class="price-grid">
            <div v-for="period in PRICE_PERIODS" :key="period.key" class="field">
              <label :for="`ap-price-${period.key}`">{{ period.label }}</label>
              <input :id="`ap-price-${period.key}`" v-model="form.prices[period.key]" class="input" type="number" min="0" step="0.01">
            </div>
          </div>
        </fieldset>
        <div class="field full"><label for="ap-content">套餐描述</label><textarea id="ap-content" v-model="form.content" class="textarea" rows="4" /></div>
      </form>
      <template #footer>
        <button type="button" class="btn" @click="form.open = false">取消</button>
        <button type="submit" form="admin-plan-form" class="btn primary" :disabled="form.saving">{{ form.saving ? '保存中…' : '保存' }}</button>
      </template>
    </Modal>
  </div>
</template>

<style scoped>
.price-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(130px, 1fr)); gap: 10px; }
</style>
