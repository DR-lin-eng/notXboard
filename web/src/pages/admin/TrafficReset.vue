<script setup lang="ts">
import { ref } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import Modal from '@/components/Modal.vue'
import Pager from '@/components/Pager.vue'
import { adminV2 } from '@/lib/api'
import { adminList, adminPost } from '@/lib/admin/list'
import { asArray, formatAnyTs } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const ui = useUi()
const action = useAction()
const days = ref(30)
const page = ref(1)
const userId = ref('')
const stats = useAsync(() => adminV2<any>('traffic-reset/stats', { query: { days: days.value } }))
const logs = useAsync(() => adminList('traffic-reset/logs', { per_page: 20, page: page.value }))
const history = ref<any | null>(null)

function targetId(): number {
  const id = Math.floor(Number(userId.value))
  if (!(id > 0)) ui.error('请输入正确的用户编号')
  return id > 0 ? id : 0
}
async function resetUser() {
  const id = targetId()
  if (!id) return
  const reason = await ui.prompt(`重置用户 #${id} 的流量？`, { message: '已用流量会立即清零。', placeholder: '填写重置原因', confirmText: '确认重置', danger: true })
  if (reason === null) return
  if (!reason.trim()) {
    ui.error('重置原因不能为空')
    return
  }
  await action.run(async () => {
    await adminPost('traffic-reset/reset-user', { user_id: id, reason: reason.trim() })
    ui.ok('流量已重置')
    page.value = 1
    await Promise.all([stats.reload(true), logs.reload(true)])
  }, ui.error)
}
async function showHistory() {
  const id = targetId()
  if (!id) return
  await action.run(async () => {
    history.value = await adminV2<any>(`traffic-reset/user/${id}/history`, { query: { limit: 20 } })
  }, ui.error)
}
</script>

<template>
  <div class="page">
    <div class="page-head"><div><h2>流量重置</h2><p>手动重置用户流量，查看重置记录。</p></div></div>
    <AsyncState :loading="stats.loading.value" :error="stats.error.value" @retry="stats.reload()">
      <div class="grid g3 mb">
        <div class="card stat"><span class="k">近 {{ days }} 天重置</span><span class="v tnum">{{ stats.data.value?.total_resets ?? 0 }}</span></div>
        <div class="card stat"><span class="k">手动重置</span><span class="v tnum">{{ stats.data.value?.manual_resets ?? 0 }}</span></div>
        <div class="card stat"><span class="k">自动重置</span><span class="v tnum">{{ stats.data.value?.auto_resets ?? 0 }}</span></div>
      </div>
    </AsyncState>
    <form class="toolbar" @submit.prevent="resetUser">
      <select v-model.number="days" class="select" aria-label="统计天数" @change="stats.reload(true)">
        <option :value="7">近 7 天</option><option :value="30">近 30 天</option><option :value="90">近 90 天</option>
      </select>
      <input v-model="userId" class="input" type="number" min="1" placeholder="用户编号" aria-label="用户编号">
      <button type="submit" class="btn danger" :disabled="action.busy.value">重置流量</button>
      <button type="button" class="btn" :disabled="action.busy.value" @click="showHistory">查看该用户历史</button>
    </form>
    <AsyncState :loading="logs.loading.value" :error="logs.error.value" @retry="logs.reload()">
      <div class="card">
        <div class="card-head"><h3>重置日志</h3></div>
        <div v-if="!logs.data.value?.items.length" class="empty">暂无重置记录。</div>
        <div v-else class="table-wrap">
          <table class="table">
            <thead><tr><th>编号</th><th>用户</th><th>类型</th><th>来源</th><th>重置时间</th><th>重置前</th><th>重置后</th></tr></thead>
            <tbody>
              <tr v-for="log in logs.data.value.items" :key="log.id">
                <td class="tnum muted">{{ log.id }}</td>
                <td>{{ log.user_email || '-' }}</td>
                <td>{{ log.reset_type_name || log.reset_type || '-' }}</td>
                <td>{{ log.trigger_source_name || log.trigger_source || '-' }}</td>
                <td class="muted nowrap">{{ formatAnyTs(log.reset_time) }}</td>
                <td class="tnum">{{ log.old_traffic?.formatted || '-' }}</td>
                <td class="tnum">{{ log.new_traffic?.formatted || '-' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <Pager :page="logs.data.value?.page || 1" :last-page="logs.data.value?.lastPage || 1" :total="logs.data.value?.total" @change="(p) => { page = p; logs.reload(true) }" />
      </div>
    </AsyncState>

    <Modal v-if="history" :title="`用户 #${history.user?.id ?? userId} 的重置历史`" wide @close="history = null">
      <dl class="kv mb">
        <dt>邮箱</dt><dd>{{ history.user?.email || '-' }}</dd>
        <dt>累计重置</dt><dd>{{ history.user?.reset_count ?? 0 }} 次</dd>
        <dt>上次重置</dt><dd>{{ formatAnyTs(history.user?.last_reset_at) }}</dd>
        <dt>下次重置</dt><dd>{{ formatAnyTs(history.user?.next_reset_at) }}</dd>
      </dl>
      <div v-if="!asArray(history.history).length" class="empty">没有重置记录。</div>
      <div v-else class="table-wrap">
        <table class="table">
          <thead><tr><th>时间</th><th>类型</th><th>来源</th><th>重置前</th><th>重置后</th></tr></thead>
          <tbody>
            <tr v-for="(item, index) in asArray(history.history)" :key="item.id ?? index">
              <td class="muted nowrap">{{ formatAnyTs(item.reset_time) }}</td>
              <td>{{ item.reset_type_name || item.reset_type || '-' }}</td>
              <td>{{ item.trigger_source_name || item.trigger_source || '-' }}</td>
              <td class="tnum">{{ item.old_traffic?.formatted || '-' }}</td>
              <td class="tnum">{{ item.new_traffic?.formatted || '-' }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </Modal>
  </div>
</template>
