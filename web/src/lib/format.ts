export function formatTs(ts: unknown): string {
  const value = Number(ts)
  if (!value) return '-'
  const d = new Date(value < 1e12 ? value * 1000 : value)
  return Number.isNaN(d.getTime()) ? String(ts) : d.toLocaleString()
}

export function formatDate(ts: unknown): string {
  const value = Number(ts)
  if (!value) return '-'
  const d = new Date(value < 1e12 ? value * 1000 : value)
  return Number.isNaN(d.getTime()) ? String(ts) : d.toLocaleDateString()
}

export function formatRelative(ts: unknown): string {
  const value = Number(ts)
  if (!value) return '-'
  const seconds = Math.floor(Date.now() / 1000) - (value < 1e12 ? value : Math.floor(value / 1000))
  if (seconds < 0) return formatTs(ts)
  if (seconds < 60) return `${seconds} 秒前`
  if (seconds < 3600) return `${Math.floor(seconds / 60)} 分钟前`
  if (seconds < 86400) return `${Math.floor(seconds / 3600)} 小时前`
  if (seconds < 86400 * 30) return `${Math.floor(seconds / 86400)} 天前`
  return formatDate(ts)
}

/** Traffic stored in KB. */
export function formatTrafficKb(kb: unknown): string {
  let value = Number(kb) || 0
  if (!Number.isFinite(value) || value < 0) value = 0
  const gb = value / 1024 / 1024
  const digits = gb >= 100 ? 0 : gb >= 10 ? 1 : 2
  return `${gb.toFixed(digits)} GB`
}

export function formatTrafficDisplay(kb: unknown, unlimited = false): string {
  return unlimited ? '无限' : formatTrafficKb(kb)
}

export function formatBytes(bytes: unknown): string {
  let num = Number(bytes)
  if (!Number.isFinite(num) || num <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']
  let idx = 0
  while (num >= 1024 && idx < units.length - 1) {
    num /= 1024
    idx += 1
  }
  const digits = num >= 100 ? 0 : num >= 10 ? 1 : 2
  return `${num.toFixed(digits)} ${units[idx]}`
}

export function formatCompactNumber(value: unknown): string {
  const num = Number(value)
  if (!Number.isFinite(num)) return '0'
  return new Intl.NumberFormat('zh-CN', { notation: 'compact', maximumFractionDigits: 1 }).format(num)
}

export function formatMoneyCent(value: unknown): string {
  const amount = Number(value)
  if (!Number.isFinite(amount)) return '¥0'
  return `¥${(amount / 100).toLocaleString(undefined, {
    minimumFractionDigits: Math.abs(amount % 100) > 0 ? 2 : 0,
    maximumFractionDigits: 2,
  })}`
}

export function formatRatePerSecond(value: unknown): string {
  let num = Number(value)
  if (!Number.isFinite(num) || num <= 0) return '0 B/s'
  const units = ['B/s', 'KB/s', 'MB/s', 'GB/s']
  let idx = 0
  while (num >= 1024 && idx < units.length - 1) {
    num /= 1024
    idx += 1
  }
  const digits = num >= 100 ? 0 : num >= 10 ? 1 : 2
  return `${num.toFixed(digits)} ${units[idx]}`
}

export function formatDurationShort(seconds: unknown): string {
  const total = Math.max(0, Math.floor(Number(seconds) || 0))
  if (total < 60) return `${total}s`
  if (total < 3600) return `${Math.floor(total / 60)}m ${total % 60}s`
  if (total < 86400) return `${Math.floor(total / 3600)}h ${Math.floor((total % 3600) / 60)}m`
  return `${Math.floor(total / 86400)}d ${Math.floor((total % 86400) / 3600)}h`
}

export function formatPercent(value: unknown): string {
  const num = Number(value)
  if (!Number.isFinite(num)) return '-'
  if (Math.abs(num) >= 100) return `${Math.round(num)}%`
  if (Math.abs(num) >= 10) return `${num.toFixed(1)}%`
  return `${num.toFixed(2)}%`
}

export function formatLatencyMs(value: unknown): string {
  const num = Number(value)
  if (!Number.isFinite(num) || num <= 0) return '-'
  return `${num >= 100 ? Math.round(num) : num.toFixed(1)} ms`
}

export function maskToken(value: unknown): string {
  const text = String(value || '')
  if (text.length <= 12) return text
  return `${text.slice(0, 6)}…${text.slice(-4)}`
}

export function truthy(value: unknown): boolean {
  return value === true || value === 1 || value === '1' || value === 'true'
}

export function asArray<T = any>(value: unknown): T[] {
  return Array.isArray(value) ? (value as T[]) : []
}

export async function copyText(text: string): Promise<boolean> {
  try {
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(text)
      return true
    }
  } catch {
    // fall through to the legacy path
  }
  const el = document.createElement('textarea')
  el.value = text
  el.setAttribute('readonly', '')
  el.style.position = 'fixed'
  el.style.opacity = '0'
  document.body.appendChild(el)
  el.select()
  let ok = false
  try {
    ok = document.execCommand('copy')
  } catch {
    ok = false
  }
  document.body.removeChild(el)
  return ok
}

/** Accepts unix seconds, numeric strings or date strings. */
export function formatAnyTs(value: unknown, fallback = '-'): string {
  if (value === null || value === undefined || value === '') return fallback
  const text = String(value).trim()
  if (/^\d+$/.test(text)) return formatTs(Number(text))
  const parsed = new Date(text)
  return Number.isNaN(parsed.getTime()) ? text || fallback : parsed.toLocaleString()
}
