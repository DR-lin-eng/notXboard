// Schema for admin settings pages backed by `config/fetch` and `config/save`.
export type ConfigFieldType = 'bool' | 'text' | 'password' | 'number' | 'float' | 'select' | 'lines' | 'textarea' | 'readonly'

export interface ConfigField {
  /** Key inside the config group and in the save payload. */
  key: string
  label: string
  type: ConfigFieldType
  options?: { value: string; label: string }[]
  hint?: string
  placeholder?: string
  fallback?: string | number | boolean
  min?: number
  max?: number
  step?: number
  /** Config group holding this field when it differs from the page group. */
  group?: string
  /** Key inside the group when it differs from the payload key. */
  source?: string
  /** Spans the full form width. */
  full?: boolean
  /** Shown only when another field has a truthy or matching value. */
  showIf?: (values: Record<string, any>) => boolean
}
export interface ConfigSection {
  title: string
  description?: string
  fields: ConfigField[]
}
export interface ConfigPage {
  /** Group returned by config/fetch, e.g. `safe` or `site`. */
  group: string
  sections: ConfigSection[]
}

export function isOn(value: unknown): boolean {
  return value === true || value === 1 || value === '1' || value === 'true'
}

/** List settings come back as an array, a JSON array string or a comma-separated string. */
function parseList(raw: unknown): string[] {
  let items: unknown = raw
  if (typeof raw === 'string' && raw.trim().startsWith('[')) {
    try {
      items = JSON.parse(raw)
    } catch {
      items = raw
    }
  }
  const list = Array.isArray(items) ? items : String(items ?? '').split(/[\r\n,]+/)
  return list.map((item) => String(item ?? '').trim()).filter(Boolean)
}

function matchOption(field: ConfigField, raw: unknown): string | undefined {
  const wanted = String(raw ?? '').trim().toLowerCase()
  return field.options?.find((option) => option.value.toLowerCase() === wanted)?.value
}

export function readConfigValues(page: ConfigPage, config: Record<string, any>): Record<string, any> {
  const values: Record<string, any> = {}
  for (const field of page.sections.flatMap((section) => section.fields)) {
    const raw = config?.[field.group || page.group]?.[field.source || field.key]
    if (field.type === 'bool') values[field.key] = raw == null ? Boolean(field.fallback) : isOn(raw)
    else if (field.type === 'lines') values[field.key] = parseList(raw).join('\n')
    else if (field.type === 'number' || field.type === 'float') values[field.key] = raw == null || raw === '' ? field.fallback ?? '' : Number(raw)
    else if (field.type === 'select') {
      // A stored value outside the options is kept as is: the select stays empty and the field is not saved.
      values[field.key] = raw == null ? String(field.fallback ?? '') : matchOption(field, raw) ?? String(raw)
    } else values[field.key] = raw == null ? String(field.fallback ?? '') : String(raw)
  }
  return values
}

export function buildConfigPayload(page: ConfigPage, values: Record<string, any>): Record<string, any> {
  const payload: Record<string, any> = {}
  for (const field of page.sections.flatMap((section) => section.fields)) {
    if (field.type === 'readonly') continue
    const value = values[field.key]
    if (field.type === 'bool') payload[field.key] = value ? 1 : 0
    else if (field.type === 'lines') payload[field.key] = parseList(value)
    else if (field.type === 'select') {
      const option = matchOption(field, value)
      if (option !== undefined) payload[field.key] = option
    }
    else if (field.type === 'number' || field.type === 'float') {
      let num = field.type === 'number' ? parseInt(String(value), 10) : parseFloat(String(value))
      if (!Number.isFinite(num)) num = Number(field.fallback ?? 0)
      if (field.min != null) num = Math.max(field.min, num)
      if (field.max != null) num = Math.min(field.max, num)
      payload[field.key] = num
    } else payload[field.key] = String(value ?? '')
  }
  return payload
}
