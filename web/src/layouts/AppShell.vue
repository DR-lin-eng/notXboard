<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { LogOut, Menu, Moon, Sun } from 'lucide-vue-next'
import { NAV, type NavItem, type Role } from '@/lib/nav'
import { brandMark, site } from '@/lib/site'
import { useSession } from '@/stores/session'
import { useUi } from '@/stores/ui'

const session = useSession()
const ui = useUi()
const route = useRoute()
const router = useRouter()
const userMenu = ref(false)

function allowed(role?: Role): boolean {
  if (role === 'super') return session.isSuperAdmin
  if (role === 'admin') return session.isAdmin
  return true
}

const groups = computed(() =>
  NAV.filter((group) => allowed(group.role))
    .map((group) => ({
      ...group,
      items: group.items.filter((item) => allowed(item.role) && (item.flag !== 'dispute' || session.disputeEnabled)),
    }))
    .filter((group) => group.items.length > 0),
)

function isActive(item: NavItem): boolean {
  const path = route.path
  const prefixes = item.match || [item.to]
  if (item.to === '/admin') return path === '/admin'
  return prefixes.some((prefix) => path === prefix || path.startsWith(prefix.endsWith('/') ? prefix : `${prefix}/`))
}

const title = computed(() => route.meta.title || site.name)
const initials = computed(() => (session.displayName || '?').slice(0, 1).toUpperCase())

watch(
  () => route.fullPath,
  () => {
    ui.navOpen = false
    userMenu.value = false
  },
)

function logout() {
  session.clear()
  router.replace('/login')
}
</script>

<template>
  <div class="shell" :class="{ 'nav-open': ui.navOpen }">
    <div class="scrim" @click="ui.navOpen = false" />
    <aside class="sidebar" aria-label="主导航">
      <RouterLink to="/dashboard" class="brand">
        <span class="brand-mark">{{ brandMark(site.name) }}</span>
        <span class="truncate">{{ site.name }}</span>
      </RouterLink>
      <nav class="nav">
        <template v-for="group in groups" :key="group.title">
          <div class="nav-group">
            {{ group.title }}
            <span v-if="group.role" class="nav-tag">{{ group.role === 'super' ? '站长' : '管理员' }}</span>
          </div>
          <RouterLink
            v-for="item in group.items"
            :key="item.to"
            :to="item.to"
            class="nav-item"
            :class="{ active: isActive(item) }"
            :aria-current="isActive(item) ? 'page' : undefined"
          >
            <component :is="item.icon" />
            <span>{{ item.label }}</span>
          </RouterLink>
        </template>
      </nav>
      <div class="sb-foot">
        <span class="avatar">{{ initials }}</span>
        <div class="who">
          <b class="truncate">{{ session.displayName }}</b>
          <span>{{ session.roleLabel }}</span>
        </div>
      </div>
    </aside>

    <div class="main">
      <header class="topbar">
        <button type="button" class="icon-btn menu-btn" aria-label="打开菜单" @click="ui.navOpen = true"><Menu /></button>
        <h1 class="truncate">{{ title }}</h1>
        <span class="sp" />
        <button
          type="button"
          class="icon-btn"
          :aria-label="ui.theme === 'dark' ? '切换到浅色' : '切换到深色'"
          @click="ui.toggleTheme()"
        >
          <Sun v-if="ui.theme === 'dark'" /><Moon v-else />
        </button>
        <div class="user">
          <button type="button" class="avatar-btn" aria-label="账户菜单" :aria-expanded="userMenu" @click="userMenu = !userMenu">
            <span class="avatar">{{ initials }}</span>
          </button>
          <div v-if="userMenu" class="menu" role="menu">
            <div class="menu-head">
              <b class="truncate">{{ session.displayName }}</b>
              <span class="muted small">{{ session.roleLabel }}</span>
            </div>
            <RouterLink to="/profile" role="menuitem">个人资料</RouterLink>
            <button type="button" role="menuitem" @click="logout"><LogOut />退出登录</button>
          </div>
        </div>
      </header>
      <main class="content"><RouterView :key="route.path" /></main>
    </div>
  </div>
