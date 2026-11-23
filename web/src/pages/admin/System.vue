<script setup lang="ts">
import { reactive, ref } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import Modal from '@/components/Modal.vue'
import Pager from '@/components/Pager.vue'
import { adminV2 } from '@/lib/api'
import { adminList, adminPost } from '@/lib/admin/list'
import { isOn } from '@/lib/admin/configSchema'
import { formatAnyTs } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const ui = useUi()
const action = useAction()
const health = useAsync(async () => {
  const [status, queue] = await Promise.all([adminV2<any>('system/getSystemStatus'), adminV2<any>('system/getQueueStats')])
  return { status: status || {}, queue: queue || {} }
})

const filter = reactive({ level: '', keyword: '' })
const logPage = ref(1)
const logs = useAsync(() => adminList('system/getSystemLog', { current: logPage.value, page_size: 20, level: filter.level || undefined, keyword: filter.keyword.trim() || undefined }))
function searchLogs() {
  logPage.value = 1
  logs.reload(true)
}
const detail = ref<any | null>(null)
const levelClass = (level: unknown) => {
  const value = String(level || '').toLowerCase()
  return value === 'error' ? 'danger' : value === 'warning' ? 'warn' : ''
}
const pretty = (value: unknown) => {
  if (value == null || value === '') return ''
  if (typeof value !== 'string') return JSON.stringify(value, null, 2)
  try {
    return JSON.stringify(JSON.parse(value), null, 2)
  } catch {
    return value
  }
}

const failedPage = ref(1)
const failed = useAsync(() => adminList('system/getHorizonFailedJobs', { current: failedPage.value, page_size: 20 }))

