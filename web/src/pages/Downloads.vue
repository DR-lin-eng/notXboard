<script setup lang="ts">
import { computed } from 'vue'
import { Download } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import CopyField from '@/components/CopyField.vue'
import { api } from '@/lib/api'
import { normalizeHttpUrl } from '@/lib/domain'
import { useAsync } from '@/lib/useAsync'

const PLATFORMS = [
  { key: 'windows', title: 'Windows 客户端' },
  { key: 'macos', title: 'macOS 客户端' },
  { key: 'android', title: 'Android 客户端' },
] as const

const { data, loading, error, reload } = useAsync(() => api<any>('/api/v1/guest/comm/config'))
const rows = computed(() => PLATFORMS.map((item) => {
  const rawUrl = String(data.value?.[`${item.key}_download_url`] || '').trim()
  return { ...item, version: String(data.value?.[`${item.key}_version`] || '').trim(), rawUrl, url: normalizeHttpUrl(rawUrl) }
}))
</script>

<template>
  <div class="page">
    <div class="page-head"><div><h2>软件下载</h2><p>选择你的系统，直接下载客户端。</p></div></div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <div v-if="!rows.some((item) => item.url)" class="notice mb">当前还没有可用的下载地址，请联系站长在后台填写。</div>
      <div class="grid g3">
        <div v-for="item in rows" :key="item.key" class="card">
          <div class="card-head">
            <h3>{{ item.title }}</h3><span class="sp" />
            <span class="badge" :class="item.url ? 'ok' : 'warn'">{{ item.url ? '可下载' : '未配置' }}</span>
          </div>
          <div class="card-body col">
            <div class="muted small">版本 {{ item.version || '未填写' }}</div>
            <CopyField v-if="item.url" :value="item.url" label="下载链接" />
            <div v-else class="notice" :class="{ error: item.rawUrl }">{{ item.rawUrl ? '下载地址格式不正确，请联系站长处理。' : '站长还没有配置这个系统的下载地址。' }}</div>
          </div>
          <div class="card-foot">
            <a v-if="item.url" class="btn primary" :href="item.url" target="_blank" rel="noopener noreferrer"><Download />立即下载</a>
            <button v-else type="button" class="btn" disabled>暂不可下载</button>
          </div>
        </div>
      </div>
    </AsyncState>
  </div>
</template>
