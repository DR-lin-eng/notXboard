<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{{ $title }} · 公共概览</title>
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
            grid-template-columns: minmax(240px, 1fr) minmax(320px, 1.4fr);
            gap: 20px;
        }

        .map {
            position: relative;
            background: radial-gradient(circle at 20% 20%, rgba(14, 165, 233, 0.12), transparent 70%), linear-gradient(180deg, rgba(15, 23, 42, 0.9), rgba(8, 15, 28, 0.95));
            border: 1px solid rgba(34, 48, 74, 0.8);
            border-radius: 20px;
            overflow: hidden;
            min-height: 360px;
        }

        .map-chart {
            position: absolute;
            inset: 0;
        }

        .map-summary {
            position: absolute;
            left: 18px;
            right: 18px;
            bottom: 18px;
            padding: 10px 14px;
            border-radius: 14px;
            font-size: 12px;
            line-height: 1.6;
            color: #dbeafe;
            background: rgba(2, 6, 23, 0.8);
            border: 1px solid rgba(96, 165, 250, 0.24);
            backdrop-filter: blur(8px);
            box-shadow: 0 14px 28px rgba(2, 6, 23, 0.35);
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
<script>
    (function () {
        var hash = String(window.location.hash || '').trim();
        if (!hash || !hash.startsWith('#/')) return;
        var target = '/app' + (window.location.search || '') + hash;
        if (window.location.pathname === '/app') return;
        window.location.replace(target);
    })();
</script>
<div class="page">
    <div class="nav">
        <div class="brand">
            <div class="brand-logo">NX</div>
            <div>{{ $title }}</div>
        </div>
        <a class="btn" href="/app/#/login">进入控制台</a>
    </div>

    <section class="hero">
        <h1>为全球服务器提供共享统一管理平台。</h1>
        <p>实时展示全站运行概览、流量消耗排行、节点活跃度与区域分布。无需登录即可快速了解平台状态。</p>

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
                    <div class="muted">消耗最多的用户（已打码）</div>
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
                    <div class="map-chart" id="region-map-chart"></div>
                    <div class="map-summary" id="map-summary"></div>
                </div>
            </div>
        </div>
    </section>

    <footer class="footer">
        Public Dashboard · v{{ $version }}
    </footer>
</div>

<script src="/assets/world-map/echarts.min.js?v={{ $version }}"></script>
<script src="/assets/world-map/world.js?v={{ $version }}"></script>
<script src="/assets/world-map/notxboard-geo-map.js?v={{ $version }}"></script>

<script>
    const endpoints = {
        overview: '/api/v1/guest/public/overview',
        leaderboards: '/api/v1/guest/public/leaderboards',
        geo: '/api/v1/guest/public/geo'
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
        const chartEl = document.getElementById('region-map-chart');
        const summary = document.getElementById('map-summary');
        if (!geoMap || !chartEl || !summary) return;

        const points = buildMapPoints(regions).slice(0, 24);
        const totalCount = points.reduce((sum, item) => sum + Number(item.count || 0), 0);
        const hub = {
            name: '平台调度中心',
            label: '平台调度中心',
            coord: [104.1954, 35.8617],
            detail: 'Public Dashboard Relay'
        };
        const links = points.map((point, index) => ({
            label: `平台 -> ${point.label}`,
            coords: [hub.coord, point.coord],
            value: point.count,
            tone: index < 2 ? 'warn' : 'stable',
            detail: `${point.label} · ${formatCompactNumber(point.count)} 来源`
        }));

        geoMap.render(chartEl, {
            mode: 'public',
            hub,
            points: points,
            links: links,
            minPointSize: 12,
            maxPointSize: 24,
            showLabels: true,
            maxLabels: 8
        });

        summary.textContent = points.length
            ? `已覆盖 ${points.length} 个热点地区 · 最活跃 ${points[0].label} · 来源 ${formatCompactNumber(totalCount)}`
            : '暂无地区热力数据';
    };

    const loadOverview = async () => {
        const res = await fetch(endpoints.overview);
        const data = await res.json();
        if (data.code !== 0) return;
        const d = data.data;
        setText('active-users', d.active_users ?? '--');
        setText('avg-up', formatMbps(d.avg_bandwidth?.upload_bps ?? 0));
        setText('avg-down', formatMbps(d.avg_bandwidth?.download_bps ?? 0));
        setText('node-count', d.node_count ?? '--');
        setText('user-count', d.registered_users ?? '--');
    };

    const loadLeaderboards = async () => {
        const res = await fetch(endpoints.leaderboards);
        const data = await res.json();
        if (data.code !== 0) return;
        fillList('user-rank', data.data.top_users || []);
        fillList('node-rank', data.data.top_nodes || []);
    };

    const loadGeo = async () => {
        const res = await fetch(endpoints.geo);
        const data = await res.json();
        if (data.code !== 0) return;
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
