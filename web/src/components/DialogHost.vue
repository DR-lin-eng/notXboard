<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useUi } from '@/stores/ui'

const ui = useUi()
const current = computed(() => ui.dialogs[0] || null)
const value = ref('')
const field = ref<HTMLInputElement | HTMLTextAreaElement | null>(null)
const okBtn = ref<HTMLButtonElement | null>(null)

watch(current, async (dialog) => {
  value.value = dialog?.value || ''
  if (!dialog) return
  await nextTick()
  ;(field.value || okBtn.value)?.focus()
})

function cancel() {
  const dialog = current.value
  if (!dialog) return
  ui.closeDialog(dialog.id, dialog.kind === 'confirm' ? false : null)
}
function accept() {
  const dialog = current.value
  if (!dialog) return
  ui.closeDialog(dialog.id, dialog.kind === 'prompt' ? value.value : true)
}
</script>

<template>
  <div v-if="current" class="overlay" @mousedown.self="cancel" @keydown.esc="cancel">
    <form class="dialog" role="dialog" aria-modal="true" :aria-label="current.title" @submit.prevent="accept">
      <h3>{{ current.title }}</h3>
      <p v-if="current.message" class="muted msg">{{ current.message }}</p>
      <template v-if="current.kind === 'prompt'">
        <textarea v-if="current.multiline" ref="field" v-model="value" class="textarea" :placeholder="current.placeholder" />
        <input v-else ref="field" v-model="value" class="input" :placeholder="current.placeholder">
      </template>
      <div class="row foot">
        <span class="sp" />
        <button v-if="current.kind !== 'alert'" type="button" class="btn" @click="cancel">取消</button>
        <button ref="okBtn" type="submit" class="btn" :class="current.danger ? 'danger solid' : 'primary'">
          {{ current.confirmText || '确定' }}
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.overlay { position: fixed; inset: 0; z-index: 80; background: var(--overlay); display: grid; place-items: center; padding: 16px; }
.dialog { width: min(460px, 100%); background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); padding: 20px; box-shadow: var(--pop-shadow); display: flex; flex-direction: column; gap: 12px; }
.dialog h3 { font-size: 16px; font-weight: 650; }
.msg { white-space: pre-wrap; word-break: break-word; }
.foot { margin-top: 4px; }
</style>
