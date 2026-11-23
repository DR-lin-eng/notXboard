import { adminV2, getAdminPath, request, ApiError } from '@/lib/api'

export interface AdminPage<T = any> {
  items: T[]
  total: number
  page: number
  lastPage: number
}

/** Admin list endpoints return rows as `data`, `data.data` or a top-level paginator. */
export async function adminList<T = any>(endpoint: string, query: Record<string, any> = {}, pageSize = 20): Promise<AdminPage<T>> {
  const adminPath = getAdminPath()
  if (!adminPath) throw new ApiError('当前账号没有管理权限', 403, null)
  const body = await request<any>(`/api/v2/${encodeURIComponent(adminPath)}/${endpoint.replace(/^\/+/, '')}`, { query })
  const data = body?.data
  const holder = Array.isArray(data) ? body : data && typeof data === 'object' ? data : body || {}
  const items: T[] = Array.isArray(data) ? data : Array.isArray(data?.data) ? data.data : []
  const pagination = holder.pagination && typeof holder.pagination === 'object' ? holder.pagination : holder
  const total = Number(pagination.total ?? holder.total ?? items.length) || 0
  const size = Number(pagination.per_page ?? pagination.pageSize ?? pagination.page_size ?? pageSize) || pageSize
  const page = Number(pagination.current_page ?? pagination.current ?? pagination.page ?? 1) || 1
  const lastPage = Number(pagination.last_page) || Math.max(1, Math.ceil(total / size))
  return { items, total, page, lastPage }
}

export function adminPost<T = any>(endpoint: string, body: unknown = {}): Promise<T> {
  return adminV2<T>(endpoint, { method: 'POST', body })
}
