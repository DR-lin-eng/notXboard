<script setup lang="ts">
import { ref } from 'vue'
import ConfigForm from '@/components/ConfigForm.vue'
import type { ConfigPage } from '@/lib/admin/configSchema'

const form = ref<InstanceType<typeof ConfigForm> | null>(null)
const appUrl = ref('')
const page: ConfigPage = {
  group: 'oauth',
  sections: [{
    title: 'Linux DO 登录',
    fields: [
      { key: 'oauth_linux_do_enable', label: '启用 Linux DO 登录', type: 'bool', full: true },
      { key: 'oauth_linux_do_client_id', label: '客户端编号（Client ID）', type: 'text' },
      { key: 'oauth_linux_do_client_secret', label: '客户端密钥（Client Secret）', type: 'password' },
      { key: 'oauth_linux_do_redirect_uri', label: '回调地址', type: 'text', full: true, hint: '必须和 Linux DO 应用后台填写的一致。' },
    ],
  }],
}
function callbackUrl(): string {
  const base = (appUrl.value || location.origin).replace(/\/+$/, '')
  return `${base}/api/v1/passport/auth/linuxdo/callback`
}
function useSuggested() {
  if (form.value) form.value.values.oauth_linux_do_redirect_uri = callbackUrl()
}
</script>

<template>
  <div class="page narrow">
    <div class="page-head"><div><h2>第三方登录</h2><p>Linux DO OAuth 与回调地址。</p></div></div>
    <ConfigForm ref="form" :page="page" @loaded="(config) => (appUrl = String(config.site?.app_url || ''))">
      <template #after-oauth_linux_do_redirect_uri>
        <div class="row wrap"><span class="muted small mono">{{ callbackUrl() }}</span><button type="button" class="btn sm" @click="useSuggested">使用推荐地址</button></div>
      </template>
    </ConfigForm>
  </div>
</template>
