<script setup lang="ts">
import { reactive, ref } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import Modal from '@/components/Modal.vue'
import Pager from '@/components/Pager.vue'
import { errorMessage } from '@/lib/api'
import { adminList, adminPost } from '@/lib/admin/list'
import { isOn } from '@/lib/admin/configSchema'
import { parseIdList, PLAN_PRICE_FIELDS } from '@/lib/domain'
import { formatMoneyCent, formatTs } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const ui = useUi()
const action = useAction()
const page = ref(1)
const { data, loading, error, reload } = useAsync(() => adminList('coupon/fetch', { current: page.value, pageSize: 20 }))
function go(next: number) {
  page.value = next
  reload(true)
}
const valueText = (coupon: any) => (Number(coupon.type) === 2 ? `${coupon.value}%` : formatMoneyCent(coupon.value))

const blank = () => ({
  open: false, name: '', type: '1', value: '', count: 1, code: '', started: '', ended: '',
  limitUse: '', limitUser: '', planIds: '', periods: [] as string[], error: '', saving: false,
})
const form = reactive(blank())
const generated = ref<string[]>([])
const toUnix = (value: string) => (value ? Math.floor(new Date(value).getTime() / 1000) : 0)

async function generate() {
  form.error = ''
  const raw = Number(form.value)
  // Fixed amounts are stored in cents; percentages as whole numbers.
  const value = form.type === '1' ? Math.round(raw * 100) : Math.round(raw)
  const payload: Record<string, any> = {
    generate_count: Math.max(1, Math.floor(Number(form.count) || 1)), name: form.name.trim(), type: form.type, value,
    started_at: toUnix(form.started), ended_at: toUnix(form.ended),
    limit_use: form.limitUse === '' ? null : Math.max(0, Math.floor(Number(form.limitUse))),
    limit_use_with_user: form.limitUser === '' ? null : Math.max(0, Math.floor(Number(form.limitUser))),
    limit_plan_ids: parseIdList(form.planIds), limit_period: form.periods,
  }
  if (form.code.trim()) payload.code = form.code.trim()
  if (!payload.name || !(value > 0) || !payload.started_at || !payload.ended_at) {
    form.error = '请完整填写名称、优惠值、开始和结束时间'
    return
  }
  if (form.type === '2' && value > 100) {
    form.error = '折扣比例不能超过 100%'
    return
  }
  if (payload.ended_at <= payload.started_at) {
    form.error = '结束时间必须晚于开始时间'
    return
  }
  form.saving = true
  try {
    const res = await adminPost<any>('coupon/generate', payload)
    generated.value = Array.isArray(res?.codes) ? res.codes.map(String) : []
    form.open = false
    ui.ok('优惠券已生成')
    page.value = 1
    await reload(true)
  } catch (e) {
    form.error = errorMessage(e, '生成失败')
  } finally {
    form.saving = false
  }
}
async function toggle(coupon: any) {
  await action.run(async () => {
    await adminPost('coupon/show', { id: coupon.id })
    await reload(true)
  }, ui.error)
}
async function remove(coupon: any) {
  if (!(await ui.confirm(`删除优惠券「${coupon.name || coupon.code}」？`, { message: '删除后无法恢复。', confirmText: '删除', danger: true }))) return
  await action.run(async () => {
    await adminPost('coupon/drop', { id: coupon.id })
    ui.ok('优惠券已删除')
    await reload(true)
  }, ui.error)
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div><h2>优惠券</h2><p>生成优惠券，控制是否可用。</p></div>
      <div class="actions"><button type="button" class="btn primary" @click="Object.assign(form, blank(), { open: true })">生成优惠券</button></div>
    </div>
    <div v-if="generated.length" class="notice mb">
      <b>刚生成的券码</b>
      <div class="codebox" style="margin-top: 8px">{{ generated.join('\n') }}</div>
    </div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <div class="card">
        <div v-if="!data?.items.length" class="empty">还没有优惠券。</div>
        <div v-else class="table-wrap">
          <table class="table">
            <thead><tr><th>编号</th><th>名称</th><th>券码</th><th>优惠</th><th>次数限制</th><th>有效期</th><th>状态</th><th class="right">操作</th></tr></thead>
            <tbody>
              <tr v-for="coupon in data.items" :key="coupon.id">
                <td class="tnum muted">{{ coupon.id }}</td>
                <td>{{ coupon.name || '-' }}</td>
                <td class="mono">{{ coupon.code }}</td>
                <td class="tnum">{{ valueText(coupon) }}</td>
                <td class="tnum">{{ coupon.limit_use ?? '不限' }} / 每人 {{ coupon.limit_use_with_user ?? '不限' }}</td>
                <td class="muted nowrap">{{ formatTs(coupon.started_at) }}<br>{{ formatTs(coupon.ended_at) }}</td>
                <td><span class="badge" :class="isOn(coupon.show) ? 'ok' : ''">{{ isOn(coupon.show) ? '可用' : '已停用' }}</span></td>
                <td class="right nowrap">
                  <button type="button" class="btn sm" :disabled="action.busy.value" @click="toggle(coupon)">{{ isOn(coupon.show) ? '停用' : '启用' }}</button>
                  <button type="button" class="btn sm danger" :disabled="action.busy.value" @click="remove(coupon)">删除</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <Pager :page="data?.page || 1" :last-page="data?.lastPage || 1" :total="data?.total" @change="go" />
      </div>
    </AsyncState>

    <Modal v-if="form.open" title="生成优惠券" wide @close="form.open = false">
      <form id="admin-coupon-form" class="form-grid" @submit.prevent="generate">
        <div v-if="form.error" class="notice error full" role="alert">{{ form.error }}</div>
        <div class="field"><label for="ac-name">名称</label><input id="ac-name" v-model="form.name" class="input" required></div>
        <div class="field"><label for="ac-type">类型</label>
          <select id="ac-type" v-model="form.type" class="select"><option value="1">固定金额</option><option value="2">折扣比例</option></select>
        </div>
        <div class="field"><label for="ac-value">{{ form.type === '1' ? '优惠金额（元）' : '折扣比例（%）' }}</label>
          <input id="ac-value" v-model="form.value" class="input" type="number" min="0" :step="form.type === '1' ? 0.01 : 1" required>
        </div>
        <div class="field"><label for="ac-count">生成数量</label><input id="ac-count" v-model="form.count" class="input" type="number" min="1" max="500"></div>
        <div class="field full"><label for="ac-code">固定券码（可空）</label><input id="ac-code" v-model="form.code" class="input" autocomplete="off">
          <span class="hint">生成多张时只有第一张使用固定券码，其余随机生成。</span>
        </div>
        <div class="field"><label for="ac-start">开始时间</label><input id="ac-start" v-model="form.started" class="input" type="datetime-local" required></div>
        <div class="field"><label for="ac-end">结束时间</label><input id="ac-end" v-model="form.ended" class="input" type="datetime-local" required></div>
        <div class="field"><label for="ac-limit">总可用次数（空 = 不限）</label><input id="ac-limit" v-model="form.limitUse" class="input" type="number" min="0"></div>
        <div class="field"><label for="ac-limit-user">每人可用次数（空 = 不限）</label><input id="ac-limit-user" v-model="form.limitUser" class="input" type="number" min="0"></div>
        <div class="field full"><label for="ac-plans">限制套餐编号（逗号分隔，空 = 不限）</label><input id="ac-plans" v-model="form.planIds" class="input"></div>
        <fieldset class="field full">
          <legend>限制周期（不选 = 不限）</legend>
          <div class="chips">
            <label v-for="period in PLAN_PRICE_FIELDS" :key="period.key" class="check"><input v-model="form.periods" type="checkbox" :value="period.key">{{ period.label }}</label>
          </div>
        </fieldset>
      </form>
      <template #footer>
        <button type="button" class="btn" @click="form.open = false">取消</button>
        <button type="submit" form="admin-coupon-form" class="btn primary" :disabled="form.saving">{{ form.saving ? '生成中…' : '生成' }}</button>
      </template>
    </Modal>
  </div>
</template>
