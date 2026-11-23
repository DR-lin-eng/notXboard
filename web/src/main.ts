import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { site } from './lib/site'
import './styles/tokens.css'
import './styles/base.css'

async function start() {
  if (site.page === 'public') {
    const { default: PublicDashboard } = await import('./pages/public/PublicDashboard.vue')
    createApp(PublicDashboard).use(createPinia()).mount('#app')
    return
  }
  const [{ default: App }, { router }] = await Promise.all([import('./App.vue'), import('./router')])
  createApp(App).use(createPinia()).use(router).mount('#app')
}

void start()
