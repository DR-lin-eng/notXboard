<script setup lang="ts">
import { computed } from 'vue'
import { Copy, Download, KeyRound } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import QuotaTable from '@/components/QuotaTable.vue'
import UsageBar from '@/components/UsageBar.vue'
import { api } from '@/lib/api'
import { asArray, copyText, formatTrafficDisplay, formatTrafficKb, formatTs } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useSession } from '@/stores/session'
import { useUi } from '@/stores/ui'

const session = useSession()
const ui = useUi()

const { data, loading, error, reload } = useAsync(async () => {
  const [info, subscribe, quota, apiKey, stats] = await Promise.all([
    api<any>('/api/v1/user/info'),
    api<any>('/api/v1/user/getSubscribe'),
    api<any>('/api/v1/user/plan/quota'),
    api<any>('/api/v1/user/api-key'),
    api<any>('/api/v1/user/access/stats'),
  ])
  return { info: info || {}, subscribe: subscribe || {}, quota: quota || {}, apiKey: apiKey || {}, stats: stats || {} }
})
const keyAction = useAction()

const quotaItems = computed(() => asArray(data.value?.quota.items))
const unlimited = computed(() => Boolean(data.value?.quota.has_unlimited_traffic))
const allowanceKb = computed(() => Number(data.value?.quota.total_allowance_kb || 0))
const usedKb = computed(() => Number(data.value?.quota.total_used_kb || 0))
const remainingKb = computed(() => Number(data.value?.quota.total_remaining_kb || 0))
const usagePercent = computed(() =>
  unlimited.value || allowanceKb.value <= 0 ? 0 : Math.max(0, Math.min(100, (usedKb.value / allowanceKb.value) * 100)),
)
const expiredAt = computed(() => Number(data.value?.subscribe.expired_at || 0))
const subscriptionActive = computed(() => expiredAt.value > Math.floor(Date.now() / 1000))
const hasActivePlan = computed(() => subscriptionActive.value || quotaItems.value.length > 0)
const subscriptionLabel = computed(() => (subscriptionActive.value ? '使用中' : quotaItems.value.length ? '套餐生效中' : '未开通'))
const subscriptionDetail = computed(() =>
  subscriptionActive.value
    ? `有效期至 ${formatTs(expiredAt.value)}`
    : quotaItems.value.length ? `${quotaItems.value.length} 个套餐实例` : '选择套餐后即可开始使用',
)
const ipLimit = computed(() => {
  const limit = Number(session.me?.concurrent_ip_limit)
  return limit > 0 ? limit : 3
})
const subscribeUrl = computed(() => String(data.value?.subscribe.subscribe_url || ''))
const roleDuty = computed(() =>
  session.isSuperAdmin ? '站长：负责站点设置和全局管理' : session.isAdmin ? '管理员：协助处理节点与工单' : '用户：提供节点并使用订阅',
)

async function copySubscribe() {
  if (!subscribeUrl.value) return
  if (await copyText(subscribeUrl.value)) ui.ok('订阅链接已复制')
  else ui.error('复制失败，请手动复制')
}

