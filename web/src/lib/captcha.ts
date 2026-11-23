// Human verification and proof-of-work helpers for the sign-in and sign-up forms.
import { api } from './api'

export type AuthMode = 'login' | 'register'
export interface GuestConfig {
  is_captcha?: number
  captcha_type?: string
  turnstile_site_key?: string
  recaptcha_site_key?: string
  recaptcha_v3_site_key?: string
  pow_enable?: number
  allow_email_register?: number
  allow_oauth_register?: number
  oauth_linux_do_enable?: number
  is_email_verify?: number
  is_invite_force?: number
  [key: string]: unknown
}
export interface PowProof {
  id: string
  nonce: string
  hash: string
  token: string
  expires_at: number
}

declare global {
  interface Window {
    turnstile?: {
      render: (el: HTMLElement, options: Record<string, unknown>) => string
      reset: (id?: string) => void
      remove?: (id: string) => void
    }
    grecaptcha?: {
      render: (el: HTMLElement, options: Record<string, unknown>) => number
      reset: (id?: number) => void
      ready: (cb: () => void) => void
      execute: (key: string, options: { action: string }) => Promise<string>
    }
  }
}

let configPromise: Promise<GuestConfig> | null = null
export function loadGuestConfig(force = false): Promise<GuestConfig> {
  if (!configPromise || force) {
    configPromise = api<GuestConfig>('/api/v1/guest/comm/config')
      .then((data) => data || {})
      .catch((error) => {
        configPromise = null
        throw error
      })
  }
  return configPromise
}

export const captchaEnabled = (config: GuestConfig) => Number(config?.is_captcha || 0) === 1
export const captchaProvider = (config: GuestConfig) => String(config?.captcha_type || '').toLowerCase()
export const powEnabled = (config: GuestConfig) => Number(config?.pow_enable || 0) === 1

const scripts = new Map<string, Promise<void>>()
function loadScript(src: string, ready: () => boolean): Promise<void> {
  if (ready()) return Promise.resolve()
  const cached = scripts.get(src)
  if (cached) return cached
  const promise = new Promise<void>((resolve, reject) => {
    const el = document.createElement('script')
    el.src = src
    el.async = true
    el.defer = true
    el.onerror = () => reject(new Error('人机验证脚本加载失败，请检查网络后重试'))
    el.onload = () => {
      let checks = 0
      const timer = window.setInterval(() => {
        checks += 1
        if (ready()) {
          window.clearInterval(timer)
          resolve()
        } else if (checks >= 50) {
          window.clearInterval(timer)
          reject(new Error('人机验证初始化超时，请刷新后重试'))
        }
      }, 100)
    }
    document.head.appendChild(el)
  }).catch((error) => {
    scripts.delete(src)
    throw error
  })
  scripts.set(src, promise)
  return promise
}

export function loadTurnstile(): Promise<void> {
  return loadScript(
    'https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit',
    () => typeof window.turnstile?.render === 'function',
  )
}
export function loadRecaptcha(): Promise<void> {
  return loadScript(
    'https://www.recaptcha.net/recaptcha/api.js?render=explicit',
    () => typeof window.grecaptcha?.render === 'function',
  )
}
export async function executeRecaptchaV3(siteKey: string, action: string): Promise<string> {
  const key = String(siteKey || '').trim()
  if (!key) throw new Error('reCAPTCHA v3 的站点密钥未配置')
  await loadScript(
    `https://www.recaptcha.net/recaptcha/api.js?render=${encodeURIComponent(key)}`,
    () => typeof window.grecaptcha?.ready === 'function' && typeof window.grecaptcha?.execute === 'function',
  )
  return new Promise((resolve, reject) => {
    window.grecaptcha!.ready(() => {
      Promise.resolve(window.grecaptcha!.execute(key, { action }))
        .then((token) => resolve(String(token || '').trim()))
        .catch(reject)
    })
  })
}

async function sha256Hex(message: string): Promise<string> {
  const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(message))
  return Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, '0')).join('')
}

interface PowChallenge {
  challenge_id: string
  seed: string
  base: string
  token: string
  difficulty?: number
  expires_at: number
}

export async function solvePow(): Promise<PowProof> {
  if (!window.crypto?.subtle) throw new Error('当前浏览器环境不支持防刷验证，请使用 HTTPS 访问')
  const challenge = await api<PowChallenge>('/api/v1/passport/auth/pow-challenge')
  if (!challenge) throw new Error('防刷验证获取失败')
  const prefix = '0'.repeat(Math.max(1, Number(challenge.difficulty || 4)))
  for (let nonce = 0; ; nonce += 1) {
    const hash = await sha256Hex(`${challenge.seed}|${challenge.base}|${challenge.token}|${nonce}`)
    if (hash.startsWith(prefix)) {
      return { id: challenge.challenge_id, nonce: String(nonce), hash, token: challenge.token, expires_at: challenge.expires_at }
    }
    if (nonce % 200 === 199) await new Promise((resolve) => setTimeout(resolve, 0))
  }
}

export function powFresh(proof: PowProof | null): boolean {
  return Boolean(proof?.expires_at && Number(proof.expires_at) > Math.floor(Date.now() / 1000) + 5)
}
