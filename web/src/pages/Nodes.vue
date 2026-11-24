<script setup lang="ts">
// Nodes hub: Xboard-Node machines, owned nodes, shared nodes and TCPing agents.
import { computed, reactive, ref } from 'vue'
import { Cpu, Plus, Server } from 'lucide-vue-next'
import AsyncState from '@/components/AsyncState.vue'
import CopyField from '@/components/CopyField.vue'
import Modal from '@/components/Modal.vue'
import UsageBar from '@/components/UsageBar.vue'
import { api, errorMessage, request } from '@/lib/api'
import { asArray, copyText, formatBytes, formatLatencyMs, formatRelative, formatTrafficKb, formatTs, maskToken } from '@/lib/format'
import { CN_PROVINCE_OPTIONS } from '@/lib/nodeConstants'
import { isDeployableProtocol, loadProtocolOptions, protocolDisplayName, protocolMap } from '@/lib/nodeProtocol'
import { agentLocations, machineStatus, nodeStatus, parseLocation, serializeLocation, tcpingStatus } from '@/lib/nodes'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

const ui = useUi()
const action = useAction()

const { data, loading, error, reload } = useAsync(async () => {
  const partial = { agentError: '', machineError: '' }
  const [owned, accessible, protocols, agents, machines] = await Promise.all([
    api<any>('/api/v1/user/server-nodes'),
    api<any>('/api/v1/user/accessible-nodes'),
    loadProtocolOptions(),
    api<any>('/api/v1/user/tcping/agents').catch((e) => { partial.agentError = errorMessage(e, '加载 TCPing Agent 失败'); return [] }),
    api<any>('/api/v1/user/server-machines').catch((e) => { partial.machineError = errorMessage(e, '加载机器失败'); return [] }),
  ])
  return {
    owned: asArray(owned), accessible: asArray(accessible), names: protocolMap(protocols),
    agents: asArray(agents), machines: asArray(machines), ...partial,
  }
})

const deployable = computed(() => (data.value?.owned || []).filter((node: any) => isDeployableProtocol(node.protocol)))
const nodeName = (id: number) => data.value?.owned.find((node: any) => Number(node.id) === Number(id))?.name || `#${id}`
const percent = (pair: any) => (Number(pair?.total) > 0 ? (Number(pair.used || 0) / Number(pair.total)) * 100 : 0)

async function run(fn: () => Promise<unknown>, done = '') {
  await action.run(async () => {
    await fn()
    if (done) ui.ok(done)
    await reload(true)
  }, ui.error)
}

// ---- deploy command dialog (machines and single nodes) ----
const deploy = ref<{ title: string; command: string; panelUrl: string; note: string } | null>(null)
async function showCommand(title: string, path: string, note: string) {
  await action.run(async () => {
    const res = await api<any>(path, { method: 'POST', body: {} })
    const command = String(res?.command || '').trim()
    if (!command) throw new Error('未生成部署命令')
    deploy.value = { title, command, panelUrl: res?.panel_url || location.origin, note }
  }, ui.error)
}

async function copyDeploy() {
  if (deploy.value && (await copyText(deploy.value.command))) ui.ok('部署命令已复制')
  else ui.error('复制失败，请手动选择复制')
}

