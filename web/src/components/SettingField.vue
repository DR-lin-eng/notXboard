<script setup lang="ts">
// One protocol setting, typed from its current value: array, boolean, enum, number or (nullable) string.
import { computed } from 'vue'
import { settingEnumOptions, settingLabel } from '@/lib/nodeProtocol'

const props = defineProps<{ path: string; value: unknown; nullable: boolean }>()
const emit = defineEmits<{ change: [value: unknown] }>()

const id = computed(() => `setting_${props.path.replace(/[^a-zA-Z0-9_]+/g, '_')}`)
const label = computed(() => settingLabel(props.path))
const kind = computed(() => {
  if (Array.isArray(props.value)) return 'array'
  if (typeof props.value === 'boolean') return 'boolean'
  if (settingEnumOptions(props.path, props.value)) return 'enum'
  if (typeof props.value === 'number') return 'number'
  return 'string'
})
const options = computed(() => settingEnumOptions(props.path, props.value) || [])
const elementType = computed(() => {
  const values = (Array.isArray(props.value) ? props.value : []).filter((item) => item !== null && item !== undefined)
  if (!values.length) return 'string'
  if (values.every((item) => typeof item === 'number')) return 'number'
  if (values.every((item) => typeof item === 'boolean')) return 'boolean'
  return 'string'
})
const text = computed(() => (props.value === null || props.value === undefined ? '' : String(props.value)))
const arrayText = computed(() => (Array.isArray(props.value) ? props.value.map((item) => String(item ?? '')).join('\n') : ''))

function onArray(raw: string) {
  const lines = raw.split('\n').map((item) => item.trim()).filter(Boolean)
  if (elementType.value === 'number') {
    emit('change', lines.map(Number).filter((item) => Number.isFinite(item)))
  } else if (elementType.value === 'boolean') {
    emit('change', lines.map((item) => ['1', 'true', 'yes', 'on'].includes(item.toLowerCase())))
  } else {
    emit('change', lines)
  }
}
function onEnum(raw: string) {
  const matched = options.value.find((item) => String(item.value ?? '') === raw)
  if (matched) emit('change', matched.value === '' && props.nullable ? null : matched.value)
}
function onNumber(raw: string) {
  const parsed = Number(raw.trim() || '0')
  emit('change', Number.isFinite(parsed) ? parsed : 0)
}
function onString(raw: string) {
  const clean = raw.trim()
  emit('change', props.nullable && clean === '' ? null : clean)
}
const valueOf = (event: Event) => (event.target as HTMLInputElement).value
</script>

<template>
  <div class="field">
    <label :for="id">{{ label }}<template v-if="kind === 'array'">（数组）</template></label>
    <template v-if="kind === 'array'">
      <textarea :id="id" class="textarea mono" rows="3" placeholder="每行一项" :value="arrayText" @change="onArray(valueOf($event))" />
      <span class="hint">每行一项，留空表示空数组。</span>
    </template>
    <select v-else-if="kind === 'boolean'" :id="id" class="select" :value="value ? '1' : '0'" @change="emit('change', valueOf($event) === '1')">
      <option value="1">是</option>
      <option value="0">否</option>
    </select>
    <select v-else-if="kind === 'enum'" :id="id" class="select" :value="text" @change="onEnum(valueOf($event))">
      <option v-for="item in options" :key="String(item.value)" :value="String(item.value ?? '')">{{ item.label }}</option>
    </select>
    <input v-else-if="kind === 'number'" :id="id" class="input" type="number" step="0.01" :value="text" @change="onNumber(valueOf($event))">
    <input v-else :id="id" class="input" :value="text" :placeholder="nullable ? '留空表示空值' : ''" @change="onString(valueOf($event))">
  </div>
</template>
