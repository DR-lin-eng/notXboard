<script setup lang="ts">
import { computed, reactive, watch } from 'vue'
import { Plus } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import CopyField from '@/components/CopyField.vue'
import { api } from '@/lib/api'
import { asArray, formatAnyTs, formatMoneyCent } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const ui = useUi()
const action = useAction()
const { data, loading, error, reload } = useAsync(async () => {
  const payload = await api<any>('/api/v1/user/invite/fetch')
  return {
    codes: asArray(payload?.codes ?? payload),
    plans: asArray(payload?.available_plans),
    stat: asArray(payload?.stat),
  }
})

const form = reactive({ plan: '', period: '' })
const periods = computed(() => asArray(data.value?.plans.find((p: any) => String(p.id) === form.plan)?.periods))
watch(periods, (list) => {
  form.period = list[0]?.value ? String(list[0].value) : ''
})

const planName = (id: unknown) => data.value?.plans.find((p: any) => Number(p.id) === Number(id))?.name || ''
function periodName(planId: unknown, period: unknown) {
  const list = asArray(data.value?.plans.find((p: any) => Number(p.id) === Number(planId))?.periods)
  return list.find((item: any) => String(item.value) === String(period))?.label || String(period || '')
}
const inviteUrl = (code: string) => `${location.origin}/app/#/login?register=1&invite_code=${encodeURIComponent(code)}`

async function create() {
  if (Boolean(form.plan) !== Boolean(form.period)) {
    ui.error('请同时选择邀请套餐和周期')
    return
  }
  await action.run(async () => {
    await api('/api/v1/user/invite/save', { method: 'POST', body: form.plan ? { assigned_plan_id: form.plan, assigned_period: form.period } : {} })
    ui.ok('邀请码已生成')
    await reload(true)
  }, ui.error)
}
</script>

<template>
  <div class="page">
    <div class="page-head"><div><h2>邀请</h2><p>生成邀请码，把注册链接发给朋友。</p></div></div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <template v-if="data">
        <div class="grid g3">
          <div class="card stat"><div class="k">邀请注册数</div><div class="v">{{ data.stat[0] ?? 0 }}</div></div>
          <div class="card stat"><div class="k">已获得佣金</div><div class="v">{{ formatMoneyCent(data.stat[1] ?? 0) }}</div></div>
          <div class="card stat"><div class="k">可用邀请码</div><div class="v">{{ data.codes.length }}</div></div>
        </div>

        <div class="card mt">
          <div class="card-head"><h3>生成邀请码</h3></div>
          <div class="card-body form-grid">
            <div class="field">
              <label for="i-plan">注册后自动赠送套餐（可选）</label>
              <select id="i-plan" v-model="form.plan" class="select">
                <option value="">仅注册，不赠送套餐</option>
                <option v-for="p in data.plans" :key="p.id" :value="String(p.id)">{{ p.name || `套餐 #${p.id}` }}</option>
              </select>
              <span class="hint">只能选择你自己创建的节点套餐。</span>
            </div>
            <div class="field">
              <label for="i-period">赠送周期</label>
              <select id="i-period" v-model="form.period" class="select" :disabled="!periods.length">
                <option v-if="!periods.length" value="">请先选择套餐</option>
                <option v-for="item in periods" :key="item.value" :value="String(item.value)">{{ item.label || item.value }}</option>
              </select>
            </div>
          </div>
          <div class="card-foot"><button type="button" class="btn primary" :disabled="action.busy.value" @click="create"><Plus />生成邀请码</button></div>
        </div>

        <div class="card mt">
          <div class="card-head"><h3>我的邀请码</h3></div>
          <div v-if="!data.codes.length" class="empty">暂无可用邀请码。</div>
          <div v-else class="table-wrap">
            <table class="table">
              <thead><tr><th>邀请码</th><th>赠送套餐</th><th>周期</th><th>邀请链接</th><th class="hide-m">创建时间</th></tr></thead>
              <tbody>
                <tr v-for="c in data.codes" :key="c.code || c">
                  <td class="mono">{{ c.code || c }}</td>
                  <td class="muted">{{ c.assigned_plan_name || planName(c.assigned_plan_id) || '-' }}</td>
                  <td class="muted">{{ c.assigned_period ? periodName(c.assigned_plan_id, c.assigned_period) : '-' }}</td>
                  <td class="link-cell"><CopyField :value="inviteUrl(String(c.code || c))" label="邀请链接" /></td>
                  <td class="muted nowrap hide-m">{{ formatAnyTs(c.created_at) }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </template>
    </AsyncState>
  </div>
</template>

<style scoped>
.link-cell { min-width: 260px; max-width: 380px; }
</style>
