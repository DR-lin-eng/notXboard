// Protocol settings model for the visual node editor: templates, mode presets and path helpers.
import { api } from './api'
import {
  DEFAULT_NODE_PROTOCOL_OPTIONS,
  NODE_PROTOCOL_EDITOR_META,
  NODE_PROTOCOL_LABELS,
  NODE_PROTOCOL_SETTINGS_TEMPLATES,
  NODE_SETTING_ENUM_OPTIONS,
  NODE_SETTING_PATH_LABELS,
  NODE_SETTING_SEGMENT_LABELS,
  XBOARD_NODE_DEPLOYABLE_PROTOCOLS,
} from './nodeConstants'

export type Settings = Record<string, any>
export interface ProtocolOption {
  value: string
  label: string
}

let optionsCache: ProtocolOption[] | null = null
let remoteTemplates: Record<string, Settings> = {}

export function isPlainObject(value: unknown): value is Settings {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value)
}
export function deepClone<T>(value: T): T {
  return value === undefined ? value : JSON.parse(JSON.stringify(value))
}

export function protocolDisplayName(protocol: unknown, map: Record<string, string> = {}): string {
  const key = String(protocol || '').toLowerCase()
  return map[key] || NODE_PROTOCOL_LABELS[key] || (key ? key.toUpperCase() : '-')
}
export function isDeployableProtocol(protocol: unknown): boolean {
  return XBOARD_NODE_DEPLOYABLE_PROTOCOLS.has(String(protocol || '').toLowerCase())
}

export async function loadProtocolOptions(force = false): Promise<ProtocolOption[]> {
  if (!force && optionsCache?.length) return optionsCache
  try {
    const rows = await api<any[]>('/api/v1/user/server-nodes/protocols')
    const templates: Record<string, Settings> = {}
    const normalized: ProtocolOption[] = []
    for (const row of Array.isArray(rows) ? rows : []) {
      const value = String(row?.value || '').toLowerCase()
      if (!value) continue
      if (isPlainObject(row?.template)) templates[value] = row.template
      normalized.push({ value, label: String(row?.label || protocolDisplayName(value)) })
    }
    if (normalized.length) {
      optionsCache = normalized
      remoteTemplates = templates
      return normalized
    }
  } catch {
    // fall back to the built-in list
  }
  optionsCache = DEFAULT_NODE_PROTOCOL_OPTIONS
  remoteTemplates = {}
  return optionsCache
}

export function protocolMap(options: ProtocolOption[]): Record<string, string> {
  return Object.fromEntries(options.map((item) => [item.value, item.label]))
}

function mergePreferDefined(primary: any, fallback: any): any {
  if (primary === null || primary === undefined) return deepClone(fallback)
  if (Array.isArray(primary)) return deepClone(primary)
  if (typeof primary !== 'object') return primary
  const fallbackObj = isPlainObject(fallback) ? fallback : {}
  const out: Settings = {}
  for (const key of new Set([...Object.keys(fallbackObj), ...Object.keys(primary)])) {
    const p = primary[key]
    const f = fallbackObj[key]
    if (p === null || p === undefined) out[key] = deepClone(f)
    else if (typeof p === 'object' && !Array.isArray(p)) out[key] = mergePreferDefined(p, f)
    else out[key] = deepClone(p)
  }
  return out
}

export function protocolTemplate(protocol: unknown): Settings {
  const key = String(protocol || '').toLowerCase()
  const local = NODE_PROTOCOL_SETTINGS_TEMPLATES[key] || {}
  const remote = remoteTemplates[key]
  return isPlainObject(remote) ? mergePreferDefined(remote, local) : deepClone(local)
}

export function splitPath(path: string): string[] {
  return String(path || '').split('.').map((item) => item.trim()).filter(Boolean)
}
export function getByPath(source: any, path: string): any {
  let cursor = source
  for (const segment of splitPath(path)) {
    if (!cursor || typeof cursor !== 'object' || !(segment in cursor)) return undefined
    cursor = cursor[segment]
  }
  return cursor
}
export function setByPath(target: Settings, path: string, value: unknown): void {
  const segments = splitPath(path)
  let cursor = target
  segments.forEach((segment, index) => {
    if (index === segments.length - 1) {
      cursor[segment] = value
      return
    }
    if (!isPlainObject(cursor[segment])) cursor[segment] = {}
    cursor = cursor[segment]
  })
}
function deleteByPath(target: Settings, path: string): void {
  const segments = splitPath(path)
  if (!segments.length || !isPlainObject(target)) return
  const walk = (cursor: any, index: number) => {
    const key = segments[index]
    if (!cursor || typeof cursor !== 'object' || !(key in cursor)) return
    if (index === segments.length - 1) {
      delete cursor[key]
      return
    }
    walk(cursor[key], index + 1)
    if (isPlainObject(cursor[key]) && !Object.keys(cursor[key]).length) delete cursor[key]
  }
  walk(target, 0)
}

