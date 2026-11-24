<script setup lang="ts">
import { onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import { ArrowRight, Moon, Sun } from 'lucide-vue-next'
import { asArray, formatCompactNumber } from '@/lib/format'
import { brandMark, setTitle, site } from '@/lib/site'
import { useUi } from '@/stores/ui'

type Phase = 'loading' | 'ready' | 'error'
interface RankItem { name?: string; code?: string; value_text?: string; count?: number }

const ui = useUi()
const count = new Intl.NumberFormat('zh-CN')
const formatMbps = (bps: unknown) => `${((Number(bps || 0) * 8) / 1_000_000).toFixed(2)} Mbps`

const overview = reactive({ phase: 'loading' as Phase, data: null as any })
const boards = reactive({ phase: 'loading' as Phase, users: [] as RankItem[], nodes: [] as RankItem[] })
const geo = reactive({ phase: 'loading' as Phase, regions: [] as RankItem[], mapText: '正在加载区域数据', mapReady: false, summary: '' })

// Public data is read without credentials, so a signed-in visitor's token never goes to these endpoints.
async function api<T>(path: string): Promise<T> {
  const controller = new AbortController()
  const timeout = window.setTimeout(() => controller.abort(), 10_000)
  try {
    const res = await fetch(path, { signal: controller.signal, headers: { Accept: 'application/json' }, credentials: 'omit' })
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    const payload = await res.json()
    if (payload?.code !== 0) throw new Error(payload?.message || '请求失败')
    return (payload.data || {}) as T
  } finally {
    window.clearTimeout(timeout)
  }
}

const boardsEl = ref<HTMLElement | null>(null)
const mapEl = ref<HTMLElement | null>(null)
const chartEl = ref<HTMLElement | null>(null)

async function loadOverview() {
  try {
    overview.data = await api<any>('/api/v1/guest/public/overview')
    overview.phase = 'ready'
  } catch {
    overview.phase = 'error'
  }
}

async function loadBoards() {
  try {
    const data = await api<any>('/api/v1/guest/public/leaderboards')
    boards.users = asArray(data?.top_users)
    boards.nodes = asArray(data?.top_nodes)
    boards.phase = 'ready'
  } catch {
    boards.phase = 'error'
  }
}

// The world map is large, so its scripts are loaded only once the map scrolls into view.
const MAP_ASSETS = ['echarts.min.js', 'world.js', 'notxboard-geo-map.js'].map(
  (file) => `/assets/world-map/${file}?v=${encodeURIComponent(site.version)}`,
)
let mapAssets: Promise<void> | null = null
function loadScript(src: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const script = document.createElement('script')
    script.src = src
    script.async = true
    script.addEventListener('load', () => resolve(), { once: true })
    script.addEventListener('error', () => reject(new Error(`无法加载 ${src}`)), { once: true })
    document.head.appendChild(script)
  })
}
function ensureMapAssets(): Promise<void> {
  if (!mapAssets) {
    mapAssets = MAP_ASSETS.reduce((chain, src) => chain.then(() => loadScript(src)), Promise.resolve())
    mapAssets.catch(() => { mapAssets = null })
  }
  return mapAssets
}

function shortRegionLabel(label: string, key: string): string {
  const cleaned = label.replace(/(特别行政区|自治区|省|市|地区)$/u, '').trim()
  return cleaned.length <= 6 ? cleaned : key || cleaned.slice(0, 6)
}

