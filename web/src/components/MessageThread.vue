<script setup lang="ts">
import { formatTs } from '@/lib/format'
// `labels` overrides the author names: [is_me, other].
const props = defineProps<{ messages: any[]; labels?: [string, string] }>()
const author = (m: any) => (props.labels ? props.labels[m.is_me ? 0 : 1] : m.is_me ? '我' : `用户 #${m.user_id}`)
const when = (value: unknown) => (typeof value === 'number' || /^\d+$/.test(String(value)) ? formatTs(value) : String(value || ''))
</script>

<template>
  <div v-if="!messages.length" class="empty">还没有消息。</div>
  <div v-else class="thread">
    <div v-for="(m, index) in messages" :key="index" class="msg" :class="{ me: m.is_me }">
      <div class="muted small">{{ author(m) }} · {{ when(m.created_at) }}</div>
      <div class="bubble">{{ m.message }}</div>
    </div>
  </div>
</template>

<style scoped>
.thread { display: flex; flex-direction: column; gap: 12px; }
.msg { max-width: min(640px, 92%); }
.msg.me { align-self: flex-end; text-align: right; }
.bubble { margin-top: 4px; padding: 9px 12px; border-radius: var(--radius-sm); background: var(--surface-2); white-space: pre-wrap; word-break: break-word; text-align: left; }
.msg.me .bubble { background: var(--accent-soft); }
</style>