async function resetKey() {
  const has = Boolean(data.value?.apiKey.has_api_key)
  if (has && !(await ui.confirm('重置 API Key？', { message: '旧的 Key 会立即失效，使用它的脚本需要更新。', confirmText: '重置', danger: true }))) return
  await keyAction.run(async () => {
    await api(has ? '/api/v1/user/api-key/reset' : '/api/v1/user/api-key/generate', { method: 'POST', body: {} })
    ui.ok(has ? 'API Key 已重置' : 'API Key 已生成')
    await reload(true)
  }, ui.error)
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div>
        <h2>使用概览</h2>
        <p>订阅、流量与节点状态</p>
      </div>
      <span v-if="data" class="badge" :class="hasActivePlan ? 'ok' : 'warn'"><i class="dot" />{{ hasActivePlan ? '服务可用' : '需要选择套餐' }}</span>
      <div class="actions">
        <RouterLink class="btn" to="/downloads"><Download />下载客户端</RouterLink>
        <button type="button" class="btn primary" :disabled="!subscribeUrl" @click="copySubscribe"><Copy />复制订阅链接</button>
      </div>
    </div>

    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <template v-if="data">
        <div class="grid g4">
          <div class="card stat">
            <div class="k">订阅状态</div>
            <div class="v" :class="hasActivePlan ? 'text-ok' : 'text-warn'">{{ subscriptionLabel }}</div>
            <div class="d">{{ subscriptionDetail }}</div>
          </div>
          <div class="card stat">
            <div class="k">剩余流量</div>
            <div class="v">{{ formatTrafficDisplay(remainingKb, unlimited) }}</div>
            <div class="d">{{ unlimited ? '当前套餐不限总流量' : `已使用 ${usagePercent.toFixed(1)}%` }}</div>
          </div>
          <div class="card stat">
            <div class="k">可用节点</div>
            <div class="v">{{ data.stats.accessible_nodes ?? '-' }}</div>
            <div class="d">当前账号可以连接</div>
          </div>
          <div class="card stat">
            <div class="k">并发 IP</div>
            <div class="v">{{ ipLimit }}</div>
            <div class="d">当前连接上限</div>
          </div>
        </div>

        <div v-if="!hasActivePlan" class="notice accent row mt">
          <span class="sp">当前账户尚未生效套餐，选择套餐后将显示订阅信息。</span>
          <RouterLink class="btn primary sm" to="/plans">选择套餐</RouterLink>
        </div>

        <div class="card mt">
          <div class="card-head">
            <h3>套餐额度</h3>
            <span class="sub">不同套餐独立扣费，额度按套餐实例统计</span>
            <span class="sp" />
            <RouterLink class="btn sm" to="/plans">管理套餐</RouterLink>
          </div>
          <div class="card-body">
            <div class="grid g3">
              <div><div class="muted small">总额度</div><div class="strong big">{{ formatTrafficDisplay(allowanceKb, unlimited) }}</div></div>
              <div><div class="muted small">已使用</div><div class="strong big">{{ formatTrafficKb(usedKb) }}</div></div>
              <div><div class="muted small">剩余</div><div class="strong big">{{ formatTrafficDisplay(remainingKb, unlimited) }}</div></div>
            </div>
            <UsageBar v-if="!unlimited" class="mt-sm" :percent="usagePercent" label="套餐总流量使用进度" />
          </div>
          <QuotaTable :items="quotaItems" />
        </div>

        <div class="grid g2 mt">
          <div class="card">
            <div class="card-head"><h3>账户信息</h3></div>
            <div class="card-body">
              <dl class="kv">
                <dt>邮箱</dt><dd>{{ data.info.email || '-' }}</dd>
                <dt>UUID</dt><dd class="mono">{{ data.info.uuid || '-' }}</dd>
                <dt>用户等级</dt><dd>{{ session.me?.trust_level ?? '-' }}</dd>
                <dt>第三方用户名</dt><dd>{{ session.me?.linux_do_username || '-' }}</dd>
                <dt>身份</dt>
                <dd class="chips">
                  <span v-if="session.me?.is_linux_do_user" class="badge ok">Linux DO Connect</span>
                  <span class="badge" :class="{ accent: session.isAdmin }">{{ session.isSuperAdmin ? '站长' : session.isAdmin ? '管理员' : '用户' }}</span>
                </dd>
                <dt>角色说明</dt><dd>{{ roleDuty }}</dd>
                <dt>到期时间</dt><dd>{{ expiredAt ? formatTs(expiredAt) : '-' }}</dd>
              </dl>
            </div>
          </div>

          <div class="col">
            <div class="card">
              <div class="card-head"><h3>API Key</h3></div>
              <div class="card-body">
                <dl class="kv">
                  <dt>是否已生成</dt><dd>{{ data.apiKey.has_api_key ? '是' : '否' }}</dd>
                  <dt>当前 Key</dt><dd class="mono">{{ data.apiKey.api_key || '-' }}</dd>
                  <dt>创建时间</dt><dd>{{ data.apiKey.created_at || '-' }}</dd>
                </dl>
              </div>
              <div class="card-foot">
                <button type="button" class="btn sm" :disabled="keyAction.busy.value" @click="resetKey">
                  <KeyRound />{{ data.apiKey.has_api_key ? '重置 Key' : '生成 Key' }}
                </button>
              </div>
            </div>
            <div class="card">
              <div class="card-head">
                <h3>节点统计</h3>
                <span class="sp" />
                <RouterLink class="link small" to="/nodes">查看节点</RouterLink>
              </div>
              <div class="card-body">
                <dl class="kv">
                  <dt>我提供的节点</dt><dd>{{ data.stats.owned_nodes ?? '-' }}</dd>
                  <dt>可用节点总数</dt><dd>{{ data.stats.accessible_nodes ?? '-' }}</dd>
                  <dt>被分享的节点</dt><dd>{{ data.stats.shared_nodes ?? '-' }}</dd>
                  <dt>并发 IP 限制</dt><dd>{{ ipLimit }}（默认 3）<RouterLink class="link small" to="/audit"> 查看审计记录</RouterLink></dd>
                </dl>
              </div>
            </div>
          </div>
        </div>
      </template>
    </AsyncState>
  </div>
</template>

<style scoped>
.big { font-size: 18px; }
</style>