function renderMap(regions: RankItem[]) {
  const geoMap = (window as any).NotXboardGeoMap
  if (!geoMap || !chartEl.value) throw new Error('地图组件不可用')
  const merged = new Map<string, { key: string; label: string; count: number; coord: number[]; shortLabel: string }>()
  for (const region of regions) {
    const key = geoMap.resolveLocationKey(region?.name, region?.code)
    const coord = geoMap.coordFor(region?.name, region?.code)
    if (!key || !coord) continue
    const label = String(region?.name || region?.code || key)
    const point = merged.get(key) || { key, label, count: 0, coord, shortLabel: shortRegionLabel(label, key) }
    point.count += Number(region?.count || 0)
    merged.set(key, point)
  }
  const points = [...merged.values()].sort((a, b) => b.count - a.count).slice(0, 24)
  const hub = { name: '平台调度中心', label: '平台调度中心', coord: [104.1954, 35.8617], detail: site.name }
  geoMap.render(chartEl.value, {
    mode: 'public',
    hub,
    points,
    links: points.map((point, index) => ({
      label: `平台 → ${point.label}`,
      coords: [hub.coord, point.coord],
      value: point.count,
      tone: index < 2 ? 'warn' : 'stable',
      detail: `${point.label} · ${formatCompactNumber(point.count)} 次来源`,
    })),
    minPointSize: 12,
    maxPointSize: 24,
    showLabels: true,
    maxLabels: 8,
  })
  const total = points.reduce((sum, point) => sum + point.count, 0)
  geo.summary = points.length
    ? `覆盖 ${points.length} 个地区，最活跃的是${points[0].label}，共 ${formatCompactNumber(total)} 次来源`
    : ''
}

async function loadGeo() {
  try {
    const data = await api<any>('/api/v1/guest/public/geo')
    geo.regions = asArray(data?.regions)
    geo.phase = 'ready'
  } catch {
    geo.phase = 'error'
    if (!geo.mapReady) geo.mapText = '区域数据加载失败'
    return
  }
  if (!geo.regions.length) {
    geo.mapReady = false
    geo.mapText = '还没有可以绘制的区域数据'
    return
  }
  try {
    if (!geo.mapReady) geo.mapText = '正在加载地图'
    await ensureMapAssets()
    renderMap(geo.regions)
    geo.mapReady = true
  } catch {
    geo.mapReady = false
    geo.mapText = '地图加载失败，左侧列表仍然可用'
  }
}

const stops: Array<() => void> = []
function poll(task: () => Promise<void>, interval: number) {
  let timer: number | undefined
  let stopped = false
  const tick = async () => {
    if (!document.hidden) await task()
    if (!stopped) timer = window.setTimeout(tick, interval)
  }
  void tick()
  stops.push(() => { stopped = true; window.clearTimeout(timer) })
}
function whenVisible(element: HTMLElement | null, start: () => void, rootMargin: string) {
  if (!element || !('IntersectionObserver' in window)) return start()
  const observer = new IntersectionObserver((entries) => {
    if (!entries.some((entry) => entry.isIntersecting)) return
    observer.disconnect()
    start()
  }, { rootMargin })
  observer.observe(element)
  stops.push(() => observer.disconnect())
}

onMounted(() => {
  setTitle('运行概览')
  // Old links such as /#/login pointed at the signed-in app before the two were split.
  if (window.location.hash.startsWith('#/')) {
    window.location.replace(`/app/${window.location.search}${window.location.hash}`)
    return
  }
  poll(loadOverview, 60_000)
  whenVisible(boardsEl.value, () => poll(loadBoards, 180_000), '220px 0px')
  whenVisible(mapEl.value, () => poll(loadGeo, 180_000), '80px 0px')
})
onBeforeUnmount(() => stops.forEach((stop) => stop()))

const rankName = (item: RankItem) => String(item.name || (item.code && item.code !== 'ZZ' ? item.code : '') || '未知地区')
const rankValue = (item: RankItem) => String(item.value_text ?? (item.count != null ? count.format(Number(item.count)) : '--'))
const metric = (value: unknown, format: (v: unknown) => string) =>
  overview.phase === 'error' && !overview.data ? '--' : format(value)
</script>

