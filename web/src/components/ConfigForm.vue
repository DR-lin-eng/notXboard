<script setup lang="ts">
// Renders an admin settings page from a schema and saves it through config/save.
import { computed, reactive, ref, watch } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import { adminV2, errorMessage } from '@/lib/api'
import { buildConfigPayload, readConfigValues, type ConfigPage } from '@/lib/admin/configSchema'
import { useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const props = defineProps<{
  page: ConfigPage
  /** Runs before saving; return false to cancel. */
  beforeSave?: (payload: Record<string, any>, original: Record<string, any>) => Promise<boolean> | boolean
}>()
const emit = defineEmits<{ saved: [payload: Record<string, any>]; loaded: [group: Record<string, any>] }>()
const ui = useUi()
const values = reactive<Record<string, any>>({})
const original = ref<Record<string, any>>({})
const saving = ref(false)
const formError = ref('')

const { loading, error, reload } = useAsync(async () => {
  const config = await adminV2<any>('config/fetch')
  const group = (config && typeof config === 'object' ? config[props.page.group] : null) || {}
  const next = readConfigValues(props.page, group)
  for (const key of Object.keys(values)) delete values[key]
  Object.assign(values, next)
  original.value = { ...next }
  emit('loaded', group)
  return true
})
watch(() => props.page, () => reload())

const dirty = computed(() => Object.keys(values).some((key) => values[key] !== original.value[key]))
defineExpose({ values, reload })

async function save() {
  formError.value = ''
  const payload = buildConfigPayload(props.page, values)
  if (props.beforeSave && !(await props.beforeSave(payload, original.value))) return
  saving.value = true
  try {
    await adminV2('config/save', { method: 'POST', body: payload })
    ui.ok('设置已保存')
    emit('saved', payload)
    await reload(true)
  } catch (e) {
    formError.value = errorMessage(e, '保存失败')
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <AsyncState :loading="loading" :error="error" @retry="reload()">
    <form class="col" @submit.prevent="save">
      <div v-if="formError" class="notice error" role="alert">{{ formError }}</div>
      <div v-for="section in page.sections" :key="section.title" class="card">
        <div class="card-head"><h3>{{ section.title }}</h3><span v-if="section.description" class="sub">{{ section.description }}</span></div>
        <div class="card-body form-grid">
          <template v-for="field in section.fields" :key="field.key">
            <div v-if="!field.showIf || field.showIf(values)" class="field" :class="{ full: field.full || field.type === 'lines' || field.type === 'textarea' }">
              <label v-if="field.type === 'bool'" class="switch">
                <input v-model="values[field.key]" type="checkbox" role="switch">
                <span>{{ field.label }}</span>
              </label>
              <template v-else>
                <label :for="`cfg-${field.key}`">{{ field.label }}</label>
                <select v-if="field.type === 'select'" :id="`cfg-${field.key}`" v-model="values[field.key]" class="select">
                  <option v-for="option in field.options" :key="option.value" :value="option.value">{{ option.label }}</option>
                </select>
                <textarea v-else-if="field.type === 'lines' || field.type === 'textarea'" :id="`cfg-${field.key}`" v-model="values[field.key]" class="textarea" rows="4" :placeholder="field.placeholder" />
                <input v-else-if="field.type === 'number' || field.type === 'float'" :id="`cfg-${field.key}`" v-model="values[field.key]" class="input" type="number" :min="field.min" :max="field.max" :step="field.step ?? (field.type === 'float' ? 0.01 : 1)">
                <input v-else :id="`cfg-${field.key}`" v-model="values[field.key]" class="input" :type="field.type === 'password' ? 'password' : 'text'" :readonly="field.type === 'readonly'" :placeholder="field.placeholder" autocomplete="off">
              </template>
              <span v-if="field.hint" class="hint">{{ field.hint }}</span>
              <slot :name="`after-${field.key}`" :values="values" />
            </div>
          </template>
        </div>
      </div>
      <div class="savebar">
        <span class="muted small">{{ dirty ? '有未保存的修改' : '所有修改已保存' }}</span><span class="sp" />
        <button type="button" class="btn" :disabled="!dirty || saving" @click="reload()">放弃修改</button>
        <button type="submit" class="btn primary" :disabled="saving">{{ saving ? '保存中…' : '保存设置' }}</button>
      </div>
    </form>
  </AsyncState>
</template>
