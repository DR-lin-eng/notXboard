import type { Component } from 'vue'
import {
  Activity, Bell, BookOpen, Box, CreditCard, Download, Gauge, Gift, Globe, HeartHandshake, Home, KeyRound, Layers, LifeBuoy,
  Megaphone, MessageSquare, Package, Puzzle, Receipt, RotateCcw, Scale, Send, Server, ShieldAlert, ShieldCheck, Terminal,
  TicketPercent, Undo2, UserCog, UserRound, Users, Wallet, Wrench,
} from 'lucide-vue-next'

export type Role = 'user' | 'admin' | 'super'

export interface NavItem {
  to: string
  label: string
  icon: Component
  /** Minimum role. Defaults to every signed-in user. */
  role?: Role
  /** Extra visibility flag resolved by the shell. */
  flag?: 'dispute'
  /** Route name prefixes that keep the item highlighted. */
  match?: string[]
}
export interface NavGroup {
  title: string
  role?: Role
  items: NavItem[]
}

export const NAV: NavGroup[] = [
  {
    title: '订阅与账单',
    items: [
      { to: '/dashboard', label: '概览', icon: Home },
      { to: '/plans', label: '套餐', icon: Box, match: ['/plans', '/plan-link'] },
      { to: '/orders', label: '订单', icon: Receipt, match: ['/orders', '/order'] },
      { to: '/refunds', label: '退款', icon: Undo2, match: ['/refunds', '/refund/'] },
      { to: '/refund-votes', label: '争议投票', icon: Scale, flag: 'dispute', match: ['/refund-votes', '/refund-vote/'] },
    ],
  },
  {
    title: '节点运营',
    items: [
      { to: '/nodes', label: '节点与机器', icon: Server },
      { to: '/node-plans', label: '节点套餐', icon: Layers },
      { to: '/node-admin', label: '节点用户', icon: Users },
      { to: '/audit', label: '审计', icon: ShieldCheck },
      { to: '/tools', label: '工具', icon: Wrench },
    ],
  },
  {
    title: '支持与资源',
    items: [
      { to: '/tickets', label: '工单', icon: MessageSquare, match: ['/tickets', '/ticket/'] },
      { to: '/notices', label: '公告', icon: Bell },
      { to: '/knowledge', label: '知识库', icon: BookOpen },
      { to: '/downloads', label: '软件下载', icon: Download },
    ],
  },
  {
    title: '账户',
    items: [
      { to: '/profile', label: '个人资料', icon: UserRound },
      { to: '/payment', label: '收款设置', icon: Wallet },
      { to: '/sponsor', label: '赞助', icon: HeartHandshake },
      { to: '/invite', label: '邀请', icon: Gift },
    ],
  },
  {
    title: '平台管理',
    role: 'admin',
    items: [
      { to: '/admin', label: '工作台', icon: Gauge, role: 'admin' },
      { to: '/admin/users', label: '用户', icon: UserCog, role: 'super' },
      { to: '/admin/tickets', label: '工单处理', icon: LifeBuoy, role: 'admin' },
      { to: '/admin/plans', label: '平台套餐', icon: Package, role: 'admin' },
      { to: '/admin/payments', label: '支付通道', icon: CreditCard, role: 'admin' },
      { to: '/admin/coupons', label: '优惠券', icon: TicketPercent, role: 'admin' },
      { to: '/admin/gift-cards', label: '礼品卡', icon: Gift, role: 'admin' },
      { to: '/admin/notices', label: '公告管理', icon: Megaphone, role: 'admin' },
      { to: '/admin/system', label: '系统', icon: Activity, role: 'admin' },
    ],
  },
  {
    title: '站点设置',
    role: 'super',
    items: [
      { to: '/admin/site', label: '站点与订阅', icon: Globe, role: 'super' },
      { to: '/admin/security', label: '登录与安全', icon: ShieldCheck, role: 'super' },
      { to: '/admin/oauth', label: 'Linux DO 登录', icon: KeyRound, role: 'super' },
      { to: '/admin/telegram', label: 'Telegram', icon: Send, role: 'super' },
      { to: '/admin/risk', label: '风险审查', icon: ShieldAlert, role: 'super' },
      { to: '/admin/traffic-reset', label: '流量重置', icon: RotateCcw, role: 'super' },
      { to: '/admin/plugins', label: '扩展', icon: Puzzle, role: 'super' },
      { to: '/admin/api', label: '接口调试', icon: Terminal, role: 'super' },
    ],
  },
]
