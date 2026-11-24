<script setup lang="ts">
defineProps<{ items: any[] }>()
</script>

<template>
  <div v-if="!items.length" class="empty">暂无证据。</div>
  <div v-else class="col">
    <div v-for="(item, index) in items" :key="index" class="notice">
      <div class="muted small">
        {{ item.role }}<template v-if="item.is_mine"> · 我</template><template v-else-if="item.user_id"> · 用户 #{{ item.user_id }}</template> · {{ item.created_at }}
      </div>
      <div class="body">{{ item.content }}</div>
    </div>
  </div>
</template>

<style scoped>
.body { white-space: pre-wrap; word-break: break-word; margin-top: 4px; }
</style>
