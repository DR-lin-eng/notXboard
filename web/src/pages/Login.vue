<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Moon, ShieldCheck, Sun } from 'lucide-vue-next'
import { errorMessage, request } from '@/lib/api'
import {
  captchaEnabled, captchaProvider, executeRecaptchaV3, loadGuestConfig, loadRecaptcha, loadTurnstile,
  powEnabled, powFresh, solvePow, type AuthMode, type GuestConfig, type PowProof,
} from '@/lib/captcha'
import { brandMark, setTitle, site } from '@/lib/site'
import { useSession } from '@/stores/session'
import { useUi } from '@/stores/ui'

const route = useRoute()
const router = useRouter()
const session = useSession()
const ui = useUi()

const config = ref<GuestConfig>({})
const configError = ref('')
const email = ref('')
const password = ref('')
const emailCode = ref('')
const inviteCode = ref(String(route.query.invite_code || '').trim())
const error = ref('')
const submitting = ref(false)
const sendingCode = ref(false)
const codeCooldown = ref(0)
const captchaToken = ref('')
const captchaHint = ref('')
const captchaFailed = ref(false)
const widgetEl = ref<HTMLElement | null>(null)
const pow = ref<PowProof | null>(null)
const powState = ref<'idle' | 'working' | 'ready' | 'error'>('idle')
const powError = ref('')
let widgetId: string | number | null = null
let cooldownTimer = 0
let powTicket = 0

const allowEmailRegister = computed(() => Number(config.value.allow_email_register || 0) === 1)
const allowOauthRegister = computed(() => Number(config.value.allow_oauth_register || 0) === 1)
const oauthEnabled = computed(() => Number(config.value.oauth_linux_do_enable || 0) === 1)
const emailVerify = computed(() => Number(config.value.is_email_verify || 0) === 1)
const inviteForced = computed(() => Number(config.value.is_invite_force || 0) === 1)
const mode = computed<AuthMode>(() => (route.query.register === '1' && allowEmailRegister.value ? 'register' : 'login'))
const provider = computed(() => (captchaEnabled(config.value) ? captchaProvider(config.value) : ''))
const needsWidget = computed(() => provider.value === 'turnstile' || provider.value === 'recaptcha')
const usesPow = computed(() => powEnabled(config.value))

const oauthUrl = computed(() => {
  const url = new URL('/api/v1/passport/oauth2/linux-do/redirect', location.origin)
  const code = inviteCode.value.trim()
  if (code) url.searchParams.set('invite_code', code)
  return url.toString()
})
const registerBlocked = computed(() => {
  if (allowEmailRegister.value || !Object.keys(config.value).length) return ''
  return oauthEnabled.value && allowOauthRegister.value
    ? '邮箱注册已关闭，请使用 Linux DO 完成首次注册。'
    : '当前站点已关闭新用户注册。'
})
const canSubmit = computed(() => {
  if (submitting.value) return false
  if (needsWidget.value && !captchaToken.value) return false
  if (usesPow.value && powState.value !== 'ready') return false
  return true
})

function switchMode(next: AuthMode) {
  const query: Record<string, string> = {}
  if (inviteCode.value.trim()) query.invite_code = inviteCode.value.trim()
  if (next === 'register') query.register = '1'
  router.replace({ path: '/login', query })
}

async function preparePow() {
  if (!usesPow.value) {
    powState.value = 'idle'
    return
  }
  if (powFresh(pow.value)) {
    powState.value = 'ready'
    return
  }
  const mine = ++powTicket
  powState.value = 'working'
  powError.value = ''
  pow.value = null
  try {
    const proof = await solvePow()
    if (mine !== powTicket) return
    pow.value = proof
    powState.value = 'ready'
  } catch (e) {
    if (mine !== powTicket) return
    powState.value = 'error'
    powError.value = errorMessage(e, '防刷验证准备失败，请刷新后重试。')
  }
}

function takePow(): PowProof | null {
  const proof = powFresh(pow.value) ? pow.value : null
  pow.value = null
  return proof
}

