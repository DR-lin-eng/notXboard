<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import MessageThread from '@/components/MessageThread.vue'
import { api } from '@/lib/api'
import { ticketStatus } from '@/lib/domain'
import { asArray } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const route = useRoute()
const router = useRouter()
const ui = useUi()
const id = computed(() => String(route.params.id))
const reply = ref('')
const action = useAction()
const { data: ticket, loading, error, reload } = useAsync(() => api<any>('/api/v1/user/ticket/fetch', { query: { id: id.value } }))

async function send() {
  if (!reply.value.trim()) return
  await action.run(async () => {
    await api('/api/v1/user/ticket/reply', { method: 'POST', body: { id: Number(id.value), message: reply.value } })
    reply.value = ''
    await reload(true)
  }, ui.error)
}
async function close() {
  if (!(await ui.confirm('关闭工单？', { message: '关闭后无法继续回复。', confirmText: '关闭工单', danger: true }))) return
  await action.run(async () => {
    await api('/api/v1/user/ticket/close', { method: 'POST', body: { id: Number(id.value) } })
    ui.ok('工单已关闭')
    router.push('/tickets')
  }, ui.error)
}
</script>

<template>
  <div class="page narrow">
    <div class="page-head">
      <div><h2>工单 #{{ id }}</h2><p>{{ ticket?.subject }}</p></div>
      <div class="actions"><RouterLink class="btn" to="/tickets"><ArrowLeft />返回</RouterLink></div>
    </div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <template v-if="ticket">
        <div class="card">
          <div class="card-head">
            <h3>消息</h3>
            <span class="badge" :class="ticketStatus(ticket.status).tone">{{ ticketStatus(ticket.status).label }}</span>
            <span class="sp" /><span class="muted small">节点 {{ ticket.node_id ? `#${ticket.node_id}` : '未指定' }}</span>
          </div>
          <div class="card-body"><MessageThread :messages="asArray(ticket.message)" /></div>
        </div>
        <form class="card mt" @submit.prevent="send">
          <div class="card-body field">
            <label for="reply">回复</label>
            <textarea id="reply" v-model="reply" class="textarea" rows="4" placeholder="输入回复" />
          </div>
          <div class="card-foot">
            <button type="submit" class="btn primary" :disabled="action.busy.value || !reply.trim()">发送回复</button>
            <button type="button" class="btn danger" :disabled="action.busy.value" @click="close">关闭工单</button>
          </div>
        </form>
      </template>
    </AsyncState>
  </div>
</template>
