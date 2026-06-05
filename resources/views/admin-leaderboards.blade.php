<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{{ $title }} · 超级管理员排行榜</title>
    <meta name="description" content="{{ $description }}">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Fira+Code:wght@400;500;600&family=Fira+Sans:wght@300;400;500;600;700&display=swap" rel="stylesheet">
    <style>
        :root {
            --bg: #0F172A;
            --panel: #111C32;
            --panel-2: #0B1426;
            --stroke: #22304A;
            --text: #F8FAFC;
            --muted: #94A3B8;
            --accent: #22C55E;
            --accent-2: #1F9D4B;
            --code: #FDE68A;
            --danger: #FB7185;
        }

        * { box-sizing: border-box; }
        body {
            margin: 0;
            font-family: "Fira Sans", "Segoe UI", sans-serif;
            color: var(--text);
            background: radial-gradient(circle at top, #1B2645 0%, #0B1120 45%, #070B14 100%);
        }

        a { color: inherit; text-decoration: none; }

        .page {
            min-height: 100vh;
            display: flex;
            flex-direction: column;
        }

        .nav {
            position: sticky;
            top: 0;
            z-index: 10;
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 18px 6vw;
            background: rgba(9, 16, 30, 0.85);
            border-bottom: 1px solid var(--stroke);
            backdrop-filter: blur(12px);
        }

        .brand {
            display: flex;
            align-items: center;
            gap: 12px;
            font-weight: 600;
        }

        .brand-logo {
            width: 36px;
            height: 36px;
            border-radius: 12px;
            background: linear-gradient(135deg, #1E293B, #1E3A8A);
            border: 1px solid var(--stroke);
            display: grid;
            place-items: center;
            font-family: "Fira Code", monospace;
            color: var(--code);
            font-size: 14px;
        }

        .btn {
            padding: 10px 16px;
            border-radius: 10px;
            background: var(--accent);
            color: #0A0F1D;
            font-weight: 600;
            transition: transform 0.2s ease, box-shadow 0.2s ease;
            box-shadow: 0 10px 30px rgba(34, 197, 94, 0.2);
        }

        .btn:hover { transform: translateY(-1px); }

        .hero {
            padding: 80px 6vw 40px;
            display: grid;
            gap: 24px;
        }

        .hero h1 {
            font-size: clamp(28px, 4vw, 48px);
            margin: 0;
        }

        .hero p {
            margin: 0;
            color: var(--muted);
            font-size: 16px;
            max-width: 680px;
        }

        .stats {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
            gap: 16px;
            margin-top: 16px;
        }

        .card {
            background: linear-gradient(180deg, rgba(15, 23, 42, 0.9), rgba(8, 15, 28, 0.9));
            border: 1px solid var(--stroke);
            border-radius: 16px;
            padding: 20px;
            box-shadow: 0 16px 40px rgba(0, 0, 0, 0.3);
        }

        .card .label {
            color: var(--muted);
            font-size: 13px;
        }

        .card .value {
            font-family: "Fira Code", monospace;
            font-size: 22px;
            margin-top: 8px;
        }

        .grid {
            padding: 20px 6vw 80px;
            display: grid;
            gap: 24px;
        }

        .section-title {
            font-size: 18px;
            margin: 0 0 12px;
        }

        .leaderboards {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 20px;
        }

        .list {
            list-style: none;
            padding: 0;
            margin: 16px 0 0;
            display: grid;
            gap: 10px;
        }

        .list-item {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 10px 12px;
            border-radius: 10px;
            background: rgba(30, 41, 59, 0.35);
            border: 1px solid rgba(34, 48, 74, 0.6);
        }

        .list-item span {
            font-size: 13px;
            color: var(--muted);
        }

        .list-item strong {
            font-family: "Fira Code", monospace;
            font-size: 13px;
        }

        .map-wrap {
            display: grid;
            grid-template-columns: minmax(240px, 1fr) minmax(280px, 1.2fr);
            gap: 20px;
        }

        .map {
            position: relative;
            border-radius: 18px;
            overflow: hidden;
            min-height: 360px;
            border: 1px solid rgba(255, 255, 255, 0.08);
            background: linear-gradient(180deg, rgba(7, 12, 22, 0.7), rgba(3, 7, 15, 0.9));
            box-shadow: inset 0 0 40px rgba(59, 130, 246, 0.2);
        }

        .map-chart {
            position: absolute;
            inset: 0;
        }

        .map-hub {
            position: absolute;
            left: 16px;
            bottom: 16px;
            padding: 12px 16px;
            background: rgba(15, 23, 42, 0.7);
            border-radius: 12px;
            border: 1px solid rgba(59, 130, 246, 0.3);
            backdrop-filter: blur(14px);
            font-size: 12px;
            color: #e0f2fe;
            display: grid;
            row-gap: 4px;
        }

        .map-summary {
            position: absolute;
            right: 18px;
            top: 18px;
            padding: 10px 14px;
            background: rgba(15, 23, 42, 0.75);
            border-radius: 10px;
            border: 1px solid rgba(59, 130, 246, 0.35);
            font-size: 11px;
            color: #d1d5db;
            display: grid;
            row-gap: 2px;
        }

        .map .hub-label {
            font-size: 14px;
            font-weight: 600;
            color: #f8fafc;
        }

        .notice {
            display: none;
            margin-top: 12px;
            padding: 10px 12px;
            border-radius: 10px;
            border: 1px solid rgba(251, 113, 133, 0.5);
            background: rgba(251, 113, 133, 0.12);
            color: #fecdd3;
            font-size: 13px;
        }

        .notice.show {
            display: block;
        }

        .footer {
            margin-top: auto;
            padding: 24px 6vw 40px;
            color: var(--muted);
            font-size: 12px;
            border-top: 1px solid var(--stroke);
        }

        .muted {
            color: var(--muted);
            font-size: 13px;
        }

        @media (max-width: 960px) {
            .map-wrap {
                grid-template-columns: 1fr;
            }
        }

        @media (prefers-reduced-motion: reduce) {
            .btn { transition: none; }
        }
    </style>
</head>
<body>
<div class="page">
    <div class="nav">
        <div class="brand">
            <div class="brand-logo">NX</div>
            <div>{{ $title }} · 超级管理员榜单</div>
        </div>
        <a class="btn" href="/{{ $secure_path }}">返回后台</a>
    </div>

    <section class="hero">
        <h1>超级管理员可见的全量排行榜。</h1>
        <p>展示全站运行概览、流量消耗排行、节点活跃度与区域分布。用户信息不做隐私隐藏，仅限超级管理员查看。</p>
        <div class="notice" id="auth-warning"></div>

        <div class="stats" id="overview-cards">
            <div class="card">
                <div class="label">正在使用的人数</div>
                <div class="value" id="active-users">--</div>
                <div class="muted">60 秒内有流量更新的用户</div>
            </div>
            <div class="card">
                <div class="label">一分钟内平均上行带宽</div>
                <div class="value" id="avg-up">--</div>
                <div class="muted">全站平均上传速率</div>
            </div>
            <div class="card">
                <div class="label">一分钟内平均下行带宽</div>
                <div class="value" id="avg-down">--</div>
                <div class="muted">全站平均下载速率</div>
            </div>
            <div class="card">
                <div class="label">已接入节点</div>
                <div class="value" id="node-count">--</div>
                <div class="muted">当前可管理的节点数量</div>
            </div>
            <div class="card">
                <div class="label">已注册人数</div>
                <div class="value" id="user-count">--</div>
                <div class="muted">累计注册用户</div>
            </div>
        </div>
    </section>

    <section class="grid">
        <div class="card">
            <h2 class="section-title">流量消耗排行</h2>
            <div class="leaderboards">
                <div>
                    <div class="muted">消耗最多的用户（未打码）</div>
                    <ul class="list" id="user-rank"></ul>
                </div>
                <div>
                    <div class="muted">消耗最多的节点</div>
                    <ul class="list" id="node-rank"></ul>
                </div>
            </div>
        </div>

        <div class="card">
            <h2 class="section-title">使用地区排行榜</h2>
            <div class="map-wrap">
                <div>
                    <div class="muted">访问来源（按地区统计）</div>
                    <ul class="list" id="region-rank"></ul>
                </div>
                <div class="map" id="region-map">
                    <div class="map-chart" id="leaderboard-map-chart"></div>
                    <div class="map-hub">
                        <span class="hub-label">Admin Operations Center</span>
                        <span id="hub-key">--</span>
                        <span id="hub-status">指令通道正常</span>
                    </div>
                    <div class="map-summary" id="map-summary">
                        <span>热点地区</span>
                        <strong id="summary-coverage">--</strong>
                        <span>流量源总量</span>
                        <strong id="summary-agents">--</strong>
                    </div>
                </div>
            </div>
        </div>
    </section>

    <footer class="footer">
        Admin Leaderboards · v{{ $version }}
    </footer>
</div>

<script src="/assets/world-map/echarts.min.js?v={{ $version }}"></script>
<script src="/assets/world-map/world.js?v={{ $version }}"></script>
<script src="/assets/world-map/notxboard-geo-map.js?v={{ $version }}"></script>

<script>
    const securePath = String(@json($secure_path ?? '')).replace(/^\\/+/, '');
    const apiBase = '/api/v2';
    const endpoints = {
        overview: `${apiBase}/${securePath}/public-dashboard/overview`,
        leaderboards: `${apiBase}/${securePath}/public-dashboard/leaderboards`,
        geo: `${apiBase}/${securePath}/public-dashboard/geo`
    };

    const warningEl = document.getElementById('auth-warning');
    let warningShown = false;

    const showWarning = (text) => {
        if (!warningEl || warningShown) return;
        warningEl.textContent = text;
        warningEl.classList.add('show');
        warningShown = true;
    };

    const normalizeBearer = (token) => {
        const clean = String(token || '').trim();
        if (!clean) return '';
        return clean.toLowerCase().startsWith('bearer ') ? clean : `Bearer ${clean}`;
    };

    const getToken = () => {
        return localStorage.getItem('auth_data')
            || localStorage.getItem('PORTAL_ACCESS_TOKEN')
            || localStorage.getItem('Portal_access_token')
            || localStorage.getItem('access_token')
            || localStorage.getItem('token');
    };

    const fetchJson = async (url) => {
        const token = normalizeBearer(getToken());
        const headers = {
            'Content-Type': 'application/json'
        };
        if (token) {
            headers.Authorization = token;
        }
        const res = await fetch(url, {
            credentials: 'same-origin',
            headers
        });
        if (res.status === 401) {
            showWarning('后台登录已失效，请重新登录超级管理员账号。');
            return null;
        }
        if (res.status === 403) {
            showWarning('需要超级管理员权限才能查看此页面数据。');
            return null;
        }
        return res.json();
    };

    const formatMbps = (bps) => {
        const value = (bps * 8) / 1_000_000;
        return value.toFixed(2) + ' Mbps';
    };

    const setText = (id, value) => {
        const el = document.getElementById(id);
        if (el) el.textContent = value;
    };

    const formatCompactNumber = (value) => new Intl.NumberFormat('zh-CN', {
        notation: 'compact',
        maximumFractionDigits: 1
    }).format(Number(value || 0));

    const shortRegionLabel = (label, key) => {
        const value = String(label || key || '').trim();
        if (!value) return key || '--';
        const cleaned = value.replace(/(特别行政区|自治区|省|市|地区)$/u, '').trim();
        if (cleaned.length <= 6) return cleaned;
        return key || cleaned.slice(0, 6);
    };

    const buildMapPoints = (regions) => {
        const geoMap = window.NotXboardGeoMap;
        const points = new Map();
        (Array.isArray(regions) ? regions : []).forEach((region) => {
            const key = geoMap ? geoMap.resolveLocationKey(region?.name, region?.code) : '';
            const coord = geoMap ? geoMap.coordFor(region?.name, region?.code) : null;
            if (!key || !coord) return;
            if (!points.has(key)) {
                points.set(key, {
                    key,
                    label: region?.name || region?.code || key,
                    count: 0,
                    coord,
                    shortLabel: shortRegionLabel(region?.name || region?.code, key),
                    detail: ''
                });
            }
            const point = points.get(key);
            point.count += Number(region?.count || 0);
            if (region?.name && (point.label === point.key || String(region.name).length < String(point.label).length)) {
                point.label = region.name;
                point.shortLabel = shortRegionLabel(region.name, key);
            }
        });
        return Array.from(points.values()).sort((left, right) => right.count - left.count);
    };

    const fillList = (id, items) => {
        const list = document.getElementById(id);
        if (!list) return;
        list.replaceChildren();
        items.forEach((item, index) => {
            const li = document.createElement('li');
            li.className = 'list-item';
            const label = document.createElement('div');
            label.textContent = `${index + 1}. ${item.name || item.code || ''}`;
            const value = document.createElement('strong');
            value.textContent = String(item.value_text ?? item.count ?? '');
            li.append(label, value);
            list.appendChild(li);
        });
    };

    const renderMapMarkers = (regions) => {
        const geoMap = window.NotXboardGeoMap;
        const chartEl = document.getElementById('leaderboard-map-chart');
        if (!geoMap || !chartEl) return;

        const points = buildMapPoints(regions).slice(0, 24);
        const totalCount = points.reduce((sum, item) => sum + Number(item.count || 0), 0);
        const top = points[0] || null;
        const hub = {
            name: 'Admin Operations Center',
            label: 'Admin Operations Center',
            coord: [104.1954, 35.8617],
            detail: 'Super Admin Relay'
        };
        const links = points.map((point, index) => ({
            label: `Admin -> ${point.label}`,
            coords: [hub.coord, point.coord],
            value: point.count,
            tone: index < 3 ? 'alert' : 'warn',
            detail: `${point.label} · ${formatCompactNumber(point.count)} 来源`
        }));

        geoMap.render(chartEl, {
            mode: 'admin',
            hub,
            points: points.map((point, index) => ({
                key: point.key,
                label: point.label,
                shortLabel: point.shortLabel,
                count: point.count,
                coord: point.coord,
                tone: index < 3 ? 'alert' : 'warn',
                detail: `${point.key} · ${formatCompactNumber(point.count)} 来源`
            })),
            links: links,
            minPointSize: 13,
            maxPointSize: 26,
            showLabels: true,
            maxLabels: 10
        });

        setText('summary-coverage', top ? `${points.length} 区域 / Top ${top.label}` : '--');
        setText('summary-agents', top ? formatCompactNumber(totalCount) : '--');
        setText('hub-key', top ? `${top.label} · ${formatCompactNumber(top.count)}` : '--');
        setText('hub-status', top ? `已绘制 ${points.length} 个热点地区，按流量源强度排序` : '暂无可绘制地区数据');
    };

    const loadOverview = async () => {
        const data = await fetchJson(endpoints.overview);
        if (!data || data.code !== 0) return;
        const d = data.data;
        setText('active-users', d.active_users ?? '--');
        setText('avg-up', formatMbps(d.avg_bandwidth?.upload_bps ?? 0));
        setText('avg-down', formatMbps(d.avg_bandwidth?.download_bps ?? 0));
        setText('node-count', d.node_count ?? '--');
        setText('user-count', d.registered_users ?? '--');
    };

    const loadLeaderboards = async () => {
        const data = await fetchJson(endpoints.leaderboards);
        if (!data || data.code !== 0) return;
        fillList('user-rank', data.data.top_users || []);
        fillList('node-rank', data.data.top_nodes || []);
    };

    const loadGeo = async () => {
        const data = await fetchJson(endpoints.geo);
        if (!data || data.code !== 0) return;
        const regions = data.data?.regions || [];
        fillList('region-rank', regions);
        renderMapMarkers(regions);
    };

    loadOverview();
    loadLeaderboards();
    loadGeo();
    setInterval(loadOverview, 60000);
    setInterval(loadLeaderboards, 180000);
    setInterval(loadGeo, 180000);
</script>
</body>
</html>