function resetCaptcha() {
  captchaToken.value = ''
  try {
    if (widgetId === null) return
    if (provider.value === 'turnstile') window.turnstile?.reset(widgetId as string)
    if (provider.value === 'recaptcha') window.grecaptcha?.reset(widgetId as number)
  } catch {
    // widget may already be gone
  }
}

async function mountCaptcha() {
  captchaToken.value = ''
  captchaHint.value = ''
  captchaFailed.value = false
  widgetId = null
  if (!needsWidget.value) return
  await nextTick()
  const el = widgetEl.value
  if (!el) return
  el.innerHTML = ''
  const action = mode.value === 'register' ? 'portal_register' : 'portal_login'
  const callbacks = {
    callback: (token: string) => {
      captchaToken.value = String(token || '')
    },
    'expired-callback': () => {
      captchaToken.value = ''
    },
    'error-callback': () => {
      captchaToken.value = ''
    },
  }
  captchaHint.value = '正在加载人机验证控件…'
  try {
    if (provider.value === 'turnstile') {
      const sitekey = String(config.value.turnstile_site_key || '').trim()
      if (!sitekey) throw new Error('Cloudflare Turnstile 的站点密钥未配置。')
      await loadTurnstile()
      widgetId = window.turnstile!.render(el, { sitekey, action, theme: ui.theme, ...callbacks })
    } else {
      const sitekey = String(config.value.recaptcha_site_key || '').trim()
      if (!sitekey) throw new Error('reCAPTCHA 的站点密钥未配置。')
      await loadRecaptcha()
      widgetId = window.grecaptcha!.render(el, { sitekey, theme: ui.theme, ...callbacks })
    }
    captchaHint.value = ''
  } catch (e) {
    captchaFailed.value = true
    captchaHint.value = errorMessage(e, '人机验证加载失败，请刷新后重试。')
  }
}

async function captchaPayload(): Promise<Record<string, string>> {
  if (!provider.value) return {}
  if (provider.value === 'turnstile') {
    if (!captchaToken.value) throw new Error('请先完成人机验证')
    return { turnstile_token: captchaToken.value }
  }
  if (provider.value === 'recaptcha') {
    if (!captchaToken.value) throw new Error('请先完成人机验证')
    return { recaptcha_data: captchaToken.value }
  }
  if (provider.value === 'recaptcha-v3') {
    const action = mode.value === 'register' ? 'portal_register' : 'portal_login'
    const token = await executeRecaptchaV3(String(config.value.recaptcha_v3_site_key || ''), action)
    if (!token) throw new Error('reCAPTCHA v3 执行失败，请刷新后重试')
    return { recaptcha_v3_token: token }
  }
  throw new Error('当前开启的人机验证类型暂不支持，请联系管理员切换为 Turnstile 或 reCAPTCHA。')
}

async function sendCode() {
  if (sendingCode.value || codeCooldown.value > 0) return
  const address = email.value.trim()
  if (!address) {
    error.value = '请先输入邮箱地址'
    return
  }
  sendingCode.value = true
  error.value = ''
  try {
    const payload = { email: address, ...(await captchaPayload()) }
    await request('/api/v1/passport/comm/sendEmailVerify', { method: 'POST', body: payload })
    ui.ok('邮箱验证码已发送，请查看收件箱。')
    codeCooldown.value = 60
    cooldownTimer = window.setInterval(() => {
      codeCooldown.value -= 1
      if (codeCooldown.value <= 0) window.clearInterval(cooldownTimer)
    }, 1000)
  } catch (e) {
    error.value = errorMessage(e, '发送验证码失败')
  } finally {
    resetCaptcha()
    sendingCode.value = false
  }
}

