<script setup lang="ts">
import { reactive, ref } from 'vue'
import { ApiError, getAdminPath, request } from '@/lib/api'
import { useUi } from '@/stores/ui'

const ui = useUi()
const form = reactive({ scope: 'v2', method: 'GET', endpoint: 'stat/getStats', body: '' })
const output = ref('')
const status = ref('')
const busy = ref(false)

function resolvePath(): string {
  const endpoint = form.endpoint.trim()
  if (!endpoint) throw new Error('请输入请求地址')
  if (form.scope === 'v2') {
    const adminPath = getAdminPath()
    if (!adminPath) throw new Error('当前账号没有管理权限')
    return `/api/v2/${encodeURIComponent(adminPath)}/${endpoint.replace(/^\/+/, '')}`
  }
  if (form.scope === 'v1') return `/api/v1/${endpoint.replace(/^\/+/, '')}`
  // The login token must never leave this site.
  const url = new URL(endpoint, window.location.origin)
  if (url.origin !== window.location.origin) throw new Error('自定义地址只能是当前站点')
  return `${url.pathname}${url.search}`
}
async function send() {
  let path: string
  let body: unknown
  try {
    path = resolvePath()
    if (form.method !== 'GET' && form.body.trim()) body = JSON.parse(form.body)
  } catch (e) {
    ui.error(e instanceof SyntaxError ? '请求内容不是有效的 JSON' : (e as Error).message)
    return
  }
  if (form.method === 'DELETE' && !(await ui.confirm('发送 DELETE 请求？', { message: `${path} 可能永久删除数据。`, confirmText: '发送', danger: true }))) return
  busy.value = true
  status.value = ''
  try {
    const result = await request<any>(path, { method: form.method as any, body })
    status.value = '请求成功'
    output.value = JSON.stringify(result, null, 2)
  } catch (e) {
    const err = e as ApiError
    status.value = `请求失败${err.status ? `（HTTP ${err.status}）` : ''}：${err.message}`
    output.value = err.payload ? JSON.stringify(err.payload, null, 2) : ''
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="page">
    <div class="page-head"><div><h2>接口调试</h2><p>用当前登录身份直接调用面板接口，用于排查问题。</p></div></div>
    <div class="card mb">
      <form class="card-body form-grid" @submit.prevent="send">
        <div class="field"><label for="api-scope">接口范围</label>
          <select id="api-scope" v-model="form.scope" class="select"><option value="v2">管理接口（v2）</option><option value="v1">用户接口（v1）</option><option value="custom">自定义路径</option></select>
        </div>
        <div class="field"><label for="api-method">方法</label>
          <select id="api-method" v-model="form.method" class="select"><option>GET</option><option>POST</option><option>PUT</option><option>DELETE</option></select>
        </div>
        <div class="field full"><label for="api-endpoint">请求地址</label>
          <input id="api-endpoint" v-model="form.endpoint" class="input mono" :placeholder="form.scope === 'custom' ? '/api/v1/guest/comm/config' : 'stat/getStats'" required>
        </div>
        <div v-if="form.method !== 'GET'" class="field full"><label for="api-body">请求内容（JSON）</label><textarea id="api-body" v-model="form.body" class="textarea mono" rows="6" spellcheck="false" placeholder="{}" /></div>
        <div class="field full"><button type="submit" class="btn primary" :disabled="busy">{{ busy ? '请求中…' : '发送请求' }}</button></div>
      </form>
    </div>
    <div class="card">
      <div class="card-head"><h3>响应</h3><span class="muted small">{{ status }}</span></div>
      <div class="card-body"><pre v-if="output" class="codebox">{{ output }}</pre><div v-else class="muted">发送请求后在这里显示响应。</div></div>
    </div>
  </div>
</template>
