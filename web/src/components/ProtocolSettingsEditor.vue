<script setup lang="ts">
// Visual editor for a node's protocol settings: recommended fields first, everything else under "advanced".
import { computed, reactive, watch } from 'vue'
import SettingField from './SettingField.vue'
import {
  advancedPaths, applyModePreset, deepClone, editorMode, getByPath, leafPaths, modePresets,
  recommendedPaths, setByPath, type Settings,
} from '@/lib/nodeProtocol'

const props = defineProps<{ protocol: string; host: string }>()
const settings = defineModel<Settings>({ required: true })

// A path that was ever null stays nullable, so clearing a filled field restores null rather than "".
const nullable = reactive(new Set<string>())
watch(settings, (value) => {
  for (const path of leafPaths(value)) if (getByPath(value, path) === null) nullable.add(path)
}, { immediate: true })

const presets = computed(() => modePresets(props.protocol))
const mode = computed(() => editorMode(props.protocol, settings.value))
const recommended = computed(() => recommendedPaths(props.protocol, settings.value).filter((path) => getByPath(settings.value, path) !== undefined))
const advanced = computed(() => advancedPaths(props.protocol, settings.value))

function update(path: string, value: unknown) {
  const next = deepClone(settings.value)
  setByPath(next, path, value)
  settings.value = next
}
function switchMode(next: string) {
  settings.value = applyModePreset(props.protocol, settings.value, next, props.host)
}
watch(() => props.host, (host) => {
  if (mode.value) settings.value = applyModePreset(props.protocol, settings.value, mode.value, host)
})
</script>

<template>
  <div class="col">
    <div v-if="presets.length" class="field">
      <label for="protocol-mode">推荐模式</label>
      <select id="protocol-mode" class="select" :value="mode" @change="switchMode(($event.target as HTMLSelectElement).value)">
        <option v-for="preset in presets" :key="preset.key" :value="preset.key">{{ preset.label }}</option>
      </select>
      <span class="hint">切换后会自动替换常见 TLS / REALITY 参数，其他高级字段会保留。</span>
    </div>
    <div v-if="recommended.length" class="form-grid">
      <SettingField v-for="path in recommended" :key="path" :path="path" :value="getByPath(settings, path)" :nullable="nullable.has(path)" @change="update(path, $event)" />
    </div>
    <div v-else class="muted">当前协议没有推荐参数。</div>
    <details v-if="advanced.length" class="fold">
      <summary>高级协议参数（大多数情况无需修改）</summary>
      <div class="form-grid mt-sm">
        <SettingField v-for="path in advanced" :key="path" :path="path" :value="getByPath(settings, path)" :nullable="nullable.has(path)" @change="update(path, $event)" />
      </div>
    </details>
  </div>
</template>