async function submit() {
  if (!canSubmit.value) return
  const isRegister = mode.value === 'register'
  error.value = ''
  if (!email.value.trim() || !password.value) {
    error.value = '请输入邮箱和密码'
    return
  }
  submitting.value = true
  try {
    const payload: Record<string, string> = { email: email.value.trim(), password: password.value }
    if (isRegister) {
      payload.invite_code = inviteCode.value.trim()
      if (emailVerify.value && emailCode.value.trim()) payload.email_code = emailCode.value.trim()
    }
    Object.assign(payload, await captchaPayload())
    if (usesPow.value) {
      const proof = takePow()
      if (!proof) throw new Error('防刷验证尚未准备完成，请稍候再试。')
      Object.assign(payload, { pow_id: proof.id, pow_nonce: proof.nonce, pow_hash: proof.hash, pow_token: proof.token })
    }
    const res = await request<any>(`/api/v1/passport/auth/${isRegister ? 'register' : 'login'}`, { method: 'POST', body: payload })
    const authData = String(res?.data?.auth_data || '')
    if (!authData) throw new Error(isRegister ? '注册失败' : '登录失败')
    session.setAuth(authData, String(res?.data?.token || ''))
    await session.loadMe(true)
    await router.replace(session.consumeRedirect() || '/dashboard')
  } catch (e) {
    error.value = errorMessage(e, isRegister ? '注册失败' : '登录失败')
    resetCaptcha()
    preparePow()
  } finally {
    submitting.value = false
  }
}

watch(mode, () => {
  error.value = ''
  setTitle(mode.value === 'register' ? '注册' : '登录')
  mountCaptcha()
  preparePow()
})

onMounted(async () => {
  try {
    config.value = await loadGuestConfig()
  } catch (e) {
    configError.value = errorMessage(e, '站点配置加载失败，请刷新后重试。')
  }
  setTitle(mode.value === 'register' ? '注册' : '登录')
  mountCaptcha()
  preparePow()
})
onBeforeUnmount(() => {
  window.clearInterval(cooldownTimer)
  powTicket += 1
})
</script>

<template>
  <div class="login">
    <section class="art">
      <div class="brand"><span class="brand-mark">{{ brandMark(site.name) }}</span>{{ site.name }}</div>
      <div>
        <h2>一个入口，<br>管理订阅、节点与平台。</h2>
        <p>普通用户与管理员使用同一个登录页。登录后系统按账号身份自动展示对应的功能。</p>
      </div>
      <ul>
        <li><ShieldCheck />登录后按身份显示菜单，管理接口始终由服务端校验</li>
        <li><ShieldCheck />节点、机器、套餐与工单集中在同一个工作台</li>
      </ul>
    </section>

    <section class="form-side">
      <button type="button" class="icon-btn theme" :aria-label="ui.theme === 'dark' ? '切换到浅色' : '切换到深色'" @click="ui.toggleTheme()">
        <Sun v-if="ui.theme === 'dark'" /><Moon v-else />
      </button>
      <form class="box" novalidate @submit.prevent="submit">
        <div class="brand only-m"><span class="brand-mark">{{ brandMark(site.name) }}</span>{{ site.name }}</div>
        <h3>{{ mode === 'register' ? '创建账户' : '登录账户' }}</h3>
        <p class="muted lead">{{ mode === 'register' ? '注册成功后会自动登录。' : '使用邮箱密码或第三方账号登录。' }}</p>

        <div v-if="allowEmailRegister" class="seg switch" role="tablist">
          <button type="button" role="tab" :aria-selected="mode === 'login'" :class="{ active: mode === 'login' }" @click="switchMode('login')">登录</button>
          <button type="button" role="tab" :aria-selected="mode === 'register'" :class="{ active: mode === 'register' }" @click="switchMode('register')">注册</button>
        </div>

        <div v-if="error" class="notice error" role="alert">{{ error }}</div>
        <div v-if="configError" class="notice error" role="alert">{{ configError }}</div>
        <div v-if="registerBlocked" class="notice">{{ registerBlocked }}</div>
        <div v-if="inviteCode && mode === 'login'" class="notice accent">
          当前邀请码：<code>{{ inviteCode }}</code>，注册时会自动绑定邀请关系。
        </div>
        <div v-if="inviteForced && !inviteCode && mode === 'register'" class="notice">当前站点要求必须通过邀请链接注册。</div>

        <div class="field">
          <label for="lg-email">邮箱</label>
          <input id="lg-email" v-model="email" class="input" type="email" autocomplete="username" placeholder="you@example.com" required>
        </div>
        <div class="field">
          <label for="lg-password">密码</label>
          <input
            id="lg-password" v-model="password" class="input" type="password" required
            :autocomplete="mode === 'register' ? 'new-password' : 'current-password'" placeholder="••••••••"
          >
        </div>
        <template v-if="mode === 'register'">
          <div v-if="emailVerify" class="field">
            <label for="lg-code">邮箱验证码</label>
            <div class="row">
              <input id="lg-code" v-model="emailCode" class="input" inputmode="numeric" maxlength="6" autocomplete="one-time-code" placeholder="6 位验证码">
              <button type="button" class="btn" :disabled="sendingCode || codeCooldown > 0" @click="sendCode">
                {{ codeCooldown > 0 ? `${codeCooldown}s 后重发` : sendingCode ? '发送中…' : '发送验证码' }}
              </button>
            </div>
          </div>
          <div class="field">
            <label for="lg-invite">邀请码{{ inviteForced ? '（必填）' : '（可空）' }}</label>
            <input id="lg-invite" v-model="inviteCode" class="input" placeholder="例如 ABCD1234" autocomplete="off">
          </div>
        </template>

        <div v-show="needsWidget" class="field">
          <span class="label">人机验证</span>
          <div ref="widgetEl" class="widget" />
        </div>
        <div v-if="captchaHint" class="notice" :class="{ error: captchaFailed }">{{ captchaHint }}</div>
        <div v-if="usesPow && powState === 'working'" class="notice">正在准备防刷验证，请稍候…</div>
        <div v-if="usesPow && powState === 'error'" class="notice error row">
          <span class="sp">{{ powError }}</span>
          <button type="button" class="btn sm" @click="preparePow">重试</button>
        </div>

        <button type="submit" class="btn primary block" :disabled="!canSubmit">
          {{ submitting ? '请稍候…' : mode === 'register' ? '立即注册' : '登录' }}
        </button>

        <template v-if="oauthEnabled && (mode === 'login' || allowOauthRegister)">
          <div class="or">或</div>
          <a class="btn block" :href="oauthUrl">
            {{ mode === 'register' ? '使用 Linux DO 注册' : allowOauthRegister ? '使用 Linux DO 登录 / 注册' : '使用 Linux DO 登录' }}
          </a>
        </template>
      </form>
    </section>
  </div>
