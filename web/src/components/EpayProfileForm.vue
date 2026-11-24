<script setup lang="ts">
// EPay merchant settings; the key is write-only and only sent when re-entered.
import { reactive, ref, watch } from 'vue'
import { api, errorMessage } from '@/lib/api'
import { useUi } from '@/stores/ui'

const props = defineProps<{ endpoint: string; idPrefix: string; profile: any }>()
const ui = useUi()
const error = ref('')
const saving = ref(false)
const form = reactive({ url: '', submit_path: '', pid: '', key: '', use_post: true, sitename: '', device: '' })

watch(() => props.profile, (profile) => {
  Object.assign(form, {
    url: profile?.url || 'https://credit.linux.do/epay',
    submit_path: profile?.submit_path || '/pay/submit.php',
    pid: profile?.pid || '', key: '',
    use_post: profile ? Boolean(profile.use_post) : true,
    sitename: profile?.sitename || '', device: profile?.device || '',
  })
}, { immediate: true })

async function save() {
  saving.value = true
  error.value = ''
  try {
    await api(props.endpoint, { method: 'PUT', body: { ...form, url: form.url.trim(), submit_path: form.submit_path.trim(), pid: form.pid.trim() } })
    form.key = ''
    ui.ok('收款设置已保存')
  } catch (e) {
    error.value = errorMessage(e, '保存失败')
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <form class="col" @submit.prevent="save">
    <div v-if="error" class="notice error" role="alert">{{ error }}</div>
    <div class="form-grid">
      <div class="field"><label :for="`${idPrefix}-url`">网关 URL</label><input :id="`${idPrefix}-url`" v-model="form.url" class="input"></div>
      <div class="field"><label :for="`${idPrefix}-path`">提交路径</label><input :id="`${idPrefix}-path`" v-model="form.submit_path" class="input"></div>
      <div class="field"><label :for="`${idPrefix}-pid`">商户号 PID</label><input :id="`${idPrefix}-pid`" v-model="form.pid" class="input"></div>
      <div class="field">
        <label :for="`${idPrefix}-key`">商户密钥</label>
        <input :id="`${idPrefix}-key`" v-model="form.key" class="input" type="password" autocomplete="off" :placeholder="profile?.has_key ? '已设置，重新输入可覆盖' : ''">
        <span class="hint">加密保存，不会回显。</span>
      </div>
      <div class="field">
        <label :for="`${idPrefix}-post`">提交方式</label>
        <select :id="`${idPrefix}-post`" v-model="form.use_post" class="select"><option :value="true">POST</option><option :value="false">GET</option></select>
      </div>
      <div class="field"><label :for="`${idPrefix}-site`">站点名称（可选）</label><input :id="`${idPrefix}-site`" v-model="form.sitename" class="input"></div>
      <div class="field"><label :for="`${idPrefix}-device`">设备标识（可选）</label><input :id="`${idPrefix}-device`" v-model="form.device" class="input"></div>
    </div>
    <div class="row"><span class="sp" /><button type="submit" class="btn primary" :disabled="saving">{{ saving ? '保存中…' : '保存' }}</button></div>
  </form>
</template>
