<script setup lang="ts">
import { computed, reactive } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import CopyField from '@/components/CopyField.vue'
import QuotaTable from '@/components/QuotaTable.vue'
import { api, errorMessage } from '@/lib/api'
import { asArray, formatTrafficDisplay, formatTrafficKb } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useSession } from '@/stores/session'
import { useUi } from '@/stores/ui'
import { useRouter } from 'vue-router'

const session = useSession()
const router = useRouter()
const ui = useUi()
const action = useAction()
const { data, loading, error, reload } = useAsync(async () => {
  const [info, subscribe, quota, stat] = await Promise.all([
    api<any>('/api/v1/user/info'),
    api<any>('/api/v1/user/getSubscribe'),
    api<any>('/api/v1/user/plan/quota'),
    api<any>('/api/v1/user/getStat'),
  ])
  return { info: info || {}, subscribe: subscribe || {}, quota: quota || {}, stat: asArray(stat) }
})
const unlimited = computed(() => Boolean(data.value?.quota.has_unlimited_traffic))

async function resetSecurity() {
  if (!(await ui.confirm('重置订阅？', { message: '订阅链接和 UUID 会更换，所有客户端需要重新导入订阅。', confirmText: '重置', danger: true }))) return
  await action.run(async () => {
    await api('/api/v1/user/resetSecurity')
    ui.ok('订阅已重置')
    await reload(true)
  }, ui.error)
}

const pwd = reactive({ old: '', next: '', error: '', saving: false })
async function changePassword() {
  if (!pwd.old || !pwd.next) {
    pwd.error = '请填写旧密码和新密码'
    return
  }
  pwd.saving = true
  pwd.error = ''
  try {
    await api('/api/v1/user/changePassword', { method: 'POST', body: { old_password: pwd.old, new_password: pwd.next } })
    pwd.old = ''
    pwd.next = ''
    await ui.alert('密码已修改', '为了安全，所有设备上的登录都已退出，请用新密码重新登录。')
    session.clear()
    router.replace('/login')
  } catch (e) {
    pwd.error = errorMessage(e, '修改失败')
  } finally {
    pwd.saving = false
  }
}
</script>

<template>
  <div class="page">
    <div class="page-head"><div><h2>个人资料</h2><p>账户信息、订阅和密码。</p></div></div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <template v-if="data">
        <div class="grid g2">
          <div class="card">
            <div class="card-head"><h3>账户信息</h3></div>
            <div class="card-body col">
              <dl class="kv">
                <dt>邮箱</dt><dd>{{ data.info.email || '-' }}</dd>
                <dt>第三方用户名</dt><dd>{{ session.me?.linux_do_username || '-' }}</dd>
                <dt>身份</dt><dd>{{ session.roleLabel }}</dd>
              </dl>
              <div class="field">
                <label>订阅链接</label>
                <CopyField v-if="data.subscribe.subscribe_url" :value="data.subscribe.subscribe_url" label="订阅链接" />
                <span v-else class="muted">暂无订阅</span>
              </div>
            </div>
            <div class="card-foot"><button type="button" class="btn danger" :disabled="action.busy.value" @click="resetSecurity">重置订阅</button></div>
          </div>
          <form class="card" @submit.prevent="changePassword">
            <div class="card-head"><h3>修改密码</h3></div>
            <div class="card-body col">
              <div v-if="pwd.error" class="notice error" role="alert">{{ pwd.error }}</div>
              <div class="field"><label for="p-old">旧密码</label><input id="p-old" v-model="pwd.old" class="input" type="password" autocomplete="current-password"></div>
              <div class="field"><label for="p-new">新密码</label><input id="p-new" v-model="pwd.next" class="input" type="password" autocomplete="new-password"></div>
              <p class="muted small">修改后所有设备都会退出登录。</p>
            </div>
            <div class="card-foot"><button type="submit" class="btn primary" :disabled="pwd.saving">{{ pwd.saving ? '修改中…' : '修改密码' }}</button></div>
          </form>
        </div>

        <div class="grid g3 mt">
          <div class="card stat"><div class="k">未付款订单</div><div class="v">{{ data.stat[0] ?? '-' }}</div></div>
          <div class="card stat"><div class="k">未关闭工单</div><div class="v">{{ data.stat[1] ?? '-' }}</div></div>
          <div class="card stat"><div class="k">邀请人数</div><div class="v">{{ data.stat[2] ?? '-' }}</div></div>
        </div>

        <div class="card mt">
          <div class="card-head">
            <h3>套餐额度使用</h3><span class="sp" />
            <span class="muted small">已用 {{ formatTrafficKb(data.quota.total_used_kb || 0) }} / {{ formatTrafficDisplay(data.quota.total_allowance_kb || 0, unlimited) }} · 剩余 {{ formatTrafficDisplay(data.quota.total_remaining_kb || 0, unlimited) }}</span>
          </div>
          <QuotaTable :items="asArray(data.quota.items)" />
        </div>
      </template>
    </AsyncState>
  </div>
</template>
