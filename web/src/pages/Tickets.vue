<script setup lang="ts">
import { reactive, ref } from 'vue'
import { Plus } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import Modal from '@/components/Modal.vue'
import { api, errorMessage } from '@/lib/api'
import { TICKET_LEVELS, ticketStatus } from '@/lib/domain'
import { asArray } from '@/lib/format'
import { useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const ui = useUi()
const { data: tickets, loading, error, reload } = useAsync(async () => asArray(await api('/api/v1/user/ticket/fetch')))

const nodes = ref<any[]>([])
const form = reactive({ open: false, node_id: '', level: '0', subject: '', message: '', error: '', saving: false })
async function openNew() {
  Object.assign(form, { open: true, node_id: '', level: '0', subject: '', message: '', error: '', saving: false })
  try {
    nodes.value = asArray(await api('/api/v1/user/accessible-nodes'))
  } catch {
    nodes.value = []
  }
}
async function submit() {
  if (!form.subject.trim() || !form.message.trim()) {
    form.error = '请填写主题和内容'
    return
  }
  form.saving = true
  form.error = ''
  try {
    await api('/api/v1/user/ticket/save', {
      method: 'POST',
      body: { subject: form.subject.trim(), level: form.level, message: form.message.trim(), node_id: form.node_id ? Number(form.node_id) : null },
    })
    form.open = false
    ui.ok('工单已提交')
    await reload(true)
  } catch (e) {
    form.error = errorMessage(e, '提交失败')
  } finally {
    form.saving = false
  }
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div><h2>工单</h2><p>遇到问题可以提交工单；指定节点时会通知该节点的负责人。</p></div>
      <div class="actions"><button type="button" class="btn primary" @click="openNew"><Plus />新建工单</button></div>
    </div>
    <div class="card">
      <AsyncState :loading="loading" :error="error" :empty="!tickets?.length" empty-title="还没有工单" @retry="reload()">
        <div class="table-wrap">
          <table class="table">
            <thead><tr><th>ID</th><th>主题</th><th>节点</th><th>状态</th><th /></tr></thead>
            <tbody>
              <tr v-for="t in tickets" :key="t.id">
                <td>#{{ t.id }}</td>
                <td class="strong">{{ t.subject }}</td>
                <td class="muted">{{ t.node_id ? `#${t.node_id}` : '-' }}</td>
                <td><span class="badge" :class="ticketStatus(t.status).tone">{{ ticketStatus(t.status).label }}</span></td>
                <td><div class="actions"><RouterLink class="btn sm" :to="`/ticket/${t.id}`">查看</RouterLink></div></td>
              </tr>
            </tbody>
          </table>
        </div>
      </AsyncState>
    </div>

    <Modal v-if="form.open" title="新建工单" @close="form.open = false">
      <form id="ticket-form" class="col" @submit.prevent="submit">
        <div v-if="form.error" class="notice error" role="alert">{{ form.error }}</div>
        <div class="form-grid">
          <div class="field">
            <label for="t-node">节点（可选）</label>
            <select id="t-node" v-model="form.node_id" class="select">
              <option value="">不指定</option>
              <option v-for="n in nodes" :key="n.id" :value="String(n.id)">{{ n.name }} (#{{ n.id }})</option>
            </select>
          </div>
          <div class="field">
            <label for="t-level">优先级</label>
            <select id="t-level" v-model="form.level" class="select"><option v-for="l in TICKET_LEVELS" :key="l.value" :value="l.value">{{ l.label }}</option></select>
          </div>
        </div>
        <div class="field"><label for="t-subject">主题</label><input id="t-subject" v-model="form.subject" class="input" placeholder="问题简述"></div>
        <div class="field"><label for="t-message">内容</label><textarea id="t-message" v-model="form.message" class="textarea" rows="6" placeholder="详细描述" /></div>
      </form>
      <template #footer>
        <button type="button" class="btn" @click="form.open = false">取消</button>
        <button type="submit" form="ticket-form" class="btn primary" :disabled="form.saving">{{ form.saving ? '提交中…' : '提交' }}</button>
      </template>
    </Modal>
  </div>
</template>
