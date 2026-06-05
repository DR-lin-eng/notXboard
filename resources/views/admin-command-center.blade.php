<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{{ $title }} · 超级管理员指挥舱</title>
    <meta name="description" content="{{ $description }}">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;600;700&family=IBM+Plex+Mono:wght@400;500;600&family=Orbitron:wght@500;700;800&display=swap" rel="stylesheet">
    <style>
        :root {
            --bg: #030712;
            --bg-2: #07111f;
            --panel: rgba(6, 17, 31, 0.86);
            --panel-strong: rgba(8, 21, 38, 0.96);
            --stroke: rgba(96, 165, 250, 0.22);
            --stroke-strong: rgba(56, 189, 248, 0.4);
            --text: #E6F1FF;
            --muted: #8CA3C6;
            --cyan: #22D3EE;
            --teal: #14B8A6;
            --blue: #60A5FA;
            --amber: #F59E0B;
            --rose: #FB7185;
            --green: #22C55E;
            --violet: #A78BFA;
            --shadow: 0 24px 60px rgba(2, 6, 23, 0.55);
            --radius-xl: 26px;
            --radius-lg: 18px;
            --radius-md: 14px;
        }

        * { box-sizing: border-box; }

        html, body {
            margin: 0;
            min-height: 100%;
            background:
                radial-gradient(1200px 680px at 12% -10%, rgba(34, 211, 238, 0.12), transparent 58%),
                radial-gradient(980px 620px at 100% 0%, rgba(96, 165, 250, 0.12), transparent 56%),
                radial-gradient(900px 560px at 50% 120%, rgba(20, 184, 166, 0.10), transparent 52%),
                linear-gradient(180deg, #02050d 0%, #030712 52%, #07111f 100%);
            color: var(--text);
            font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
        }

        body::before {
            content: "";
            position: fixed;
            inset: 0;
            pointer-events: none;
            background:
                linear-gradient(rgba(96, 165, 250, 0.06) 1px, transparent 1px),
                linear-gradient(90deg, rgba(96, 165, 250, 0.05) 1px, transparent 1px);
            background-size: 120px 120px;
            mask-image: radial-gradient(circle at center, rgba(0, 0, 0, 0.85), transparent 92%);
            opacity: 0.35;
        }

        a { color: inherit; text-decoration: none; }
        button, a.button {
            border: 0;
            border-radius: 999px;
            padding: 11px 18px;
            font-weight: 600;
            cursor: pointer;
            transition: transform 0.18s ease, box-shadow 0.18s ease, border-color 0.18s ease, background 0.18s ease;
        }
        button:hover, a.button:hover {
            transform: translateY(-1px);
        }

        .page {
            min-height: 100vh;
            padding: 22px;
        }

        .shell {
            position: relative;
            min-height: calc(100vh - 44px);
            border-radius: 28px;
            border: 1px solid rgba(96, 165, 250, 0.16);
            background: linear-gradient(180deg, rgba(5, 12, 24, 0.92), rgba(3, 8, 18, 0.98));
            box-shadow: var(--shadow);
            overflow: hidden;
        }

        .shell::before {
            content: "";
            position: absolute;
            inset: 0;
            pointer-events: none;
            background:
                radial-gradient(540px 320px at 10% 0%, rgba(34, 211, 238, 0.08), transparent 70%),
                radial-gradient(460px 280px at 92% 10%, rgba(167, 139, 250, 0.08), transparent 72%);
        }

        .topbar {
            position: sticky;
            top: 0;
            z-index: 10;
            display: flex;
            justify-content: space-between;
            align-items: center;
            gap: 18px;
            padding: 20px 26px;
            border-bottom: 1px solid rgba(96, 165, 250, 0.1);
            background: linear-gradient(180deg, rgba(5, 12, 24, 0.94), rgba(5, 12, 24, 0.7));
            backdrop-filter: blur(18px);
        }

        .brand {
            display: flex;
            align-items: center;
            gap: 16px;
            min-width: 0;
        }

        .brand-mark {
            width: 52px;
            height: 52px;
            border-radius: 18px;
            border: 1px solid rgba(34, 211, 238, 0.28);
            background:
                linear-gradient(135deg, rgba(34, 211, 238, 0.18), rgba(96, 165, 250, 0.04)),
                rgba(8, 21, 38, 0.94);
            display: grid;
            place-items: center;
            font-family: "Orbitron", sans-serif;
            font-size: 17px;
            font-weight: 800;
            color: var(--cyan);
            box-shadow: inset 0 0 0 1px rgba(34, 211, 238, 0.08);
            overflow: hidden;
        }

        .brand-mark img {
            width: 100%;
            height: 100%;
            object-fit: cover;
        }

        .brand-copy {
            min-width: 0;
        }

        .brand-kicker {
            color: var(--cyan);
            font-size: 11px;
            letter-spacing: 0.28em;
            text-transform: uppercase;
            margin-bottom: 6px;
        }

        .brand-title {
            margin: 0;
            font-family: "Orbitron", sans-serif;
            font-size: clamp(22px, 3vw, 36px);
            letter-spacing: 0.04em;
        }

        .brand-subtitle {
            margin: 6px 0 0;
            color: var(--muted);
            font-size: 14px;
        }

        .topbar-actions {
            display: flex;
            align-items: center;
            gap: 10px;
            flex-wrap: wrap;
            justify-content: flex-end;
        }

        .chip {
            display: inline-flex;
            align-items: center;
            gap: 8px;
            padding: 9px 14px;
            border-radius: 999px;
            border: 1px solid rgba(96, 165, 250, 0.16);
            background: rgba(8, 21, 38, 0.86);
            color: var(--muted);
            font-size: 13px;
        }

        .chip strong {
            color: var(--text);
            font-family: "IBM Plex Mono", monospace;
            font-weight: 600;
        }

        .chip[data-tone="ok"] {
            border-color: rgba(34, 197, 94, 0.28);
            color: #B7F7C5;
        }

        .chip[data-tone="warn"] {
            border-color: rgba(245, 158, 11, 0.28);
            color: #FDE68A;
        }

        .chip[data-tone="bad"] {
            border-color: rgba(251, 113, 133, 0.32);
            color: #FFC1CB;
        }

        .btn-primary {
            background: linear-gradient(135deg, rgba(34, 211, 238, 0.88), rgba(96, 165, 250, 0.92));
            color: #03101f;
            box-shadow: 0 14px 34px rgba(34, 211, 238, 0.22);
        }

        .btn-ghost {
            border: 1px solid rgba(96, 165, 250, 0.18);
            background: rgba(8, 21, 38, 0.72);
            color: var(--text);
        }

        .dashboard {
            position: relative;
            padding: 26px;
            display: grid;
            gap: 22px;
        }

        .hero {
            display: grid;
            grid-template-columns: minmax(340px, 1.35fr) minmax(320px, 0.9fr);
            gap: 22px;
        }

        .hero-panel, .panel {
            position: relative;
            border-radius: var(--radius-xl);
            border: 1px solid rgba(96, 165, 250, 0.14);
            background: linear-gradient(180deg, rgba(8, 21, 38, 0.9), rgba(4, 12, 24, 0.94));
            overflow: hidden;
        }

        .hero-panel::before, .panel::before {
            content: "";
            position: absolute;
            inset: 0;
            pointer-events: none;
            background: linear-gradient(135deg, rgba(34, 211, 238, 0.05), transparent 34%, transparent 72%, rgba(96, 165, 250, 0.04));
        }

        .hero-main {
            padding: 28px;
            display: grid;
            gap: 18px;
            min-height: 300px;
        }

        .hero-main h2 {
            margin: 0;
            font-family: "Orbitron", sans-serif;
            font-size: clamp(24px, 3vw, 44px);
            line-height: 1.1;
            letter-spacing: 0.04em;
            max-width: 12ch;
        }

        .hero-main p {
            margin: 0;
            color: var(--muted);
            font-size: 15px;
            max-width: 74ch;
        }

        .hero-badges {
            display: flex;
            flex-wrap: wrap;
            gap: 10px;
        }

        .hero-radar {
            margin-top: auto;
            display: grid;
            grid-template-columns: repeat(3, minmax(0, 1fr));
            gap: 14px;
        }

        .focus-card {
            padding: 16px;
            border-radius: 18px;
            border: 1px solid rgba(96, 165, 250, 0.12);
            background: rgba(3, 10, 22, 0.8);
        }

        .focus-card span {
            display: block;
            color: var(--muted);
            font-size: 11px;
            letter-spacing: 0.18em;
            text-transform: uppercase;
            margin-bottom: 10px;
        }

        .focus-card strong {
            display: block;
            font-family: "Orbitron", sans-serif;
            font-size: 24px;
            margin-bottom: 8px;
        }

        .focus-card small {
            display: block;
            color: var(--muted);
            font-size: 12px;
        }

        .signal-panel {
            padding: 22px;
            display: grid;
            gap: 14px;
            align-content: start;
        }

        .panel-kicker {
            color: var(--cyan);
            font-size: 11px;
            letter-spacing: 0.24em;
            text-transform: uppercase;
            margin-bottom: 8px;
        }

        .panel-title {
            margin: 0;
            font-size: 20px;
            font-weight: 700;
        }

        .panel-subtitle {
            margin: 0;
            color: var(--muted);
            font-size: 14px;
        }

        .signal-grid {
            display: grid;
            grid-template-columns: repeat(2, minmax(0, 1fr));
            gap: 12px;
        }

        .signal-tile {
            border-radius: 18px;
            border: 1px solid rgba(96, 165, 250, 0.12);
            background: rgba(4, 12, 24, 0.86);
            padding: 16px;
        }

        .signal-tile span {
            display: block;
            color: var(--muted);
            font-size: 12px;
            margin-bottom: 10px;
        }

        .signal-tile strong {
            display: block;
            font-family: "Orbitron", sans-serif;
            font-size: 26px;
            line-height: 1.1;
        }

        .signal-tile small {
            display: block;
            margin-top: 8px;
            color: var(--muted);
            font-size: 12px;
        }

        .metrics-grid {
            display: grid;
            grid-template-columns: repeat(6, minmax(0, 1fr));
            gap: 14px;
        }

        .metric-card {
            position: relative;
            overflow: hidden;
            border-radius: 20px;
            padding: 18px;
            border: 1px solid rgba(96, 165, 250, 0.12);
            background: linear-gradient(180deg, rgba(8, 21, 38, 0.92), rgba(4, 12, 24, 0.96));
        }

        .metric-card::after {
            content: "";
            position: absolute;
            right: -24px;
            top: -24px;
            width: 92px;
            height: 92px;
            border-radius: 999px;
            background: radial-gradient(circle, rgba(255,255,255,0.18), transparent 72%);
            opacity: 0.6;
        }

        .metric-card span {
            display: block;
            color: var(--muted);
            font-size: 12px;
            margin-bottom: 12px;
        }

        .metric-card strong {
            display: block;
            font-family: "Orbitron", sans-serif;
            font-size: 28px;
            line-height: 1.05;
        }

        .metric-card small {
            display: block;
            margin-top: 10px;
            color: var(--muted);
            font-size: 12px;
        }

        .metric-card[data-tone="cyan"] { box-shadow: inset 0 0 0 1px rgba(34, 211, 238, 0.06); }
        .metric-card[data-tone="teal"] { box-shadow: inset 0 0 0 1px rgba(20, 184, 166, 0.06); }
        .metric-card[data-tone="amber"] { box-shadow: inset 0 0 0 1px rgba(245, 158, 11, 0.06); }
        .metric-card[data-tone="rose"] { box-shadow: inset 0 0 0 1px rgba(251, 113, 133, 0.06); }
        .metric-card[data-tone="blue"] { box-shadow: inset 0 0 0 1px rgba(96, 165, 250, 0.06); }
        .metric-card[data-tone="violet"] { box-shadow: inset 0 0 0 1px rgba(167, 139, 250, 0.06); }

        .main-grid {
            display: grid;
            grid-template-columns: minmax(0, 1.45fr) minmax(340px, 0.85fr);
            gap: 22px;
            align-items: start;
        }

        .stack {
            display: grid;
            gap: 22px;
        }

        .panel-head {
            display: flex;
            justify-content: space-between;
            align-items: flex-start;
            gap: 14px;
            margin-bottom: 18px;
        }

        .panel-head h3 {
            margin: 4px 0 0;
            font-size: 22px;
        }

        .panel-head p {
            margin: 6px 0 0;
            color: var(--muted);
            font-size: 13px;
        }

        .panel-inline-stats {
            display: flex;
            gap: 10px;
            flex-wrap: wrap;
        }

        .inline-stat {
            border-radius: 16px;
            border: 1px solid rgba(96, 165, 250, 0.12);
            padding: 10px 12px;
            background: rgba(3, 10, 22, 0.8);
            min-width: 112px;
        }

        .inline-stat span {
            display: block;
            color: var(--muted);
            font-size: 11px;
            margin-bottom: 8px;
        }

        .inline-stat strong {
            display: block;
            font-family: "IBM Plex Mono", monospace;
            font-size: 13px;
        }

        .panel-body {
            padding: 22px;
        }

        .trend-grid {
            display: grid;
            grid-template-columns: repeat(7, minmax(0, 1fr));
            gap: 12px;
            align-items: end;
            min-height: 220px;
        }

        .trend-col {
            display: grid;
            gap: 10px;
            justify-items: center;
        }

        .trend-stack {
            position: relative;
            width: min(72px, 100%);
            height: 172px;
            border-radius: 18px;
            border: 1px solid rgba(96, 165, 250, 0.12);
            background: linear-gradient(180deg, rgba(7, 17, 31, 0.74), rgba(3, 8, 18, 0.92));
            display: flex;
            flex-direction: column;
            justify-content: flex-end;
            gap: 2px;
            padding: 10px;
        }

        .trend-bar {
            width: 100%;
            border-radius: 12px;
            min-height: 3px;
        }

        .trend-bar.download {
            background: linear-gradient(180deg, rgba(34, 211, 238, 0.96), rgba(14, 165, 233, 0.82));
        }

        .trend-bar.upload {
            background: linear-gradient(180deg, rgba(20, 184, 166, 0.95), rgba(34, 197, 94, 0.82));
        }

        .trend-label {
            text-align: center;
        }

        .trend-label strong {
            display: block;
            font-size: 12px;
            color: var(--text);
        }

        .trend-label span {
            display: block;
            margin-top: 4px;
            font-size: 11px;
            color: var(--muted);
        }

        .node-grid {
            display: grid;
            grid-template-columns: repeat(2, minmax(0, 1fr));
            gap: 14px;
        }

        .node-card {
            border-radius: 22px;
            border: 1px solid rgba(96, 165, 250, 0.12);
            background: rgba(3, 10, 22, 0.84);
            padding: 18px;
            display: grid;
            gap: 14px;
        }

        .node-head {
            display: flex;
            justify-content: space-between;
            gap: 12px;
            align-items: flex-start;
        }

        .node-head h4 {
            margin: 0 0 6px;
            font-size: 18px;
        }

        .node-subline, .node-owner, .feed-meta, .feed-sub {
            color: var(--muted);
            font-size: 12px;
        }

        .node-badges, .feed-top {
            display: flex;
            align-items: center;
            gap: 8px;
            flex-wrap: wrap;
        }

        .status-badge {
            display: inline-flex;
            align-items: center;
            border-radius: 999px;
            padding: 6px 10px;
            font-size: 11px;
            font-weight: 600;
            border: 1px solid rgba(96, 165, 250, 0.18);
            background: rgba(8, 21, 38, 0.82);
            color: var(--text);
        }

        .status-badge[data-tone="ok"] {
            border-color: rgba(34, 197, 94, 0.28);
            color: #B7F7C5;
        }

        .status-badge[data-tone="warn"] {
            border-color: rgba(245, 158, 11, 0.32);
            color: #FDE68A;
        }

        .status-badge[data-tone="bad"] {
            border-color: rgba(251, 113, 133, 0.34);
            color: #FFC1CB;
        }

        .status-badge[data-tone="violet"] {
            border-color: rgba(167, 139, 250, 0.34);
            color: #DDD6FE;
        }

        .status-badge[data-tone="cyan"] {
            border-color: rgba(34, 211, 238, 0.34);
            color: #CFFAFE;
        }

        .node-meter {
            display: grid;
            gap: 8px;
        }

        .node-meter-top, .node-meter-bottom {
            display: flex;
            justify-content: space-between;
            gap: 12px;
            font-size: 12px;
            color: var(--muted);
        }

        .meter {
            height: 10px;
            border-radius: 999px;
            background: rgba(148, 163, 184, 0.14);
            overflow: hidden;
        }

        .meter > span {
            display: block;
            height: 100%;
            border-radius: inherit;
            background: linear-gradient(90deg, rgba(34, 211, 238, 0.92), rgba(96, 165, 250, 0.92));
        }

        .meter.warn > span {
            background: linear-gradient(90deg, rgba(245, 158, 11, 0.94), rgba(249, 115, 22, 0.94));
        }

        .meter.bad > span {
            background: linear-gradient(90deg, rgba(251, 113, 133, 0.96), rgba(239, 68, 68, 0.94));
        }

        .node-stats {
            display: grid;
            grid-template-columns: repeat(2, minmax(0, 1fr));
            gap: 10px;
        }

        .node-stat {
            border-radius: 16px;
            border: 1px solid rgba(96, 165, 250, 0.1);
            padding: 10px 12px;
            background: rgba(6, 17, 31, 0.84);
        }

        .node-stat span {
            display: block;
            color: var(--muted);
            font-size: 11px;
            margin-bottom: 8px;
        }

        .node-stat strong {
            display: block;
            font-family: "IBM Plex Mono", monospace;
            font-size: 13px;
        }

        .feed-list {
            display: grid;
            gap: 12px;
        }

        .feed-item {
            border-radius: 18px;
            border: 1px solid rgba(96, 165, 250, 0.1);
            background: rgba(3, 10, 22, 0.84);
            padding: 14px;
            display: grid;
            gap: 8px;
        }

        .feed-top strong {
            font-size: 14px;
        }

        .bars {
            display: grid;
            gap: 12px;
        }

        .bar-row {
            display: grid;
            gap: 8px;
        }

        .bar-meta {
            display: flex;
            justify-content: space-between;
            gap: 12px;
            font-size: 12px;
        }

        .bar-meta span {
            color: var(--muted);
        }

        .bar-meta strong {
            font-family: "IBM Plex Mono", monospace;
            font-size: 12px;
        }

        .bar-track {
            height: 10px;
            border-radius: 999px;
            background: rgba(148, 163, 184, 0.14);
            overflow: hidden;
        }

        .bar-track > span {
            display: block;
            height: 100%;
            border-radius: inherit;
            background: linear-gradient(90deg, rgba(34, 211, 238, 0.92), rgba(96, 165, 250, 0.92));
        }

        .bar-track[data-tone="violet"] > span {
            background: linear-gradient(90deg, rgba(167, 139, 250, 0.92), rgba(96, 165, 250, 0.92));
        }

        .bar-track[data-tone="amber"] > span {
            background: linear-gradient(90deg, rgba(245, 158, 11, 0.94), rgba(249, 115, 22, 0.92));
        }

        .bar-track[data-tone="teal"] > span {
            background: linear-gradient(90deg, rgba(20, 184, 166, 0.94), rgba(34, 197, 94, 0.92));
        }

        .split-grid {
            display: grid;
            grid-template-columns: repeat(2, minmax(0, 1fr));
            gap: 16px;
        }

        .ops-theater .panel-body {
            display: grid;
            gap: 20px;
        }

        .ops-theater-grid {
            display: grid;
            grid-template-columns: minmax(260px, 0.82fr) minmax(520px, 1.68fr) minmax(290px, 0.94fr);
            gap: 18px;
            align-items: stretch;
        }

        .ops-map-side {
            display: grid;
            gap: 14px;
            align-content: start;
        }

        .ops-side-section {
            border-radius: 22px;
            border: 1px solid rgba(96, 165, 250, 0.12);
            background:
                linear-gradient(180deg, rgba(8, 21, 38, 0.86), rgba(3, 10, 22, 0.9)),
                radial-gradient(circle at top left, rgba(34, 211, 238, 0.08), transparent 38%);
            padding: 16px;
            display: grid;
            gap: 12px;
        }

        .ops-side-title {
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 10px;
            font-size: 13px;
            font-weight: 600;
            letter-spacing: 0.12em;
            text-transform: uppercase;
            color: #D8EAFE;
        }

        .ops-side-title small {
            color: var(--muted);
            font-size: 11px;
            letter-spacing: 0.08em;
            font-weight: 500;
        }

        .ops-brief-grid {
            display: grid;
            grid-template-columns: repeat(2, minmax(0, 1fr));
            gap: 12px;
        }

        .ops-brief-card {
            min-height: 106px;
            border-radius: 18px;
            border: 1px solid rgba(96, 165, 250, 0.12);
            background: rgba(4, 12, 24, 0.82);
            padding: 14px;
            display: grid;
            gap: 8px;
            align-content: start;
        }

        .ops-brief-card span {
            color: var(--muted);
            font-size: 11px;
            letter-spacing: 0.14em;
            text-transform: uppercase;
        }

        .ops-brief-card strong {
            font-family: "Orbitron", sans-serif;
            font-size: 24px;
            line-height: 1.05;
        }

        .ops-brief-card small {
            color: var(--muted);
            font-size: 12px;
        }

        .ops-region-list {
            display: grid;
            gap: 10px;
        }

        .ops-region-card {
            border-radius: 16px;
            border: 1px solid rgba(96, 165, 250, 0.12);
            background: rgba(3, 10, 22, 0.84);
            padding: 12px 14px;
            display: grid;
            gap: 8px;
        }

        .ops-region-head {
            display: flex;
            justify-content: space-between;
            gap: 10px;
            align-items: center;
        }

        .ops-region-head strong {
            font-size: 14px;
        }

        .ops-region-head span {
            color: var(--muted);
            font-size: 11px;
            letter-spacing: 0.08em;
            text-transform: uppercase;
        }

        .ops-region-meta {
            display: flex;
            justify-content: space-between;
            gap: 10px;
            color: var(--muted);
            font-size: 12px;
        }

        .ops-map-shell {
            position: relative;
            border-radius: 26px;
            border: 1px solid rgba(96, 165, 250, 0.16);
            background:
                radial-gradient(circle at 50% 14%, rgba(34, 211, 238, 0.08), transparent 34%),
                linear-gradient(180deg, rgba(5, 12, 24, 0.96), rgba(3, 8, 18, 0.98));
            box-shadow:
                inset 0 0 0 1px rgba(34, 211, 238, 0.04),
                0 24px 54px rgba(2, 6, 23, 0.34);
            overflow: hidden;
        }

        .ops-map-stage {
            position: relative;
            min-height: 580px;
            padding: 18px;
        }

        .ops-map-canvas {
            position: absolute;
            inset: 18px;
            z-index: 1;
        }

        .ops-map-shell::before {
            content: "";
            position: absolute;
            inset: 0;
            pointer-events: none;
            background:
                linear-gradient(180deg, rgba(34, 211, 238, 0.04), transparent 16%),
                radial-gradient(circle at center, rgba(34, 211, 238, 0.05), transparent 44%);
        }

        .ops-map-legend {
            position: absolute;
            left: 18px;
            right: 18px;
            bottom: 18px;
            z-index: 3;
            display: flex;
            justify-content: space-between;
            gap: 12px;
            flex-wrap: wrap;
        }

        .ops-legend-strip,
        .ops-map-summary {
            display: inline-flex;
            align-items: center;
            gap: 10px;
            flex-wrap: wrap;
            padding: 10px 12px;
            border-radius: 999px;
            border: 1px solid rgba(96, 165, 250, 0.14);
            background: rgba(5, 12, 24, 0.86);
            backdrop-filter: blur(8px);
            color: var(--muted);
            font-size: 12px;
        }

        .ops-legend-strip span::before {
            content: "";
            display: inline-block;
            width: 8px;
            height: 8px;
            margin-right: 6px;
            border-radius: 999px;
            vertical-align: middle;
            background: rgba(34, 211, 238, 0.9);
            box-shadow: 0 0 12px rgba(34, 211, 238, 0.28);
        }

        .ops-legend-strip span:nth-child(2)::before {
            background: rgba(245, 158, 11, 0.9);
            box-shadow: 0 0 12px rgba(245, 158, 11, 0.28);
        }

        .ops-legend-strip span:nth-child(3)::before {
            background: rgba(251, 113, 133, 0.92);
            box-shadow: 0 0 12px rgba(251, 113, 133, 0.34);
        }

        .ops-map-summary strong {
            color: var(--text);
            font-family: "IBM Plex Mono", monospace;
        }

        .ops-alert-card strong,
        .ops-watch-card strong,
        .ops-ticker-card strong {
            display: block;
            font-size: 13px;
            color: #F8FBFF;
        }

        .ops-alert-card small,
        .ops-watch-card small,
        .ops-ticker-card small {
            display: block;
            margin-top: 4px;
            color: var(--muted);
            font-size: 11px;
        }

        .ops-alert-list,
        .ops-watch-list {
            display: grid;
            gap: 10px;
        }

        .ops-alert-card,
        .ops-watch-card {
            border-radius: 16px;
            border: 1px solid rgba(96, 165, 250, 0.12);
            background: rgba(3, 10, 22, 0.84);
            padding: 12px;
            display: grid;
            gap: 8px;
        }

        .ops-alert-head,
        .ops-watch-head {
            display: flex;
            justify-content: space-between;
            gap: 10px;
            align-items: flex-start;
        }

        .ops-watch-metrics {
            display: flex;
            justify-content: space-between;
            gap: 10px;
            color: var(--muted);
            font-size: 11px;
        }

        .ops-sparkline-wrap {
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 10px;
        }

        .ops-sparkline {
            width: 118px;
            height: 34px;
            flex: 0 0 auto;
        }

        .ops-sparkline-empty {
            color: var(--muted);
            font-size: 11px;
        }

        .ops-map-ticker {
            display: grid;
            grid-template-columns: repeat(6, minmax(0, 1fr));
            gap: 12px;
        }

        .ops-ticker-card {
            border-radius: 18px;
            border: 1px solid rgba(96, 165, 250, 0.12);
            background: rgba(3, 10, 22, 0.84);
            padding: 14px;
            display: grid;
            gap: 8px;
            min-height: 132px;
        }

        .ops-ticker-card p {
            margin: 0;
            color: var(--muted);
            font-size: 12px;
            line-height: 1.5;
        }

        .empty {
            padding: 26px 18px;
            border-radius: 18px;
            border: 1px dashed rgba(96, 165, 250, 0.2);
            color: var(--muted);
            text-align: center;
            font-size: 14px;
            background: rgba(6, 17, 31, 0.5);
        }

        .locked {
            min-height: calc(100vh - 112px);
            display: grid;
            place-items: center;
            padding: 26px;
        }

        .locked-card {
            width: min(720px, 100%);
            border-radius: 30px;
            border: 1px solid rgba(96, 165, 250, 0.16);
            background: linear-gradient(180deg, rgba(8, 21, 38, 0.94), rgba(4, 12, 24, 0.98));
            box-shadow: var(--shadow);
            padding: 34px;
            display: grid;
            gap: 18px;
        }

        .locked-card h2 {
            margin: 0;
            font-family: "Orbitron", sans-serif;
            font-size: clamp(28px, 4vw, 46px);
            line-height: 1.08;
        }

        .locked-card p {
            margin: 0;
            color: var(--muted);
            font-size: 15px;
        }

        .locked-actions {
            display: flex;
            flex-wrap: wrap;
            gap: 12px;
        }

        .mono {
            font-family: "IBM Plex Mono", monospace;
        }

        @media (max-width: 1380px) {
            .metrics-grid {
                grid-template-columns: repeat(3, minmax(0, 1fr));
            }

            .ops-theater-grid {
                grid-template-columns: minmax(260px, 0.88fr) minmax(0, 1.35fr);
            }

            .ops-map-side:last-child {
                grid-column: 1 / -1;
                grid-template-columns: repeat(2, minmax(0, 1fr));
                align-items: start;
            }

            .ops-map-ticker {
                grid-template-columns: repeat(3, minmax(0, 1fr));
            }

            .main-grid {
                grid-template-columns: 1fr;
            }
        }

        @media (max-width: 1180px) {
            .hero {
                grid-template-columns: 1fr;
            }

            .ops-theater-grid,
            .ops-map-side:last-child {
                grid-template-columns: 1fr;
            }

            .node-grid, .split-grid {
                grid-template-columns: 1fr;
            }
        }

        @media (max-width: 860px) {
            .page {
                padding: 10px;
            }

            .topbar {
                padding: 18px;
            }

            .dashboard {
                padding: 18px;
            }

            .metrics-grid {
                grid-template-columns: repeat(2, minmax(0, 1fr));
            }

            .hero-radar {
                grid-template-columns: 1fr;
            }

            .ops-brief-grid,
            .ops-map-ticker {
                grid-template-columns: 1fr;
            }

            .signal-grid {
                grid-template-columns: 1fr;
            }

            .trend-grid {
                grid-template-columns: repeat(4, minmax(0, 1fr));
            }

            .ops-map-stage {
                min-height: 460px;
            }

            .ops-map-legend {
                position: static;
                margin-top: 12px;
            }
        }

        @media (max-width: 560px) {
            .metrics-grid {
                grid-template-columns: 1fr;
            }

            .ops-map-stage {
                min-height: 380px;
                padding: 12px;
            }

            .ops-map-canvas {
                inset: 12px;
            }

            .trend-grid {
                grid-template-columns: repeat(2, minmax(0, 1fr));
            }
        }
    </style>
</head>
<body>
<div class="page">
    <div class="shell">
        <header class="topbar">
            <div class="brand">
                <div class="brand-mark">
                    @if(!empty($logo))
                        <img src="{{ $logo }}" alt="logo">
                    @else
                        NX
                    @endif
                </div>
                <div class="brand-copy">
                    <div class="brand-kicker">Super Admin Internal Command Deck</div>
                    <h1 class="brand-title">NotXboard Ops Atlas</h1>
                    <p class="brand-subtitle">完全独立于公开大屏的超级管理员指挥舱，只读取内部快照与实时系统状态。</p>
                </div>
            </div>
            <div class="topbar-actions">
                <div class="chip" id="sessionChip" data-tone="warn">会话 <strong>未认证</strong></div>
                <div class="chip">刷新倒计时 <strong id="refreshCountdown">--</strong></div>
                <div class="chip">最近快照 <strong id="lastUpdated">--</strong></div>
                <button class="btn-primary" id="refreshBtn" type="button">立即刷新</button>
                <a class="button btn-ghost" href="/{{ $secure_path }}">完整后台</a>
                <a class="button btn-ghost" href="/app/#/admin">前台超管入口</a>
            </div>
        </header>

        <main id="appRoot" class="dashboard"></main>
    </div>
</div>

<script src="/assets/world-map/echarts.min.js?v={{ $version }}"></script>
<script src="/assets/world-map/world.js?v={{ $version }}"></script>
<script src="/assets/world-map/notxboard-geo-map.js?v={{ $version }}"></script>

<script>
    (function () {
        const APP_ROOT = document.getElementById('appRoot');
        const refreshBtn = document.getElementById('refreshBtn');
        const refreshCountdownEl = document.getElementById('refreshCountdown');
        const lastUpdatedEl = document.getElementById('lastUpdated');
        const sessionChipEl = document.getElementById('sessionChip');
        const securePath = String(@json($secure_path ?? '') || '').replace(/^\/+|\/+$/g, '');
        const sharedAuthKeys = ['auth_data', 'PORTAL_ACCESS_TOKEN', 'Portal_access_token', 'access_token', 'token'];
        const endpoints = {
            me: '/api/v1/user/me',
            snapshot: '/api/v1/admin/command-center'
        };

        const state = {
            token: '',
            me: null,
            refreshTimer: null,
            countdownTimer: null,
            nextRefreshIn: 0
        };

        function escapeHtml(value) {
            return String(value ?? '')
                .replace(/&/g, '&amp;')
                .replace(/</g, '&lt;')
                .replace(/>/g, '&gt;')
                .replace(/"/g, '&quot;')
                .replace(/'/g, '&#39;');
        }

        function asArray(value) {
            return Array.isArray(value) ? value : [];
        }

        function normalizeBearer(token) {
            const clean = String(token || '').trim();
            if (!clean) return '';
            return clean.toLowerCase().startsWith('bearer ') ? clean : `Bearer ${clean}`;
        }

        function readStoredToken() {
            for (const key of sharedAuthKeys) {
                const value = normalizeBearer(localStorage.getItem(key) || '');
                if (value) return value;
            }
            return '';
        }

        function setSessionChip(text, tone = 'warn') {
            sessionChipEl.dataset.tone = tone;
            sessionChipEl.innerHTML = `会话 <strong>${escapeHtml(text)}</strong>`;
        }

        function clearTimers() {
            if (state.refreshTimer) {
                window.clearTimeout(state.refreshTimer);
                state.refreshTimer = null;
            }
            if (state.countdownTimer) {
                window.clearInterval(state.countdownTimer);
                state.countdownTimer = null;
            }
        }

        function formatCompactNumber(value) {
            const number = Number(value || 0);
            return new Intl.NumberFormat('zh-CN', {
                notation: 'compact',
                maximumFractionDigits: number >= 1000 ? 1 : 0
            }).format(number);
        }

        function formatTrafficKb(kb) {
            const value = Number(kb || 0);
            if (value <= 0) return '0 KB';
            const units = ['KB', 'MB', 'GB', 'TB', 'PB'];
            let size = value;
            let index = 0;
            while (size >= 1024 && index < units.length - 1) {
                size /= 1024;
                index += 1;
            }
            const digits = size >= 100 ? 0 : (size >= 10 ? 1 : 2);
            return `${size.toFixed(digits)} ${units[index]}`;
        }

        function formatMoneyCent(value) {
            const amount = Number(value || 0) / 100;
            return new Intl.NumberFormat('zh-CN', {
                style: 'currency',
                currency: 'CNY',
                minimumFractionDigits: 2
            }).format(amount);
        }

        function formatPercent(value) {
            const number = Number(value || 0);
            return `${number.toFixed(number >= 100 ? 0 : 1)}%`;
        }

        function formatRatePerSecond(value) {
            const units = ['B/s', 'KB/s', 'MB/s', 'GB/s', 'TB/s'];
            let size = Number(value || 0);
            let index = 0;
            while (size >= 1024 && index < units.length - 1) {
                size /= 1024;
                index += 1;
            }
            return `${size.toFixed(size >= 100 ? 0 : (size >= 10 ? 1 : 2))} ${units[index]}`;
        }

        function formatLatencyMs(value) {
            if (value === null || value === undefined || value === '') return '--';
            const number = Number(value);
            if (!Number.isFinite(number)) return '--';
            if (number >= 1000) return `${(number / 1000).toFixed(2)} s`;
            return `${Math.round(number)} ms`;
        }

        function formatDurationShort(seconds) {
            const total = Math.max(0, Number(seconds || 0));
            const days = Math.floor(total / 86400);
            const hours = Math.floor((total % 86400) / 3600);
            const minutes = Math.floor((total % 3600) / 60);
            if (days > 0) return `${days}d ${hours}h`;
            if (hours > 0) return `${hours}h ${minutes}m`;
            return `${minutes}m`;
        }

        function formatAnyTimestamp(value) {
            if (!value) return '--';
            let timestamp = Number(value);
            if (!Number.isFinite(timestamp)) return '--';
            if (timestamp < 1e12) timestamp *= 1000;
            const date = new Date(timestamp);
            if (Number.isNaN(date.getTime())) return '--';
            return date.toLocaleString('zh-CN', { hour12: false });
        }

        function badge(label, tone = 'cyan') {
            return `<span class="status-badge" data-tone="${escapeHtml(tone)}">${escapeHtml(label)}</span>`;
        }

        function empty(text) {
            return `<div class="empty">${escapeHtml(text)}</div>`;
        }

        function shortRegionLabel(label, anchorKey) {
            const value = String(label || anchorKey || '').trim();
            if (!value) return anchorKey || '--';
            const cleaned = value.replace(/(特别行政区|自治区|省|市|地区)$/u, '').trim();
            if (cleaned.length <= 6) return cleaned;
            if (anchorKey && anchorKey.length <= 3) return anchorKey;
            return cleaned.slice(0, 6);
        }

        function renderSparkline(samples) {
            const rows = asArray(samples).slice(-12);
            if (!rows.length) return `<div class="ops-sparkline-empty">无 24h 样本</div>`;

            const values = rows.map((item) => {
                if (item?.is_reachable === false) return null;
                const latency = Number(item?.latency_ms);
                return Number.isFinite(latency) && latency >= 0 ? latency : null;
            });

            const finiteValues = values.filter((item) => item !== null);
            if (!finiteValues.length) {
                return `<div class="ops-sparkline-empty">全部离线</div>`;
            }

            const width = 118;
            const height = 34;
            const padding = 3;
            const max = Math.max(1, ...finiteValues);
            const min = Math.min(...finiteValues);
            const points = values.map((value, index) => {
                const x = padding + (index / Math.max(1, values.length - 1)) * (width - padding * 2);
                const y = value === null
                    ? height - padding
                    : height - padding - ((value - min) / Math.max(1, max - min || 1)) * (height - padding * 2);
                return { x, y, gap: value === null };
            });

            let path = '';
            points.forEach((point, index) => {
                if (point.gap) return;
                const needsMove = index === 0 || points[index - 1].gap;
                path += `${needsMove ? 'M' : ' L'}${point.x.toFixed(1)} ${point.y.toFixed(1)}`;
            });

            return `
                <svg class="ops-sparkline" viewBox="0 0 ${width} ${height}" aria-hidden="true">
                    <path d="${path}" fill="none" stroke="rgba(34, 211, 238, 0.94)" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"></path>
                    ${points.filter((point) => !point.gap).map((point) => `
                        <circle cx="${point.x.toFixed(1)}" cy="${point.y.toFixed(1)}" r="1.8" fill="#E0F2FE"></circle>
                    `).join('')}
                </svg>
            `;
        }

        function buildMapPoints(regionDistribution, watchlist, tcpingAlerts) {
            const geoMap = window.NotXboardGeoMap;
            const points = new Map();

            const ensurePoint = (locationKey, coord, label) => {
                if (!locationKey || !coord) return null;
                if (!points.has(locationKey)) {
                    points.set(locationKey, {
                        key: locationKey,
                        label: label || locationKey,
                        shortLabel: shortRegionLabel(label, locationKey),
                        coord: coord,
                        total: 0,
                        online: 0,
                        alerts: 0,
                        riskyNodes: 0,
                        watchScore: 0,
                        protocols: new Set(),
                        samples: [],
                        tooltipNote: '',
                        spotlightNode: '',
                    });
                }
                const point = points.get(locationKey);
                if (label && (point.label === point.key || String(label).length < String(point.label).length)) {
                    point.label = label;
                    point.shortLabel = shortRegionLabel(label, locationKey);
                }
                return point;
            };

            asArray(regionDistribution).forEach((region) => {
                const locationKey = geoMap ? geoMap.resolveLocationKey(region?.location_name, region?.location_code) : '';
                const coord = geoMap ? geoMap.coordFor(region?.location_name, region?.location_code) : null;
                const point = ensurePoint(locationKey, coord, region?.location_name || region?.location_code || locationKey);
                if (!point) return;
                point.total += Math.max(0, Number(region?.total || 0));
                point.online += Math.max(0, Number(region?.online || 0));
            });

            asArray(watchlist).forEach((node) => {
                const locationKey = geoMap ? geoMap.resolveLocationKey(node?.location_name, node?.location_code) : '';
                const coord = geoMap ? geoMap.coordFor(node?.location_name, node?.location_code) : null;
                const point = ensurePoint(locationKey, coord, node?.location_name || node?.location_code || locationKey);
                if (!point) return;
                point.total += point.total > 0 ? 0 : 1;
                point.online += node?.online_status === 'online' && point.online === 0 ? 1 : 0;
                point.riskyNodes += 1;
                point.watchScore = Math.max(point.watchScore, Number(node?.watch_score || 0));
                if (node?.active_alert) {
                    point.alerts += Math.max(1, Number(node?.active_alert?.count || 0));
                    point.tooltipNote = node?.active_alert?.latest_error || node?.tcping_last_error || point.tooltipNote;
                } else if (node?.tcping_last_error && !point.tooltipNote) {
                    point.tooltipNote = node.tcping_last_error;
                }
                if (node?.protocol_label) point.protocols.add(node.protocol_label);
                if (!point.spotlightNode && node?.name) point.spotlightNode = node.name;
                if ((!point.samples || !point.samples.length) && asArray(node?.tcping_samples).length) {
                    point.samples = asArray(node.tcping_samples);
                }
            });

            asArray(tcpingAlerts).forEach((alert) => {
                const locationKey = geoMap ? geoMap.resolveLocationKey(alert?.node_location_name, alert?.node_location_code) : '';
                const coord = geoMap ? geoMap.coordFor(alert?.node_location_name, alert?.node_location_code) : null;
                const point = ensurePoint(locationKey, coord, alert?.node_location_name || locationKey);
                if (!point) return;
                point.alerts += 1;
                if (!point.tooltipNote) {
                    point.tooltipNote = alert?.latest_error || '';
                }
            });

            return Array.from(points.values())
                .map((point) => {
                    const score = point.total * 5
                        + point.online * 3
                        + point.riskyNodes * 9
                        + point.alerts * 18
                        + Math.min(200, point.watchScore) * 0.25;
                    return {
                        ...point,
                        protocols: Array.from(point.protocols),
                        tone: point.alerts > 0 ? 'rose' : (point.riskyNodes > 0 ? 'amber' : 'cyan'),
                        score,
                    };
                })
                .sort((left, right) => right.score - left.score || right.total - left.total);
        }

        function renderOpsGeoMap(points, overview) {
            const geoMap = window.NotXboardGeoMap;
            const chartEl = document.getElementById('opsMapChart');
            if (!geoMap || !chartEl) return;

            const hub = {
                name: '超管指挥中心',
                label: '超管指挥中心',
                coord: [104.1954, 35.8617],
                detail: `覆盖 ${formatCompactNumber(overview?.tcping_enabled_nodes || points.length)} 个监控节点`
            };

            geoMap.render(chartEl, {
                mode: 'command',
                hub: hub,
                points: points.map((point) => ({
                    key: point.key,
                    label: point.label,
                    shortLabel: point.shortLabel,
                    coord: point.coord,
                    count: point.total,
                    online: point.online,
                    tone: point.alerts > 0 ? 'alert' : (point.riskyNodes > 0 ? 'warn' : 'stable'),
                    detail: point.tooltipNote || (point.spotlightNode ? `焦点节点：${point.spotlightNode}` : '当前无额外异常')
                })),
                links: points.map((point) => ({
                    label: `${hub.label} -> ${point.label}`,
                    coords: [hub.coord, point.coord],
                    value: point.total,
                    tone: point.alerts > 0 ? 'alert' : (point.riskyNodes > 0 ? 'warn' : 'stable'),
                    detail: `${point.label} · 在线 ${formatCompactNumber(point.online)} / 节点 ${formatCompactNumber(point.total)}`
                })),
                minPointSize: 14,
                maxPointSize: 30,
                showLabels: true,
                maxLabels: 10
            });
        }

        function renderOpsTheater(regionDistribution, watchlist, tcpingAlerts, auditStream, overview) {
            const points = buildMapPoints(regionDistribution, watchlist, tcpingAlerts).slice(0, 18);
            if (!points.length) return empty('暂无可绘制的节点地区分布数据');

            const maxNodes = Math.max(1, ...points.map((item) => Math.max(1, Number(item.total || 0))));
            const hotRegionCount = points.filter((item) => item.alerts > 0 || item.riskyNodes > 0).length;
            const onlineTotal = points.reduce((sum, item) => sum + Number(item.online || 0), 0);
            const totalRegionalNodes = points.reduce((sum, item) => sum + Math.max(1, Number(item.total || 0)), 0);
            const topRegions = points.slice(0, 4);
            const alertRows = asArray(tcpingAlerts).slice(0, 4);
            const watchRows = asArray(watchlist).slice(0, 4);
            const auditRows = asArray(auditStream).slice(0, 6);

            return `
                <section class="panel ops-theater">
                    <div class="panel-body">
                        <div class="panel-head">
                            <div>
                                <div class="panel-kicker">Operations Theater</div>
                                <h3>全球节点态势图</h3>
                                <p>把地区热度、TCPing 告警、风险节点和审计流集中在一个战区视图里，不再只是通用卡片列表。</p>
                            </div>
                            <div class="panel-inline-stats">
                                <div class="inline-stat">
                                    <span>地图覆盖</span>
                                    <strong>${escapeHtml(formatCompactNumber(points.length))} 区域</strong>
                                </div>
                                <div class="inline-stat">
                                    <span>热点区域</span>
                                    <strong>${escapeHtml(formatCompactNumber(hotRegionCount))}</strong>
                                </div>
                                <div class="inline-stat">
                                    <span>在线密度</span>
                                    <strong>${escapeHtml(formatPercent(totalRegionalNodes > 0 ? (onlineTotal / totalRegionalNodes) * 100 : 0))}</strong>
                                </div>
                            </div>
                        </div>

                        <div class="ops-theater-grid">
                            <aside class="ops-map-side">
                                <div class="ops-side-section">
                                    <div class="ops-side-title">战区摘要 <small>REGION SNAPSHOT</small></div>
                                    <div class="ops-brief-grid">
                                        <article class="ops-brief-card">
                                            <span>活跃地区</span>
                                            <strong>${escapeHtml(formatCompactNumber(points.length))}</strong>
                                            <small>已解析到世界地图锚点的落地区域</small>
                                        </article>
                                        <article class="ops-brief-card">
                                            <span>热点战区</span>
                                            <strong>${escapeHtml(formatCompactNumber(hotRegionCount))}</strong>
                                            <small>存在告警或高风险节点的地区</small>
                                        </article>
                                        <article class="ops-brief-card">
                                            <span>在线节点</span>
                                            <strong>${escapeHtml(formatCompactNumber(onlineTotal))}</strong>
                                            <small>按地区聚合后的在线节点总数</small>
                                        </article>
                                        <article class="ops-brief-card">
                                            <span>节点总量</span>
                                            <strong>${escapeHtml(formatCompactNumber(overview?.total_nodes || totalRegionalNodes))}</strong>
                                            <small>来自超级管理员快照的全局节点数</small>
                                        </article>
                                    </div>
                                </div>

                                <div class="ops-side-section">
                                    <div class="ops-side-title">地区热区 <small>TOP REGIONS</small></div>
                                    <div class="ops-region-list">
                                        ${topRegions.map((point) => `
                                            <article class="ops-region-card">
                                                <div class="ops-region-head">
                                                    <strong>${escapeHtml(point.label || point.key)}</strong>
                                                    ${badge(point.alerts > 0 ? `告警 ${point.alerts}` : (point.riskyNodes > 0 ? `关注 ${point.riskyNodes}` : '稳定'), point.tone)}
                                                </div>
                                                <div class="ops-region-meta">
                                                    <span>节点 ${escapeHtml(formatCompactNumber(point.total || 0))}</span>
                                                    <span>在线 ${escapeHtml(formatCompactNumber(point.online || 0))}</span>
                                                </div>
                                                <div class="bar-track" data-tone="${escapeHtml(point.tone)}">
                                                    <span style="width:${Math.max(8, (Math.max(1, Number(point.total || 0)) / maxNodes) * 100).toFixed(2)}%"></span>
                                                </div>
                                                <div class="feed-sub">${escapeHtml(point.spotlightNode ? `焦点节点：${point.spotlightNode}` : (point.tooltipNote || '当前没有额外异常说明'))}</div>
                                            </article>
                                        `).join('')}
                                    </div>
                                </div>
                            </aside>

                            <div class="ops-map-shell">
                                <div class="ops-map-stage">
                                    <div class="ops-map-canvas" id="opsMapChart"></div>
                                    <div class="ops-map-legend">
                                        <div class="ops-legend-strip">
                                            <span>稳定区域</span>
                                            <span>高风险区域</span>
                                            <span>活跃告警区域</span>
                                        </div>
                                        <div class="ops-map-summary">
                                            <span>24h 审计事件 <strong>${escapeHtml(formatCompactNumber(auditRows.length))}</strong></span>
                                            <span>活跃告警 <strong>${escapeHtml(formatCompactNumber(asArray(tcpingAlerts).length))}</strong></span>
                                            <span>风险节点 <strong>${escapeHtml(formatCompactNumber(asArray(watchlist).length))}</strong></span>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <aside class="ops-map-side">
                                <div class="ops-side-section">
                                    <div class="ops-side-title">实时异常 <small>TCPING ALERTS</small></div>
                                    ${alertRows.length ? `
                                        <div class="ops-alert-list">
                                            ${alertRows.map((alert) => `
                                                <article class="ops-alert-card">
                                                    <div class="ops-alert-head">
                                                        <div>
                                                            <strong>${escapeHtml(alert?.node_name || '-')}</strong>
                                                            <small>${escapeHtml(alert?.node_location_name || '-')}</small>
                                                        </div>
                                                        ${badge(`持续 ${formatDurationShort(alert?.duration_seconds || 0)}`, 'rose')}
                                                    </div>
                                                    <div class="feed-meta">${escapeHtml(alert?.user_email || '-')} · ${escapeHtml(alert?.node_protocol || '-')}</div>
                                                    <div class="feed-sub">${escapeHtml(alert?.latest_error || '当前无异常说明')}</div>
                                                </article>
                                            `).join('')}
                                        </div>
                                    ` : empty('暂无活跃 TCPing 告警')}
                                </div>

                                <div class="ops-side-section">
                                    <div class="ops-side-title">高危节点 <small>WATCH SCORE</small></div>
                                    ${watchRows.length ? `
                                        <div class="ops-watch-list">
                                            ${watchRows.map((node) => `
                                                <article class="ops-watch-card">
                                                    <div class="ops-watch-head">
                                                        <div>
                                                            <strong>${escapeHtml(node?.name || '-')}</strong>
                                                            <small>${escapeHtml(node?.location_name || '-')} · ${escapeHtml(node?.protocol_label || node?.protocol || '-')}</small>
                                                        </div>
                                                        ${badge(node?.active_alert ? `告警 ${Number(node?.active_alert?.count || 1)}` : `分数 ${formatCompactNumber(node?.watch_score || 0)}`, node?.active_alert ? 'rose' : 'amber')}
                                                    </div>
                                                    <div class="ops-watch-metrics">
                                                        <span>用户 ${escapeHtml(formatCompactNumber(node?.online_users || 0))}</span>
                                                        <span>连接 ${escapeHtml(formatCompactNumber(node?.active_connections || 0))}</span>
                                                        <span>延迟 ${escapeHtml(formatLatencyMs(node?.tcping_last_latency_ms))}</span>
                                                    </div>
                                                    <div class="ops-sparkline-wrap">
                                                        ${renderSparkline(node?.tcping_samples)}
                                                        <div class="feed-sub">${escapeHtml(node?.tcping_last_error || node?.active_alert?.latest_error || '最近 24h 未记录额外异常')}</div>
                                                    </div>
                                                </article>
                                            `).join('')}
                                        </div>
                                    ` : empty('暂无高危节点')}
                                </div>
                            </aside>
                        </div>

                        <div class="ops-map-ticker">
                            ${auditRows.length ? auditRows.map((log) => `
                                <article class="ops-ticker-card">
                                    <strong>${escapeHtml(log?.user_email || '-')}</strong>
                                    <small>${escapeHtml(formatAnyTimestamp(log?.created_at))}</small>
                                    <p>${escapeHtml(log?.target_domain || log?.target_protocol || '未提供目标')}</p>
                                    <div class="feed-meta">${escapeHtml(log?.node_name || '-')} · ${escapeHtml(log?.node_location_name || '-')}</div>
                                    <div class="feed-sub">${escapeHtml(log?.ip_address || '-')} · ${escapeHtml(log?.action_taken || 'logged')}</div>
                                </article>
                            `).join('') : empty('当前没有审计事件流')}
                        </div>
                    </div>
                </section>
            `;
        }

        function renderBars(items, options = {}) {
            const rows = asArray(items);
            if (!rows.length) return empty(options.emptyText || '暂无数据');
            const max = Math.max(1, ...rows.map((item) => Number(item?.value || 0)));
            const tone = options.tone || 'cyan';
            return `
                <div class="bars">
                    ${rows.map((item) => {
                        const value = Number(item?.value || 0);
                        const width = Math.max(4, (value / max) * 100);
                        const subtitle = typeof options.subtitle === 'function' ? options.subtitle(item) : '';
                        const valueText = typeof options.valueFormatter === 'function'
                            ? options.valueFormatter(value, item)
                            : String(value);
                        return `
                            <div class="bar-row">
                                <div class="bar-meta">
                                    <span>${escapeHtml(item?.label || '-')}</span>
                                    <strong>${escapeHtml(valueText)}</strong>
                                </div>
                                <div class="bar-track" data-tone="${escapeHtml(tone)}"><span style="width:${width.toFixed(2)}%"></span></div>
                                ${subtitle ? `<div class="feed-sub">${escapeHtml(subtitle)}</div>` : ''}
                            </div>
                        `;
                    }).join('')}
                </div>
            `;
        }

        function renderTrendChart(points) {
            const rows = asArray(points);
            if (!rows.length) return empty('最近 7 天还没有流量记录');
            const max = Math.max(1, ...rows.map((item) => Number(item?.total_kb || 0)));
            return `
                <div class="trend-grid">
                    ${rows.map((item) => {
                        const upload = Math.max(0, Number(item?.upload_kb || 0));
                        const download = Math.max(0, Number(item?.download_kb || 0));
                        const total = Math.max(0, Number(item?.total_kb || 0));
                        const scaled = Math.max(12, (total / max) * 140);
                        const uploadHeight = total > 0 ? Math.max(3, (upload / total) * scaled) : 3;
                        const downloadHeight = total > 0 ? Math.max(3, (download / total) * scaled) : 3;
                        return `
                            <div class="trend-col">
                                <div class="trend-stack" title="上传 ${escapeHtml(formatTrafficKb(upload))} / 下载 ${escapeHtml(formatTrafficKb(download))}">
                                    <div class="trend-bar upload" style="height:${uploadHeight.toFixed(2)}px"></div>
                                    <div class="trend-bar download" style="height:${downloadHeight.toFixed(2)}px"></div>
                                </div>
                                <div class="trend-label">
                                    <strong>${escapeHtml(String(item?.date || '').slice(5) || '-')}</strong>
                                    <span>${escapeHtml(formatTrafficKb(total))}</span>
                                </div>
                            </div>
                        `;
                    }).join('')}
                </div>
            `;
        }

        function renderWatchlist(nodes) {
            const rows = asArray(nodes).slice(0, 8);
            if (!rows.length) return empty('当前没有节点监控数据');
            return `
                <div class="node-grid">
                    ${rows.map((node) => {
                        const progress = Math.max(0, Math.min(100, Number(node?.traffic_usage_percentage || 0)));
                        const meterTone = progress >= 90 ? 'bad' : (progress >= 70 ? 'warn' : 'ok');
                        const tcpingTone = node?.tcping_status === 'offline'
                            ? 'warn'
                            : (node?.tcping_status === 'online' ? 'ok' : (node?.tcping_status === 'unsupported' ? 'violet' : 'cyan'));
                        return `
                            <article class="node-card">
                                <div class="node-head">
                                    <div>
                                        <h4>${escapeHtml(node?.name || '-')}</h4>
                                        <div class="node-subline">${escapeHtml(node?.host || '-')} : ${escapeHtml(node?.port || '-')} · ${escapeHtml(node?.location_name || '-')}</div>
                                    </div>
                                    <div class="node-badges">
                                        ${badge(node?.online_status === 'online' ? '节点在线' : '节点离线', node?.online_status === 'online' ? 'ok' : 'bad')}
                                        ${badge(node?.protocol_label || node?.protocol || '-', 'cyan')}
                                    </div>
                                </div>
                                <div class="node-owner">${escapeHtml(node?.owner_name || '-')} · ${escapeHtml(node?.owner_email || '-')}</div>
                                <div class="node-badges">
                                    ${badge(node?.tcping_status === 'unsupported'
                                        ? 'TCPing 不支持 UDP 探测'
                                        : (node?.tcping_status === 'offline'
                                            ? 'TCPing 异常'
                                            : (node?.tcping_status === 'online' ? 'TCPing 正常' : 'TCPing 等待')), tcpingTone)}
                                    ${node?.active_alert ? badge(`告警 ${node.active_alert.count || 1}`, 'bad') : ''}
                                </div>
                                <div class="node-meter">
                                    <div class="node-meter-top">
                                        <span>节点总流量</span>
                                        <strong>${node?.traffic_limit_kb > 0
                                            ? `${escapeHtml(formatTrafficKb(node?.traffic_used_kb || 0))} / ${escapeHtml(formatTrafficKb(node?.traffic_limit_kb || 0))}`
                                            : `${escapeHtml(formatTrafficKb(node?.traffic_used_kb || 0))} / 无限`}</strong>
                                    </div>
                                    <div class="meter ${meterTone}"><span style="width:${node?.traffic_limit_kb > 0 ? progress.toFixed(2) : 100}%"></span></div>
                                    <div class="node-meter-bottom">
                                        <span>${node?.traffic_limit_kb > 0 ? `负载 ${formatPercent(progress)}` : '未设置节点总流量上限'}</span>
                                        <strong>${escapeHtml(formatLatencyMs(node?.tcping_last_latency_ms))}</strong>
                                    </div>
                                </div>
                                <div class="node-stats">
                                    <div class="node-stat">
                                        <span>在线用户</span>
                                        <strong>${escapeHtml(formatCompactNumber(node?.online_users || 0))}</strong>
                                    </div>
                                    <div class="node-stat">
                                        <span>活跃连接</span>
                                        <strong>${escapeHtml(formatCompactNumber(node?.active_connections || 0))}</strong>
                                    </div>
                                    <div class="node-stat">
                                        <span>7 天流量</span>
                                        <strong>${escapeHtml(formatTrafficKb(node?.weekly_traffic_kb || 0))}</strong>
                                    </div>
                                    <div class="node-stat">
                                        <span>关注分数</span>
                                        <strong>${escapeHtml(formatCompactNumber(node?.watch_score || 0))}</strong>
                                    </div>
                                </div>
                                <div class="feed-sub">${escapeHtml(node?.active_alert?.latest_error || node?.tcping_last_error || '未检测到最新异常说明')}</div>
                            </article>
                        `;
                    }).join('')}
                </div>
            `;
        }

        function renderFeed(items, mapper, emptyText) {
            const rows = asArray(items);
            if (!rows.length) return empty(emptyText);
            return `<div class="feed-list">${rows.map(mapper).join('')}</div>`;
        }

        function renderDashboard(payload) {
            const overview = payload?.overview || {};
            const system = payload?.system || {};
            const trend = asArray(payload?.traffic_trend);
            const watchlist = asArray(payload?.node_watchlist);
            const topUsers = asArray(payload?.top_users).slice(0, 8);
            const protocolDistribution = asArray(payload?.protocol_distribution).slice(0, 6);
            const regionDistribution = asArray(payload?.region_distribution).slice(0, 6);
            const tickets = asArray(payload?.tickets).slice(0, 5);
            const refunds = asArray(payload?.refunds).slice(0, 5);
            const tcpingAgents = asArray(payload?.tcping_agents).slice(0, 5);
            const tcpingAlerts = asArray(payload?.tcping_alerts).slice(0, 5);
            const auditStream = asArray(payload?.audit_stream).slice(0, 6);

            const totalNodes = Number(overview?.total_nodes || 0);
            const onlineNodes = Number(overview?.online_nodes || 0);
            const nodeOnlineRatio = totalNodes > 0 ? (onlineNodes / totalNodes) * 100 : 0;
            const alertCount = Number(overview?.tcping_alerts_active || 0);
            const metrics = [
                {
                    label: '节点在线率',
                    value: formatPercent(nodeOnlineRatio),
                    meta: `${onlineNodes} / ${totalNodes || 0} 节点在线`,
                    tone: 'cyan'
                },
                {
                    label: '今日流量',
                    value: formatTrafficKb(overview?.traffic_today_kb || 0),
                    meta: `实际用户 ${formatCompactNumber(overview?.traffic_today_unique_users || 0)}`,
                    tone: 'teal'
                },
                {
                    label: '活跃告警',
                    value: formatCompactNumber(alertCount),
                    meta: `覆盖节点 ${formatCompactNumber(overview?.tcping_enabled_nodes || 0)}`,
                    tone: 'rose'
                },
                {
                    label: '探针健康',
                    value: `${formatCompactNumber(overview?.tcping_agents_online || 0)}/${formatCompactNumber(overview?.tcping_agents_total || 0)}`,
                    meta: '独立 TCPing Agent',
                    tone: 'violet'
                },
                {
                    label: '待处理事务',
                    value: `${formatCompactNumber(overview?.open_tickets || 0)} / ${formatCompactNumber(overview?.pending_refunds || 0)}`,
                    meta: '工单 / 退款',
                    tone: 'amber'
                },
                {
                    label: '24h 收入',
                    value: formatMoneyCent(overview?.revenue_24h_amount || 0),
                    meta: `完成订单 ${formatCompactNumber(overview?.completed_orders_24h || 0)}`,
                    tone: 'blue'
                }
            ];

            APP_ROOT.innerHTML = `
                <section class="hero">
                    <article class="hero-panel hero-main">
                        <div>
                            <div class="panel-kicker">Internal Command Center</div>
                            <h2>超级管理员作战视图</h2>
                            <p>这不是公开大屏的换皮版本，而是超级管理员独占的内部指挥界面。节点、用户、退款、探针、审计流都直接按原始数据展示，不做脱敏，也不复用公共概览结构。</p>
                        </div>
                        <div class="hero-badges">
                            ${badge(system?.schedule_ok ? 'Scheduler 正常' : 'Scheduler 异常', system?.schedule_ok ? 'ok' : 'bad')}
                            ${badge(system?.horizon?.available ? (system?.horizon?.ok ? 'Horizon 正常' : 'Horizon 异常') : 'Horizon 未启用', system?.horizon?.available ? (system?.horizon?.ok ? 'ok' : 'warn') : 'cyan')}
                            ${badge(alertCount > 0 ? `活跃告警 ${alertCount}` : '当前无活跃告警', alertCount > 0 ? 'warn' : 'ok')}
                            ${badge(state.me?.email || '-', 'violet')}
                        </div>
                        <div class="hero-radar">
                            <div class="focus-card">
                                <span>Live Throughput</span>
                                <strong>${escapeHtml(formatRatePerSecond(overview?.throughput_download_bps || 0))}</strong>
                                <small>下行主导，上行 ${escapeHtml(formatRatePerSecond(overview?.throughput_upload_bps || 0))}</small>
                            </div>
                            <div class="focus-card">
                                <span>User Footprint</span>
                                <strong>${escapeHtml(formatCompactNumber(overview?.live_users || 0))}</strong>
                                <small>当前仍在产生流量更新的用户数</small>
                            </div>
                            <div class="focus-card">
                                <span>Watch Score</span>
                                <strong>${escapeHtml(formatCompactNumber((watchlist[0] && watchlist[0].watch_score) || 0))}</strong>
                                <small>最高风险节点：${escapeHtml((watchlist[0] && watchlist[0].name) || '无')}</small>
                            </div>
                        </div>
                    </article>

                    <aside class="hero-panel signal-panel">
                        <div>
                            <div class="panel-kicker">Signal Stack</div>
                            <h2 class="panel-title">系统体征</h2>
                            <p class="panel-subtitle">最近一次任务运行、日志压力、探针状态与异常告警在这里做聚合判读。</p>
                        </div>
                        <div class="signal-grid">
                            <div class="signal-tile">
                                <span>计划任务</span>
                                <strong>${escapeHtml(formatAnyTimestamp(system?.schedule_last_runtime))}</strong>
                                <small>${system?.schedule_ok ? '调度链路稳定' : '需要检查 scheduler 容器'}</small>
                            </div>
                            <div class="signal-tile">
                                <span>错误日志 24h</span>
                                <strong>${escapeHtml(formatCompactNumber(system?.logs?.errors_last_24h || 0))}</strong>
                                <small>Warning ${escapeHtml(formatCompactNumber(system?.logs?.warnings_last_24h || 0))}</small>
                            </div>
                            <div class="signal-tile">
                                <span>活跃退款</span>
                                <strong>${escapeHtml(formatCompactNumber(overview?.pending_refunds || 0))}</strong>
                                <small>待处理争议与退款流程</small>
                            </div>
                            <div class="signal-tile">
                                <span>活跃工单</span>
                                <strong>${escapeHtml(formatCompactNumber(overview?.open_tickets || 0))}</strong>
                                <small>需要后台继续跟进</small>
                            </div>
                        </div>
                    </aside>
                </section>

                <section class="metrics-grid">
                    ${metrics.map((item) => `
                        <article class="metric-card" data-tone="${escapeHtml(item.tone)}">
                            <span>${escapeHtml(item.label)}</span>
                            <strong>${escapeHtml(item.value)}</strong>
                            <small>${escapeHtml(item.meta)}</small>
                        </article>
                    `).join('')}
                </section>

                ${renderOpsTheater(regionDistribution, watchlist, tcpingAlerts, auditStream, overview)}

                <section class="main-grid">
                    <div class="stack">
                        <article class="panel">
                            <div class="panel-body">
                                <div class="panel-head">
                                    <div>
                                        <div class="panel-kicker">Traffic Heatfield</div>
                                        <h3>最近 7 天流量走势</h3>
                                        <p>按日查看上传与下载的热度变化，用于判断全站负载波动。</p>
                                    </div>
                                    <div class="panel-inline-stats">
                                        <div class="inline-stat">
                                            <span>今日流量</span>
                                            <strong>${escapeHtml(formatTrafficKb(overview?.traffic_today_kb || 0))}</strong>
                                        </div>
                                        <div class="inline-stat">
                                            <span>实时下载</span>
                                            <strong>${escapeHtml(formatRatePerSecond(overview?.throughput_download_bps || 0))}</strong>
                                        </div>
                                        <div class="inline-stat">
                                            <span>实时上传</span>
                                            <strong>${escapeHtml(formatRatePerSecond(overview?.throughput_upload_bps || 0))}</strong>
                                        </div>
                                    </div>
                                </div>
                                ${renderTrendChart(trend)}
                            </div>
                        </article>

                        <article class="panel">
                            <div class="panel-body">
                                <div class="panel-head">
                                    <div>
                                        <div class="panel-kicker">Node Watchlist</div>
                                        <h3>风险节点墙</h3>
                                        <p>按关注分数排序，优先显示离线、告警、流量压力高的节点。</p>
                                    </div>
                                </div>
                                ${renderWatchlist(watchlist)}
                            </div>
                        </article>

                        <article class="panel">
                            <div class="panel-body">
                                <div class="panel-head">
                                    <div>
                                        <div class="panel-kicker">User Pressure</div>
                                        <h3>高流量用户榜</h3>
                                        <p>直接显示原始用户信息，不做邮箱打码，便于超管排查套餐与节点占用。</p>
                                    </div>
                                </div>
                                ${renderBars(topUsers.map((item) => ({
                                    label: `${item?.display_name || item?.email || '-'} · ${item?.plan_name || '-'}`,
                                    value: item?.traffic_kb || 0,
                                    detail: item?.email || '-'
                                })), {
                                    tone: 'amber',
                                    emptyText: '当前没有用户流量排行',
                                    subtitle: (item) => item?.detail || '',
                                    valueFormatter: (value) => formatTrafficKb(value)
                                })}
                            </div>
                        </article>

                        <article class="panel">
                            <div class="panel-body">
                                <div class="panel-head">
                                    <div>
                                        <div class="panel-kicker">Distribution</div>
                                        <h3>协议与地区分布</h3>
                                        <p>从节点编排视角快速判断协议占比与落地地区热度。</p>
                                    </div>
                                </div>
                                <div class="split-grid">
                                    <div>
                                        ${renderBars(protocolDistribution.map((item) => ({
                                            label: item?.label || item?.protocol || '-',
                                            value: item?.total || 0,
                                            online: item?.online || 0
                                        })), {
                                            tone: 'cyan',
                                            emptyText: '暂无协议分布数据',
                                            subtitle: (item) => `在线 ${formatCompactNumber(item?.online || 0)} 个`,
                                            valueFormatter: (value) => `${formatCompactNumber(value)} 个`
                                        })}
                                    </div>
                                    <div>
                                        ${renderBars(regionDistribution.map((item) => ({
                                            label: item?.location_name || item?.location_code || '-',
                                            value: item?.total || 0,
                                            online: item?.online || 0
                                        })), {
                                            tone: 'violet',
                                            emptyText: '暂无地区分布数据',
                                            subtitle: (item) => `在线 ${formatCompactNumber(item?.online || 0)} 个`,
                                            valueFormatter: (value) => `${formatCompactNumber(value)} 个`
                                        })}
                                    </div>
                                </div>
                            </div>
                        </article>
                    </div>

                    <div class="stack">
                        <article class="panel">
                            <div class="panel-body">
                                <div class="panel-head">
                                    <div>
                                        <div class="panel-kicker">TCPing Alerts</div>
                                        <h3>活动告警</h3>
                                        <p>按最新触发时间展示，优先定位断连与连通性波动。</p>
                                    </div>
                                </div>
                                ${renderFeed(tcpingAlerts, (alert) => `
                                    <div class="feed-item">
                                        <div class="feed-top">
                                            <strong>${escapeHtml(alert?.node_name || '-')}</strong>
                                            ${badge(alert?.status || 'active', 'bad')}
                                        </div>
                                        <div class="feed-meta">${escapeHtml(alert?.user_email || '-')} · ${escapeHtml(alert?.node_location_name || '-')}</div>
                                        <div class="feed-sub">${escapeHtml(formatDurationShort(alert?.duration_seconds || 0))} · ${escapeHtml(alert?.latest_error || '无错误信息')}</div>
                                    </div>
                                `, '暂无活跃 TCPing 告警')}
                            </div>
                        </article>

                        <article class="panel">
                            <div class="panel-body">
                                <div class="panel-head">
                                    <div>
                                        <div class="panel-kicker">Probe Health</div>
                                        <h3>探针健康</h3>
                                        <p>独立于 V2bX 的 TCPing Agent 运行情况。</p>
                                    </div>
                                </div>
                                ${renderFeed(tcpingAgents, (agent) => `
                                    <div class="feed-item">
                                        <div class="feed-top">
                                            <strong>${escapeHtml(agent?.name || '-')}</strong>
                                            ${badge(agent?.is_online ? '在线' : '离线', agent?.is_online ? 'ok' : 'warn')}
                                        </div>
                                        <div class="feed-meta">${escapeHtml(agent?.owner_name || '-')} · ${escapeHtml(agent?.owner_email || '-')}</div>
                                        <div class="feed-sub">心跳 ${escapeHtml(formatAnyTimestamp(agent?.last_heartbeat_at))} · 同步 ${escapeHtml(formatAnyTimestamp(agent?.last_sync_at))}</div>
                                    </div>
                                `, '暂无探针数据')}
                            </div>
                        </article>

                        <article class="panel">
                            <div class="panel-body">
                                <div class="panel-head">
                                    <div>
                                        <div class="panel-kicker">Support Queue</div>
                                        <h3>工单与退款</h3>
                                        <p>同屏查看客服与资金流异常，减少切换页面。</p>
                                    </div>
                                </div>
                                <div class="split-grid">
                                    <div>
                                        ${renderFeed(tickets, (ticket) => `
                                            <div class="feed-item">
                                                <div class="feed-top">
                                                    <strong>${escapeHtml(ticket?.subject || '-')}</strong>
                                                    ${badge(Number(ticket?.status || 0) === 0 ? 'OPEN' : 'CLOSED', Number(ticket?.status || 0) === 0 ? 'warn' : 'cyan')}
                                                </div>
                                                <div class="feed-meta">${escapeHtml(ticket?.user_email || '-')} · ${escapeHtml(ticket?.node_name || '-')}</div>
                                                <div class="feed-sub">更新 ${escapeHtml(formatAnyTimestamp(ticket?.updated_at))} · 负责人 ${escapeHtml(ticket?.assigned_admin_email || '-')}</div>
                                            </div>
                                        `, '当前没有开启工单')}
                                    </div>
                                    <div>
                                        ${renderFeed(refunds, (refund) => `
                                            <div class="feed-item">
                                                <div class="feed-top">
                                                    <strong>${escapeHtml(refund?.user_email || '-')}</strong>
                                                    ${badge(refund?.status || 'pending', refund?.status === 'voting' ? 'warn' : 'amber')}
                                                </div>
                                                <div class="feed-meta">${escapeHtml(refund?.plan_name || '-')} · trade_no ${escapeHtml(refund?.trade_no || '-')}</div>
                                                <div class="feed-sub">申请 ${escapeHtml(formatMoneyCent(refund?.refund_amount || 0))} / 原金额 ${escapeHtml(formatMoneyCent(refund?.gateway_amount || 0))}</div>
                                            </div>
                                        `, '当前没有待处理退款')}
                                    </div>
                                </div>
                            </div>
                        </article>

                        <article class="panel">
                            <div class="panel-body">
                                <div class="panel-head">
                                    <div>
                                        <div class="panel-kicker">Audit Stream</div>
                                        <h3>审计事件流</h3>
                                        <p>最近发生的审计事件，直接暴露目标域名与用户来源信息。</p>
                                    </div>
                                </div>
                                ${renderFeed(auditStream, (log) => `
                                    <div class="feed-item">
                                        <div class="feed-top">
                                            <strong>${escapeHtml(log?.user_email || '-')}</strong>
                                            ${badge(log?.action_taken || 'logged', log?.action_taken === 'blocked' ? 'bad' : (log?.action_taken === 'allowed' ? 'ok' : 'cyan'))}
                                        </div>
                                        <div class="feed-meta">${escapeHtml(log?.node_name || '-')} · ${escapeHtml(log?.node_location_name || '-')} · ${escapeHtml(log?.ip_address || '-')}</div>
                                        <div class="feed-sub">${escapeHtml(log?.target_domain || log?.target_protocol || '未提供目标')} · ${escapeHtml(formatAnyTimestamp(log?.created_at))}</div>
                                    </div>
                                `, '当前没有审计事件')}
                            </div>
                        </article>
                    </div>
                </section>
            `;

            renderOpsGeoMap(points, overview);
        }

        function renderLocked(message) {
            clearTimers();
            refreshCountdownEl.textContent = '--';
            lastUpdatedEl.textContent = '--';
            APP_ROOT.innerHTML = `
                <section class="locked">
                    <article class="locked-card">
                        <div class="panel-kicker">Super Admin Command Center</div>
                        <h2>独立监控大屏已与公开大屏彻底分离</h2>
                        <p>${escapeHtml(message || '未检测到可用的超级管理员登录态。请先登录超级管理员账号，然后再进入本页。')}</p>
                        <p>本页会自动继承前台主题后台与原后台的本地登录态。若你刚完成登录，保持此页打开即可自动接管新的认证状态。</p>
                        <div class="locked-actions">
                            <a class="button btn-primary" href="/app/#/login">去前台登录</a>
                            <a class="button btn-ghost" href="/{{ $secure_path }}">去原后台登录</a>
                        </div>
                        <div class="chip mono" data-tone="warn">后台路径 <strong>/${escapeHtml(securePath || '-')}</strong></div>
                    </article>
                </section>
            `;
        }

        async function fetchJson(url) {
            const token = normalizeBearer(state.token);
            const headers = {
                'Accept': 'application/json'
            };
            if (token) {
                headers.Authorization = token;
            }

            const response = await fetch(url, {
                credentials: 'same-origin',
                headers
            });

            const text = await response.text();
            let json = null;
            try {
                json = text ? JSON.parse(text) : null;
            } catch (_) {
                json = null;
            }

            const message = json?.message || json?.error || `请求失败 (${response.status})`;
            if (response.status === 401) {
                throw new Error('后台登录已失效，请重新登录');
            }
            if (response.status === 403) {
                throw new Error('需要超级管理员权限才能查看本页');
            }
            if (!response.ok) {
                throw new Error(message);
            }
            return json;
        }

        async function loadSession() {
            state.token = readStoredToken();

            try {
                const response = await fetchJson(endpoints.me);
                const me = response?.data || {};
                if (!me?.is_super_admin) {
                    state.me = null;
                    setSessionChip('权限不足', 'bad');
                    renderLocked('当前账号已登录，但不是超级管理员。');
                    return false;
                }

                state.me = me;
                setSessionChip(me.email || 'super-admin', 'ok');
                return true;
            } catch (error) {
                state.me = null;
                setSessionChip('失效', 'bad');
                renderLocked(error.message || '加载登录状态失败');
                return false;
            }
        }

        function scheduleRefresh(seconds) {
            clearTimers();
            const interval = Math.max(5, Math.min(300, Number(seconds || 20)));
            state.nextRefreshIn = interval;
            refreshCountdownEl.textContent = `${interval}s`;

            state.countdownTimer = window.setInterval(() => {
                state.nextRefreshIn = Math.max(0, state.nextRefreshIn - 1);
                refreshCountdownEl.textContent = `${state.nextRefreshIn}s`;
            }, 1000);

            state.refreshTimer = window.setTimeout(() => {
                refreshSnapshot(false).catch(() => {});
            }, interval * 1000);
        }

        async function refreshSnapshot(manual) {
            refreshBtn.disabled = true;
            refreshBtn.textContent = manual ? '刷新中...' : '同步中...';

            const ready = await loadSession();
            if (!ready) {
                refreshBtn.disabled = false;
                refreshBtn.textContent = '立即刷新';
                return;
            }

            try {
                const response = await fetchJson(endpoints.snapshot);
                const payload = response?.data || {};
                renderDashboard(payload);
                lastUpdatedEl.textContent = formatAnyTimestamp(payload?.generated_at || Math.floor(Date.now() / 1000));
                scheduleRefresh(payload?.refresh_interval_seconds || 20);
            } catch (error) {
                setSessionChip('异常', 'bad');
                renderLocked(error.message || '加载监控快照失败');
            } finally {
                refreshBtn.disabled = false;
                refreshBtn.textContent = '立即刷新';
            }
        }

        refreshBtn.addEventListener('click', () => {
            refreshSnapshot(true).catch(() => {});
        });

        window.addEventListener('storage', (event) => {
            if (![...sharedAuthKeys, 'me'].includes(event.key || '')) return;
            refreshSnapshot(true).catch(() => {});
        });

        loadSession().then((ready) => {
            if (!ready) return;
            refreshSnapshot(true).catch(() => {});
        });
    })();
</script>
</body>
</html>
