<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import CopyField from '@/components/CopyField.vue'
import { api, errorMessage } from '@/lib/api'
import { PLAN_PRICE_FIELDS, hasTrialQuota, parseIdList, planShareLink, planVisibility, type PlanVisibility } from '@/lib/domain'
import { asArray } from '@/lib/format'
import { TRUST_LEVEL_OPTIONS } from '@/lib/nodeConstants'
import { useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const route = useRoute()
const router = useRouter()
const ui = useUi()
const planId = computed(() => (route.params.id ? String(route.params.id) : ''))
const formError = ref('')
const saving = ref(false)

const form = reactive({
  name: '', content: '', visibility: 'public' as PlanVisibility, show: true, sell: true, renew: true, sort: 0,
  min_trust: '' as number | '', access_user_ids: '', share_token: '', node_ids: [] as number[],
  allow_trial: false, paid_quota: 0 as number | '', unlimited: false,
  free_quota: {} as Record<number, number | ''>, prices: {} as Record<string, number | ''>,
})

const { data: nodes, loading, error, reload } = useAsync(async () => {
  const owned = asArray(await api('/api/v1/user/server-nodes'))
  for (const level of TRUST_LEVEL_OPTIONS) form.free_quota[level] = ''
  for (const field of PLAN_PRICE_FIELDS) form.prices[field.key] = ''
  if (!planId.value) {
    form.prices.month_price = 0
    return owned
  }
  const plan = asArray(await api('/api/v1/user/node-plans')).find((item: any) => String(item.id) === planId.value)
  if (!plan) throw new Error('套餐不存在')
  const quota = plan.free_quota_gb_by_trust_level || {}
  Object.assign(form, {
    name: plan.name || '', content: plan.content || '', visibility: planVisibility(plan.visibility_scope),
    show: Boolean(plan.show), sell: Boolean(plan.sell), renew: Boolean(plan.renew), sort: Number(plan.sort ?? 0),
    min_trust: plan.min_trust_level ?? '', access_user_ids: asArray(plan.access_user_ids).join(','),
    share_token: String(plan.share_token || ''), node_ids: asArray(plan.node_ids).map(Number),
    allow_trial: Boolean(plan.allow_trial || hasTrialQuota(quota)),
    paid_quota: Number(plan.paid_quota_gb ?? plan.transfer_enable ?? 0), unlimited: Boolean(plan.is_unlimited_traffic),
  })
  for (const level of TRUST_LEVEL_OPTIONS) {
    const value = Number(quota[String(level)] ?? 0)
    form.free_quota[level] = value > 0 ? value : ''
  }
  for (const field of PLAN_PRICE_FIELDS) {
    const fen = plan[field.key]
    form.prices[field.key] = fen === null || fen === undefined ? '' : Number((Number(fen) / 100).toFixed(2))
  }
  return owned
})

watch(() => form.visibility, (scope) => {
  if (scope !== 'public') form.show = false
})

async function save() {
  formError.value = ''
  const prices: Record<string, number> = {}
  for (const field of PLAN_PRICE_FIELDS) {
    const raw = form.prices[field.key]
    if (raw === '' || raw === null || raw === undefined) continue
    const value = Number(raw)
    if (!Number.isFinite(value) || value < 0) {
      formError.value = `${field.label}价格必须大于等于 0`
      return
    }
    prices[field.key] = Number(value.toFixed(2))
  }
  if (!Object.keys(prices).length) {
    formError.value = '至少设置一个购买价格'
    return
  }
  const accessUserIds = form.visibility === 'assigned_only' ? parseIdList(form.access_user_ids) : []
  if (form.visibility === 'assigned_only' && !accessUserIds.length) {
    formError.value = '指定用户模式下，至少填写一个可购买用户 ID'
    return
  }
  let freeQuota: Record<number, number> | null = null
  if (form.allow_trial) {
    const quota = Object.fromEntries(TRUST_LEVEL_OPTIONS.map((level) => {
      const value = Number(form.free_quota[level] || 0)
      return [level, Number.isFinite(value) && value > 0 ? Number(value.toFixed(2)) : 0]
    }))
    if (Object.values(quota).some((value) => value > 0)) freeQuota = quota
  }
  const payload = {
    name: form.name.trim(), content: form.content, prices,
    show: form.show, sell: form.sell, renew: form.renew, sort: Number(form.sort),
    transfer_enable: form.unlimited ? 0 : Number(form.paid_quota || 0),
    is_unlimited_traffic: form.unlimited,
    node_ids: form.node_ids,
    min_trust_level: form.min_trust === '' ? null : Number(form.min_trust),
    allow_trial: form.allow_trial,
    free_quota_gb_by_trust_level: freeQuota,
    visibility_scope: form.visibility,
    access_user_ids: accessUserIds,
    share_token: form.visibility === 'link_only' ? form.share_token.trim() || null : null,
  }
  saving.value = true
  try {
    if (planId.value) await api(`/api/v1/user/node-plans/${planId.value}`, { method: 'PUT', body: payload })
    else await api('/api/v1/user/node-plans', { method: 'POST', body: payload })
    ui.ok('套餐已保存')
    router.push('/node-plans')
  } catch (e) {
    formError.value = errorMessage(e, '保存失败')
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="page narrow">
    <div class="page-head">
      <div><h2>{{ planId ? `编辑节点套餐 #${planId}` : '新建节点套餐' }}</h2></div>
      <div class="actions"><RouterLink class="btn" to="/node-plans"><ArrowLeft />返回</RouterLink></div>
    </div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <form class="col" novalidate @submit.prevent="save">
        <div v-if="formError" class="notice error" role="alert">{{ formError }}</div>

        <div class="card">
          <div class="card-head"><h3>基本信息</h3></div>
          <div class="card-body form-grid">
            <div class="field full"><label for="np-name">名称</label><input id="np-name" v-model="form.name" class="input"></div>
            <div class="field full"><label for="np-content">描述</label><textarea id="np-content" v-model="form.content" class="textarea" rows="4" /></div>
            <div class="field"><label for="np-sort">显示顺序</label><input id="np-sort" v-model.number="form.sort" class="input" type="number"><span class="hint">数字越小越靠前。</span></div>
            <div class="field"><label for="np-trust">最低用户等级（0-4）</label><input id="np-trust" v-model="form.min_trust" class="input" type="number" min="0" max="4" placeholder="留空 = 不限制"></div>
          </div>
        </div>

        <div class="card">
          <div class="card-head"><h3>可见与销售</h3></div>
          <div class="card-body form-grid">
            <div class="field">
              <label for="np-vis">可见方式</label>
              <select id="np-vis" v-model="form.visibility" class="select">
                <option value="public">公开展示</option>
                <option value="link_only">仅专属链接购买</option>
                <option value="assigned_only">仅指定用户购买</option>
              </select>
            </div>
            <div class="field checks">
              <label class="check"><input v-model="form.show" type="checkbox" :disabled="form.visibility !== 'public'">上架展示</label>
              <label class="check"><input v-model="form.sell" type="checkbox">允许新用户购买</label>
              <label class="check"><input v-model="form.renew" type="checkbox">允许已购用户续费</label>
            </div>
            <div v-if="form.visibility === 'assigned_only'" class="field full">
              <label for="np-users">可购买用户 ID</label>
              <input id="np-users" v-model="form.access_user_ids" class="input" placeholder="例如 1,2,3">
              <span class="hint">多个用逗号分隔。</span>
            </div>
            <div v-if="form.visibility === 'link_only'" class="field full">
              <label for="np-token">专属购买口令</label>
              <input id="np-token" v-model="form.share_token" class="input mono" placeholder="留空自动生成，例如 Abc12345xyz">
              <CopyField v-if="form.share_token.trim()" :value="planShareLink(form.share_token)" label="专属购买链接" />
              <span v-else class="hint">保存后生成专属链接。</span>
            </div>
          </div>
        </div>

        <div class="card">
          <div class="card-head"><h3>包含节点</h3></div>
          <div class="card-body">
            <div v-if="!nodes?.length" class="empty">你还没有节点。<RouterLink class="link" to="/nodes/new">新增节点</RouterLink></div>
            <div v-else class="node-grid">
              <label v-for="n in nodes" :key="n.id" class="check pick">
                <input v-model="form.node_ids" type="checkbox" :value="Number(n.id)">
                <span>{{ n.name }} <span class="muted small">#{{ n.id }} · {{ n.protocol }}</span></span>
              </label>
            </div>
          </div>
        </div>

        <div class="card">
          <div class="card-head"><h3>流量额度</h3></div>
          <div class="card-body form-grid">
            <div class="field">
              <label for="np-quota">付费额度（GB，购买后可用）</label>
              <input id="np-quota" v-model="form.paid_quota" class="input" type="number" min="0" :disabled="form.unlimited" :placeholder="form.unlimited ? '无限流量' : ''">
              <label class="check small"><input v-model="form.unlimited" type="checkbox">无限流量（购买后不限总流量）</label>
            </div>
            <div class="field">
              <label>试用</label>
              <label class="check"><input v-model="form.allow_trial" type="checkbox">允许试用（使用免费额度）</label>
            </div>
            <template v-if="form.allow_trial">
              <div class="full muted small">试用免费额度：按用户等级，每月 GB，作用于本套餐节点。</div>
              <div v-for="level in TRUST_LEVEL_OPTIONS" :key="level" class="field">
                <label :for="`np-fq${level}`">等级 {{ level }}</label>
                <input :id="`np-fq${level}`" v-model="form.free_quota[level]" class="input" type="number" min="0" step="0.01" placeholder="0 表示不提供">
              </div>
            </template>
          </div>
        </div>

        <div class="card">
          <div class="card-head"><h3>价格</h3><span class="sub">单位：元；留空表示不售卖该周期</span></div>
          <div class="card-body form-grid">
            <div v-for="field in PLAN_PRICE_FIELDS" :key="field.key" class="field">
              <label :for="`np-${field.key}`">{{ field.label }}</label>
              <input :id="`np-${field.key}`" v-model="form.prices[field.key]" class="input" type="number" min="0" step="0.01" placeholder="不售卖">
            </div>
          </div>
        </div>

        <div class="row"><span class="sp" /><RouterLink class="btn" to="/node-plans">取消</RouterLink><button type="submit" class="btn primary" :disabled="saving">{{ saving ? '保存中…' : '保存' }}</button></div>
      </form>
    </AsyncState>
  </div>
</template>

<style scoped>
.checks { gap: 10px; justify-content: center; }
.node-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 8px; }
.pick { border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 10px; }
</style>
