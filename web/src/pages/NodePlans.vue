<script setup lang="ts">
import { Plus } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import CopyField from '@/components/CopyField.vue'
import { api } from '@/lib/api'
import { hasTrialQuota, planShareLink, planVisibility } from '@/lib/domain'
import { asArray } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const ui = useUi()
const action = useAction()
const { data: plans, loading, error, reload } = useAsync(async () => asArray(await api('/api/v1/user/node-plans')))

function visibility(plan: any): { label: string; tone: string } {
  const scope = planVisibility(plan.visibility_scope)
  if (scope === 'link_only') return { label: '仅专属链接', tone: 'info' }
  if (scope === 'assigned_only') return { label: '仅指定用户', tone: 'info' }
  return plan.show ? { label: '公开展示', tone: 'ok' } : { label: '不公开展示', tone: '' }
}
async function remove(plan: any) {
  if (!(await ui.confirm(`删除套餐「${plan.name}」？`, { confirmText: '删除', danger: true }))) return
  await action.run(async () => {
    await api(`/api/v1/user/node-plans/${plan.id}`, { method: 'DELETE' })
    ui.ok('套餐已删除')
    await reload(true)
  }, ui.error)
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div><h2>节点套餐</h2><p>发布你自己的节点套餐；套餐只能包含你管理的节点。</p></div>
      <div class="actions"><RouterLink class="btn primary" to="/node-plans/new"><Plus />新建套餐</RouterLink></div>
    </div>
    <div class="card">
      <AsyncState :loading="loading" :error="error" :empty="!plans?.length" empty-title="还没有发布节点套餐" @retry="reload()">
        <div class="table-wrap">
          <table class="table">
            <thead><tr><th>套餐</th><th>节点</th><th>可见方式</th><th>试用</th><th>付费额度</th><th>专属购买链接</th><th /></tr></thead>
            <tbody>
              <tr v-for="p in plans" :key="p.id">
                <td><div class="strong">{{ p.name }}</div><div class="muted small">#{{ p.id }}</div></td>
                <td class="muted">{{ asArray(p.node_ids).join(', ') || '-' }}</td>
                <td><span class="badge" :class="visibility(p).tone">{{ visibility(p).label }}</span></td>
                <td><span class="badge" :class="p.allow_trial || hasTrialQuota(p.free_quota_gb_by_trust_level) ? 'ok' : ''">{{ p.allow_trial || hasTrialQuota(p.free_quota_gb_by_trust_level) ? '允许' : '不允许' }}</span></td>
                <td class="nowrap">{{ p.is_unlimited_traffic ? '无限' : `${Number(p.paid_quota_gb ?? p.transfer_enable ?? 0) || 0} GB` }}</td>
                <td class="share"><CopyField v-if="p.share_token" :value="planShareLink(p.share_token)" label="专属购买链接" /><span v-else class="muted">未启用</span></td>
                <td>
                  <div class="actions">
                    <RouterLink class="btn sm" :to="`/node-plans/edit/${p.id}`">编辑</RouterLink>
                    <button type="button" class="btn sm danger" :disabled="action.busy.value" @click="remove(p)">删除</button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </AsyncState>
    </div>
  </div>
</template>

<style scoped>
.share { min-width: 240px; max-width: 320px; }
</style>
