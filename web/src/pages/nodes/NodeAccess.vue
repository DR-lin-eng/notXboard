<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import { api, errorMessage } from '@/lib/api'
import { parseIdList } from '@/lib/domain'
import { asArray } from '@/lib/format'
import { TRUST_LEVEL_OPTIONS } from '@/lib/nodeConstants'
import { useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const route = useRoute()
const router = useRouter()
const ui = useUi()
const nodeId = computed(() => String(route.params.id))
const form = reactive({ min_trust: 0, user_ids: '', quota: {} as Record<number, number> })
const formError = ref('')
const saving = ref(false)

const { data: node, loading, error, reload } = useAsync(async () => {
  const data = await api<any>(`/api/v1/user/server-nodes/${encodeURIComponent(nodeId.value)}`)
  const quota = data?.access_control?.free_quota_gb_by_trust_level || {}
  form.min_trust = Number(data?.access_control?.min_trust_level ?? 0)
  form.user_ids = asArray(data?.access_control?.authorized_users).join(',')
  for (const level of TRUST_LEVEL_OPTIONS) form.quota[level] = Number(quota[String(level)] ?? 0) || 0
  return data || {}
})

async function save() {
  formError.value = ''
  saving.value = true
  try {
    await api(`/api/v1/user/server-nodes/${encodeURIComponent(nodeId.value)}/access`, {
      method: 'POST',
      body: {
        min_trust_level: Number.isFinite(Number(form.min_trust)) ? Number(form.min_trust) : null,
        authorized_users: parseIdList(form.user_ids),
        free_quota_gb_by_trust_level: Object.fromEntries(TRUST_LEVEL_OPTIONS.map((level) => [level, Number(form.quota[level]) || 0])),
      },
    })
    ui.ok('分享设置已保存')
    router.push('/nodes')
  } catch (e) {
    formError.value = errorMessage(e, '保存失败')
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="page narrow">
    <div class="page-head">
      <div><h2>分享与权限：{{ node?.name || `#${nodeId}` }}</h2><p>按用户等级开放，或指定可用用户。</p></div>
      <div class="actions"><RouterLink class="btn" to="/nodes"><ArrowLeft />返回</RouterLink></div>
    </div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <form class="col" @submit.prevent="save">
        <div v-if="formError" class="notice error" role="alert">{{ formError }}</div>
        <div class="card">
          <div class="card-head"><h3>谁可以使用</h3></div>
          <div class="card-body form-grid">
            <div class="field">
              <label for="na-trust">最低用户等级</label>
              <select id="na-trust" v-model.number="form.min_trust" class="select">
                <option v-for="level in TRUST_LEVEL_OPTIONS" :key="level" :value="level">等级 {{ level }}</option>
              </select>
            </div>
            <div class="field">
              <label for="na-users">指定可用用户 ID</label>
              <input id="na-users" v-model="form.user_ids" class="input" placeholder="例如 1,2,3">
              <span class="hint">多个用逗号分隔。</span>
            </div>
          </div>
        </div>
        <div class="card">
          <div class="card-head"><h3>免费额度</h3><span class="sub">按用户等级，GB / 月</span></div>
          <div class="card-body">
            <p class="muted small">为不同等级用户提供该节点的免费使用额度，超出后不再下发到该节点。</p>
            <div class="form-grid mt-sm">
              <div v-for="level in TRUST_LEVEL_OPTIONS" :key="level" class="field">
                <label :for="`na-q${level}`">等级 {{ level }}</label>
                <input :id="`na-q${level}`" v-model.number="form.quota[level]" class="input" type="number" min="0" step="0.01">
              </div>
            </div>
          </div>
        </div>
        <div class="row"><span class="sp" /><button type="submit" class="btn primary" :disabled="saving">{{ saving ? '保存中…' : '保存' }}</button></div>
      </form>
    </AsyncState>
  </div>
</template>
