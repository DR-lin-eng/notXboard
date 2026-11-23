<script setup lang="ts">
import { X } from 'lucide-vue-next'
defineProps<{ title: string; wide?: boolean }>()
const emit = defineEmits<{ close: [] }>()
</script>

<template>
  <div class="overlay" @mousedown.self="emit('close')" @keydown.esc="emit('close')">
    <section class="modal" :class="{ wide }" role="dialog" aria-modal="true" :aria-label="title">
      <header class="row">
        <h3>{{ title }}</h3>
        <span class="sp" />
        <button type="button" class="icon-btn" aria-label="关闭" @click="emit('close')"><X /></button>
      </header>
      <div class="body"><slot /></div>
      <footer v-if="$slots.footer" class="row"><span class="sp" /><slot name="footer" /></footer>
    </section>
  </div>
</template>

<style scoped>
.overlay { position: fixed; inset: 0; z-index: 70; background: var(--overlay); display: grid; place-items: center; padding: 16px; }
.modal { width: min(560px, 100%); max-height: calc(100vh - 32px); background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); box-shadow: var(--pop-shadow); display: flex; flex-direction: column; }
.modal.wide { width: min(880px, 100%); }
header { padding: 14px 16px; border-bottom: 1px solid var(--border); }
header h3 { font-size: 15px; font-weight: 650; }
.body { padding: 16px; overflow: auto; }
footer { padding: 12px 16px; border-top: 1px solid var(--border); }
</style>
