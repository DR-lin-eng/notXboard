<script setup lang="ts">
import { reactive } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import Modal from '@/components/Modal.vue'
import { errorMessage } from '@/lib/api'
import { adminList, adminPost } from '@/lib/admin/list'
import { isOn } from '@/lib/admin/configSchema'
import { formatAnyTs } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const ui = useUi()
const action = useAction()
const { data, loading, error, reload } = useAsync(async () => (await adminList('notice/fetch')).items)

const blank = () => ({ open: false, id: 0, title: '', content: '', img_url: '', tags: '', show: true, popup: false, scope_type: 'global', target_plan_ids: [] as number[], error: '', saving: false })
const form = reactive(blank())
function openNew() {
  Object.assign(form, blank(), { open: true })
}
function edit(notice: any) {
  Object.assign(form, blank(), {
    open: true, id: Number(notice.id), title: notice.title || '', content: notice.content || '', img_url: notice.img_url || '',
    tags: Array.isArray(notice.tags) ? notice.tags.join(', ') : '', show: isOn(notice.show), popup: isOn(notice.popup),
    scope_type: notice.scope_type === 'plan_subscribers' ? 'plan_subscribers' : 'global',
    target_plan_ids: (Array.isArray(notice.target_plan_ids) ? notice.target_plan_ids : []).map(Number).filter((id: number) => id > 0),
  })
}
async function save() {
  form.error = ''
  const payload: Record<string, any> = {
    title: form.title.trim(), content: form.content.trim(), img_url: form.img_url.trim() || null,
    tags: form.tags.split(/[,，]/).map((item) => item.trim()).filter(Boolean),
    show: form.show ? 1 : 0, popup: form.popup ? 1 : 0,
  }
  if (!payload.title || !payload.content) {
    form.error = '标题和内容不能为空'
    return
  }
  if (form.scope_type === 'plan_subscribers' && form.target_plan_ids.length) {
    payload.scope_type = 'plan_subscribers'
    payload.target_plan_ids = form.target_plan_ids
  }
  if (form.id) payload.id = form.id
  form.saving = true
  try {
    await adminPost('notice/save', payload)
    form.open = false
    ui.ok('公告已保存')
    await reload(true)
  } catch (e) {
    form.error = errorMessage(e, '保存失败')
  } finally {
    form.saving = false
  }
}
async function toggle(notice: any) {
  await action.run(async () => {
    await adminPost('notice/show', { id: notice.id })
    await reload(true)
  }, ui.error)
}
async function remove(notice: any) {
  if (!(await ui.confirm(`删除公告「${notice.title || notice.id}」？`, { message: '删除后无法恢复。', confirmText: '删除', danger: true }))) return
  await action.run(async () => {
    await adminPost('notice/drop', { id: notice.id })
    ui.ok('公告已删除')
    await reload(true)
  }, ui.error)
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div><h2>公告管理</h2><p>全站公告的发布、显示和弹窗设置。</p></div>
      <div class="actions"><button type="button" class="btn primary" @click="openNew">新建公告</button></div>
    </div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <div class="card">
        <div v-if="!data?.length" class="empty">还没有公告。</div>
        <div v-else class="table-wrap">
          <table class="table">
            <thead><tr><th>编号</th><th>标题</th><th>标签</th><th>显示</th><th>弹窗</th><th>更新时间</th><th class="right">操作</th></tr></thead>
            <tbody>
              <tr v-for="notice in data" :key="notice.id">
                <td class="tnum muted">{{ notice.id }}</td>
                <td>{{ notice.title || '-' }} <span v-if="notice.scope_type === 'plan_subscribers'" class="badge">套餐订阅者</span></td>
                <td><span v-for="tag in notice.tags || []" :key="tag" class="badge">{{ tag }}</span></td>
                <td><span class="badge" :class="isOn(notice.show) ? 'ok' : ''">{{ isOn(notice.show) ? '显示中' : '已隐藏' }}</span></td>
                <td>{{ isOn(notice.popup) ? '是' : '否' }}</td>
                <td class="muted">{{ formatAnyTs(notice.updated_at || notice.created_at) }}</td>
                <td class="right nowrap">
                  <button type="button" class="btn sm" @click="edit(notice)">编辑</button>
                  <button type="button" class="btn sm" :disabled="action.busy.value" @click="toggle(notice)">{{ isOn(notice.show) ? '隐藏' : '显示' }}</button>
                  <button type="button" class="btn sm danger" :disabled="action.busy.value" @click="remove(notice)">删除</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </AsyncState>

    <Modal v-if="form.open" :title="form.id ? `编辑公告 #${form.id}` : '新建公告'" wide @close="form.open = false">
      <form id="admin-notice-form" class="form-grid" @submit.prevent="save">
        <div v-if="form.error" class="notice error full" role="alert">{{ form.error }}</div>
        <div v-if="form.scope_type === 'plan_subscribers'" class="notice full">
          <template v-if="form.target_plan_ids.length">
            仅对以下套餐的订阅者可见：
            <span v-for="id in form.target_plan_ids" :key="id" class="badge">套餐 #{{ id }}
              <button type="button" class="chip-x" :aria-label="`移除套餐 ${id}`" @click="form.target_plan_ids = form.target_plan_ids.filter((item) => item !== id)">×</button>
            </span>
          </template>
          <template v-else>已移除全部目标套餐，保存后将变为全站公告。</template>
        </div>
        <div class="field full"><label for="an-title">标题</label><input id="an-title" v-model="form.title" class="input" required></div>
        <div class="field full"><label for="an-content">内容</label><textarea id="an-content" v-model="form.content" class="textarea" rows="8" required /></div>
        <div class="field"><label for="an-img">图片地址（可空）</label><input id="an-img" v-model="form.img_url" class="input" placeholder="https://"></div>
        <div class="field"><label for="an-tags">标签（逗号分隔）</label><input id="an-tags" v-model="form.tags" class="input"></div>
        <div class="field"><label class="switch"><input v-model="form.show" type="checkbox" role="switch"><span>对用户显示</span></label></div>
        <div class="field"><label class="switch"><input v-model="form.popup" type="checkbox" role="switch"><span>登录后弹窗提示</span></label></div>
      </form>
      <template #footer>
        <button type="button" class="btn" @click="form.open = false">取消</button>
        <button type="submit" form="admin-notice-form" class="btn primary" :disabled="form.saving">{{ form.saving ? '保存中…' : '保存' }}</button>
      </template>
    </Modal>
  </div>
</template>

<style scoped>
.chip-x { border: 0; background: none; color: inherit; cursor: pointer; padding: 0 0 0 4px; font: inherit; }
</style>