// ---- machines ----
const machineForm = reactive({ open: false, id: 0, name: '', notes: '', error: '', saving: false })
function openMachine(machine?: any) {
  Object.assign(machineForm, { open: true, id: machine?.id || 0, name: machine?.name || '', notes: machine?.notes || '', error: '', saving: false })
}
async function saveMachine() {
  if (!machineForm.name.trim()) {
    machineForm.error = '请填写机器名称'
    return
  }
  machineForm.saving = true
  machineForm.error = ''
  try {
    const body = { name: machineForm.name.trim(), notes: machineForm.notes.trim() }
    if (machineForm.id) await api(`/api/v1/user/server-machines/${machineForm.id}`, { method: 'PUT', body })
    else await api('/api/v1/user/server-machines', { method: 'POST', body })
    machineForm.open = false
    ui.ok(machineForm.id ? '机器已更新' : '机器已创建')
    await reload(true)
  } catch (e) {
    machineForm.error = errorMessage(e, '保存失败')
  } finally {
    machineForm.saving = false
  }
}
const bind = reactive({ open: false, machine: null as any, selected: [] as number[], error: '', saving: false })
function openBind(machine: any) {
  Object.assign(bind, { open: true, machine, selected: asArray(machine.node_ids).map(Number), error: '', saving: false })
}
async function saveBind() {
  bind.saving = true
  bind.error = ''
  try {
    await api(`/api/v1/user/server-machines/${bind.machine.id}/nodes`, { method: 'POST', body: { node_ids: bind.selected } })
    bind.open = false
    ui.ok('绑定已更新，节点变更会实时推送到机器')
    await reload(true)
  } catch (e) {
    bind.error = errorMessage(e, '绑定失败')
  } finally {
    bind.saving = false
  }
}
async function rotateMachine(machine: any) {
  if (!(await ui.confirm('轮换这台机器的凭据？', { message: '旧实例会立即失效，请随后在服务器上执行新的部署命令。', confirmText: '轮换凭据', danger: true }))) return
  await showCommand(`机器 ${machine.name}`, `/api/v1/user/server-machines/${machine.id}/deploy-command/rotate-token`, '凭据已轮换，旧实例已失效。请在目标服务器上以 root 执行新命令。')
}
async function toggleMachine(machine: any) {
  await run(() => api(`/api/v1/user/server-machines/${machine.id}`, { method: 'PUT', body: { is_active: !machine.is_active } }), machine.is_active ? '机器已停用' : '机器已启用')
}
async function deleteMachine(machine: any) {
  if (!(await ui.confirm(`删除机器「${machine.name}」？`, { message: '已绑定的节点会解绑，节点本身不会删除。', confirmText: '删除', danger: true }))) return
  await run(() => api(`/api/v1/user/server-machines/${machine.id}`, { method: 'DELETE' }), '机器已删除')
}

// ---- nodes ----
async function deployNode(node: any) {
  await action.run(async () => {
    const res = await request<any>(`/api/v1/user/server-nodes/${node.id}/deploy`, { method: 'POST' })
    let commandData: any = {}
    try {
      commandData = await api<any>(`/api/v1/user/server-nodes/${node.id}/deploy-command`, { method: 'POST', body: {} })
    } catch {
      // the deploy itself succeeded; the command can be fetched again later
    }
    await reload(true)
    deploy.value = {
      title: `节点 #${node.id} ${node.name || ''}`,
      command: String(commandData?.command || ''),
      panelUrl: commandData?.panel_url || location.origin,
      note: res?.message || '在目标服务器上以 root 执行一次即可。',
    }
  }, ui.error)
}
async function rotateNode(node: any) {
  if (!(await ui.confirm('轮换这个节点的 Xboard-Node 凭据？', { message: '旧实例会立即失效，请随后执行新的部署命令。', confirmText: '轮换凭据', danger: true }))) return
  await showCommand(`节点 #${node.id} ${node.name || ''}`, `/api/v1/user/server-nodes/${node.id}/deploy-command/rotate-token`, '凭据已轮换，旧实例已失效。')
  await reload(true)
}
async function deleteNode(node: any) {
  if (!(await ui.confirm(`删除节点「${node.name || node.id}」？`, { message: '删除后无法恢复。', confirmText: '删除', danger: true }))) return
  await run(() => api(`/api/v1/user/server-nodes/${node.id}`, { method: 'DELETE' }), '节点已删除')
}

