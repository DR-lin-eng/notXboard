import type { Component } from 'vue'
import {
  Bell, BookOpen, Box, Download, Gauge, Gift, HeartHandshake, Home, Layers, LifeBuoy, MessageSquare,
  Receipt, Scale, Server, ShieldCheck, Undo2, UserRound, Users, Wallet, Wrench,
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
      { to: '/admin/support', label: '支持', icon: LifeBuoy, role: 'admin' },
    ],
  },
]
