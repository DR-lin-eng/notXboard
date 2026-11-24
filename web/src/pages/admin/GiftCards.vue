<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import JsonField from '@/components/JsonField.vue'
import Modal from '@/components/Modal.vue'
import Pager from '@/components/Pager.vue'
import { adminV2, errorMessage } from '@/lib/api'
import { adminList, adminPost } from '@/lib/admin/list'
import { isOn } from '@/lib/admin/configSchema'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const ui = useUi()
const action = useAction()
const codePage = ref(1)
const { data, loading, error, reload } = useAsync(async () => {
  const [templates, types, stats, codes] = await Promise.all([
    adminList('gift-card/templates', { per_page: 100, page: 1 }),
    adminV2<any>('gift-card/types'), adminV2<any>('gift-card/statistics'),
    adminList('gift-card/codes', { per_page: 20, page: codePage.value }),
  ])
  const typeMap: Record<string, string> = types && typeof types === 'object' ? types.data || types : {}
  return { templates: templates.items, types: typeMap, stats: stats?.total_stats || {}, codes }
})
const typeOptions = computed(() => {
  const entries = Object.entries(data.value?.types || {})
  return entries.length ? entries.map(([value, label]) => ({ value, label: String(label) })) : [{ value: '1', label: '通用礼品卡' }]
})
function goCodes(page: number) {
  codePage.value = page
  reload(true)
}

const blank = () => ({
  open: false, id: 0, name: '', type: '1', status: true, description: '', themeColor: '#4f46e5',
  rewards: { balance: 1000 } as Record<string, any>, conditions: {} as Record<string, any>, limits: {} as Record<string, any>,
  jsonOk: { rewards: true, conditions: true, limits: true }, error: '', saving: false,
})
const form = reactive(blank())
const asObject = (value: unknown) => (value && typeof value === 'object' && !Array.isArray(value) ? { ...(value as Record<string, any>) } : {})
function editTemplate(template?: any) {
  Object.assign(form, blank(), { open: true })
  if (!template) return
  Object.assign(form, {
    id: Number(template.id), name: template.name || '', type: String(template.type ?? '1'), status: isOn(template.status),
    description: template.description || '', themeColor: template.theme_color || '#4f46e5',
    rewards: asObject(template.rewards), conditions: asObject(template.conditions), limits: asObject(template.limits),
  })
}
async function saveTemplate() {
  form.error = ''
  if (!form.name.trim()) {
    form.error = '模板名称不能为空'
    return
  }
  if (!form.jsonOk.rewards || !form.jsonOk.conditions || !form.jsonOk.limits) {
    form.error = '奖励、条件或限制里有无效的 JSON，请先修正'
    return
  }
  if (!Object.keys(form.rewards).length) {
    form.error = '奖励设置不能为空'
    return
  }
  const payload: Record<string, any> = {
    name: form.name.trim(), type: Number(form.type) || 1, status: Boolean(form.status), description: form.description,
    theme_color: form.themeColor || '#4f46e5', rewards: form.rewards, conditions: form.conditions, limits: form.limits,
  }
  form.saving = true
  try {
    if (form.id) await adminPost('gift-card/update-template', { ...payload, id: form.id })
    else await adminPost('gift-card/create-template', payload)
    form.open = false
    ui.ok('模板已保存')
    await reload(true)
  } catch (e) {
    form.error = errorMessage(e, '保存失败')
  } finally {
    form.saving = false
  }
}
async function removeTemplate(template: any) {
  if (!(await ui.confirm(`删除模板「${template.name || template.id}」？`, { message: '删除后无法恢复。', confirmText: '删除', danger: true }))) return
  await action.run(async () => {
    await adminPost('gift-card/delete-template', { id: template.id })
    ui.ok('模板已删除')
    await reload(true)
  }, ui.error)
}

