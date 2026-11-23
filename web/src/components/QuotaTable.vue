<script setup lang="ts">
import UsageBar from './UsageBar.vue'
import { formatTrafficDisplay, formatTrafficKb, formatTs } from '@/lib/format'

defineProps<{ items: any[] }>()
</script>

<template>
  <div v-if="!items.length" class="empty">暂无生效中的套餐额度。</div>
  <div v-else class="table-wrap">
    <table class="table">
      <thead>
        <tr><th>套餐</th><th>周期</th><th>适用节点</th><th>有效期</th><th>已用 / 总量</th><th>剩余</th><th>进度</th></tr>
      </thead>
      <tbody>
        <tr v-for="(item, index) in items" :key="index">
          <td class="strong">{{ item.plan_name || `套餐 #${item.plan_id}` }}</td>
          <td class="muted">{{ item.period || '-' }}</td>
          <td class="muted">{{ Array.isArray(item.node_ids) && item.node_ids.length ? item.node_ids.join(', ') : '全部节点' }}</td>
          <td class="muted nowrap">{{ item.expired_at ? formatTs(item.expired_at) : '长期有效' }}</td>
          <td class="muted nowrap">
            {{ formatTrafficKb(item.used_traffic_kb) }} / {{ formatTrafficDisplay(item.traffic_allowance_kb, Boolean(item.is_unlimited_traffic)) }}
          </td>
          <td class="muted nowrap">{{ formatTrafficDisplay(item.remaining_traffic_kb, Boolean(item.is_unlimited_traffic)) }}</td>
          <td>
            <span v-if="item.is_unlimited_traffic" class="muted">无限</span>
            <UsageBar v-else :percent="item.usage_percent || 0" />
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
