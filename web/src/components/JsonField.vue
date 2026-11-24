<script setup lang="ts">
// Textarea bound to a JSON object; emits only when the text parses to an object.
import { ref, watch } from 'vue'
const props = defineProps<{ modelValue: Record<string, any>; id: string; rows?: number }>()
const emit = defineEmits<{ 'update:modelValue': [value: Record<string, any>]; valid: [ok: boolean] }>()
const text = ref(JSON.stringify(props.modelValue ?? {}, null, 2))
const invalid = ref(false)
let last = text.value
watch(() => props.modelValue, (value) => {
  const next = JSON.stringify(value ?? {}, null, 2)
  if (next !== last) text.value = last = next
})
function onInput() {
  try {
    const parsed = JSON.parse(text.value || '{}')
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) throw new Error('not an object')
    invalid.value = false
    last = JSON.stringify(parsed, null, 2)
    emit('update:modelValue', parsed)
  } catch {
    invalid.value = true
  }
  emit('valid', !invalid.value)
}
</script>

<template>
  <textarea :id="id" v-model="text" class="textarea mono" :rows="rows || 4" spellcheck="false" :aria-invalid="invalid" @input="onInput" />
  <span v-if="invalid" class="hint" style="color: var(--danger)">不是有效的 JSON 对象，修正后才会保存这一项。</span>
</template>
