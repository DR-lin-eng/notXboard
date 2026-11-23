<script setup lang="ts">
import { computed, ref } from 'vue'
import { Search } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import { api } from '@/lib/api'
import { asArray } from '@/lib/format'
import { useAsync } from '@/lib/useAsync'

const keyword = ref('')
const { data: items, loading, error, reload } = useAsync(async () => asArray(await api('/api/v1/user/knowledge/fetch')))
const filtered = computed(() => {
  const q = keyword.value.trim().toLowerCase()
  const list = items.value || []
  return q ? list.filter((k: any) => `${k.title} ${k.category} ${k.content}`.toLowerCase().includes(q)) : list
})
</script>

<template>
  <div class="page narrow">
    <div class="page-head">
      <div><h2>知识库</h2><p>使用教程和常见问题。</p></div>
      <div class="actions"><label class="search"><Search /><input v-model="keyword" class="input" type="search" placeholder="搜索文章" aria-label="搜索文章"></label></div>
    </div>
    <AsyncState :loading="loading" :error="error" :empty="!filtered.length" :empty-title="keyword ? '没有匹配的文章' : '知识库还没有内容'" @retry="reload()">
      <div class="col">
        <details v-for="k in filtered" :key="k.id ?? k.title" class="fold">
          <summary>{{ k.title || '-' }} <span v-if="k.category" class="badge">{{ k.category }}</span></summary>
          <div class="prose">{{ k.content }}</div>
        </details>
      </div>
    </AsyncState>
  </div>
</template>

<style scoped>
.search { position: relative; display: block; }
.search svg { position: absolute; left: 10px; top: 50%; transform: translateY(-50%); width: 15px; height: 15px; color: var(--muted); }
.search .input { padding-left: 32px; min-width: 220px; }
</style>
