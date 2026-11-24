<script setup lang="ts">
defineProps<{ loading: boolean; error?: string; empty?: boolean; emptyTitle?: string; emptyText?: string }>()
defineEmits<{ retry: [] }>()
</script>

<template>
  <div v-if="loading" class="card-body col" aria-busy="true">
    <div class="skeleton" style="width: 40%" />
    <div class="skeleton" style="width: 85%" />
    <div class="skeleton" style="width: 65%" />
  </div>
  <div v-else-if="error" class="card-body">
    <div class="notice error row">
      <span class="sp">{{ error }}</span>
      <button class="btn sm" type="button" @click="$emit('retry')">重试</button>
    </div>
  </div>
  <div v-else-if="empty" class="empty">
    <div class="t">{{ emptyTitle || '暂无数据' }}</div>
    <div v-if="emptyText">{{ emptyText }}</div>
    <div v-if="$slots.action" class="mt-sm"><slot name="action" /></div>
  </div>
  <slot v-else />
</template>
