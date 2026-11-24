// Node, location and TCPing presentation helpers shared by the node pages.
import { NODE_LOCATION_OPTIONS, TCPING_AGENT_LOCATION_OPTIONS } from './nodeConstants'

export interface Tone {
  label: string
  tone: string
}

export function nodeStatus(node: any): Tone {
  const online = String(node?.online_status || '').toLowerCase()
  const raw = node?.is_online
  if (online === 'online' || raw === true || raw === 1 || raw === '1') return { label: '在线', tone: 'ok' }
  if (online === 'offline' || raw === false || raw === 0 || raw === '0') return { label: '离线', tone: 'danger' }
  const status = String(node?.status || 'unknown').toLowerCase()
  if (status === 'active') return { label: '可用', tone: 'ok' }
  if (status === 'deploying') return { label: '部署中', tone: 'warn' }
  if (status === 'maintenance') return { label: '维护中', tone: 'warn' }
  if (status === 'inactive') return { label: '未启用', tone: '' }
  return { label: String(node?.status || '未知'), tone: '' }
}

export function tcpingStatus(status: unknown, enabled = true): Tone {
  const key = String(status || 'unknown').toLowerCase()
  if (key === 'unsupported') return { label: '不支持 UDP 探测', tone: '' }
  if (!enabled) return { label: '未开启', tone: '' }
  if (key === 'online') return { label: '在线', tone: 'ok' }
  if (key === 'offline') return { label: '离线', tone: 'danger' }
  return { label: '待探测', tone: 'warn' }
}

export function machineStatus(machine: any): Tone {
  if (!machine?.is_active) return { label: '已停用', tone: '' }
  if (machine?.is_online) return { label: '在线', tone: 'ok' }
  if (!machine?.last_seen_at) return { label: '未连接', tone: 'warn' }
  return { label: '离线', tone: 'danger' }
}

export interface LocationOption {
  code: string
  name: string
}
export interface LocationSelection {
  location_code: string
  location_name: string
}
export const nodeLocations: readonly LocationOption[] = NODE_LOCATION_OPTIONS
export const agentLocations: readonly LocationOption[] = TCPING_AGENT_LOCATION_OPTIONS

export function serializeLocation(option: LocationOption): string {
  return `${option.code} | ${option.name}`
}
export function locationInputValue(code: unknown, name: unknown): string {
  const c = String(code || '').trim().toUpperCase()
  const n = String(name || '').trim()
  if (!c && !n) return ''
  const matched = NODE_LOCATION_OPTIONS.find((item) => item.code === c)
  if (matched) return serializeLocation(matched)
  return c ? `${c} | ${n || c}` : n
}
export function parseLocation(raw: unknown, options: readonly LocationOption[] = nodeLocations): LocationSelection | null {
  const text = String(raw || '').trim()
  if (!text) return null
  const normalized = text.toLowerCase()
  const exact = options.find((item) =>
    serializeLocation(item).toLowerCase() === normalized || item.code.toLowerCase() === normalized || item.name.toLowerCase() === normalized)
  if (exact) return { location_code: exact.code, location_name: exact.name }
  const parts = text.split('|').map((item) => item.trim()).filter(Boolean)
  if (parts.length >= 2) return { location_code: parts[0].toUpperCase(), location_name: parts.slice(1).join(' | ') }
  return null
}

/** Maps a 422 response's `errors` to one message per field key. */
export function fieldErrors(error: any): Record<string, string> {
  const errors = error?.status === 422 ? error?.payload?.errors : null
  const out: Record<string, string> = {}
  if (!errors || typeof errors !== 'object') return out
  for (const [key, raw] of Object.entries(errors)) {
    const first = (Array.isArray(raw) ? raw : [raw]).map((item) => String(item ?? '').trim()).find(Boolean)
    if (first) out[key] = first
  }
  return out
}
