<script setup lang="ts">
import { ref } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import ConfigForm from '@/components/ConfigForm.vue'
import Pager from '@/components/Pager.vue'
import { adminList, adminPost } from '@/lib/admin/list'
import { isOn, type ConfigPage } from '@/lib/admin/configSchema'
import { formatAnyTs } from '@/lib/format'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const ui = useUi()
const action = useAction()
const tab = ref<'reviews' | 'bans' | 'settings'>('reviews')
const reviewPage = ref(1)
const banPage = ref(1)
const notify = ref({ risk: false, banned: false })
const form = ref<InstanceType<typeof ConfigForm> | null>(null)
const llmOn = (v: Record<string, any>) => Boolean(v.user_risk_review_llm_enable)

const page: ConfigPage = {
  group: 'risk_review',
  sections: [
    {
      title: '共享 IP 风险识别',
      fields: [
        { key: 'user_risk_review_enable', label: '启用定期审查', type: 'bool', full: true },
        { key: 'user_risk_review_schedule_minutes', label: '审查间隔（分钟）', type: 'number', fallback: 30, min: 5, max: 1440 },
        { key: 'user_risk_review_time_window_minutes', label: '共享 IP 统计窗口（分钟）', type: 'number', fallback: 60, min: 5, max: 1440 },
        { key: 'user_risk_review_context_hours', label: '上下文分析时长（小时）', type: 'number', fallback: 24, min: 1, max: 168 },
        { key: 'user_risk_review_min_shared_ip_users', label: '最少命中用户数', type: 'number', fallback: 2, min: 2, max: 50 },
        { key: 'user_risk_review_scan_limit', label: '单次最大扫描 IP 数', type: 'number', fallback: 20, min: 1, max: 200 },
        { key: 'user_risk_review_notify_cooldown_minutes', label: '提醒冷却（分钟）', type: 'number', fallback: 60, min: 5, max: 10080 },
      ],
    },
    {
      title: 'LLM 预审查',
      description: '使用 OpenAI 兼容接口对命中的用户做预审',
      fields: [
        { key: 'user_risk_review_llm_enable', label: '启用 LLM 预审', type: 'bool', full: true },
        { key: 'user_risk_review_llm_base_url', label: 'Base URL', type: 'text', placeholder: 'https://api.openai.com/v1', showIf: llmOn },
        { key: 'user_risk_review_llm_api_key', label: 'API Key', type: 'password', showIf: llmOn },
        { key: 'user_risk_review_llm_model', label: '模型名称', type: 'text', showIf: llmOn },
        { key: 'user_risk_review_llm_timeout_seconds', label: '超时（秒）', type: 'number', fallback: 20, min: 5, max: 120, showIf: llmOn },
        { key: 'user_risk_review_llm_temperature', label: 'Temperature', type: 'float', fallback: 0.2, min: 0, max: 1, step: 0.1, showIf: llmOn },
      ],
    },
  ],
}

const reviews = useAsync(() => adminList('risk-review/fetch', { current: reviewPage.value, pageSize: 20 }))
const bans = useAsync(() => adminList('user/ban-records', { current: banPage.value, pageSize: 20 }))
const matched = (review: any) => (review.matched_users || []).map((item: any) => item.email || `#${item.id}`).join(', ') || String(review.matched_user_count || 0)

