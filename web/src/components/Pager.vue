<script setup lang="ts">
defineProps<{ page: number; lastPage: number; total?: number; disabled?: boolean }>()
const emit = defineEmits<{ change: [page: number] }>()
</script>

<template>
  <div class="pager row">
    <span v-if="total != null" class="muted small">共 {{ total }} 条</span>
    <span class="sp" />
    <button type="button" class="btn sm" :disabled="disabled || page <= 1" @click="emit('change', page - 1)">上一页</button>
    <span class="muted small tnum">第 {{ page }} / {{ Math.max(1, lastPage) }} 页</span>
    <button type="button" class="btn sm" :disabled="disabled || page >= lastPage" @click="emit('change', page + 1)">下一页</button>
  </div>
</template>

<style scoped>
.pager { padding: 10px 16px; border-top: 1px solid var(--border); gap: 10px; }
</style>
