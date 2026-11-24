<script setup lang="ts">
import { reactive, ref } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import MessageThread from '@/components/MessageThread.vue'
import Modal from '@/components/Modal.vue'
import Pager from '@/components/Pager.vue'
import { adminV2 } from '@/lib/api'
import { adminList, adminPost } from '@/lib/admin/list'
import { ticketStatus, TICKET_LEVELS } from '@/lib/domain'
import { asArray, formatAnyTs } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const ui = useUi()
const action = useAction()
const filter = reactive({ page: 1, status: '', email: '' })
const { data, loading, error, reload } = useAsync(() => adminList('ticket/fetch', {
  current: filter.page, pageSize: 20,
  status: filter.status === '' ? undefined : filter.status,
  email: filter.email.trim() || undefined,
}))
function search() {
  filter.page = 1
  reload()
}
function go(page: number) {
  filter.page = page
  reload(true)
}
// reply_status 1 = the user wrote last and is waiting for staff.
const replyState = (ticket: any) => (Number(ticket.status) === 1 ? { label: '-', tone: '' }
  : Number(ticket.reply_status) === 1 ? { label: '待回复', tone: 'warn' } : { label: '已回复', tone: 'ok' })
const levelLabel = (level: unknown) => TICKET_LEVELS.find((item) => item.value === String(level))?.label || '-'

const detail = ref<any | null>(null)
const reply = ref('')
async function open(ticket: any) {
  await action.run(async () => {
    detail.value = await adminV2<any>('ticket/fetch', { query: { id: ticket.id } })
    reply.value = ''
  }, ui.error)
}
async function send() {
  if (!detail.value || !reply.value.trim()) return
  await action.run(async () => {
    await adminPost('ticket/reply', { id: detail.value.id, message: reply.value.trim() })
    reply.value = ''
    detail.value = await adminV2<any>('ticket/fetch', { query: { id: detail.value.id } })
    await reload(true)
  }, ui.error)
}
async function close(ticket: any) {
  if (!(await ui.confirm(`关闭工单 #${ticket.id}？`, { message: '关闭后用户无法继续回复。', confirmText: '关闭工单', danger: true }))) return
  await action.run(async () => {
    await adminPost('ticket/close', { id: ticket.id })
    ui.ok('工单已关闭')
    detail.value = null
    await reload(true)
  }, ui.error)
}
</script>

<template>
  <div class="page">
    <div class="page-head"><div><h2>工单管理</h2><p>查看、回复和关闭全站工单。</p></div></div>
    <form class="toolbar" @submit.prevent="search">
      <select v-model="filter.status" class="select" aria-label="工单状态" @change="search">
        <option value="">全部状态</option><option value="0">处理中</option><option value="1">已关闭</option>
      </select>
      <input v-model="filter.email" class="input" type="search" placeholder="按用户邮箱筛选" aria-label="用户邮箱">
      <button type="submit" class="btn">筛选</button>
    </form>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <div class="card">
        <div v-if="!data?.items.length" class="empty">没有符合条件的工单。</div>
        <div v-else class="table-wrap">
          <table class="table">
            <thead><tr><th>编号</th><th>主题</th><th>用户</th><th>优先级</th><th>状态</th><th>回复</th><th>更新时间</th><th class="right">操作</th></tr></thead>
            <tbody>
              <tr v-for="ticket in data.items" :key="ticket.id">
                <td class="tnum muted">{{ ticket.id }}</td>
                <td>{{ ticket.subject || '-' }}</td>
                <td>{{ ticket.user?.email || `#${ticket.user_id}` }}</td>
                <td>{{ levelLabel(ticket.level) }}</td>
                <td><span class="badge" :class="ticketStatus(ticket.status).tone">{{ ticketStatus(ticket.status).label }}</span></td>
                <td><span class="badge" :class="replyState(ticket).tone">{{ replyState(ticket).label }}</span></td>
                <td class="muted">{{ formatAnyTs(ticket.updated_at) }}</td>
                <td class="right nowrap">
                  <button type="button" class="btn sm" :disabled="action.busy.value" @click="open(ticket)">处理</button>
                  <button v-if="Number(ticket.status) !== 1" type="button" class="btn sm danger" :disabled="action.busy.value" @click="close(ticket)">关闭</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <Pager :page="data?.page || 1" :last-page="data?.lastPage || 1" :total="data?.total" @change="go" />
      </div>
    </AsyncState>

    <Modal v-if="detail" :title="`工单 #${detail.id} · ${detail.subject || ''}`" wide @close="detail = null">
      <dl class="kv mb">
        <dt>用户</dt><dd>{{ detail.user?.email || `#${detail.user_id}` }}</dd>
        <template v-if="detail.node_id"><dt>关联节点</dt><dd>#{{ detail.node_id }}</dd></template>
        <dt>状态</dt><dd>{{ ticketStatus(detail.status).label }}</dd>
      </dl>
      <MessageThread :messages="asArray(detail.messages)" :labels="['用户', '客服']" />
      <form v-if="Number(detail.status) !== 1" class="col" style="margin-top: 14px" @submit.prevent="send">
        <label for="admin-ticket-reply" class="sr-only">回复内容</label>
        <textarea id="admin-ticket-reply" v-model="reply" class="textarea" rows="4" placeholder="输入回复内容" required />
        <div class="row"><span class="sp" />
          <button type="button" class="btn danger" :disabled="action.busy.value" @click="close(detail)">关闭工单</button>
          <button type="submit" class="btn primary" :disabled="action.busy.value || !reply.trim()">发送回复</button>
        </div>
      </form>
      <p v-else class="muted small" style="margin-top: 14px">工单已关闭。</p>
    </Modal>
  </div>
</template>
