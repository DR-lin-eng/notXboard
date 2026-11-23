<script setup lang="ts">
import { reactive, ref } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import Modal from '@/components/Modal.vue'
import Pager from '@/components/Pager.vue'
import { errorMessage } from '@/lib/api'
import { adminList, adminPost } from '@/lib/admin/list'
import { formatBytes, formatTs, truthy } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useSession } from '@/stores/session'
import { useUi } from '@/stores/ui'

const GB = 1024 ** 3
const ui = useUi()
const session = useSession()
const action = useAction()
const filter = reactive({ field: 'email', search: '', status: '' })
const page = ref(1)
const pageSize = ref(20)

function buildFilters(): Array<{ id: string; value: unknown }> {
  const filters: Array<{ id: string; value: unknown }> = []
  const search = filter.search.trim()
  if (search) {
    if (filter.field === 'id') {
      const id = Number(search)
      if (!Number.isInteger(id) || id <= 0) throw new Error('用户编号必须是正整数')
      filters.push({ id: 'id', value: id })
    } else filters.push({ id: 'email', value: search })
  }
  if (filter.status) filters.push({ id: 'banned', value: filter.status === 'banned' })
  return filters
}
const { data, loading, error, reload } = useAsync(() =>
  adminList('user/fetch', {}, pageSize.value, { current: page.value, pageSize: pageSize.value, filter: buildFilters(), sort: [{ id: 'id', desc: true }] }),
)
const plans = useAsync(async () => (await adminList('plan/fetch')).items)
function search() {
  page.value = 1
  reload(true)
}
const money = (value: unknown) => `¥${(Number(value) || 0).toFixed(2)}`
const roleText = (user: any) => (truthy(user.is_super_admin) ? '站长' : truthy(user.is_admin) ? '管理员' : '用户')

async function toggleBan(user: any) {
  const banned = truthy(user.banned)
  const reason = await ui.prompt(banned ? `解封 ${user.email}？` : `封禁 ${user.email}？`, {
    message: banned ? '将恢复该账号的访问。' : '将立即阻止该用户登录和使用服务。',
    placeholder: banned ? '解封说明（可选）' : '填写封禁原因', multiline: true, confirmText: banned ? '确认解封' : '确认封禁', danger: !banned,
  })
  if (reason === null) return
  if (!banned && !reason.trim()) {
    ui.error('封禁原因不能为空')
    return
  }
  await action.run(async () => {
    const payload: Record<string, any> = { id: user.id, banned: banned ? 0 : 1 }
    if (reason.trim()) payload.ban_reason = reason.trim()
    await adminPost('user/update', payload)
    ui.ok(banned ? '用户已解封' : '用户已封禁')
    await reload(true)
  }, ui.error)
}
async function resetSecret(user: any) {
  if (!(await ui.confirm(`重置 ${user.email} 的订阅密钥？`, { message: '旧的订阅地址会立即失效，用户需要重新导入订阅。', confirmText: '重置', danger: true }))) return
  await action.run(async () => {
    await adminPost('user/resetSecret', { id: user.id })
    ui.ok('订阅密钥已重置')
    await reload(true)
  }, ui.error)
}

