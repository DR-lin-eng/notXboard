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
      { path: 'nodes/:id/access', component: () => import('@/pages/nodes/NodeAccess.vue'), meta: { title: '分享与权限' } },
      { path: 'nodes/:id/audit-rules', component: () => import('@/pages/nodes/NodeAuditRules.vue'), meta: { title: '审计规则' } },
      { path: 'tickets', component: () => import('@/pages/Tickets.vue'), meta: { title: '工单' } },
      { path: 'ticket/:id', component: () => import('@/pages/TicketDetail.vue'), meta: { title: '工单详情' } },
      { path: 'node-plans', component: () => import('@/pages/NodePlans.vue'), meta: { title: '节点套餐' } },
      { path: 'node-plans/new', component: () => import('@/pages/NodePlanForm.vue'), meta: { title: '新建节点套餐' } },
      { path: 'node-plans/edit/:id', component: () => import('@/pages/NodePlanForm.vue'), meta: { title: '编辑节点套餐' } },
      { path: 'node-admin', component: () => import('@/pages/nodeadmin/NodeAdmin.vue'), meta: { title: '节点用户' } },
      { path: 'node-admin/refunds', component: () => import('@/pages/nodeadmin/NodeAdminRefunds.vue'), meta: { title: '节点退款' } },
      { path: 'node-admin/refund/:id', component: () => import('@/pages/nodeadmin/NodeAdminRefundDetail.vue'), meta: { title: '处理退款' } },
      { path: 'node-admin/node/:id', component: () => import('@/pages/nodeadmin/NodeAdminUsers.vue'), meta: { title: '节点用户流量' } },
      { path: 'node-admin/ticket/:id', component: () => import('@/pages/nodeadmin/NodeAdminTicket.vue'), meta: { title: '处理工单' } },
      { path: 'profile', component: () => import('@/pages/Profile.vue'), meta: { title: '个人资料' } },
      { path: 'invite', component: () => import('@/pages/Invite.vue'), meta: { title: '邀请' } },
      { path: 'notices', component: () => import('@/pages/Notices.vue'), meta: { title: '公告' } },
      { path: 'knowledge', component: () => import('@/pages/Knowledge.vue'), meta: { title: '知识库' } },
      { path: 'downloads', component: () => import('@/pages/Downloads.vue'), meta: { title: '软件下载' } },
      { path: 'tools', component: () => import('@/pages/Tools.vue'), meta: { title: '工具' } },
      { path: 'payment', component: () => import('@/pages/PaymentProfile.vue'), meta: { title: '收款设置' } },
      { path: 'sponsor', component: () => import('@/pages/Sponsor.vue'), meta: { title: '赞助' } },
      { path: 'sponsor/:tradeNo', component: () => import('@/pages/SponsorDetail.vue'), meta: { title: '赞助订单' } },
      { path: 'audit', component: () => import('@/pages/AuditLogs.vue'), meta: { title: '审计日志' } },
      { path: 'admin', component: () => import('@/pages/admin/Home.vue'), meta: { title: '工作台', role: 'admin' } },
      { path: 'admin/command-center', component: () => import('@/pages/admin/CommandCenter.vue'), meta: { title: '指挥中心', role: 'super' } },
      { path: 'admin/users', component: () => import('@/pages/admin/Users.vue'), meta: { title: '用户', role: 'super' } },
      { path: 'admin/tickets', component: () => import('@/pages/admin/Tickets.vue'), meta: { title: '工单处理', role: 'admin' } },
      { path: 'admin/plans', component: () => import('@/pages/admin/Plans.vue'), meta: { title: '平台套餐', role: 'admin' } },
      { path: 'admin/payments', component: () => import('@/pages/admin/Payments.vue'), meta: { title: '支付通道', role: 'admin' } },
      { path: 'admin/coupons', component: () => import('@/pages/admin/Coupons.vue'), meta: { title: '优惠券', role: 'admin' } },
      { path: 'admin/gift-cards', component: () => import('@/pages/admin/GiftCards.vue'), meta: { title: '礼品卡', role: 'admin' } },
      { path: 'admin/notices', component: () => import('@/pages/admin/Notices.vue'), meta: { title: '公告管理', role: 'admin' } },
      { path: 'admin/system', component: () => import('@/pages/admin/System.vue'), meta: { title: '系统', role: 'admin' } },
      { path: 'admin/site', component: () => import('@/pages/admin/Site.vue'), meta: { title: '站点与订阅', role: 'super' } },
      { path: 'admin/security', component: () => import('@/pages/admin/Security.vue'), meta: { title: '登录与安全', role: 'super' } },
      { path: 'admin/oauth', component: () => import('@/pages/admin/OAuth.vue'), meta: { title: 'Linux DO 登录', role: 'super' } },
      { path: 'admin/telegram', component: () => import('@/pages/admin/Telegram.vue'), meta: { title: 'Telegram', role: 'super' } },
      { path: 'admin/risk', component: () => import('@/pages/admin/RiskReview.vue'), meta: { title: '风险审查', role: 'super' } },
      { path: 'admin/traffic-reset', component: () => import('@/pages/admin/TrafficReset.vue'), meta: { title: '流量重置', role: 'super' } },
      { path: 'admin/plugins', component: () => import('@/pages/admin/Plugins.vue'), meta: { title: '扩展', role: 'super' } },
      { path: 'admin/api', component: () => import('@/pages/admin/ApiTool.vue'), meta: { title: '接口调试', role: 'super' } },
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
