<script setup lang="ts">
import { computed, reactive } from 'vue'
import { useRoute } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import { api } from '@/lib/api'
import { asArray } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const TYPES: Record<string, string> = { domain: '域名', protocol: '协议特征', ip: 'IP / 网段' }
const ACTIONS: Record<string, string> = { block: '拦截', allow: '放行', log: '仅记录' }

const route = useRoute()
const ui = useUi()
const nodeId = computed(() => String(route.params.id))
const base = computed(() => `/api/v1/user/server-nodes/${encodeURIComponent(nodeId.value)}/audit-rules`)
const form = reactive({ rule_type: 'domain', rule_pattern: '', action: 'block', is_active: true })
const action = useAction()
const { data: rules, loading, error, reload } = useAsync(async () => asArray(await api(base.value)))

async function add() {
  if (!form.rule_pattern.trim()) {
    ui.error('请填写匹配内容')
    return
  }
  await action.run(async () => {
    await api(base.value, { method: 'POST', body: { ...form, rule_pattern: form.rule_pattern.trim() } })
    form.rule_pattern = ''
    ui.ok('规则已新增')
    await reload(true)
  }, ui.error)
}
async function remove(rule: any) {
  if (!(await ui.confirm('删除该规则？', { confirmText: '删除', danger: true }))) return
  await action.run(async () => {
    await api(`${base.value}/${rule.id}`, { method: 'DELETE' })
    ui.ok('规则已删除')
    await reload(true)
  }, ui.error)
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div><h2>审计规则：节点 #{{ nodeId }}</h2><p>按域名、协议特征或 IP 网段拦截、放行或记录流量。</p></div>
      <div class="actions"><RouterLink class="btn" to="/nodes"><ArrowLeft />返回</RouterLink></div>
    </div>
    <form class="card" @submit.prevent="add">
      <div class="card-body rule-form">
        <div class="field"><label for="r-type">类型</label><select id="r-type" v-model="form.rule_type" class="select"><option v-for="(label, key) in TYPES" :key="key" :value="key">{{ label }}</option></select></div>
        <div class="field grow"><label for="r-pattern">匹配内容</label><input id="r-pattern" v-model="form.rule_pattern" class="input mono" placeholder="example.com / tcp / 1.2.3.* / 1.2.3.0/24"></div>
        <div class="field"><label for="r-action">处理方式</label><select id="r-action" v-model="form.action" class="select"><option v-for="(label, key) in ACTIONS" :key="key" :value="key">{{ label }}</option></select></div>
        <label class="check"><input v-model="form.is_active" type="checkbox">启用</label>
        <button type="submit" class="btn primary" :disabled="action.busy.value">新增规则</button>
      </div>
    </form>
    <div class="card mt">
      <AsyncState :loading="loading" :error="error" :empty="!rules?.length" empty-title="还没有审计规则" @retry="reload()">
        <div class="table-wrap">
          <table class="table">
            <thead><tr><th>ID</th><th>类型</th><th>匹配内容</th><th>处理方式</th><th>状态</th><th /></tr></thead>
            <tbody>
              <tr v-for="rule in rules" :key="rule.id">
                <td>{{ rule.id }}</td>
                <td><span class="badge">{{ TYPES[rule.rule_type] || rule.rule_type }}</span></td>
                <td class="mono">{{ rule.rule_pattern }}</td>
                <td><span class="badge" :class="rule.action === 'block' ? 'danger' : rule.action === 'allow' ? 'ok' : ''">{{ ACTIONS[rule.action] || rule.action }}</span></td>
                <td><span class="badge" :class="rule.is_active ? 'ok' : ''">{{ rule.is_active ? '启用' : '停用' }}</span></td>
                <td><div class="actions"><button type="button" class="btn sm danger" :disabled="action.busy.value" @click="remove(rule)">删除</button></div></td>
              </tr>
            </tbody>
          </table>
        </div>
      </AsyncState>
    </div>
  </div>
</template>

<style scoped>
.rule-form { display: flex; flex-wrap: wrap; gap: 12px; align-items: flex-end; }
.rule-form .field { min-width: 140px; }
.rule-form .grow { flex: 1 1 240px; }
.rule-form .check { height: 36px; }
</style>