const toLocal = (ts: unknown) => {
  const n = Number(ts)
  if (!(n > 0)) return ''
  const d = new Date(n * 1000)
  const pad = (v: number) => String(v).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`
}
const blank = () => ({
  open: false, user: null as any, email: '', planId: '', quotaGb: '', expires: '', balance: '', deviceLimit: '', speedLimit: '',
  remarks: '', password: '', isAdmin: false, error: '', saving: false,
})
const form = reactive(blank())
let original: Record<string, any> = {}
function snapshot() {
  return {
    email: form.email.trim().toLowerCase(), plan_id: form.planId ? Number(form.planId) : null,
    transfer_enable: form.quotaGb === '' ? 0 : Math.round(Number(form.quotaGb) * GB),
    expired_at: form.expires ? Math.floor(new Date(form.expires).getTime() / 1000) : null,
    balance: Number(form.balance) || 0, device_limit: form.deviceLimit === '' ? null : Number(form.deviceLimit),
    speed_limit: form.speedLimit === '' ? null : Number(form.speedLimit), remarks: form.remarks.trim() || null, is_admin: form.isAdmin,
  } as Record<string, any>
}
function edit(user: any) {
  Object.assign(form, blank(), {
    open: true, user, email: user.email || '', planId: user.plan_id ? String(user.plan_id) : '',
    quotaGb: user.transfer_enable ? String(+(Number(user.transfer_enable) / GB).toFixed(2)) : '',
    expires: toLocal(user.expired_at), balance: String(Number(user.balance) || 0),
    deviceLimit: user.device_limit == null ? '' : String(user.device_limit), speedLimit: user.speed_limit == null ? '' : String(user.speed_limit),
    remarks: user.remarks || '', isAdmin: truthy(user.is_admin),
  })
  original = snapshot()
}
async function save() {
  form.error = ''
  const next = snapshot()
  if (!next.email) {
    form.error = '邮箱不能为空'
    return
  }
  for (const key of ['transfer_enable', 'balance', 'device_limit', 'speed_limit']) {
    if (next[key] != null && !(Number.isFinite(next[key]) && next[key] >= 0)) {
      form.error = '流量、余额、设备数和限速必须是不小于 0 的数字'
      return
    }
  }
  if (form.expires && !Number.isFinite(next.expired_at)) {
    form.error = '到期时间格式不正确'
    return
  }
  if (form.password && form.password.length < 8) {
    form.error = '新密码至少 8 位'
    return
  }
  // Send only what changed so an edit never overwrites fields it did not touch.
  const payload: Record<string, any> = { id: form.user.id }
  for (const key of Object.keys(next)) if (next[key] !== original[key]) payload[key] = next[key]
  if (form.password) payload.password = form.password
  if (Object.keys(payload).length === 1) {
    form.open = false
    return
  }
  if (payload.is_admin === true && !(await ui.confirm(`把 ${next.email} 设为管理员？`, { message: '管理员可以管理套餐、支付、工单和公告。', confirmText: '设为管理员' }))) return
  form.saving = true
  try {
    await adminPost('user/update', payload)
    form.open = false
    ui.ok('用户已更新')
    await reload(true)
  } catch (e) {
    form.error = errorMessage(e, '保存失败')
  } finally {
    form.saving = false
  }
}
</script>

<template>
  <div class="page wide">
    <div class="page-head"><div><h2>用户</h2><p>查找用户，调整套餐、余额和账号状态。</p></div></div>
    <form class="toolbar" @submit.prevent="search">
      <select v-model="filter.field" class="select" aria-label="搜索字段" style="min-width: 110px; flex-basis: 110px"><option value="email">邮箱</option><option value="id">用户编号</option></select>
      <input v-model="filter.search" class="input" type="search" :placeholder="filter.field === 'id' ? '输入用户编号' : '输入邮箱关键字'" aria-label="搜索">
      <select v-model="filter.status" class="select" aria-label="账号状态" @change="search"><option value="">全部状态</option><option value="active">正常</option><option value="banned">已封禁</option></select>
      <button type="submit" class="btn">查询</button>
    </form>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <div class="card">
        <div v-if="!data?.items.length" class="empty">没有符合条件的用户。</div>
        <div v-else class="table-wrap">
          <table class="table">
            <thead><tr><th>编号</th><th>邮箱</th><th>套餐</th><th>已用 / 总流量</th><th>到期</th><th>余额</th><th>状态</th><th>身份</th><th class="right">操作</th></tr></thead>
            <tbody>
              <tr v-for="user in data.items" :key="user.id">
                <td class="tnum muted">{{ user.id }}</td>
                <td>{{ user.email }}<div v-if="user.remarks" class="muted small">{{ user.remarks }}</div></td>
                <td>{{ user.plan?.name || '-' }}</td>
                <td class="tnum nowrap">{{ formatBytes(user.total_used) }} / {{ formatBytes(user.transfer_enable) }}</td>
                <td class="muted nowrap">{{ user.expired_at ? formatTs(user.expired_at) : '长期有效' }}</td>
                <td class="tnum">{{ money(user.balance) }}</td>
                <td><span class="badge" :class="truthy(user.banned) ? 'danger' : 'ok'">{{ truthy(user.banned) ? '已封禁' : '正常' }}</span><div v-if="truthy(user.banned) && user.ban_reason" class="muted small">{{ user.ban_reason }}</div></td>
                <td>{{ roleText(user) }}</td>
                <td class="right nowrap">
                  <button type="button" class="btn sm" @click="edit(user)">编辑</button>
                  <button type="button" class="btn sm" :disabled="action.busy.value" @click="resetSecret(user)">重置密钥</button>
                  <button type="button" class="btn sm" :class="{ danger: !truthy(user.banned) }" :disabled="action.busy.value || user.id === session.me?.id" @click="toggleBan(user)">{{ truthy(user.banned) ? '解封' : '封禁' }}</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <Pager :page="data?.page || 1" :last-page="data?.lastPage || 1" :total="data?.total" @change="(p) => { page = p; reload(true) }" />
      </div>
    </AsyncState>

    <Modal v-if="form.open" :title="`编辑用户 #${form.user?.id}`" wide @close="form.open = false">
      <form id="admin-user-form" class="form-grid" @submit.prevent="save">
        <div v-if="form.error" class="notice error full" role="alert">{{ form.error }}</div>
        <div class="field"><label for="au-email">邮箱</label><input id="au-email" v-model="form.email" class="input" type="email" required></div>
        <div class="field"><label for="au-plan">套餐</label>
          <select id="au-plan" v-model="form.planId" class="select">
            <option value="">无套餐</option>
            <option v-for="plan in plans.data.value || []" :key="plan.id" :value="String(plan.id)">{{ plan.name }}</option>
          </select>
        </div>
        <div class="field"><label for="au-quota">总流量（GB）</label><input id="au-quota" v-model="form.quotaGb" class="input" type="number" min="0" step="any"></div>
        <div class="field"><label for="au-expire">到期时间</label><input id="au-expire" v-model="form.expires" class="input" type="datetime-local"><span class="hint">留空表示长期有效。</span></div>
        <div class="field"><label for="au-balance">余额（元）</label><input id="au-balance" v-model="form.balance" class="input" type="number" min="0" step="0.01"></div>
        <div class="field"><label for="au-device">设备数上限</label><input id="au-device" v-model="form.deviceLimit" class="input" type="number" min="0" placeholder="不限制"></div>
        <div class="field"><label for="au-speed">限速（Mbps）</label><input id="au-speed" v-model="form.speedLimit" class="input" type="number" min="0" placeholder="不限制"></div>
        <div class="field"><label for="au-password">新密码</label><input id="au-password" v-model="form.password" class="input" type="password" autocomplete="new-password" placeholder="留空则不修改"></div>
        <div class="field full"><label for="au-remarks">备注</label><input id="au-remarks" v-model="form.remarks" class="input"></div>
        <div v-if="!truthy(form.user?.is_super_admin)" class="field full"><label class="switch"><input v-model="form.isAdmin" type="checkbox" role="switch"><span>管理员</span></label></div>
      </form>
      <template #footer>
        <button type="button" class="btn" @click="form.open = false">取消</button>
        <button type="submit" form="admin-user-form" class="btn primary" :disabled="form.saving">{{ form.saving ? '保存中…' : '保存' }}</button>
      </template>
    </Modal>
  </div>
</template>
