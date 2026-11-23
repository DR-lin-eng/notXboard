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
      { path: 'plans', component: () => import('@/pages/Plans.vue'), meta: { title: '套餐' } },
      { path: 'plan-link/:token', component: () => import('@/pages/Plans.vue'), meta: { title: '专属套餐' } },
      { path: 'orders', component: () => import('@/pages/Orders.vue'), meta: { title: '订单' } },
      { path: 'order/:tradeNo', component: () => import('@/pages/OrderDetail.vue'), meta: { title: '订单详情' } },
      { path: 'refunds', component: () => import('@/pages/Refunds.vue'), meta: { title: '退款' } },
      { path: 'refund/:id', component: () => import('@/pages/RefundDetail.vue'), meta: { title: '退款详情' } },
      { path: 'refund-votes', component: () => import('@/pages/RefundVotes.vue'), meta: { title: '争议投票', flag: 'dispute' } },
      { path: 'refund-vote/:id', component: () => import('@/pages/RefundVoteDetail.vue'), meta: { title: '争议投票', flag: 'dispute' } },
      { path: 'nodes', component: () => import('@/pages/Nodes.vue'), meta: { title: '节点与机器' } },
      { path: 'nodes/new', component: () => import('@/pages/nodes/NodeForm.vue'), meta: { title: '新增节点' } },
      { path: 'nodes/edit/:id', component: () => import('@/pages/nodes/NodeForm.vue'), meta: { title: '编辑节点' } },
      { path: 'nodes/:id/monitor', component: () => import('@/pages/nodes/NodeMonitor.vue'), meta: { title: '节点监控' } },
      // user-routes
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