const clear = reactive({ days: 30, level: 'all', limit: 1000 })
async function clearLogs() {
  const days = Math.max(0, Math.floor(Number(clear.days) || 0))
  const limit = Math.max(1, Math.floor(Number(clear.limit) || 1000))
  const scope = clear.level === 'all' ? '全部级别' : `${clear.level} 级别`
  if (!(await ui.confirm('清理系统日志？', { message: `将删除 ${days} 天前的${scope}日志，最多 ${limit} 条。删除后无法恢复。`, confirmText: '清理', danger: true }))) return
  await action.run(async () => {
    const result = await adminPost<any>('system/clearSystemLog', { days, level: clear.level, limit })
    ui.ok(`已清理 ${result?.deleted_count ?? 0} 条日志`)
    logPage.value = 1
    await logs.reload(true)
  }, ui.error)
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div><h2>系统</h2><p>调度与队列状态、系统日志和失败任务。</p></div>
      <div class="actions"><button type="button" class="btn" @click="health.reload(true); logs.reload(true); failed.reload(true)">刷新</button></div>
    </div>
    <AsyncState :loading="health.loading.value" :error="health.error.value" @retry="health.reload()">
      <div class="grid g3 mb">
        <div class="card stat"><span class="k">定时调度</span><span class="v"><span class="badge" :class="isOn(health.data.value?.status.schedule) ? 'ok' : 'danger'">{{ isOn(health.data.value?.status.schedule) ? '运行中' : '未运行' }}</span></span>
          <span class="muted small">上次运行 {{ formatAnyTs(health.data.value?.status.schedule_last_runtime) }}</span></div>
        <div class="card stat"><span class="k">队列</span><span class="v"><span class="badge" :class="isOn(health.data.value?.queue.status) ? 'ok' : 'danger'">{{ isOn(health.data.value?.queue.status) ? '运行中' : '未运行' }}</span></span>
          <span class="muted small">每分钟 {{ health.data.value?.queue.jobsPerMinute ?? 0 }} 个任务，近 1 小时 {{ health.data.value?.queue.recentJobs ?? 0 }} 个</span></div>
        <div class="card stat"><span class="k">失败任务（近 7 天）</span><span class="v tnum">{{ health.data.value?.queue.failedJobs ?? 0 }}</span></div>
      </div>
    </AsyncState>

    <div class="card mb">
      <div class="card-head"><h3>系统日志</h3></div>
      <div class="card-body">
        <form class="toolbar" @submit.prevent="searchLogs">
          <select v-model="filter.level" class="select" aria-label="日志级别" @change="searchLogs">
            <option value="">全部级别</option><option value="info">info</option><option value="warning">warning</option><option value="error">error</option>
          </select>
          <input v-model="filter.keyword" class="input" type="search" placeholder="关键字" aria-label="关键字">
          <button type="submit" class="btn">查询</button>
        </form>
      </div>
      <AsyncState :loading="logs.loading.value" :error="logs.error.value" @retry="logs.reload()">
        <div v-if="!logs.data.value?.items.length" class="empty">没有符合条件的日志。</div>
        <div v-else class="table-wrap">
          <table class="table">
            <thead><tr><th>编号</th><th>级别</th><th>标题</th><th>时间</th><th class="right">操作</th></tr></thead>
            <tbody>
              <tr v-for="log in logs.data.value.items" :key="log.id">
                <td class="tnum muted">{{ log.id }}</td>
                <td><span class="badge" :class="levelClass(log.level)">{{ String(log.level || '-').toLowerCase() }}</span></td>
                <td class="log-title">{{ log.title || '-' }}</td>
                <td class="muted nowrap">{{ formatAnyTs(log.created_at) }}</td>
                <td class="right"><button type="button" class="btn sm" @click="detail = log">详情</button></td>
              </tr>
            </tbody>
          </table>
        </div>
        <Pager :page="logPage" :last-page="Math.max(1, Math.ceil((logs.data.value?.total || 0) / 20))" :total="logs.data.value?.total" @change="(p) => { logPage = p; logs.reload(true) }" />
      </AsyncState>
    </div>

    <div class="card mb">
      <div class="card-head"><h3>失败任务</h3></div>
      <AsyncState :loading="failed.loading.value" :error="failed.error.value" @retry="failed.reload()">
        <div v-if="!failed.data.value?.items.length" class="empty">没有失败任务。</div>
        <div v-else class="table-wrap">
          <table class="table">
            <thead><tr><th>编号</th><th>连接</th><th>队列</th><th>失败时间</th></tr></thead>
            <tbody>
              <tr v-for="job in failed.data.value.items" :key="job.id">
                <td class="tnum muted">{{ job.id }}</td><td>{{ job.connection || '-' }}</td><td>{{ job.queue || '-' }}</td><td class="muted nowrap">{{ job.failed_at || '-' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <Pager :page="failedPage" :last-page="Math.max(1, Math.ceil((failed.data.value?.total || 0) / 20))" :total="failed.data.value?.total" @change="(p) => { failedPage = p; failed.reload(true) }" />
      </AsyncState>
    </div>

    <div class="card">
      <div class="card-head"><h3>清理日志</h3></div>
      <form class="card-body form-grid" @submit.prevent="clearLogs">
        <div class="field"><label for="asc-days">清理多少天前的日志</label><input id="asc-days" v-model="clear.days" class="input" type="number" min="0"></div>
        <div class="field"><label for="asc-level">级别</label>
          <select id="asc-level" v-model="clear.level" class="select"><option value="all">全部</option><option value="info">info</option><option value="warning">warning</option><option value="error">error</option></select>
        </div>
        <div class="field"><label for="asc-limit">单次最多清理条数</label><input id="asc-limit" v-model="clear.limit" class="input" type="number" min="1"></div>
        <div class="field" style="align-self: end"><button type="submit" class="btn danger" :disabled="action.busy.value">清理日志</button></div>
      </form>
    </div>

    <Modal v-if="detail" :title="`日志 #${detail.id}`" wide @close="detail = null">
      <dl class="kv mb">
        <dt>级别</dt><dd>{{ detail.level || '-' }}</dd>
        <dt>时间</dt><dd>{{ formatAnyTs(detail.created_at) }}</dd>
        <dt>请求</dt><dd class="mono">{{ detail.uri || '-' }}</dd>
        <dt>标题</dt><dd>{{ detail.title || '-' }}</dd>
      </dl>
      <template v-if="pretty(detail.data)"><h4 class="small muted">请求数据</h4><pre class="codebox mb">{{ pretty(detail.data) }}</pre></template>
      <template v-if="pretty(detail.context)"><h4 class="small muted">上下文</h4><pre class="codebox mb">{{ pretty(detail.context) }}</pre></template>
    </Modal>
  </div>
</template>

<style scoped>
.log-title { max-width: 520px; white-space: normal; word-break: break-word; }
</style>
