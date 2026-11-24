import { defineStore } from 'pinia'
import { api, readAuth, setAdminPath, writeAuth } from '@/lib/api'
import { truthy } from '@/lib/format'

export interface Me {
  id: number
  email: string
  is_admin: boolean | number
  is_super_admin: boolean | number
  trust_level: number
  is_silenced?: boolean | number
  is_linux_do_user?: boolean | number
  linux_do_username?: string | null
  linux_do_name?: string | null
  linux_do_avatar?: string | null
  concurrent_ip_limit?: number | null
  refund_dispute_enable?: boolean | number
  secure_path?: string | null
  [key: string]: unknown
}

const POST_LOGIN_KEY = 'post_login_hash'

function readCachedMe(): Me | null {
  try {
    return JSON.parse(localStorage.getItem('me') || 'null')
  } catch {
    return null
  }
}

export const useSession = defineStore('session', {
  state: () => ({
    auth: readAuth(),
    me: readCachedMe() as Me | null,
    loaded: false,
  }),
  getters: {
    authed: (state) => Boolean(state.auth),
    isSuperAdmin: (state) => truthy(state.me?.is_super_admin),
    /** Staff: anyone who may see platform management menus. */
    isAdmin: (state) => truthy(state.me?.is_admin) || truthy(state.me?.is_super_admin),
    disputeEnabled: (state) => truthy(state.me?.refund_dispute_enable),
    displayName: (state) => state.me?.linux_do_name || state.me?.linux_do_username || state.me?.email || '',
    roleLabel(): string {
      if (this.isSuperAdmin) return '站长'
      if (truthy(this.me?.is_admin)) return '管理员'
      return `用户 · 信任等级 ${Number(this.me?.trust_level ?? 0)}`
    },
  },
  actions: {
    setAuth(authData: string, legacyToken = '') {
      writeAuth(authData)
      localStorage.setItem('token', legacyToken || '')
      this.auth = readAuth()
      this.setMe(null)
      this.loaded = false
    },
    setMe(me: Me | null) {
      this.me = me
      localStorage.setItem('me', JSON.stringify(me))
      setAdminPath(me?.secure_path)
    },
    async loadMe(force = false): Promise<Me | null> {
      if (!this.auth) return null
      if (this.loaded && !force) return this.me
      const me = await api<Me>('/api/v1/user/me')
      this.setMe(me)
      this.loaded = true
      return me
    },
    clear() {
      writeAuth('')
      localStorage.setItem('token', '')
      this.auth = ''
      this.setMe(null)
      this.loaded = false
    },
    rememberRedirect(fullPath: string) {
      if (!fullPath || fullPath.startsWith('/login')) return
      try {
        sessionStorage.setItem(POST_LOGIN_KEY, fullPath)
      } catch {
        // sessionStorage may be unavailable
      }
    },
    consumeRedirect(): string {
      try {
        const target = String(sessionStorage.getItem(POST_LOGIN_KEY) || '').replace(/^#/, '')
        sessionStorage.removeItem(POST_LOGIN_KEY)
        if (!target.startsWith('/') || target.startsWith('/login')) return ''
        return target
      } catch {
        return ''
      }
    },
  },
})
