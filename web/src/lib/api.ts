// HTTP client. Storage keys are shared with the previous portal so existing sessions survive the upgrade.
const AUTH_KEYS = ['auth_data', 'PORTAL_ACCESS_TOKEN', 'Portal_access_token', 'access_token'] as const
const ADMIN_PATH_RE = /^[A-Za-z0-9][A-Za-z0-9_-]{7,63}$/

export class ApiError extends Error {
  status: number
  payload: any
  constructor(message: string, status: number, payload: unknown) {
    super(message)
    this.status = status
    this.payload = payload
  }
}

function normalizeBearer(token: string | null | undefined): string {
  const clean = String(token || '').trim()
  if (!clean) return ''
  return clean.toLowerCase().startsWith('bearer ') ? clean : `Bearer ${clean}`
}

export function readAuth(): string {
  for (const key of AUTH_KEYS) {
    const value = normalizeBearer(localStorage.getItem(key))
    if (value) return value
  }
  return ''
}

export function writeAuth(value: string): void {
  const token = normalizeBearer(value)
  for (const key of AUTH_KEYS) {
    if (token) localStorage.setItem(key, token)
    else localStorage.removeItem(key)
  }
}

let adminPath = ''
let onUnauthorized: (() => void) | null = null

export function setAdminPath(path: string | null | undefined): void {
  const value = String(path || '').trim()
  adminPath = ADMIN_PATH_RE.test(value) ? value : ''
}
export function getAdminPath(): string {
  return adminPath
}
export function setUnauthorizedHandler(handler: () => void): void {
  onUnauthorized = handler
}

export interface RequestOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'
  body?: unknown
  headers?: Record<string, string>
  query?: Record<string, string | number | boolean | null | undefined>
  signal?: AbortSignal
}

function withQuery(path: string, query?: RequestOptions['query']): string {
  if (!query) return path
  const params = new URLSearchParams()
  for (const [key, value] of Object.entries(query)) {
    if (value === undefined || value === null || value === '') continue
    params.set(key, String(value))
  }
  const text = params.toString()
  if (!text) return path
  return `${path}${path.includes('?') ? '&' : '?'}${text}`
}

export async function request<T = any>(path: string, options: RequestOptions = {}): Promise<T> {
  const headers: Record<string, string> = { 'Content-Type': 'application/json', ...(options.headers || {}) }
  const auth = readAuth()
  if (auth) headers.Authorization = auth
  if (adminPath) headers['X-Admin-Path'] = adminPath

  const res = await fetch(withQuery(path, options.query), {
    method: options.method || 'GET',
    headers,
    body: options.body !== undefined ? JSON.stringify(options.body) : undefined,
    signal: options.signal,
  })
  const text = await res.text()
  let json: any = null
  try {
    json = text ? JSON.parse(text) : null
  } catch {
    json = null
  }

  if (res.status === 401) {
    onUnauthorized?.()
    throw new ApiError('登录已失效，请重新登录', 401, json)
  }
  if (!res.ok) {
    throw new ApiError(json?.message || json?.error || `HTTP ${res.status}`, res.status, json)
  }
  return json as T
}

/** Returns the `data` field of a `{ data }` envelope. */
export async function api<T = any>(path: string, options: RequestOptions = {}): Promise<T> {
  const json = await request<any>(path, options)
  return (json?.data ?? null) as T
}

/** Calls an endpoint under the secure admin prefix, `/api/v2/{secure_path}/...`. */
export function adminV2<T = any>(endpoint: string, options: RequestOptions = {}): Promise<T> {
  if (!adminPath) return Promise.reject(new ApiError('当前账号没有管理权限', 403, null))
  const cleaned = String(endpoint).replace(/^\/+/, '')
  return api<T>(`/api/v2/${encodeURIComponent(adminPath)}/${cleaned}`, options)
}

export function errorMessage(error: unknown, fallback = '操作失败'): string {
  if (error instanceof Error && error.message) return error.message
  return fallback
}
