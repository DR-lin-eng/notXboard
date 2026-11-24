<script setup lang="ts">
import { useUi } from '@/stores/ui'
const ui = useUi()
</script>

<template>
  <div class="toasts" aria-live="polite">
    <div v-for="toast in ui.toasts" :key="toast.id" class="toast" :class="toast.kind" role="status">{{ toast.text }}</div>
  </div>
</template>

<style scoped>
.toasts { position: fixed; z-index: 90; top: 16px; left: 50%; transform: translateX(-50%); display: flex; flex-direction: column; gap: 8px; width: min(440px, calc(100vw - 32px)); pointer-events: none; }
.toast { pointer-events: auto; background: var(--surface); border: 1px solid var(--border); border-left: 3px solid var(--accent); border-radius: var(--radius-sm); padding: 10px 14px; box-shadow: var(--pop-shadow); word-break: break-word; }
.toast.ok { border-left-color: var(--ok); }
.toast.error { border-left-color: var(--danger); }
</style>
