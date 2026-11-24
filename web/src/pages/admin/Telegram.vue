<script setup lang="ts">
import ConfigForm from '@/components/ConfigForm.vue'
import { adminV2 } from '@/lib/api'
import type { ConfigField, ConfigPage } from '@/lib/admin/configSchema'
import { useAction } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const ui = useUi()
const action = useAction()
const toggle = (key: string, label: string): ConfigField => ({ key, label, type: 'bool' })
const page: ConfigPage = {
  group: 'telegram',
  sections: [
    {
      title: 'Bot 基础配置',
      fields: [
        toggle('telegram_bot_enable', '启用 Telegram Bot'),
        { key: 'telegram_bot_token', label: 'Bot Token', type: 'password' },
        { key: 'telegram_discuss_link', label: '讨论群 / 频道链接', type: 'text', full: true },
      ],
    },
    {
      title: '功能与通知',
      fields: [
        toggle('telegram_user_ticket_enable', '允许用户在 Telegram 处理工单'),
        toggle('telegram_notify_ticket_created', '工单创建通知'),
        toggle('telegram_notify_ticket_replied', '工单回复通知'),
        toggle('telegram_notify_ticket_closed', '工单关闭通知'),
        toggle('telegram_notify_payment_success', '支付成功通知'),
        toggle('telegram_notify_notice_published', '公告发布通知'),
        toggle('telegram_notify_tcping_alert', 'TCPing 告警通知'),
        toggle('telegram_notify_tcping_recover', 'TCPing 恢复通知'),
        toggle('telegram_notify_refund_vote', '争议投票通知'),
        toggle('telegram_notify_refund_status', '争议状态变更通知'),
        toggle('telegram_notify_user_risk_detected', '用户风险审查通知'),
        toggle('telegram_notify_user_banned', '用户封禁通知'),
      ],
    },
  ],
}
async function registerWebhook() {
  await action.run(async () => {
    await adminV2('config/setTelegramWebhook', { method: 'POST', body: {} })
    ui.ok('Webhook 已注册')
  }, ui.error)
}
</script>

<template>
  <div class="page narrow">
    <div class="page-head">
      <div><h2>通知设置</h2><p>Telegram Bot 与事件通知。</p></div>
      <div class="actions"><button type="button" class="btn" :disabled="action.busy.value" @click="registerWebhook">注册 Webhook</button></div>
    </div>
    <p class="muted small mb">修改 Bot Token 后请先保存，再注册 Webhook。</p>
    <ConfigForm :page="page" />
  </div>
</template>