export function leafPaths(settings: unknown, prefix = ''): string[] {
  if (!isPlainObject(settings)) return prefix ? [prefix] : []
  return Object.entries(settings).flatMap(([key, value]) => {
    const path = prefix ? `${prefix}.${key}` : key
    return isPlainObject(value) ? leafPaths(value, path) : [path]
  })
}

export function settingLabel(path: string): string {
  const normalized = String(path || '').trim().toLowerCase()
  if (NODE_SETTING_PATH_LABELS[normalized]) return NODE_SETTING_PATH_LABELS[normalized]
  const parts = splitPath(normalized).map((item) => NODE_SETTING_SEGMENT_LABELS[item] || item.toUpperCase())
  return parts.length ? parts.join(' / ') : '参数'
}

export interface EnumOption {
  value: unknown
  label: string
}
export function settingEnumOptions(path: string, current: unknown): EnumOption[] | null {
  const normalized = String(path || '').trim().toLowerCase()
  const last = normalized.split('.').pop() || normalized
  const options: EnumOption[] | undefined = NODE_SETTING_ENUM_OPTIONS[normalized] || NODE_SETTING_ENUM_OPTIONS[last]
  if (!Array.isArray(options) || !options.length) return null
  const text = current === null || current === undefined ? '' : String(current)
  if (text === '' || options.some((item) => String(item?.value ?? '') === text)) return options
  return [...options, { value: current, label: `自定义值 (${text})` }]
}

function replacePlaceholders(value: any, host: string): any {
  if (Array.isArray(value)) return value.map((item) => replacePlaceholders(item, host))
  if (isPlainObject(value)) {
    return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, replacePlaceholders(item, host)]))
  }
  if (value === '__HOST__') return host.trim() || null
  return value
}

export function editorMode(protocol: unknown, settings: Settings): string {
  if (String(protocol || '').toLowerCase() !== 'vless') return ''
  return Number(getByPath(settings, 'tls') || 0) === 2 ? 'reality' : 'tls'
}
export function modePresets(protocol: unknown): Array<{ key: string; label: string }> {
  const presets = NODE_PROTOCOL_EDITOR_META[String(protocol || '').toLowerCase()]?.modePresets
  return presets ? Object.entries<any>(presets).map(([key, preset]) => ({ key, label: preset?.label || key })) : []
}
export function applyModePreset(protocol: unknown, settings: Settings, mode: string, host = ''): Settings {
  const meta = NODE_PROTOCOL_EDITOR_META[String(protocol || '').toLowerCase()]
  const preset = meta?.modePresets?.[String(mode || '').toLowerCase()]
  const next = isPlainObject(settings) ? deepClone(settings) : {}
  if (!preset) return next
  for (const path of preset.managedPaths || []) deleteByPath(next, path)
  return mergePreferDefined(replacePlaceholders(deepClone(preset.settings || {}), host), next)
}

/** Settings shown in the editor: saved values layered over the protocol template. */
export function settingsForEditor(protocol: unknown, current: Settings | null = null): Settings {
  const key = String(protocol || '').toLowerCase()
  const template = protocolTemplate(protocol)
  if (isPlainObject(current)) return mergePreferDefined(current, template)
  const defaultMode = NODE_PROTOCOL_EDITOR_META[key]?.defaultMode
  return defaultMode ? applyModePreset(protocol, template, defaultMode) : template
}

export function recommendedPaths(protocol: unknown, settings: Settings): string[] {
  const meta = NODE_PROTOCOL_EDITOR_META[String(protocol || '').toLowerCase()] || {}
  const mode = editorMode(protocol, settings)
  const paths: string[] = mode && meta.recommendedPathsByMode ? meta.recommendedPathsByMode[mode] || [] : meta.recommendedPaths || []
  return Array.from(new Set(paths.map((item) => String(item || '').trim()).filter(Boolean)))
}
export function advancedPaths(protocol: unknown, settings: Settings): string[] {
  const rest = deepClone(settings)
  for (const path of recommendedPaths(protocol, settings)) deleteByPath(rest, path)
  return leafPaths(rest)
}

export function randomNodePort(lo = 50000, hi = 65535): number {
  const span = hi - lo + 1
  const bytes = new Uint32Array(1)
  crypto.getRandomValues(bytes)
  return lo + (bytes[0] % span)
}
