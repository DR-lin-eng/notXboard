<script setup lang="ts">
import { reactive } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import { api, errorMessage, request } from '@/lib/api'
import { asArray, formatAnyTs } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const ui = useUi()
const action = useAction()
const { data, loading, error, reload } = useAsync(async () => {
  const res = await request<any>('/api/v1/user/notice/fetch')
  return {
    notices: asArray(res?.data), plans: asArray(res?.plan_options),
    manageable: asArray(res?.manageable), canPublish: Boolean(res?.can_publish),
  }
})

const form = reactive({ title: '', tags: '', content: '', plans: [] as number[], error: '', saving: false })
async function publish() {
  if (!form.plans.length) {
    form.error = '至少选择一个目标套餐'
    return
  }
  form.saving = true
  form.error = ''
  try {
    await api('/api/v1/user/notice', {
      method: 'POST',
      body: {
        title: form.title.trim(), content: form.content.trim(),
        tags: form.tags.split(/[,，]/).map((item) => item.trim()).filter(Boolean),
        target_plan_ids: form.plans,
      },
    })
    Object.assign(form, { title: '', tags: '', content: '', plans: [] })
    ui.ok('公告已发布')
    await reload(true)
  } catch (e) {
    form.error = errorMessage(e, '发布公告失败')
  } finally {
    form.saving = false
  }
}
async function toggle(notice: any) {
  await action.run(async () => {
    await api(`/api/v1/user/notice/${notice.id}/toggle`, { method: 'POST', body: {} })
    await reload(true)
  }, ui.error)
}
async function remove(notice: any) {
  if (!(await ui.confirm('删除这条公告？', { confirmText: '删除', danger: true }))) return
  await action.run(async () => {
    await api(`/api/v1/user/notice/${notice.id}`, { method: 'DELETE' })
    ui.ok('公告已删除')
    await reload(true)
  }, ui.error)
}
</script>

<template>
  <div class="page">
    <div class="page-head"><div><h2>公告</h2><p>站点公告，以及你订阅的套餐发布者发给你的通知。</p></div></div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <template v-if="data">
        <div v-if="!data.notices.length" class="card"><div class="empty">当前没有可见公告。</div></div>
        <div v-else class="col">
          <article v-for="n in data.notices" :key="n.id" class="card">
            <div class="card-head">
              <h3>{{ n.title || '-' }}</h3>
              <span v-for="tag in asArray(n.tags)" :key="tag" class="badge">{{ tag }}</span>
              <span class="sp" /><span class="muted small nowrap">{{ formatAnyTs(n.created_at) }}</span>
            </div>
            <div class="card-body prose">{{ n.content }}</div>
          </article>
        </div>

        <div class="grid g2 mt">
          <form class="card" @submit.prevent="publish">
            <div class="card-head"><h3>按套餐发布公告</h3></div>
            <div v-if="!data.canPublish" class="empty">你还没有自己创建的套餐，暂时不能发布定向公告。</div>
            <template v-else>
              <div class="card-body col">
                <div v-if="form.error" class="notice error" role="alert">{{ form.error }}</div>
                <div class="field"><label for="n-title">标题</label><input id="n-title" v-model="form.title" class="input" placeholder="例如：今晚 23:00 节点维护"></div>
                <div class="field"><label for="n-tags">标签（可选，逗号分隔）</label><input id="n-tags" v-model="form.tags" class="input" placeholder="维护, 香港, 迁移"></div>
                <div class="field"><label for="n-content">内容</label><textarea id="n-content" v-model="form.content" class="textarea" rows="6" placeholder="只会发给你自己套餐的有效订阅用户" /></div>
                <div class="field">
                  <label>目标套餐（至少选择一个）</label>
                  <label v-for="p in data.plans" :key="p.id" class="check"><input v-model="form.plans" type="checkbox" :value="Number(p.id)">{{ p.name || `套餐 #${p.id}` }}</label>
                </div>
              </div>
              <div class="card-foot"><button type="submit" class="btn primary" :disabled="form.saving">{{ form.saving ? '发布中…' : '发布公告' }}</button></div>
            </template>
          </form>
          <div class="card">
            <div class="card-head"><h3>我发布的公告</h3></div>
            <div v-if="!data.manageable.length" class="empty">你还没有发布过定向公告。</div>
            <div v-else class="table-wrap">
              <table class="table">
                <thead><tr><th>公告</th><th>套餐范围</th><th>状态</th><th /></tr></thead>
                <tbody>
                  <tr v-for="n in data.manageable" :key="n.id">
                    <td><div class="strong">{{ n.title || '-' }}</div><div class="muted small">#{{ n.id }} · {{ formatAnyTs(n.created_at) }}</div></td>
                    <td class="muted">{{ asArray(n.target_plan_ids).join(', ') || '-' }}</td>
                    <td><span class="badge" :class="n.show ? 'ok' : 'warn'">{{ n.show ? '显示中' : '已隐藏' }}</span></td>
                    <td>
                      <div class="actions">
                        <button type="button" class="btn sm" :disabled="action.busy.value" @click="toggle(n)">{{ n.show ? '隐藏' : '显示' }}</button>
                        <button type="button" class="btn sm danger" :disabled="action.busy.value" @click="remove(n)">删除</button>
                      </div>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </template>
    </AsyncState>
  </div>
</template>
