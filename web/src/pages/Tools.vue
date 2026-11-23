<script setup lang="ts">
import { reactive } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import RewardList from '@/components/RewardList.vue'
import UsageBar from '@/components/UsageBar.vue'
import { api, errorMessage } from '@/lib/api'
import { periodLabel } from '@/lib/domain'
import { asArray, formatAnyTs, formatLatencyMs, formatMoneyCent, formatTrafficKb, formatTs } from '@/lib/format'
import { protocolDisplayName } from '@/lib/nodeProtocol'
import { nodeStatus, tcpingStatus } from '@/lib/nodes'
import { useAsync } from '@/lib/useAsync'

const coupon = reactive({ code: '', busy: false, error: '', data: null as any })
async function checkCoupon() {
  if (!coupon.code.trim()) return
  Object.assign(coupon, { busy: true, error: '', data: null })
  try {
    coupon.data = (await api('/api/v1/user/coupon/check', { method: 'POST', body: { code: coupon.code.trim() } })) || {}
  } catch (e) {
    coupon.error = errorMessage(e, '查询失败')
  } finally {
    coupon.busy = false
  }
}
function discount(data: any): string {
  const type = Number(data?.type || 0)
  if (type === 1) return `立减 ${formatMoneyCent(data.value)}`
  if (type === 2) return `${Number(data.value || 0)}% 折扣`
  return '未识别'
}

const gift = reactive({ code: '', busy: false, error: '', check: null as any, redeemed: null as any })
async function giftAction(kind: 'check' | 'redeem') {
  if (!gift.code.trim()) return
  Object.assign(gift, { busy: true, error: '', check: null, redeemed: null })
  try {
    const data = (await api(`/api/v1/user/gift-card/${kind}`, { method: 'POST', body: { code: gift.code.trim() } })) || {}
    if (kind === 'check') gift.check = data
    else gift.redeemed = data
  } catch (e) {
    gift.error = errorMessage(e, kind === 'check' ? '查询失败' : '兑换失败')
  } finally {
    gift.busy = false
  }
}

const traffic = useAsync(async () => asArray(await api('/api/v1/user/node-traffic')))
const logs = useAsync(async () => asArray(await api('/api/v1/user/traffic-usage-logs', { query: { days: 7, limit: 100 } })))
</script>