</template>

<style scoped>
.login { min-height: 100%; display: grid; grid-template-columns: minmax(0, 1.05fr) minmax(0, 1fr); }
.art { background: var(--sb-bg); border-right: 1px solid var(--sb-border); padding: 48px; display: flex; flex-direction: column; justify-content: space-between; gap: 32px; }
.brand { display: flex; align-items: center; gap: 10px; font-weight: 650; font-size: 15px; }
.brand-mark { width: 26px; height: 26px; border-radius: 7px; background: var(--accent); color: var(--on-accent); display: grid; place-items: center; font-size: 12px; font-weight: 700; }
.art h2 { font-size: 34px; line-height: 1.2; letter-spacing: -.02em; font-weight: 700; }
.art p { margin-top: 14px; max-width: 46ch; color: var(--sb-muted); font-size: 15px; }
.art ul { list-style: none; padding: 0; margin: 0; display: grid; gap: 12px; color: var(--sb-muted); }
.art li { display: flex; gap: 10px; align-items: center; }
.art li svg { width: 18px; height: 18px; color: var(--accent); flex: none; }
.form-side { position: relative; display: grid; place-items: center; padding: 48px 32px; }
.theme { position: absolute; top: 16px; right: 16px; }
.box { width: min(380px, 100%); display: flex; flex-direction: column; gap: 14px; }
.box h3 { font-size: 22px; font-weight: 700; }
.lead { margin-top: -8px; }
.switch { align-self: flex-start; }
.switch button { padding: 6px 18px; font-size: 13.5px; }
.widget { min-height: 65px; }
.or { display: flex; align-items: center; gap: 12px; color: var(--muted); font-size: 12.5px; }
.or::before, .or::after { content: ""; height: 1px; flex: 1; background: var(--border); }
@media (max-width: 820px) {
  .login { grid-template-columns: minmax(0, 1fr); }
  .art { display: none; }
  .form-side { padding: 56px 20px 32px; place-items: start center; }
}
</style>
