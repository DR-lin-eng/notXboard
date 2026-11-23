import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'
import { useSession } from '@/stores/session'
import { setUnauthorizedHandler } from '@/lib/api'
import { setTitle } from '@/lib/site'

declare module 'vue-router' {
  interface RouteMeta {
    title?: string
    public?: boolean
    role?: 'admin' | 'super'
    flag?: 'dispute'
  }
}

const routes: RouteRecordRaw[] = [
  { path: '/login', component: () => import('@/pages/Login.vue'), meta: { public: true, title: '登录' } },
  {
    path: '/',
    component: () => import('@/layouts/AppShell.vue'),
    children: [
      { path: '', redirect: '/dashboard' },
      { path: 'dashboard', component: () => import('@/pages/Dashboard.vue'), meta: { title: '概览' } },
    ],
  },
  { path: '/:pathMatch(.*)*', redirect: '/dashboard' },
]

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
  scrollBehavior: () => ({ top: 0 }),
})

router.beforeEach(async (to) => {
  const session = useSession()
  if (to.meta.public) {
    if (to.path === '/login' && session.authed && !to.query.register) {
      try {
        await session.loadMe()
        return session.consumeRedirect() || '/dashboard'
      } catch {
        return true
      }
    }
    return true
  }
  if (!session.authed) {
    session.rememberRedirect(to.fullPath)
    return '/login'
  }
  try {
    await session.loadMe()
  } catch {
    if (!session.authed) return '/login'
  }
  if (to.meta.role === 'super' && !session.isSuperAdmin) return '/dashboard'
  if (to.meta.role === 'admin' && !session.isAdmin) return '/dashboard'
  if (to.meta.flag === 'dispute' && !session.disputeEnabled) return '/refunds'
  return true
})

router.afterEach((to) => setTitle(to.meta.title))

setUnauthorizedHandler(() => {
  const session = useSession()
  const current = router.currentRoute.value
  session.clear()
  if (current.path !== '/login') {
    session.rememberRedirect(current.fullPath)
    router.replace('/login')
  }
})
