<script setup lang="ts">
import { computed } from 'vue'
import { formatMoneyCent } from '@/lib/format'

const LABELS: Record<string, string> = {
  balance: '余额奖励', transfer_enable: '流量奖励', device_limit: '设备数奖励', expire_days: '有效期延长（天）',
  plan_id: '赠送套餐 ID', plan_validity_days: '套餐有效期（天）', reset_package: '赠送流量重置', invite_reward_rate: '邀请奖励比例',
}
const props = defineProps<{ rewards: unknown }>()
const entries = computed(() => (props.rewards && typeof props.rewards === 'object' ? Object.entries(props.rewards as Record<string, unknown>) : []))
function text(key: string, value: unknown): string {
  if (key === 'balance') return formatMoneyCent(value)
  if (typeof value === 'boolean') return value ? '是' : '否'
  if (value === null || value === undefined || value === '') return '-'
  return String(value)
}
</script>

<template>
  <div v-if="!entries.length" class="muted small">无奖励明细</div>
  <dl v-else class="kv">
    <template v-for="[key, value] in entries" :key="key"><dt>{{ LABELS[key] || key }}</dt><dd>{{ text(key, value) }}</dd></template>
  </dl>
</template>
