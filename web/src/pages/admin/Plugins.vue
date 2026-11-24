<script setup lang="ts">
import { computed, reactive } from 'vue'
import AsyncState from '@/components/AsyncState.vue'
import Modal from '@/components/Modal.vue'
import { errorMessage } from '@/lib/api'
import { adminList, adminPost } from '@/lib/admin/list'
import { useAction, useAsync } from '@/lib/useAsync'
import { useUi } from '@/stores/ui'

type PluginAction = 'install' | 'uninstall' | 'enable' | 'disable' | 'upgrade' | 'delete'
const ACTION_TEXT: Record<PluginAction, string> = {
  install: '已安装', uninstall: '已卸载', enable: '已启用', disable: '已禁用', upgrade: '已升级', delete: '已删除',
}
const TYPE_TEXT: Record<string, string> = { feature: '功能', payment: '支付方式' }

const ui = useUi()
const action = useAction()
const { data, loading, error, reload } = useAsync(async () => (await adminList('plugin/getPlugins')).items)

async function run(plugin: any, name: PluginAction) {
  if (name === 'uninstall' || name === 'delete') {
    const ok = await ui.confirm(name === 'delete' ? `删除扩展 ${plugin.name || plugin.code}？` : `卸载扩展 ${plugin.name || plugin.code}？`, {
      message: name === 'delete' ? '扩展的文件和记录都会被移除，无法恢复。' : '扩展会停止工作，已保存的设置会被清除。',
      confirmText: name === 'delete' ? '删除' : '卸载', danger: true,
    })
    if (!ok) return
  }
  await action.run(async () => {
    await adminPost(`plugin/${name}`, { code: plugin.code })
    ui.ok(`${plugin.name || plugin.code} ${ACTION_TEXT[name]}`)
    await reload(true)
  }, ui.error)
}

// Plugin config is a map of key -> { type, label, description, value }.
const cfg = reactive({ open: false, plugin: null as any, values: {} as Record<string, any>, error: '', saving: false })
const cfgFields = computed(() =>
  Object.entries((cfg.plugin?.config || {}) as Record<string, any>).map(([key, field]) => ({
    key, type: String(field?.type || 'string'), label: String(field?.label || key), description: String(field?.description || ''),
  })),
)
const isSecret = (key: string) => /key|secret|token|private|password/i.test(key)
function openConfig(plugin: any) {
  const values: Record<string, any> = {}
  for (const [key, field] of Object.entries((plugin.config || {}) as Record<string, any>)) {
    const value = field?.value
    values[key] = field?.type === 'boolean' ? value === true || value === 1 || value === '1' || value === 'true' : value ?? ''
  }
  Object.assign(cfg, { open: true, plugin, values, error: '' })
}
async function saveConfig() {
  cfg.error = ''
  cfg.saving = true
  try {
    await adminPost('plugin/config', { code: cfg.plugin.code, config: { ...cfg.values } })
    cfg.open = false
    ui.ok('扩展设置已保存')
    await reload(true)
  } catch (e) {
    cfg.error = errorMessage(e, '保存失败')
  } finally {
    cfg.saving = false
  }
}
</script>

<template>
  <div class="page">
    <div class="page-head"><div><h2>扩展</h2><p>安装、启用和配置支付与功能扩展。</p></div></div>
    <AsyncState :loading="loading" :error="error" @retry="reload()">
      <div class="card">
        <div v-if="!data?.length" class="empty">没有可用的扩展。</div>
        <div v-else class="table-wrap">
          <table class="table">
            <thead><tr><th>扩展</th><th>版本</th><th>类型</th><th>状态</th><th class="right">操作</th></tr></thead>
            <tbody>
              <tr v-for="plugin in data" :key="plugin.code">
                <td><b>{{ plugin.name || plugin.code }}</b><span class="muted small mono"> {{ plugin.code }}</span><div v-if="plugin.description" class="muted small">{{ plugin.description }}</div></td>
                <td class="tnum">{{ plugin.version || '-' }}<span v-if="plugin.need_upgrade" class="badge warn" style="margin-left: 6px">可升级</span></td>
                <td>{{ TYPE_TEXT[plugin.type] || plugin.type || '-' }}</td>
                <td>
                  <span v-if="!plugin.is_installed" class="badge">未安装</span>
                  <span v-else class="badge" :class="plugin.is_enabled ? 'ok' : 'warn'">{{ plugin.is_enabled ? '已启用' : '已禁用' }}</span>
                </td>
                <td class="right nowrap">
                  <button v-if="!plugin.is_installed" type="button" class="btn sm primary" :disabled="action.busy.value" @click="run(plugin, 'install')">安装</button>
                  <template v-else>
                    <button v-if="Object.keys(plugin.config || {}).length" type="button" class="btn sm" @click="openConfig(plugin)">设置</button>
                    <button v-if="plugin.need_upgrade" type="button" class="btn sm" :disabled="action.busy.value" @click="run(plugin, 'upgrade')">升级</button>
                    <button type="button" class="btn sm" :disabled="action.busy.value" @click="run(plugin, plugin.is_enabled ? 'disable' : 'enable')">{{ plugin.is_enabled ? '禁用' : '启用' }}</button>
                    <button type="button" class="btn sm" :disabled="action.busy.value" @click="run(plugin, 'uninstall')">卸载</button>
                  </template>
                  <button v-if="plugin.can_be_deleted" type="button" class="btn sm danger" :disabled="action.busy.value" @click="run(plugin, 'delete')">删除</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </AsyncState>

    <Modal v-if="cfg.open" :title="`${cfg.plugin?.name || cfg.plugin?.code} 设置`" @close="cfg.open = false">
      <form id="admin-plugin-form" class="form-grid" @submit.prevent="saveConfig">
        <div v-if="cfg.error" class="notice error full" role="alert">{{ cfg.error }}</div>
        <div v-for="field in cfgFields" :key="field.key" class="field full">
          <label v-if="field.type === 'boolean'" class="switch"><input v-model="cfg.values[field.key]" type="checkbox" role="switch"><span>{{ field.label }}</span></label>
          <template v-else>
            <label :for="`apc-${field.key}`">{{ field.label }}</label>
            <textarea v-if="field.type === 'text'" :id="`apc-${field.key}`" v-model="cfg.values[field.key]" class="textarea" rows="4" />
            <input v-else :id="`apc-${field.key}`" v-model="cfg.values[field.key]" class="input" :type="field.type === 'number' || field.type === 'integer' ? 'number' : isSecret(field.key) ? 'password' : 'text'" :step="field.type === 'number' ? 'any' : undefined" autocomplete="off">
          </template>
          <span v-if="field.description" class="hint">{{ field.description }}</span>
        </div>
      </form>
      <template #footer>
        <button type="button" class="btn" @click="cfg.open = false">取消</button>
        <button type="submit" form="admin-plugin-form" class="btn primary" :disabled="cfg.saving">{{ cfg.saving ? '保存中…' : '保存' }}</button>
      </template>
    </Modal>
  </div>
</template>