</template>

<style scoped>
.shell { display: grid; grid-template-columns: 240px minmax(0, 1fr); height: 100%; }
.sidebar { background: var(--sb-bg); border-right: 1px solid var(--sb-border); display: flex; flex-direction: column; min-height: 0; }
.brand { display: flex; align-items: center; gap: 10px; height: 56px; padding: 0 16px; color: var(--text); font-weight: 650; font-size: 15px; flex: none; }
.brand-mark { width: 26px; height: 26px; border-radius: 7px; background: var(--accent); color: var(--on-accent); display: grid; place-items: center; font-size: 12px; font-weight: 700; flex: none; }
.nav { flex: 1; overflow-y: auto; padding: 4px 10px 12px; }
.nav-group { font-size: 11.5px; font-weight: 600; color: var(--sb-muted); padding: 14px 8px 6px; display: flex; align-items: center; gap: 6px; }
.nav-tag { font-size: 10.5px; padding: 1px 6px; border-radius: 99px; background: var(--sb-active-bg); color: var(--sb-active-text); }
.nav-item { display: flex; align-items: center; gap: 10px; height: 34px; padding: 0 8px; border-radius: var(--radius-sm); color: var(--sb-text); }
.nav-item:hover { background: var(--sb-hover); }
.nav-item.active { background: var(--sb-active-bg); color: var(--sb-active-text); font-weight: 600; }
.nav-item svg { width: 17px; height: 17px; flex: none; }
.sb-foot { border-top: 1px solid var(--sb-border); padding: 12px 14px; display: flex; align-items: center; gap: 10px; flex: none; }
.avatar { width: 30px; height: 30px; border-radius: 99px; background: var(--accent); color: var(--on-accent); display: grid; place-items: center; font-size: 12px; font-weight: 700; flex: none; }
.who { min-width: 0; line-height: 1.25; display: flex; flex-direction: column; }
.who b { font-size: 13px; font-weight: 600; color: var(--text); }
.who span { font-size: 12px; color: var(--sb-muted); }
.main { display: flex; flex-direction: column; min-width: 0; min-height: 0; }
.topbar { height: 56px; flex: none; background: var(--surface); border-bottom: 1px solid var(--border); display: flex; align-items: center; gap: 12px; padding: 0 20px; }
.topbar h1 { font-size: 15px; font-weight: 650; }
.menu-btn { display: none; }
.content { flex: 1; overflow: auto; padding: 24px; }
.scrim { display: none; }
.user { position: relative; }
.avatar-btn { border: 0; background: none; padding: 0; cursor: pointer; border-radius: 99px; display: block; }
.menu { position: absolute; right: 0; top: 40px; z-index: 30; width: 220px; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); box-shadow: var(--pop-shadow); padding: 6px; }
.menu-head { padding: 8px 10px 10px; border-bottom: 1px solid var(--border); margin-bottom: 6px; display: flex; flex-direction: column; }
.menu a, .menu button { display: flex; align-items: center; gap: 8px; width: 100%; height: 34px; padding: 0 10px; border: 0; background: none; border-radius: var(--radius-sm); cursor: pointer; text-align: left; }
.menu a:hover, .menu button:hover { background: var(--surface-2); }
.menu svg { width: 16px; height: 16px; }

@media (max-width: 820px) {
  .shell { grid-template-columns: minmax(0, 1fr); }
  .sidebar { position: fixed; inset: 0 auto 0 0; width: 264px; z-index: 50; transform: translateX(-100%); transition: transform .18s ease; }
  .nav-open .sidebar { transform: none; }
  .nav-open .scrim { display: block; position: fixed; inset: 0; z-index: 40; background: var(--overlay); }
  .menu-btn { display: grid; }
  .topbar { padding: 0 12px; }
  .content { padding: 16px 12px 28px; }
  .nav-item { height: 40px; }
}
</style>