<template>
  <div class="pub">
    <header class="pub-top">
      <div class="pub-inner pub-bar">
        <a class="brand" href="/" :aria-label="`${site.name} 首页`"><span class="mark" aria-hidden="true">{{ brandMark(site.name) }}</span><span>{{ site.name }}</span></a>
        <span class="sp" />
        <button type="button" class="icon-btn" :aria-label="ui.theme === 'dark' ? '切换到浅色' : '切换到深色'" @click="ui.toggleTheme()">
          <Sun v-if="ui.theme === 'dark'" /><Moon v-else />
        </button>
        <a class="btn primary" href="/app/#/login">登录 <ArrowRight :size="15" /></a>
      </div>
    </header>

    <main class="pub-inner pub-main">
      <section class="hero" aria-labelledby="pub-title">
        <div>
          <h1 id="pub-title">{{ site.name }} 运行概览</h1>
          <p class="muted">平台的活跃用户、实时带宽、节点规模和访问来源，每分钟更新一次。</p>
        </div>
        <span class="badge" :class="overview.phase === 'error' ? 'danger' : overview.phase === 'ready' ? 'ok' : ''" role="status">
          {{ overview.phase === 'error' ? '暂时无法更新' : overview.phase === 'ready' ? '数据已更新' : '正在加载' }}
        </span>
      </section>

      <div class="metrics">
        <div class="card stat"><div class="k">60 秒内活跃用户</div><div class="v tnum"><span v-if="overview.phase === 'loading'" class="skeleton" /><template v-else>{{ metric(overview.data?.active_users, (v) => count.format(Number(v || 0))) }}</template></div></div>
        <div class="card stat"><div class="k">平均上行带宽</div><div class="v tnum"><span v-if="overview.phase === 'loading'" class="skeleton" /><template v-else>{{ metric(overview.data?.avg_bandwidth?.upload_bps, formatMbps) }}</template></div><div class="d">最近一分钟</div></div>
        <div class="card stat"><div class="k">平均下行带宽</div><div class="v tnum"><span v-if="overview.phase === 'loading'" class="skeleton" /><template v-else>{{ metric(overview.data?.avg_bandwidth?.download_bps, formatMbps) }}</template></div><div class="d">最近一分钟</div></div>
        <div class="card stat"><div class="k">已接入节点</div><div class="v tnum"><span v-if="overview.phase === 'loading'" class="skeleton" /><template v-else>{{ metric(overview.data?.node_count, (v) => count.format(Number(v || 0))) }}</template></div></div>
        <div class="card stat"><div class="k">注册用户</div><div class="v tnum"><span v-if="overview.phase === 'loading'" class="skeleton" /><template v-else>{{ metric(overview.data?.registered_users, (v) => count.format(Number(v || 0))) }}</template></div></div>
      </div>

      <section ref="boardsEl" class="block" aria-labelledby="pub-boards">
        <div class="block-head"><h2 id="pub-boards">流量排行</h2><span class="muted small">用户名已脱敏</span></div>
        <div class="grid g2">
          <div v-for="board in [{ title: '用户流量', items: boards.users, empty: '还没有用户流量数据' }, { title: '节点流量', items: boards.nodes, empty: '还没有节点流量数据' }]" :key="board.title" class="card">
            <div class="card-head"><h3>{{ board.title }}</h3></div>
            <div v-if="boards.phase === 'loading'" class="card-body col" aria-busy="true"><div v-for="n in 5" :key="n" class="skeleton" /></div>
            <div v-else-if="boards.phase === 'error' && !board.items.length" class="empty">排行加载失败，稍后会自动重试。</div>
            <div v-else-if="!board.items.length" class="empty">{{ board.empty }}</div>
            <ol v-else class="rank">
              <li v-for="(item, index) in board.items" :key="index"><span class="idx tnum">{{ String(index + 1).padStart(2, '0') }}</span><span class="name">{{ rankName(item) }}</span><b class="tnum">{{ rankValue(item) }}</b></li>
            </ol>
          </div>
        </div>
      </section>

      <section class="block" aria-labelledby="pub-geo">
        <div class="block-head"><h2 id="pub-geo">访问来源</h2><span class="muted small">按地区统计的访问次数</span></div>
        <div class="geo">
          <div class="card">
            <div class="card-head"><h3>来源地区</h3></div>
            <div v-if="geo.phase === 'loading'" class="card-body col" aria-busy="true"><div v-for="n in 5" :key="n" class="skeleton" /></div>
            <div v-else-if="geo.phase === 'error' && !geo.regions.length" class="empty">区域数据加载失败，稍后会自动重试。</div>
            <div v-else-if="!geo.regions.length" class="empty">还没有区域数据。</div>
            <ol v-else class="rank">
              <li v-for="(item, index) in geo.regions" :key="index"><span class="idx tnum">{{ String(index + 1).padStart(2, '0') }}</span><span class="name">{{ rankName(item) }}</span><b class="tnum">{{ rankValue(item) }}</b></li>
            </ol>
          </div>
          <div ref="mapEl" class="map">
            <div ref="chartEl" class="map-chart" role="img" aria-label="访问来源地区分布图" />
            <div v-if="!geo.mapReady" class="map-state">{{ geo.mapText }}</div>
            <div v-else-if="geo.summary" class="map-summary">{{ geo.summary }}</div>
          </div>
        </div>
      </section>
    </main>

    <footer class="pub-inner pub-foot"><span>{{ site.name }}</span><span>v{{ site.version }}</span></footer>
  </div>
