<script setup lang="ts">
import { Copy } from 'lucide-vue-next'
import { copyText } from '@/lib/format'
import { useUi } from '@/stores/ui'

const props = defineProps<{ value: string; label?: string; display?: string }>()
const ui = useUi()

async function copy() {
  if (await copyText(props.value)) ui.ok(`${props.label || '内容'}已复制`)
  else ui.error('复制失败，请手动选择复制')
}
</script>

<template>
  <div class="copy">
    <span class="mono" :title="display || value">{{ display || value }}</span>
    <button type="button" class="btn sm ghost" :aria-label="`复制${label || ''}`" @click="copy"><Copy />复制</button>
  </div>
</template>