<template>
  <div class="page">
    <div class="page-head"><div><h2>工具</h2><p>优惠券、礼品卡和流量明细。</p></div></div>
    <div class="grid g2">
      <div class="card">
        <div class="card-head"><h3>优惠券查询</h3></div>
        <div class="card-body col">
          <form class="row" @submit.prevent="checkCoupon">
            <input v-model="coupon.code" class="input" placeholder="请输入优惠码" aria-label="优惠码">
            <button type="submit" class="btn" :disabled="coupon.busy">查询</button>
          </form>
          <div v-if="coupon.error" class="notice error">{{ coupon.error }}</div>
          <dl v-if="coupon.data" class="kv">
            <dt>名称</dt><dd>{{ coupon.data.name || '-' }}</dd>
            <dt>优惠码</dt><dd class="mono">{{ coupon.data.code || '-' }}</dd>
            <dt>优惠内容</dt><dd>{{ discount(coupon.data) }}</dd>
            <dt>开始时间</dt><dd>{{ formatAnyTs(coupon.data.started_at, '未设置') }}</dd>
            <dt>结束时间</dt><dd>{{ formatAnyTs(coupon.data.ended_at, '未设置') }}</dd>
            <dt>剩余总次数</dt><dd>{{ coupon.data.limit_use == null ? '不限' : coupon.data.limit_use }}</dd>
            <dt>每用户可用</dt><dd>{{ coupon.data.limit_use_with_user == null ? '不限' : `${coupon.data.limit_use_with_user} 次` }}</dd>
            <dt>适用套餐</dt><dd>{{ asArray(coupon.data.limit_plan_ids).join(', ') || '全部套餐' }}</dd>
            <dt>适用周期</dt><dd>{{ asArray(coupon.data.limit_period).map(periodLabel).join(' / ') || '全部周期' }}</dd>
          </dl>
        </div>
      </div>
      <div class="card">
        <div class="card-head"><h3>礼品卡</h3></div>
        <div class="card-body col">
          <form class="row" @submit.prevent="giftAction('check')">
            <input v-model="gift.code" class="input" placeholder="请输入礼品卡兑换码" aria-label="礼品卡兑换码">
            <button type="submit" class="btn" :disabled="gift.busy">查询</button>
            <button type="button" class="btn primary" :disabled="gift.busy" @click="giftAction('redeem')">兑换</button>
          </form>
          <div v-if="gift.error" class="notice error">{{ gift.error }}</div>
          <template v-if="gift.check">
            <dl class="kv">
              <dt>兑换状态</dt><dd><span class="badge" :class="gift.check.can_redeem ? 'ok' : 'warn'">{{ gift.check.can_redeem ? '可兑换' : '暂不可兑换' }}</span></dd>
              <template v-if="gift.check.reason"><dt>说明</dt><dd>{{ gift.check.reason }}</dd></template>
              <dt>礼品卡类型</dt><dd>{{ gift.check.code_info?.template?.type_name || gift.check.code_info?.template?.name || '-' }}</dd>
              <dt>卡片状态</dt><dd>{{ gift.check.code_info?.status_name || '-' }}</dd>
              <dt>过期时间</dt><dd>{{ formatAnyTs(gift.check.code_info?.expires_at, '不限') }}</dd>
              <dt>可用次数</dt><dd>{{ gift.check.code_info?.usage_count || 0 }} / {{ gift.check.code_info?.max_usage || 0 }}</dd>
            </dl>
            <div><div class="muted small mb-xs">奖励预览</div><RewardList :rewards="gift.check.reward_preview" /></div>
          </template>
          <template v-if="gift.redeemed">
            <div class="notice ok">{{ gift.redeemed.message || '兑换成功' }}</div>
            <div><div class="muted small mb-xs">本次到账</div><RewardList :rewards="gift.redeemed.rewards" /></div>
            <div v-if="gift.redeemed.invite_rewards"><div class="muted small mb-xs">邀请奖励</div><RewardList :rewards="gift.redeemed.invite_rewards" /></div>
          </template>
        </div>
      </div>
    </div>

    <div class="card mt">
      <div class="card-head"><h3>今日节点流量</h3><span class="sub">我在各节点上的用量</span></div>
      <AsyncState :loading="traffic.loading.value" :error="traffic.error.value" :empty="!traffic.data.value?.length" empty-title="今天还没有流量记录" @retry="traffic.reload()">
        <div class="table-wrap">
          <table class="table">
            <thead><tr><th>节点</th><th>状态</th><th>今日上行</th><th>今日下行</th><th>节点流量上限</th><th>TCPing</th></tr></thead>
            <tbody>
              <tr v-for="item in traffic.data.value" :key="item.node_id">
                <td>
                  <div class="strong">{{ item.node_name || `节点 #${item.node_id || '-'}` }}</div>
                  <div class="muted small">{{ item.location_name || '-' }} · {{ protocolDisplayName(item.protocol) }} · 倍率 ×{{ Number(item.traffic_multiplier || 1).toFixed(2) }}</div>
                </td>
                <td><span class="badge" :class="nodeStatus(item).tone">{{ nodeStatus(item).label }}</span></td>
                <td class="muted nowrap">{{ formatTrafficKb(item.today?.upload || 0) }}</td>
                <td class="muted nowrap">{{ formatTrafficKb(item.today?.download || 0) }}</td>
                <td class="limit">
                  <template v-if="Number(item.traffic_limit) > 0">
                    <div class="small">{{ formatTrafficKb(item.traffic_used) }} / {{ formatTrafficKb(item.traffic_limit) }}</div>
                    <UsageBar :percent="Number(item.traffic_usage_percentage || 0)" />
                  </template>
                  <span v-else class="muted">无限</span>
                </td>
                <td>
                  <span class="badge" :class="tcpingStatus(item.tcping_status, Boolean(item.tcping_enabled)).tone">{{ tcpingStatus(item.tcping_status, Boolean(item.tcping_enabled)).label }}</span>
                  <span v-if="item.tcping_last_latency_ms != null" class="muted small"> {{ formatLatencyMs(item.tcping_last_latency_ms) }}</span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </AsyncState>
    </div>

    <div class="card mt">
      <div class="card-head"><h3>流量使用记录</h3><span class="sub">最近 7 天</span></div>
      <p class="muted small pad">同时展示原始流量和套餐实际扣费流量。倍率只影响套餐消耗，不影响节点的已用总流量。</p>
      <AsyncState :loading="logs.loading.value" :error="logs.error.value" :empty="!logs.data.value?.length" empty-title="最近还没有流量使用记录" @retry="logs.reload()">
        <div class="table-wrap">
          <table class="table">
            <thead><tr><th>时间</th><th>节点</th><th>原始流量</th><th>计费流量</th><th>倍率</th><th>来源</th></tr></thead>
            <tbody>
              <tr v-for="(log, index) in logs.data.value" :key="index">
                <td class="muted nowrap">{{ formatTs(log.recorded_at) }}</td>
                <td>
                  <div>{{ log.node_name || `节点 #${log.node_id || '-'}` }}</div>
                  <div class="muted small">{{ log.location_name || '-' }} · {{ protocolDisplayName(log.protocol) }}</div>
                </td>
                <td class="muted nowrap">{{ formatTrafficKb(log.raw_traffic_kb || 0) }}</td>
                <td class="muted nowrap">{{ formatTrafficKb(log.billed_traffic_kb || 0) }}</td>
                <td><span class="badge">×{{ Number(log.multiplier_snapshot || 1).toFixed(2) }}</span></td>
                <td class="muted">{{ log.source || '-' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </AsyncState>
    </div>
  </div>
</template>

<style scoped>
.limit { min-width: 150px; }
.mb-xs { margin-bottom: 6px; }
.pad { padding: 10px 16px 0; }
</style>
