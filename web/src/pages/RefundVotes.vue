<script setup lang="ts">
import AsyncState from '@/components/AsyncState.vue'
import { api } from '@/lib/api'
import { refundStatus } from '@/lib/domain'
import { asArray } from '@/lib/format'
import { useAsync } from '@/lib/useAsync'

const { data: items, loading, error, reload } = useAsync(async () => asArray(await api('/api/v1/user/refund-votes')))
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div><h2>争议投票</h2><p>所有成员可参与投票；投票结束后系统自动裁定，站长可覆盖。</p></div>
    </div>
    <div class="card">
      <AsyncState :loading="loading" :error="error" :empty="!items?.length" empty-title="当前没有进行中的争议" @retry="reload()">
        <div class="table-wrap">
          <table class="table">
            <thead><tr><th>案件</th><th>套餐</th><th>投票结束</th><th>状态</th><th /></tr></thead>
            <tbody>
              <tr v-for="r in items" :key="r.id">
                <td>#{{ r.id }}</td>
                <td class="strong">{{ r.plan?.name || r.plan_id }}</td>
                <td class="muted nowrap">{{ r.voting_ends_at || '-' }}</td>
                <td><span class="badge" :class="refundStatus(r.status).tone">{{ refundStatus(r.status).label }}</span></td>
                <td><div class="actions"><RouterLink class="btn sm" :to="`/refund-vote/${r.id}`">进入</RouterLink></div></td>
              </tr>
            </tbody>
          </table>
        </div>
      </AsyncState>
    </div>
  </div>
</template>