</template>

<style scoped>
.pub { min-height: 100vh; display: flex; flex-direction: column; background: var(--bg); color: var(--text); }
.pub-inner { width: 100%; max-width: 1180px; margin: 0 auto; padding: 0 20px; }
.pub-top { position: sticky; top: 0; z-index: 10; background: var(--surface); border-bottom: 1px solid var(--border); }
.pub-bar { display: flex; align-items: center; gap: 8px; height: 56px; }
.brand { display: inline-flex; align-items: center; gap: 9px; font-weight: 650; color: var(--text); text-decoration: none; }
.mark { display: grid; place-items: center; width: 28px; height: 28px; border-radius: 7px; background: var(--accent); color: var(--on-accent); font-size: 11.5px; font-weight: 700; }
.pub-main { flex: 1; padding-top: 28px; padding-bottom: 40px; }
.hero { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; margin-bottom: 18px; }
.hero h1 { margin: 0 0 6px; font-size: 24px; line-height: 1.25; }
.hero p { margin: 0; }
.metrics { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: 12px; }
.metrics .skeleton { display: block; width: 60%; height: 22px; }
.block { margin-top: 30px; }
.block-head { display: flex; align-items: baseline; gap: 10px; margin-bottom: 12px; }
.block-head h2 { margin: 0; font-size: 17px; }
.rank { list-style: none; margin: 0; padding: 0; }
.rank li { display: grid; grid-template-columns: 30px minmax(0, 1fr) max-content; gap: 10px; align-items: center; padding: 10px 16px; border-top: 1px solid var(--border); }
.rank li:first-child { border-top: 0; }
.rank .idx { color: var(--muted); font-size: 12px; }
.rank .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.geo { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 2fr); gap: 12px; align-items: start; }
/* The map artwork is drawn for a dark backdrop, so this panel stays dark in both themes. */
.map { position: relative; min-height: 420px; border-radius: var(--radius); border: 1px solid var(--border); background: #0c1724; overflow: hidden; }
.map-chart { position: absolute; inset: 0; }
.map-state { position: absolute; inset: 0; display: grid; place-items: center; padding: 20px; text-align: center; color: #cbd5e1; background: #0c1724; }
.map-summary { position: absolute; left: 12px; bottom: 12px; right: 12px; width: fit-content; max-width: calc(100% - 24px); padding: 6px 10px; border-radius: var(--radius-sm); font-size: 12.5px; color: #dbe7f3; background: rgba(8, 18, 31, 0.88); }
.pub-foot { display: flex; justify-content: space-between; padding-top: 16px; padding-bottom: 24px; color: var(--muted); font-size: 12.5px; border-top: 1px solid var(--border); }
@media (max-width: 960px) {
  .metrics { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .geo { grid-template-columns: 1fr; }
  .map { min-height: 320px; }
}
@media (max-width: 560px) {
  .hero { flex-direction: column; }
  .hero h1 { font-size: 20px; }
}
</style>
