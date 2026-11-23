<script setup lang="ts">
import { ref } from 'vue'
import ConfigForm from '@/components/ConfigForm.vue'
import { getAdminPath } from '@/lib/api'
import type { ConfigPage } from '@/lib/admin/configSchema'
import { useSession } from '@/stores/session'
import { useUi } from '@/stores/ui'

const ui = useUi()
const session = useSession()
const form = ref<InstanceType<typeof ConfigForm> | null>(null)
const captchaOn = (v: Record<string, any>) => Boolean(v.captcha_enable)
const powOn = (v: Record<string, any>) => Boolean(v.pow_enable)

const page: ConfigPage = {
  group: 'safe',
  sections: [
    {
      title: '访问安全策略',
      fields: [
        { key: 'safe_mode_enable', label: '安全模式', type: 'bool' },
        { key: 'email_verify', label: '注册邮箱验证', type: 'bool' },
        { key: 'register_mode', label: '注册方式', type: 'select', fallback: 'all', options: [
          { value: 'all', label: '邮箱 + OAuth' }, { value: 'email_only', label: '仅邮箱注册' },
          { value: 'oauth_only', label: '仅 OAuth 注册' }, { value: 'closed', label: '关闭注册' },
        ] },
        { key: 'email_gmail_limit_enable', label: 'Gmail 邮箱规则限制', type: 'bool' },
        { key: 'secure_path', label: '管理接口路径', type: 'text', hint: '管理接口使用的私有路径前缀，修改后需要重新登录。' },
        { key: 'login_token_expire_days', label: '登录态有效期（天，0 = 永不过期）', type: 'number', fallback: 365, min: 0, max: 3650 },
        { key: 'email_whitelist_enable', label: '邮箱后缀白名单', type: 'bool' },
        { key: 'email_whitelist_suffix', label: '允许的邮箱后缀（每行一个）', type: 'lines', showIf: (v) => Boolean(v.email_whitelist_enable) },
      ],
    },
    {
      title: '登录与注册防护',
      fields: [
        { key: 'register_limit_by_ip_enable', label: '按网络地址限制注册', type: 'bool' },
        { key: 'register_limit_count', label: '注册次数上限', type: 'number', fallback: 3, min: 1 },
        { key: 'register_limit_expire', label: '统计时长（分钟）', type: 'number', fallback: 60, min: 1 },
        { key: 'password_limit_enable', label: '密码错误限制', type: 'bool' },
        { key: 'password_limit_count', label: '密码错误上限', type: 'number', fallback: 5, min: 1 },
        { key: 'password_limit_expire', label: '锁定时长（分钟）', type: 'number', fallback: 60, min: 1 },
      ],
    },
    {
      title: '验证码',
      fields: [
        { key: 'captcha_enable', label: '启用验证码', type: 'bool' },
        { key: 'captcha_type', label: '验证码类型', type: 'select', fallback: 'recaptcha', options: [
          { value: 'recaptcha', label: 'Google 验证（常规）' }, { value: 'recaptcha-v3', label: 'Google 验证（无感）' }, { value: 'turnstile', label: 'Cloudflare 验证' },
        ] },
        { key: 'recaptcha_key', label: 'Google 验证服务端密钥', type: 'password', showIf: (v) => captchaOn(v) && v.captcha_type === 'recaptcha' },
        { key: 'recaptcha_site_key', label: 'Google 验证网页端密钥', type: 'text', showIf: (v) => captchaOn(v) && v.captcha_type === 'recaptcha' },
        { key: 'recaptcha_v3_secret_key', label: 'Google 无感服务端密钥', type: 'password', showIf: (v) => captchaOn(v) && v.captcha_type === 'recaptcha-v3' },
        { key: 'recaptcha_v3_site_key', label: 'Google 无感网页端密钥', type: 'text', showIf: (v) => captchaOn(v) && v.captcha_type === 'recaptcha-v3' },
        { key: 'recaptcha_v3_score_threshold', label: '无感验证分数线', type: 'float', fallback: 0.5, min: 0, max: 1, step: 0.1, showIf: (v) => captchaOn(v) && v.captcha_type === 'recaptcha-v3' },
        { key: 'turnstile_secret_key', label: 'Cloudflare 服务端密钥', type: 'password', showIf: (v) => captchaOn(v) && v.captcha_type === 'turnstile' },
        { key: 'turnstile_site_key', label: 'Cloudflare 网页端密钥', type: 'text', showIf: (v) => captchaOn(v) && v.captcha_type === 'turnstile' },
      ],
    },
    {
      title: '防刷验证（工作量证明）',
      fields: [
        { key: 'pow_enable', label: '启用防刷验证', type: 'bool' },
        { key: 'pow_require_ja3', label: '设备指纹校验', type: 'bool', showIf: powOn },
        { key: 'pow_auto_scale_enable', label: '自动调节难度', type: 'bool', showIf: powOn },
        { key: 'pow_difficulty', label: '防刷难度', type: 'number', fallback: 4, min: 1, showIf: powOn },
        { key: 'pow_auto_max_difficulty', label: '自动调节上限', type: 'number', fallback: 7, min: 1, showIf: powOn },
        { key: 'pow_effective_difficulty', label: '当前生效难度', type: 'readonly', showIf: powOn },
        { key: 'pow_ttl', label: '有效时间（秒）', type: 'number', fallback: 120, min: 10, showIf: powOn },
        { key: 'pow_seed_salt', label: '防刷密钥', type: 'password', showIf: powOn },
        { key: 'pow_base_value', label: '防刷基础值', type: 'text', showIf: powOn },
      ],
    },
  ],
}

function randomPath() {
  const bytes = crypto.getRandomValues(new Uint8Array(8))
  if (form.value) form.value.values.secure_path = Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')
}
async function beforeSave(payload: Record<string, any>) {
  payload.secure_path = String(payload.secure_path || '').trim()
  if (!payload.secure_path || payload.secure_path === getAdminPath()) return true
  return ui.confirm('修改管理接口路径？', { message: `保存后旧路径立即失效，新路径为 ${payload.secure_path}。页面会自动刷新登录信息。`, confirmText: '确认修改', danger: true })
}
async function onSaved() {
  // secure_path may have changed; refresh it from the server.
  await session.loadMe(true).catch(() => null)
}
</script>

<template>
  <div class="page narrow">
    <div class="page-head"><div><h2>登录与安全</h2><p>登录保护、验证码和管理接口路径。</p></div></div>
    <ConfigForm ref="form" :page="page" :before-save="beforeSave" @saved="onSaved">
      <template #after-secure_path><button type="button" class="btn sm" @click="randomPath">随机生成</button></template>
    </ConfigForm>
  </div>
</template>
