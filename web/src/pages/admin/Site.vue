<script setup lang="ts">
import ConfigForm from '@/components/ConfigForm.vue'
import type { ConfigPage } from '@/lib/admin/configSchema'

const page: ConfigPage = {
  group: 'site',
  sections: [
    {
      title: '站点基础信息',
      fields: [
        { key: 'app_name', label: '站点名称', type: 'text' },
        { key: 'app_url', label: '站点地址', type: 'text', placeholder: 'https://example.com' },
        { key: 'app_description', label: '站点描述', type: 'text', full: true },
        { key: 'logo', label: 'Logo 地址', type: 'text' },
        { key: 'tos_url', label: '服务条款地址', type: 'text' },
        { key: 'currency', label: '货币代码', type: 'text', placeholder: 'CNY' },
        { key: 'currency_symbol', label: '货币符号', type: 'text', placeholder: '¥' },
        { key: 'try_out_plan_id', label: '试用套餐编号（0 = 不提供试用）', type: 'number', fallback: 0, min: 0 },
        { key: 'try_out_hour', label: '试用时长（小时）', type: 'number', fallback: 1, min: 1 },
        { key: 'force_https', label: '强制 HTTPS', type: 'bool' },
      ],
    },
    {
      title: '订阅',
      fields: [
        { key: 'subscribe_url', label: '订阅域名（可空）', type: 'text' },
        { key: 'subscribe_path', label: '订阅路径', type: 'text', group: 'subscribe' },
        { key: 'subscribe_root_domains', label: '订阅根域名（每行一个，用于随机子域名）', type: 'textarea' },
        { key: 'plan_change_enable', label: '允许变更套餐', type: 'bool', group: 'subscribe' },
        { key: 'surplus_enable', label: '变更时保留剩余价值', type: 'bool', group: 'subscribe' },
      ],
    },
    {
      title: '旧版主题外观',
      description: '只影响旧版界面，新界面的深浅色由用户自己切换',
      fields: [
        { key: 'frontend_theme_sidebar', label: '侧边栏风格', type: 'select', group: 'frontend', fallback: 'light', options: [{ value: 'light', label: '浅色' }, { value: 'dark', label: '深色' }] },
        { key: 'frontend_theme_header', label: '顶部风格', type: 'select', group: 'frontend', fallback: 'dark', options: [{ value: 'dark', label: '深色' }, { value: 'light', label: '浅色' }] },
        { key: 'frontend_theme_color', label: '主题配色', type: 'select', group: 'frontend', fallback: 'default', options: [
          { value: 'default', label: '默认' }, { value: 'darkblue', label: '深蓝' }, { value: 'black', label: '黑色' }, { value: 'green', label: '绿色' },
        ] },
        { key: 'frontend_background_url', label: '背景图地址（可空）', type: 'text', group: 'frontend' },
      ],
    },
  ],
}
</script>

<template>
  <div class="page narrow">
    <div class="page-head"><div><h2>站点与订阅</h2><p>站点信息、货币、试用和订阅地址。</p></div></div>
    <ConfigForm :page="page" />
  </div>
</template>
