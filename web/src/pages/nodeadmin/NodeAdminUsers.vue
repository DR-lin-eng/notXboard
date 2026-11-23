<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import { api } from '@/lib/api'
import { asArray, formatTrafficKb } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const route = useRoute()
const ui = useUi()
const nodeId = computed(() => String(route.params.id))
const base = computed(() => `/api/v1/user/node-admin/server-nodes/${encodeURIComponent(nodeId.value)}`)
const action = useAction()
const { data, loading, error, reload } = useAsync(() => api<any>(`${base.value}/users-traffic`, { query: { days: 30 } }))
const users = computed(() => asArray(data.value?.users))

async function blacklist(user: any) {
  const reason = await ui.prompt(`拉黑用户 #${user.user_id}？`, { message: '拉黑后该用户无法使用这个节点。可填写原因（可选）。', confirmText: '拉黑', danger: true })
  if (reason === null) return
  await action.run(async () => {
    await api(`${base.value}/blacklist`, { method: 'POST', body: { user_id: Number(user.user_id), reason } })
    ui.ok('已拉黑')
    await reload(true)
  }, ui.error)
}
async function unblacklist(user: any) {
  await action.run(async () => {
    await api(`${base.value}/unblacklist`, { method: 'POST', body: { user_id: Number(user.user_id) } })
    ui.ok('已取消拉黑')
    await reload(true)
  }, ui.error)
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div><h2>节点用户流量：{{ data?.node?.name || `#${nodeId}` }}</h2><p>最近 30 天。</p></div>
      <div class="actions"><RouterLink class="btn" to="/node-admin"><ArrowLeft />返回</RouterLink></div>
    </div>
    <div class="card">
      <AsyncState :loading="loading" :error="error" :empty="!users.length" empty-title="最近 30 天没有用户流量" @retry="reload()">
        <div class="table-wrap">
          <table class="table">
            <thead><tr><th>用户</th><th>总量</th><th>上行</th><th>下行</th><th>状态</th><th /></tr></thead>
            <tbody>
              <tr v-for="u in users" :key="u.user_id">
                <td><span class="strong">#{{ u.user_id }}</span> <span class="muted">{{ u.email }}</span></td>
                <td class="nowrap">{{ formatTrafficKb(u.total) }}</td>
                <td class="muted nowrap">{{ formatTrafficKb(u.upload) }}</td>
                <td class="muted nowrap">{{ formatTrafficKb(u.download) }}</td>
                <td><span class="badge" :class="u.is_blacklisted ? 'danger' : 'ok'">{{ u.is_blacklisted ? '已拉黑' : '正常' }}</span></td>
                <td>
                  <div class="actions">
                    <button v-if="u.is_blacklisted" type="button" class="btn sm" :disabled="action.busy.value" @click="unblacklist(u)">取消拉黑</button>
                    <button v-else type="button" class="btn sm danger" :disabled="action.busy.value" @click="blacklist(u)">拉黑</button>
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