const gen = reactive({ open: false, templateId: '', count: 10, prefix: 'GC', hours: '', maxUsage: 1, error: '', saving: false })
function openGenerate(template?: any) {
  Object.assign(gen, { open: true, templateId: template ? String(template.id) : String(data.value?.templates[0]?.id || ''), count: 10, prefix: 'GC', hours: '', maxUsage: 1, error: '' })
}
async function generate() {
  gen.error = ''
  const payload: Record<string, any> = {
    template_id: Number(gen.templateId), count: Math.min(10000, Math.max(1, Math.floor(Number(gen.count) || 0))),
    prefix: gen.prefix.trim() || 'GC', max_usage: Math.max(1, Math.floor(Number(gen.maxUsage) || 1)),
  }
  const hours = Math.floor(Number(gen.hours) || 0)
  if (hours > 0) payload.expires_hours = hours
  if (!payload.template_id) {
    gen.error = '请选择模板'
    return
  }
  gen.saving = true
  try {
    await adminPost('gift-card/generate-codes', payload)
    gen.open = false
    ui.ok(`已生成 ${payload.count} 个兑换码`)
    codePage.value = 1
    await reload(true)
  } catch (e) {
    gen.error = errorMessage(e, '生成失败')
  } finally {
    gen.saving = false
  }
}
// Status 3 means the code is disabled.
async function toggleCode(code: any) {
  await action.run(async () => {
    await adminPost('gift-card/toggle-code', { id: code.id, action: Number(code.status) === 3 ? 'enable' : 'disable' })
    await reload(true)
  }, ui.error)
}
async function removeCode(code: any) {
  if (!(await ui.confirm(`删除兑换码 ${code.code}？`, { message: '删除后无法恢复。', confirmText: '删除', danger: true }))) return
  await action.run(async () => {
    await adminPost('gift-card/delete-code', { id: code.id })
    ui.ok('兑换码已删除')
    await reload(true)
  }, ui.error)
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div><h2>礼品卡</h2><p>礼品卡模板、兑换码和使用统计。</p></div>
      <div class="actions">
        <button type="button" class="btn" :disabled="!data?.templates.length" @click="openGenerate()">生成兑换码</button>
        <button type="button" class="btn primary" @click="editTemplate()">新建模板</button>
      </div>
    </div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <div class="grid g3 mb">
        <div class="card stat"><span class="k">模板数量</span><span class="v tnum">{{ data?.stats.templates_count ?? 0 }}</span></div>
        <div class="card stat"><span class="k">兑换码总数</span><span class="v tnum">{{ data?.stats.codes_count ?? 0 }}</span></div>
        <div class="card stat"><span class="k">已使用</span><span class="v tnum">{{ data?.stats.used_codes_count ?? 0 }}</span></div>
      </div>
      <div class="card mb">
        <div class="card-head"><h3>模板</h3></div>
        <div v-if="!data?.templates.length" class="empty">还没有礼品卡模板。</div>
        <div v-else class="table-wrap">
          <table class="table">
            <thead><tr><th>编号</th><th>名称</th><th>类型</th><th>状态</th><th>兑换码</th><th class="right">操作</th></tr></thead>
            <tbody>
              <tr v-for="template in data.templates" :key="template.id">
                <td class="tnum muted">{{ template.id }}</td>
                <td>{{ template.name || '-' }}</td>
                <td>{{ template.type_name || data.types[String(template.type)] || template.type }}</td>
                <td><span class="badge" :class="isOn(template.status) ? 'ok' : 'warn'">{{ isOn(template.status) ? '启用' : '停用' }}</span></td>
                <td class="tnum">{{ template.codes_count || 0 }}</td>
                <td class="right nowrap">
                  <button type="button" class="btn sm" @click="openGenerate(template)">生成兑换码</button>
                  <button type="button" class="btn sm" @click="editTemplate(template)">编辑</button>
                  <button type="button" class="btn sm danger" :disabled="action.busy.value" @click="removeTemplate(template)">删除</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
      <div class="card">
        <div class="card-head"><h3>兑换码</h3></div>
        <div v-if="!data?.codes.items.length" class="empty">还没有兑换码。</div>
        <div v-else class="table-wrap">
          <table class="table">
            <thead><tr><th>编号</th><th>兑换码</th><th>模板</th><th>状态</th><th>已用 / 上限</th><th class="right">操作</th></tr></thead>
            <tbody>
              <tr v-for="code in data.codes.items" :key="code.id">
                <td class="tnum muted">{{ code.id }}</td>
                <td class="mono">{{ code.code }}</td>
                <td>{{ code.template_name || code.template_id }}</td>
                <td><span class="badge" :class="Number(code.status) === 3 ? 'warn' : ''">{{ code.status_name || code.status }}</span></td>
                <td class="tnum">{{ code.usage_count || 0 }} / {{ code.max_usage || 1 }}</td>
                <td class="right nowrap">
                  <button type="button" class="btn sm" :disabled="action.busy.value" @click="toggleCode(code)">{{ Number(code.status) === 3 ? '启用' : '禁用' }}</button>
                  <button type="button" class="btn sm danger" :disabled="action.busy.value" @click="removeCode(code)">删除</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <Pager :page="data?.codes.page || 1" :last-page="data?.codes.lastPage || 1" :total="data?.codes.total" @change="goCodes" />
      </div>
    </AsyncState>

    <Modal v-if="form.open" :title="form.id ? `编辑模板 #${form.id}` : '新建模板'" wide @close="form.open = false">
      <form id="admin-gift-form" class="form-grid" @submit.prevent="saveTemplate">
        <div v-if="form.error" class="notice error full" role="alert">{{ form.error }}</div>
        <div class="field"><label for="ag-name">模板名称</label><input id="ag-name" v-model="form.name" class="input" required></div>
        <div class="field"><label for="ag-type">类型</label>
          <select id="ag-type" v-model="form.type" class="select"><option v-for="item in typeOptions" :key="item.value" :value="item.value">{{ item.label }}</option></select>
        </div>
        <div class="field"><label for="ag-color">主题色</label><input id="ag-color" v-model="form.themeColor" class="input" type="color"></div>
        <div class="field"><label class="switch"><input v-model="form.status" type="checkbox" role="switch"><span>启用模板</span></label></div>
        <div class="field full"><label for="ag-desc">描述</label><textarea id="ag-desc" v-model="form.description" class="textarea" rows="2" /></div>
        <div class="field full"><label for="ag-rewards">奖励设置（JSON，必填）</label>
          <JsonField id="ag-rewards" v-model="form.rewards" :rows="5" @valid="form.jsonOk.rewards = $event" />
          <span class="hint">例如 {"balance": 1000} 表示赠送 10 元余额；余额单位是分。</span>
        </div>
        <div class="field"><label for="ag-conditions">使用条件（JSON）</label>
          <JsonField id="ag-conditions" v-model="form.conditions" @valid="form.jsonOk.conditions = $event" />
          <span class="hint">可用键：new_user_only、new_user_max_days、paid_user_only、require_invite、allowed_plans。</span>
        </div>
        <div class="field"><label for="ag-limits">使用限制（JSON）</label>
          <JsonField id="ag-limits" v-model="form.limits" @valid="form.jsonOk.limits = $event" />
          <span class="hint">可用键：max_use_per_user、cooldown_hours。</span>
        </div>
      </form>
      <template #footer>
        <button type="button" class="btn" @click="form.open = false">取消</button>
        <button type="submit" form="admin-gift-form" class="btn primary" :disabled="form.saving">{{ form.saving ? '保存中…' : '保存' }}</button>
      </template>
    </Modal>

    <Modal v-if="gen.open" title="生成兑换码" @close="gen.open = false">
      <form id="admin-gift-gen" class="form-grid" @submit.prevent="generate">
        <div v-if="gen.error" class="notice error full" role="alert">{{ gen.error }}</div>
        <div class="field full"><label for="agg-template">模板</label>
          <select id="agg-template" v-model="gen.templateId" class="select" required>
            <option v-for="template in data?.templates || []" :key="template.id" :value="String(template.id)">#{{ template.id }} {{ template.name }}</option>
          </select>
        </div>
        <div class="field"><label for="agg-count">生成数量</label><input id="agg-count" v-model="gen.count" class="input" type="number" min="1" max="10000"></div>
        <div class="field"><label for="agg-prefix">前缀</label><input id="agg-prefix" v-model="gen.prefix" class="input"></div>
        <div class="field"><label for="agg-hours">有效小时（空 = 不过期）</label><input id="agg-hours" v-model="gen.hours" class="input" type="number" min="1"></div>
        <div class="field"><label for="agg-max">每个码可用次数</label><input id="agg-max" v-model="gen.maxUsage" class="input" type="number" min="1"></div>
      </form>
      <template #footer>
        <button type="button" class="btn" @click="gen.open = false">取消</button>
        <button type="submit" form="admin-gift-gen" class="btn primary" :disabled="gen.saving">{{ gen.saving ? '生成中…' : '生成' }}</button>
      </template>
    </Modal>
  </div>
</template>
