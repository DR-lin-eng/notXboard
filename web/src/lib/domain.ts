// Shared business vocabulary: price periods, order states and payment helpers.
export const PLAN_PRICE_FIELDS = [
  { key: 'month_price', label: '月付' },
  { key: 'quarter_price', label: '季付' },
  { key: 'half_year_price', label: '半年付' },
  { key: 'year_price', label: '年付' },
  { key: 'two_year_price', label: '两年付' },
  { key: 'three_year_price', label: '三年付' },
  { key: 'onetime_price', label: '一次性' },
  { key: 'reset_price', label: '重置流量' },
] as const

export function periodLabel(key: unknown): string {
  return PLAN_PRICE_FIELDS.find((item) => item.key === key)?.label || String(key || '-')
}

export interface PeriodOption {
  key: string
  label: string
  price: number
}
export function planPeriodOptions(plan: Record<string, unknown> | null | undefined): PeriodOption[] {
  const options: PeriodOption[] = []
  for (const field of PLAN_PRICE_FIELDS) {
    const raw = plan?.[field.key]
    if (raw === null || raw === undefined || raw === '') continue
    const price = Number(raw)
    if (Number.isFinite(price) && price >= 0) options.push({ key: field.key, label: field.label, price })
  }
  return options
}

export function ownerName(item: any): string {
  return String(
    item?.owner_display_name || item?.owner?.display_name || item?.owner?.linux_do_name ||
    item?.owner?.linux_do_username || item?.owner?.email || '未知',
  )
}

const ORDER_STATUS: Record<string, { label: string; tone: string }> = {
  '0': { label: '待支付', tone: 'warn' },
  '1': { label: '开通中', tone: 'info' },
  '2': { label: '已取消', tone: '' },
  '3': { label: '已完成', tone: 'ok' },
  '4': { label: '已折抵', tone: '' },
}
export function orderStatus(status: unknown): { label: string; tone: string } {
  return ORDER_STATUS[String(status)] || { label: String(status ?? '-'), tone: '' }
}

export function normalizeHttpUrl(raw: unknown): string {
  const text = String(raw || '').trim()
  if (!text) return ''
  try {
    const parsed = new URL(text, location.origin)
    return parsed.protocol === 'http:' || parsed.protocol === 'https:' ? parsed.toString() : ''
  } catch {
    return ''
  }
}

/** Rebuilds a gateway-returned POST form from its hidden inputs only, so no markup from the response is executed. */
export function submitGatewayForm(raw: string): void {
  const source = new DOMParser().parseFromString(String(raw || ''), 'text/html').querySelector('form')
  const action = normalizeHttpUrl(source?.getAttribute('action') || '')
  const method = String(source?.getAttribute('method') || '').trim().toLowerCase()
  if (!source || !action || method !== 'post') throw new Error('支付网关返回了无效表单')
  const form = document.createElement('form')
  form.method = 'post'
  form.action = action
  form.hidden = true
  source.querySelectorAll('input[type="hidden"]').forEach((input) => {
    const name = String(input.getAttribute('name') || '').trim()
    if (!name || name.length > 256) return
    const el = document.createElement('input')
    el.type = 'hidden'
    el.name = name
    el.value = input.getAttribute('value') || ''
    form.append(el)
  })
  document.body.append(form)
  HTMLFormElement.prototype.submit.call(form)
}

export function parseIdList(raw: unknown): number[] {
  return String(raw || '')
    .split(/[\n,，\s]+/)
    .map((item) => Number(item.trim()))
    .filter((value) => Number.isFinite(value) && value > 0)
}

const REFUND_STATUS: Record<string, { label: string; tone: string }> = {
  pending: { label: '待处理', tone: 'warn' },
  voting: { label: '投票中', tone: 'info' },
  approved: { label: '已同意', tone: 'ok' },
  refunded: { label: '已退款', tone: 'ok' },
  denied: { label: '已拒绝', tone: 'danger' },
}
export function refundStatus(status: unknown): { label: string; tone: string } {
  return REFUND_STATUS[String(status)] || { label: String(status ?? '-'), tone: '' }
}
export function voteLabel(vote: unknown): string {
  if (vote === 'approve') return '同意退款'
  if (vote === 'deny') return '拒绝退款'
  return '-'
}