// ---- TCPing agents ----
const agentForm = reactive({ open: false, name: '', location: '', province: '', error: '', saving: false })
const agentIsCn = computed(() => parseLocation(agentForm.location, agentLocations)?.location_code === 'CN' || agentForm.location.trim().toUpperCase().startsWith('CN'))
function openAgent() {
  Object.assign(agentForm, { open: true, name: `agent-${Date.now()}`, location: '', province: '', error: '', saving: false })
}
async function saveAgent() {
  const location = parseLocation(agentForm.location, agentLocations)
  if (!agentForm.name.trim()) agentForm.error = '请输入 Agent 名称'
  else if (!location?.location_code || !location.location_name) agentForm.error = '请选择有效的 TCPing Agent 归属地区'
  else if (location.location_code === 'CN' && !agentForm.province.trim()) agentForm.error = '中国大陆地区的探针必须填写省份归属'
  else agentForm.error = ''
  if (agentForm.error || !location) return
  agentForm.saving = true
  try {
    await api('/api/v1/user/tcping/agents', {
      method: 'POST',
      body: { name: agentForm.name.trim(), ...location, location_province: location.location_code === 'CN' ? agentForm.province.trim() : null },
    })
    agentForm.open = false
    ui.ok('Agent 已创建')
    await reload(true)
  } catch (e) {
    agentForm.error = errorMessage(e, '创建失败')
  } finally {
    agentForm.saving = false
  }
}
async function copyAgentInstall(agent: any) {
  await action.run(async () => {
    const res = await api<any>(`/api/v1/user/tcping/agents/${agent.id}/install-command`)
    const command = String(res?.command || '').trim()
    if (!command) throw new Error('未生成安装命令')
    if (!(await copyText(command))) throw new Error('复制失败，请重试')
    ui.ok('安装命令已复制。请在独立探测机上执行，不要和节点混装。')
  }, ui.error)
}
async function rotateAgent(agent: any) {
  if (!(await ui.confirm('轮换这个 Agent 的 Token？', { message: '轮换后旧实例需要重新部署。', confirmText: '轮换', danger: true }))) return
  await run(() => api(`/api/v1/user/tcping/agents/${agent.id}/rotate-token`, { method: 'POST', body: {} }), 'Token 已轮换')
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div><h2>节点与机器</h2><p>管理 Xboard-Node 机器、你的节点，以及他人分享给你的节点。</p></div>
      <div class="actions">
        <button type="button" class="btn" @click="openMachine()"><Cpu />新增机器</button>
        <RouterLink class="btn primary" to="/nodes/new"><Plus />新增节点</RouterLink>
      </div>
    </div>

    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <template v-if="data">
        <!-- machines -->
        <h3 class="section-title">Xboard-Node 机器</h3>
        <p class="muted small">机器模式：一台服务器运行一个 Xboard-Node，同时承载绑定到它的多个自有节点，节点增减会实时推送。</p>
        <div v-if="data.machineError" class="notice error mt-sm">{{ data.machineError }}</div>
        <div v-if="!data.machines.length" class="card mt-sm"><div class="empty"><Server />还没有机器。单节点部署可直接使用下方节点的「部署」。</div></div>
        <div v-else class="grid g2 mt-sm">
          <article v-for="machine in data.machines" :key="machine.id" class="card machine" :data-machine-id="machine.id">
            <div class="card-head">
              <h3 class="truncate">{{ machine.name || `#${machine.id}` }}</h3>
              <span class="badge" :class="machineStatus(machine).tone"><i class="dot" />{{ machineStatus(machine).label }}</span>
              <span class="sp" />
              <span class="muted small nowrap">{{ machine.last_seen_at ? `最后上报 ${formatRelative(machine.last_seen_at)}` : '从未上报' }}</span>
            </div>
            <div class="card-body col">
              <p v-if="machine.notes" class="muted small">{{ machine.notes }}</p>
              <div v-if="machine.load_status" class="load">
                <div><span class="muted small">CPU</span><UsageBar :percent="Number(machine.load_status.cpu || 0)" /><span class="small">{{ Number(machine.load_status.cpu || 0).toFixed(1) }}%</span></div>
                <div><span class="muted small">内存</span><UsageBar :percent="percent(machine.load_status.mem)" /><span class="small">{{ formatBytes(machine.load_status.mem?.used) }} / {{ formatBytes(machine.load_status.mem?.total) }}</span></div>
                <div v-if="Number(machine.load_status.disk?.total) > 0"><span class="muted small">磁盘</span><UsageBar :percent="percent(machine.load_status.disk)" /><span class="small">{{ formatBytes(machine.load_status.disk?.used) }} / {{ formatBytes(machine.load_status.disk?.total) }}</span></div>
              </div>
              <div v-else class="muted small">暂无负载数据。机器连上面板后会定期上报。</div>
              <div>
                <div class="muted small">绑定节点</div>
                <div v-if="asArray(machine.node_ids).length" class="chips mt-sm">
                  <span v-for="id in machine.node_ids" :key="id" class="chip">#{{ id }} {{ nodeName(id) }}</span>
                </div>
                <div v-else class="muted small mt-sm">未绑定</div>
              </div>
            </div>
            <div class="card-foot">
              <button type="button" class="btn sm" :disabled="action.busy.value" @click="openBind(machine)">绑定节点</button>
              <button type="button" class="btn sm" :disabled="action.busy.value" @click="showCommand(`机器 ${machine.name}`, `/api/v1/user/server-machines/${machine.id}/deploy-command`, '请在目标服务器上以 root 执行。')">部署命令</button>
              <button type="button" class="btn sm" :disabled="action.busy.value" @click="rotateMachine(machine)">轮换凭据</button>
              <button type="button" class="btn sm" :disabled="action.busy.value" @click="openMachine(machine)">编辑</button>
              <button type="button" class="btn sm" :disabled="action.busy.value" @click="toggleMachine(machine)">{{ machine.is_active ? '停用' : '启用' }}</button>
              <button type="button" class="btn sm danger" :disabled="action.busy.value" @click="deleteMachine(machine)">删除</button>
            </div>
          </article>
        </div>

        <!-- owned nodes -->
        <h3 class="section-title mt">我的节点</h3>
        <div class="card mt-sm">
          <div v-if="!data.owned.length" class="empty">你还没有创建节点。<RouterLink class="link" to="/nodes/new">新增节点</RouterLink></div>
          <div v-else class="table-wrap">
            <table class="table">
              <thead><tr><th>节点</th><th>协议 / 入口</th><th>状态</th><th>节点流量</th><th>倍率</th><th>TCPing</th><th /></tr></thead>
              <tbody>
                <tr v-for="node in data.owned" :key="node.id">
                  <td>
                    <div class="strong">{{ node.name || '-' }}</div>
                    <div class="muted small">#{{ node.id }} · {{ node.location_name || '未设置地区' }}</div>
                  </td>
                  <td>
                    <span class="badge">{{ protocolDisplayName(node.protocol, data.names) }}</span>
                    <div class="muted small mono">{{ node.host || '-' }}</div>
                    <div class="muted small">访问 {{ node.port ?? '-' }} / 服务 {{ node.effective_service_port ?? node.service_port ?? node.port ?? '-' }}</div>
                  </td>
                  <td>
                    <span class="badge" :class="nodeStatus(node).tone">{{ nodeStatus(node).label }}</span>
                    <div class="muted small">在线用户 {{ node.online_users ?? 0 }}</div>
                  </td>
                  <td class="traffic">
                    <div class="small">{{ formatTrafficKb(node.traffic_used) }} / {{ Number(node.traffic_limit) > 0 ? formatTrafficKb(node.traffic_limit) : '无限' }}</div>
                    <UsageBar v-if="Number(node.traffic_limit) > 0" :percent="Number(node.traffic_usage_percentage || 0)" />
                  </td>
                  <td class="nowrap">×{{ Number(node.traffic_multiplier || 1).toFixed(2) }}</td>
                  <td>
                    <span class="badge" :class="tcpingStatus(node.tcping_status, Boolean(node.tcping_enabled)).tone">{{ tcpingStatus(node.tcping_status, Boolean(node.tcping_enabled)).label }}</span>
                    <div v-if="String(node.tcping_status).toLowerCase() !== 'unsupported'" class="muted small">
                      {{ node.tcping_last_latency_ms == null ? '暂无延迟数据' : `最近 ${formatLatencyMs(node.tcping_last_latency_ms)}` }}
                    </div>
                  </td>
                  <td>
                    <div class="actions">
                      <RouterLink class="btn sm" :to="`/nodes/${node.id}/monitor`">监控</RouterLink>
                      <RouterLink class="btn sm" :to="`/nodes/edit/${node.id}`">编辑</RouterLink>
                      <button v-if="isDeployableProtocol(node.protocol)" type="button" class="btn sm" :disabled="action.busy.value" @click="deployNode(node)">部署</button>
                      <button v-else type="button" class="btn sm" disabled title="该协议暂不支持 Xboard-Node 一键部署">部署不可用</button>
                      <button v-if="isDeployableProtocol(node.protocol) && node.v2bx_token_configured" type="button" class="btn sm" :disabled="action.busy.value" @click="rotateNode(node)">轮换凭据</button>
                      <RouterLink class="btn sm" :to="`/nodes/${node.id}/access`">分享/权限</RouterLink>
                      <RouterLink class="btn sm" :to="`/nodes/${node.id}/audit-rules`">审计规则</RouterLink>
                      <button type="button" class="btn sm danger" :disabled="action.busy.value" @click="deleteNode(node)">删除</button>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <!-- shared nodes -->
        <h3 class="section-title mt">我可用的节点（含他人分享）</h3>
        <p class="muted small">只展示已授权且可下发给你的节点，未上线节点不会出现在订阅里。</p>
        <div class="card mt-sm">
          <div v-if="!data.accessible.length" class="empty">当前没有可访问的共享节点。</div>
          <div v-else class="table-wrap">
            <table class="table">
              <thead><tr><th>节点</th><th>所有者</th><th>协议</th><th>状态</th><th>倍率</th><th>TCPing</th><th /></tr></thead>
              <tbody>
                <tr v-for="node in data.accessible" :key="node.id">
                  <td><div class="strong">{{ node.name || '-' }}</div><div class="muted small">#{{ node.id }} · {{ node.location_name || '未设置地区' }}</div></td>
                  <td class="muted">{{ node.owner_email || node.user_id || '-' }}</td>
                  <td><span class="badge">{{ protocolDisplayName(node.protocol, data.names) }}</span></td>
                  <td><span v-if="node.free_quota_exhausted" class="badge warn" title="本月在该节点的免费额度已用完，下月 1 日恢复">本月额度已用完</span><span v-else class="badge" :class="nodeStatus(node).tone">{{ nodeStatus(node).label }}</span></td>
                  <td class="nowrap">×{{ Number(node.traffic_multiplier || 1).toFixed(2) }}</td>
                  <td><span class="badge" :class="tcpingStatus(node.tcping_status, Boolean(node.tcping_enabled)).tone">{{ tcpingStatus(node.tcping_status, Boolean(node.tcping_enabled)).label }}</span></td>
                  <td><div class="actions"><RouterLink class="btn sm" :to="`/nodes/${node.id}/monitor`">监控</RouterLink></div></td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <!-- agents -->
        <div class="row mt">
          <h3 class="section-title">TCPing Agent</h3><span class="sp" />
          <button type="button" class="btn sm" @click="openAgent"><Plus />新增 Agent</button>
        </div>
        <p class="muted small">独立部署在探测服务器上，不和 Xboard-Node 节点混装。创建后复制一行命令即可接入。</p>
        <div v-if="data.agentError" class="notice error mt-sm">{{ data.agentError }}</div>
        <div class="card mt-sm">
          <div v-if="!data.agents.length" class="empty">还没有 TCPing Agent。建议至少部署 1 个，节点监控和被墙提醒才会开始工作。</div>
          <div v-else class="table-wrap">
            <table class="table">
              <thead><tr><th>名称</th><th>归属</th><th>Token</th><th>最后心跳</th><th class="hide-m">最后同步</th><th /></tr></thead>
              <tbody>
                <tr v-for="agent in data.agents" :key="agent.id">
                  <td><div class="strong">{{ agent.name || '-' }}</div><div class="muted small">#{{ agent.id }}</div></td>
                  <td class="muted">{{ agent.location_display || '未设置' }}</td>
                  <td class="mono small">{{ maskToken(agent.token) }}</td>
                  <td class="muted nowrap">{{ formatTs(agent.last_heartbeat_at) }}</td>
                  <td class="muted nowrap hide-m">{{ formatTs(agent.last_sync_at) }}</td>
                  <td>
                    <div class="actions">
                      <button type="button" class="btn sm" :disabled="action.busy.value" @click="copyAgentInstall(agent)">复制安装命令</button>
                      <button type="button" class="btn sm" :disabled="action.busy.value" @click="rotateAgent(agent)">轮换 Token</button>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </template>
    </AsyncState>

    <Modal v-if="deploy" :title="`部署信息 · ${deploy.title}`" wide @close="deploy = null">
      <div class="col">
        <div class="notice">{{ deploy.note }}</div>
        <div class="field"><label>本站链接</label><CopyField :value="deploy.panelUrl" label="本站链接" /></div>
        <div class="field">
          <label>一次性部署命令</label>
          <pre v-if="deploy.command" class="codebox" data-testid="deploy-command">{{ deploy.command }}</pre>
          <div v-else class="notice warn">未能生成部署命令，请稍后在节点列表重试。</div>
        </div>
      </div>
      <template #footer>
        <button v-if="deploy.command" type="button" class="btn" @click="copyDeploy">复制命令</button>
        <button type="button" class="btn primary" @click="deploy = null">我知道了</button>
      </template>
    </Modal>

    <Modal v-if="machineForm.open" :title="machineForm.id ? '编辑机器' : '新增机器'" @close="machineForm.open = false">
      <form id="machine-form" class="col" @submit.prevent="saveMachine">
        <div v-if="machineForm.error" class="notice error" role="alert">{{ machineForm.error }}</div>
        <div class="field"><label for="m-name">机器名称</label><input id="m-name" v-model="machineForm.name" class="input" maxlength="255" placeholder="例如 hk-box-01" required></div>
        <div class="field"><label for="m-notes">备注（可选）</label><textarea id="m-notes" v-model="machineForm.notes" class="textarea" maxlength="2000" /></div>
      </form>
      <template #footer>
        <button type="button" class="btn" @click="machineForm.open = false">取消</button>
        <button type="submit" form="machine-form" class="btn primary" :disabled="machineForm.saving">{{ machineForm.saving ? '保存中…' : '保存' }}</button>
      </template>
    </Modal>

    <Modal v-if="bind.open" :title="`绑定节点 · ${bind.machine?.name}`" @close="bind.open = false">
      <div class="col">
        <div v-if="bind.error" class="notice error" role="alert">{{ bind.error }}</div>
        <p class="muted small">勾选要由这台机器承载的节点；全部取消表示解绑全部。只能绑定支持 Xboard-Node 的自有节点。</p>
        <div v-if="!deployable.length" class="empty">没有可绑定的节点。</div>
        <label v-for="node in deployable" :key="node.id" class="check">
          <input v-model="bind.selected" type="checkbox" :value="Number(node.id)">
          <span>#{{ node.id }} {{ node.name }} <span class="muted small">{{ protocolDisplayName(node.protocol, data?.names) }}</span></span>
        </label>
      </div>
      <template #footer>
        <button type="button" class="btn" @click="bind.open = false">取消</button>
        <button type="button" class="btn primary" :disabled="bind.saving" @click="saveBind">{{ bind.saving ? '保存中…' : '保存绑定' }}</button>
      </template>
    </Modal>

    <Modal v-if="agentForm.open" title="新增 TCPing Agent" @close="agentForm.open = false">
      <form id="agent-form" class="col" @submit.prevent="saveAgent">
        <div v-if="agentForm.error" class="notice error" role="alert">{{ agentForm.error }}</div>
        <p class="muted small">Agent 用于 TCPing 探测，必须填写归属地区；中国大陆需填写到省份。</p>
        <div class="field"><label for="a-name">Agent 名称</label><input id="a-name" v-model="agentForm.name" class="input" placeholder="例如 agent-hk-01"></div>
        <div class="field">
          <label for="a-loc">归属国家 / 地区</label>
          <input id="a-loc" v-model="agentForm.location" class="input" list="agent-locations" placeholder="可搜索，例如 CN | China Mainland 中国大陆">
          <datalist id="agent-locations"><option v-for="item in agentLocations" :key="item.code" :value="serializeLocation(item)" /></datalist>
        </div>
        <div v-if="agentIsCn" class="field">
          <label for="a-prov">中国大陆省份归属</label>
          <input id="a-prov" v-model="agentForm.province" class="input" list="cn-provinces" placeholder="例如 广东 / 上海">
          <datalist id="cn-provinces"><option v-for="name in CN_PROVINCE_OPTIONS" :key="name" :value="name" /></datalist>
        </div>
      </form>
      <template #footer>
        <button type="button" class="btn" @click="agentForm.open = false">取消</button>
        <button type="submit" form="agent-form" class="btn primary" :disabled="agentForm.saving">{{ agentForm.saving ? '创建中…' : '创建' }}</button>
      </template>
    </Modal>
  </div>
</template>

<style scoped>
.machine .card-foot { flex-wrap: wrap; }
.load { display: grid; gap: 8px; }
.load > div { display: grid; grid-template-columns: 36px minmax(60px, 1fr) auto; gap: 10px; align-items: center; }
.traffic { min-width: 150px; }
td .actions { flex-wrap: wrap; justify-content: flex-end; max-width: 330px; margin-left: auto; }
</style>
