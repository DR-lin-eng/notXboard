<script setup lang="ts">
// Create and edit share one form; the protocol is fixed once a node exists.
import { computed, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import ProtocolSettingsEditor from '@/components/ProtocolSettingsEditor.vue'
import { api, errorMessage } from '@/lib/api'
import { TRUST_LEVEL_OPTIONS } from '@/lib/nodeConstants'
import {
  loadProtocolOptions, protocolDisplayName, protocolMap, randomNodePort, settingsForEditor,
  type ProtocolOption, type Settings,
} from '@/lib/nodeProtocol'
import { fieldErrors, locationInputValue, nodeLocations, nodeStatus, parseLocation, serializeLocation } from '@/lib/nodes'
import { useAsync } from '@/lib/useAsync'
import { useSession } from '@/stores/session'
import { useUi } from '@/stores/ui'

const route = useRoute()
const router = useRouter()
const session = useSession()
const ui = useUi()
const nodeId = computed(() => (route.params.id ? String(route.params.id) : ''))
const editing = computed(() => Boolean(nodeId.value))

const protocols = ref<ProtocolOption[]>([])
const node = ref<any>(null)
const settings = ref<Settings>({})
const saving = ref(false)
const formError = ref('')
const errors = ref<Record<string, string>>({})
const portLinked = ref(true)

const port = randomNodePort()
const form = reactive({
  name: '', protocol: 'vmess', location: '', min_trust: 0, host: '',
  port: port as number | '', service_port: port as number | '',
  traffic_limit_gb: 0, traffic_multiplier: 1, cross_node_ip_limit: 0, device_limit: 0, connection_limit: 0,
  speed_limit_up: 0, speed_limit_down: 0, concurrent_ip_limit: 0,
  tcping_host: '', tcping_port: '' as number | '', tcping_interval_seconds: 60, tcping_timeout_ms: 3000,
  tcping_alert_after_seconds: 300, tcping_recover_after_seconds: 120,
})

const { loading, error, reload } = useAsync(async () => {
  protocols.value = await loadProtocolOptions()
  if (!editing.value) {
    form.protocol = protocols.value[0]?.value || 'vmess'
    settings.value = settingsForEditor(form.protocol)
    return true
  }
  const data = await api<any>(`/api/v1/user/server-nodes/${encodeURIComponent(nodeId.value)}`)
  node.value = data || {}
  Object.assign(form, {
    name: data?.name || '',
    protocol: String(data?.protocol || ''),
    location: locationInputValue(data?.location_code, data?.location_name),
    min_trust: Number(data?.access_control?.min_trust_level ?? 0),
    host: data?.host || '',
    port: data?.port || 443,
    service_port: data?.service_port ?? '',
    traffic_limit_gb: data?.traffic_limit ? Number(data.traffic_limit) / 1024 / 1024 : 0,
    traffic_multiplier: Number(data?.traffic_multiplier || 1) || 1,
    cross_node_ip_limit: Number(data?.cross_node_ip_limit || 0),
    device_limit: Number(data?.device_limit || 0),
    connection_limit: Number(data?.connection_limit || 0),
    speed_limit_up: Number(data?.speed_limit_up || 0),
    speed_limit_down: Number(data?.speed_limit_down || 0),
    concurrent_ip_limit: Number(data?.concurrent_ip_limit || 0),
    tcping_host: data?.tcping_host || '',
    tcping_port: data?.tcping_port || '',
    tcping_interval_seconds: Number(data?.tcping_interval_seconds || 60),
    tcping_timeout_ms: Number(data?.tcping_timeout_ms || 3000),
    tcping_alert_after_seconds: Number(data?.tcping_alert_after_seconds || 300),
    tcping_recover_after_seconds: Number(data?.tcping_recover_after_seconds || 120),
  })
  portLinked.value = (data?.service_port ?? null) === null || Number(data.service_port) === Number(data.port)
  settings.value = settingsForEditor(form.protocol, data?.settings ?? null)
  return true
})

function onProtocolChange() {
  settings.value = settingsForEditor(form.protocol)
}
async function resetSettings() {
  if (!(await ui.confirm('重置协议参数？', { message: '当前协议的参数将恢复为推荐值。', confirmText: '重置' }))) return
  settings.value = settingsForEditor(form.protocol)
}
watch([portLinked, () => form.port], () => {
  if (portLinked.value) form.service_port = form.port
})

const err = (...keys: string[]) => keys.map((key) => errors.value[key]).find(Boolean) || ''

async function save() {
  formError.value = ''
  errors.value = {}
  const location = parseLocation(form.location)
  if (!location?.location_code || !location.location_name) {
    errors.value = { location_code: '请选择有效的节点落地地区' }
    formError.value = '请选择有效的节点落地地区'
    return
  }
  const accessPort = Number(form.port)
  const servicePort = Number(form.service_port)
  const tcpingPort = Number(form.tcping_port)
  const limitGb = Number(form.traffic_limit_gb) || 0
  const payload: Record<string, unknown> = {
    name: form.name.trim(),
    ...location,
    host: form.host.trim(),
    port: accessPort,
    service_port: portLinked.value ? accessPort : (Number.isFinite(servicePort) && servicePort > 0 ? servicePort : null),
    settings: settings.value,
    traffic_limit: limitGb > 0 ? Math.floor(limitGb * 1024 * 1024) : 0,
    traffic_multiplier: Number(form.traffic_multiplier) || 1,
    device_limit: Number(form.device_limit) || 0,
    connection_limit: Number(form.connection_limit) || 0,
    speed_limit_up: Number(form.speed_limit_up) || 0,
    speed_limit_down: Number(form.speed_limit_down) || 0,
    cross_node_ip_limit: Number(form.cross_node_ip_limit) || 0,
    tcping_host: form.tcping_host.trim() || null,
    tcping_port: Number.isFinite(tcpingPort) && tcpingPort > 0 ? tcpingPort : null,
    tcping_interval_seconds: Number(form.tcping_interval_seconds) || 60,
    tcping_timeout_ms: Number(form.tcping_timeout_ms) || 3000,
    tcping_alert_after_seconds: Number(form.tcping_alert_after_seconds) || 300,
    tcping_recover_after_seconds: Number(form.tcping_recover_after_seconds) || 120,
    access_control: { min_trust_level: Number(form.min_trust) || 0 },
  }
  if (!editing.value) payload.protocol = form.protocol
  if (session.isSuperAdmin) payload.concurrent_ip_limit = Number(form.concurrent_ip_limit) || 0

  saving.value = true
  try {
    if (editing.value) {
      await api(`/api/v1/user/server-nodes/${encodeURIComponent(nodeId.value)}`, { method: 'PUT', body: payload })
      ui.ok('节点已保存')
    } else {
      await api('/api/v1/user/server-nodes', { method: 'POST', body: payload })
      ui.ok('节点已创建')
    }
    router.push('/nodes')
  } catch (e) {
    errors.value = fieldErrors(e)
    formError.value = Object.values(errors.value)[0] || errorMessage(e, editing.value ? '保存失败' : '创建失败')
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="page narrow">
    <div class="page-head">
      <div>
        <h2>{{ editing ? `编辑节点：${node?.name || nodeId}` : '新增节点' }}</h2>
        <p>常用参数已预设；地区必选，其余大多数配置折叠在高级设置里。</p>
      </div>
      <div class="actions"><RouterLink class="btn" to="/nodes"><ArrowLeft />返回</RouterLink></div>
    </div>

    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <form class="col" novalidate @submit.prevent="save">
        <div v-if="formError" class="notice error" role="alert">{{ formError }}</div>

        <div class="card">
          <div class="card-head">
            <h3>基本信息</h3><span class="sp" />
            <span v-if="editing" class="badge" :class="nodeStatus(node).tone">{{ nodeStatus(node).label }}</span>
          </div>
          <div class="card-body form-grid">
            <div class="field">
              <label for="n-name">节点名称</label>
              <input id="n-name" v-model="form.name" class="input" placeholder="例如：香港 01" :aria-invalid="Boolean(err('name'))">
              <span v-if="err('name')" class="err">{{ err('name') }}</span>
            </div>
            <div class="field">
              <label for="n-protocol">协议</label>
              <input v-if="editing" id="n-protocol" class="input" disabled :value="`${protocolDisplayName(form.protocol, protocolMap(protocols))} (${form.protocol})`">
              <select v-else id="n-protocol" v-model="form.protocol" class="select" :aria-invalid="Boolean(err('protocol'))" @change="onProtocolChange">
                <option v-for="item in protocols" :key="item.value" :value="item.value">{{ item.label }}</option>
              </select>
              <span v-if="err('protocol')" class="err">{{ err('protocol') }}</span>
            </div>
            <div class="field">
              <label for="n-location">落地国家 / 地区</label>
              <input id="n-location" v-model="form.location" class="input" list="node-locations" placeholder="必选，可搜索，例如 HK | Hong Kong 香港" :aria-invalid="Boolean(err('location_code', 'location_name'))">
              <datalist id="node-locations"><option v-for="item in nodeLocations" :key="item.code" :value="serializeLocation(item)" /></datalist>
              <span v-if="err('location_code', 'location_name')" class="err">{{ err('location_code', 'location_name') }}</span>
            </div>
            <div class="field">
              <label for="n-trust">最低用户等级</label>
              <select id="n-trust" v-model.number="form.min_trust" class="select">
                <option v-for="level in TRUST_LEVEL_OPTIONS" :key="level" :value="level">等级 {{ level }}</option>
              </select>
              <span v-if="err('access_control', 'access_control.min_trust_level')" class="err">{{ err('access_control', 'access_control.min_trust_level') }}</span>
            </div>
            <div class="field">
              <label for="n-host">节点域名 / IP</label>
              <input id="n-host" v-model.lazy="form.host" class="input" placeholder="例如 example.com 或 1.2.3.4" :aria-invalid="Boolean(err('host'))">
              <span v-if="err('host')" class="err">{{ err('host') }}</span>
            </div>
            <div class="field">
              <label for="n-port">用户访问端口</label>
              <input id="n-port" v-model.number="form.port" class="input" type="number" min="1" max="65535" :aria-invalid="Boolean(err('port'))">
              <span v-if="err('port')" class="err">{{ err('port') }}</span>
            </div>
            <div class="field">
              <label for="n-service-port">服务端口</label>
              <input id="n-service-port" v-model.number="form.service_port" class="input" type="number" min="1" max="65535" :readonly="portLinked" placeholder="例如 8443" :aria-invalid="Boolean(err('service_port'))">
              <label class="check small"><input v-model="portLinked" type="checkbox">服务端口跟随访问端口</label>
              <span v-if="err('service_port')" class="err">{{ err('service_port') }}</span>
            </div>
            <div class="field">
              <label for="n-traffic">节点总流量上限（GB，0 = 不限）</label>
              <input id="n-traffic" v-model.number="form.traffic_limit_gb" class="input" type="number" min="0" step="0.01" :aria-invalid="Boolean(err('traffic_limit'))">
              <span v-if="err('traffic_limit')" class="err">{{ err('traffic_limit') }}</span>
            </div>
          </div>
        </div>

        <div class="card">
          <div class="card-head">
            <h3>协议参数</h3><span class="sub">可视化编辑，不需要手填 JSON</span><span class="sp" />
            <button type="button" class="btn sm" @click="resetSettings">恢复推荐参数</button>
          </div>
          <div class="card-body">
            <div v-if="err('settings')" class="notice error">{{ err('settings') }}</div>
            <ProtocolSettingsEditor v-model="settings" :protocol="form.protocol" :host="form.host" />
          </div>
        </div>

        <details class="fold">
          <summary>高级设置（限速 / 设备 / TCPing 参数）</summary>
          <div>
            <div class="form-grid">
              <div class="field"><label for="n-mult">套餐扣费倍率</label><input id="n-mult" v-model.number="form.traffic_multiplier" class="input" type="number" min="0.1" max="100" step="0.1"><span v-if="err('traffic_multiplier')" class="err">{{ err('traffic_multiplier') }}</span></div>
              <div class="field"><label for="n-cross">同一 IP 跨节点限制</label><input id="n-cross" v-model.number="form.cross_node_ip_limit" class="input" type="number" min="0"><span v-if="err('cross_node_ip_limit')" class="err">{{ err('cross_node_ip_limit') }}</span></div>
              <div class="field"><label for="n-device">节点设备限制（0 = 不限）</label><input id="n-device" v-model.number="form.device_limit" class="input" type="number" min="0"><span v-if="err('device_limit')" class="err">{{ err('device_limit') }}</span></div>
              <div class="field"><label for="n-conn">节点连接数限制（0 = 不限）</label><input id="n-conn" v-model.number="form.connection_limit" class="input" type="number" min="0"><span v-if="err('connection_limit')" class="err">{{ err('connection_limit') }}</span></div>
              <div class="field"><label for="n-up">上传限速（Mbps，0 = 不限）</label><input id="n-up" v-model.number="form.speed_limit_up" class="input" type="number" min="0"><span v-if="err('speed_limit_up')" class="err">{{ err('speed_limit_up') }}</span></div>
              <div class="field"><label for="n-down">下载限速（Mbps，0 = 不限）</label><input id="n-down" v-model.number="form.speed_limit_down" class="input" type="number" min="0"><span v-if="err('speed_limit_down')" class="err">{{ err('speed_limit_down') }}</span></div>
              <div class="field">
                <label for="n-cip">同用户多 IP 并发上限</label>
                <input id="n-cip" v-model.number="form.concurrent_ip_limit" class="input" type="number" min="0" :disabled="!session.isSuperAdmin">
                <span v-if="!session.isSuperAdmin" class="hint">仅站长可修改。</span>
                <span v-if="err('concurrent_ip_limit')" class="err">{{ err('concurrent_ip_limit') }}</span>
              </div>
              <div class="field"><label for="n-th">TCPing 探测主机</label><input id="n-th" v-model="form.tcping_host" class="input" placeholder="留空则跟随节点域名/IP"><span v-if="err('tcping_host')" class="err">{{ err('tcping_host') }}</span></div>
              <div class="field"><label for="n-tp">TCPing 探测端口</label><input id="n-tp" v-model.number="form.tcping_port" class="input" type="number" min="1" max="65535" placeholder="留空则跟随访问端口"><span class="hint">UDP 协议留空则关闭探测。</span><span v-if="err('tcping_port')" class="err">{{ err('tcping_port') }}</span></div>
              <div class="field"><label for="n-ti">TCPing 间隔（秒）</label><input id="n-ti" v-model.number="form.tcping_interval_seconds" class="input" type="number" min="15" max="3600"><span v-if="err('tcping_interval_seconds')" class="err">{{ err('tcping_interval_seconds') }}</span></div>
              <div class="field"><label for="n-tt">TCPing 超时（ms）</label><input id="n-tt" v-model.number="form.tcping_timeout_ms" class="input" type="number" min="500" max="60000"><span v-if="err('tcping_timeout_ms')" class="err">{{ err('tcping_timeout_ms') }}</span></div>
              <div class="field"><label for="n-ta">连续不可达多久告警（秒）</label><input id="n-ta" v-model.number="form.tcping_alert_after_seconds" class="input" type="number" min="60" max="86400"><span v-if="err('tcping_alert_after_seconds')" class="err">{{ err('tcping_alert_after_seconds') }}</span></div>
              <div class="field"><label for="n-tr">恢复稳定多久解除告警（秒）</label><input id="n-tr" v-model.number="form.tcping_recover_after_seconds" class="input" type="number" min="30" max="86400"><span v-if="err('tcping_recover_after_seconds')" class="err">{{ err('tcping_recover_after_seconds') }}</span></div>
            </div>
            <p class="muted small mt-sm">TCPing 由独立 Agent 探测，系统统一开启，不支持单独关闭。Agent 掉线不会影响节点转发，只会影响监控数据。</p>
          </div>
        </details>

        <div class="row">
          <span class="sp" />
          <RouterLink class="btn" to="/nodes">取消</RouterLink>
          <button type="submit" class="btn primary" :disabled="saving">{{ saving ? '提交中…' : editing ? '保存' : '创建节点' }}</button>
        </div>
      </form>
    </AsyncState>
  </div>
</template>