async function runNow() {
  await action.run(async () => {
    await adminPost('risk-review/run', { limit: Number(form.value?.values.user_risk_review_scan_limit) || 20 })
    ui.ok('审查已执行')
    reviewPage.value = 1
    await reviews.reload(true)
  }, ui.error)
}
async function ban(review: any) {
  const reason = await ui.prompt(`封禁 ${review.user_email || `#${review.user_id}`}？`, {
    message: '将立即阻止该用户登录和使用服务。', placeholder: '填写封禁原因', value: String(review.summary || ''), multiline: true, confirmText: '确认封禁', danger: true,
  })
  if (reason === null) return
  if (!reason.trim()) {
    ui.error('封禁原因不能为空')
    return
  }
  await action.run(async () => {
    await adminPost('user/update', { id: review.user_id, banned: 1, ban_reason: reason.trim() })
    ui.ok('用户已封禁')
    await Promise.all([reviews.reload(true), bans.reload(true)])
  }, ui.error)
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div><h2>风险审查</h2><p>共享 IP 风险识别、预审查结果和封禁记录。</p></div>
      <div class="actions"><button type="button" class="btn" :disabled="action.busy.value" @click="runNow">立即执行一次审查</button></div>
    </div>
    <div class="seg mb" role="tablist">
      <button type="button" role="tab" :aria-selected="tab === 'reviews'" :class="{ active: tab === 'reviews' }" @click="tab = 'reviews'">审查结果</button>
      <button type="button" role="tab" :aria-selected="tab === 'bans'" :class="{ active: tab === 'bans' }" @click="tab = 'bans'">封禁记录</button>
      <button type="button" role="tab" :aria-selected="tab === 'settings'" :class="{ active: tab === 'settings' }" @click="tab = 'settings'">审查设置</button>
    </div>

    <AsyncState v-show="tab === 'reviews'" :loading="reviews.loading.value" :error="reviews.error.value" @retry="reviews.reload()">
      <div class="card">
        <div v-if="!reviews.data.value?.items.length" class="empty">暂无风险审查记录。</div>
        <div v-else class="table-wrap">
          <table class="table">
            <thead><tr><th>时间</th><th>用户</th><th>共享 IP</th><th>命中用户</th><th>风险</th><th>状态</th><th>意见摘要</th><th>处理建议</th><th class="right">操作</th></tr></thead>
            <tbody>
              <tr v-for="(review, index) in reviews.data.value.items" :key="review.id ?? index">
                <td class="muted nowrap">{{ formatAnyTs(review.reviewed_at) }}</td>
                <td><b>#{{ review.user_id }}</b><div class="muted small">{{ review.user_email || '-' }}</div></td>
                <td class="mono">{{ review.shared_ip || '-' }}</td>
                <td class="small">{{ matched(review) }}</td>
                <td><span class="badge" :class="review.risk_level === 'high' ? 'danger' : 'warn'">{{ review.risk_level || 'medium' }} · {{ review.suspicion_score || 0 }}</span><div class="muted small">{{ review.llm_model || 'heuristic' }}</div></td>
                <td><span class="badge" :class="review.user_banned ? 'danger' : 'ok'">{{ review.user_banned ? '已封禁' : '正常' }}</span><div v-if="review.user_ban_reason" class="muted small">{{ review.user_ban_reason }}</div></td>
                <td class="small wrap-cell">{{ review.summary || '-' }}</td>
                <td class="small wrap-cell">{{ review.recommendation || '-' }}</td>
                <td class="right nowrap">
                  <span v-if="review.user_banned" class="muted small">已处理</span>
                  <button v-else type="button" class="btn sm danger" :disabled="action.busy.value" @click="ban(review)">封禁</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <Pager :page="reviews.data.value?.page || 1" :last-page="reviews.data.value?.lastPage || 1" :total="reviews.data.value?.total" @change="(p) => { reviewPage = p; reviews.reload(true) }" />
      </div>
    </AsyncState>

    <AsyncState v-show="tab === 'bans'" :loading="bans.loading.value" :error="bans.error.value" @retry="bans.reload()">
      <div class="card">
        <div v-if="!bans.data.value?.items.length" class="empty">暂无封禁记录。</div>
        <div v-else class="table-wrap">
          <table class="table">
            <thead><tr><th>时间</th><th>用户</th><th>操作人</th><th>动作</th><th>来源</th><th>原因</th></tr></thead>
            <tbody>
              <tr v-for="(record, index) in bans.data.value.items" :key="record.id ?? index">
                <td class="muted nowrap">{{ formatAnyTs(record.created_at) }}</td>
                <td><b>#{{ record.user_id }}</b><div class="muted small">{{ record.user_email || '-' }}</div></td>
                <td>{{ record.admin_email || '系统' }}<span v-if="record.admin_id" class="muted small"> #{{ record.admin_id }}</span></td>
                <td><span class="badge" :class="record.action === 'ban' ? 'danger' : 'ok'">{{ record.action === 'ban' ? '封禁' : record.action === 'unban' ? '解封' : record.action || '-' }}</span></td>
                <td>{{ record.source || '-' }}</td>
                <td class="small wrap-cell">{{ record.reason || '-' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <Pager :page="bans.data.value?.page || 1" :last-page="bans.data.value?.lastPage || 1" :total="bans.data.value?.total" @change="(p) => { banPage = p; bans.reload(true) }" />
      </div>
    </AsyncState>

    <div v-show="tab === 'settings'" style="max-width: 820px">
      <p class="muted small mb">
        Telegram 通知：风险提醒{{ notify.risk ? '已开启' : '未开启' }}，封禁通知{{ notify.banned ? '已开启' : '未开启' }}。
        <RouterLink to="/admin/telegram">通知设置</RouterLink>
      </p>
      <ConfigForm ref="form" :page="page" @loaded="(config) => (notify = { risk: isOn(config.telegram?.telegram_notify_user_risk_detected), banned: isOn(config.telegram?.telegram_notify_user_banned) })" />
    </div>
  </div>
</template>

<style scoped>
.wrap-cell { min-width: 180px; max-width: 320px; white-space: normal; }
</style>
