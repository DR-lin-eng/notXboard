<!DOCTYPE html>
<html lang="zh-CN">

<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{{ $title }} - Admin Console</title>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
  <link href="https://fonts.googleapis.com/css2?family=Fira+Code:wght@400;500;600;700&family=Fira+Sans:wght@300;400;500;600;700&family=Noto+Sans+SC:wght@400;500;600;700&display=swap" rel="stylesheet" />
  <script>
    window.settings = {
      base_url: "/",
      title: "{{ $title }}",
      version: "{{ $version }}",
      logo: "{{ $logo }}",
      secure_path: "{{ $secure_path }}",
      command_center_only: {{ !empty($command_center_only) ? 'true' : 'false' }}
    };
  </script>
  <style>
    :root {
      --bg-0: #020617;
      --bg-1: #08111f;
      --bg-2: #101c31;
      --ink-0: #f8fafc;
      --ink-1: #8ea4bf;
      --line: rgba(94, 119, 158, 0.24);
      --card: rgba(9, 17, 31, 0.88);
      --primary: #22c55e;
      --primary-strong: #16a34a;
      --accent: #38bdf8;
      --ok: #22c55e;
      --warn: #f59e0b;
      --bad: #f87171;
      --shadow: 0 28px 70px rgba(2, 6, 23, 0.44);
      --radius-lg: 18px;
      --radius-md: 12px;
      --font-sans: "Fira Sans", "Noto Sans SC", "PingFang SC", "Microsoft Yahei", sans-serif;
      --font-display: "Fira Code", "Fira Sans", "Noto Sans SC", sans-serif;
      --font-mono: "Fira Code", "SFMono-Regular", Consolas, monospace;
    }

    * {
      box-sizing: border-box;
    }

    body {
      margin: 0;
      min-height: 100vh;
      font-family: var(--font-sans);
      color: var(--ink-0);
      background:
        radial-gradient(circle at 10% 12%, rgba(34, 197, 94, 0.16) 0%, transparent 24%),
        radial-gradient(circle at 90% 0%, rgba(56, 189, 248, 0.16) 0%, transparent 28%),
        radial-gradient(circle at 50% 100%, rgba(59, 130, 246, 0.16) 0%, transparent 34%),
        linear-gradient(160deg, var(--bg-0) 0%, var(--bg-1) 48%, var(--bg-2) 100%);
    }

    .hidden {
      display: none !important;
    }

    .login-view {
      min-height: 100vh;
      display: grid;
      place-items: center;
      padding: 20px;
    }

    .login-card {
      width: min(460px, 100%);
      border: 1px solid var(--line);
      border-radius: var(--radius-lg);
      background: var(--card);
      box-shadow: var(--shadow);
      backdrop-filter: blur(8px);
      padding: 20px;
      display: grid;
      gap: 12px;
    }

    .login-title {
      margin: 0;
      font-family: "Space Grotesk", "Noto Sans SC", sans-serif;
      font-size: 22px;
      font-weight: 700;
    }

    .login-subtitle {
      margin: 0;
      color: var(--ink-1);
      font-size: 13px;
      line-height: 1.6;
    }

    .login-subtitle.error {
      color: var(--bad);
    }

    .login-error {
      min-height: 20px;
      margin: 0;
      font-size: 13px;
      color: var(--bad);
    }

    .layout {
      display: grid;
      grid-template-columns: 280px 1fr;
      min-height: 100vh;
      gap: 16px;
      padding: 16px;
    }

    .sidebar,
    .content {
      border: 1px solid var(--line);
      border-radius: var(--radius-lg);
      background: var(--card);
      backdrop-filter: blur(8px);
      box-shadow: var(--shadow);
    }

    .sidebar {
      padding: 22px 16px;
      display: flex;
      flex-direction: column;
      gap: 16px;
      position: sticky;
      top: 16px;
      max-height: calc(100vh - 32px);
      overflow: auto;
    }

    .brand {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 10px;
      border-radius: 14px;
      background: linear-gradient(130deg, rgba(15, 118, 110, 0.14), rgba(239, 111, 62, 0.14));
    }

    .brand-logo {
      width: 42px;
      height: 42px;
      border-radius: 12px;
      overflow: hidden;
      background: #fff;
      border: 1px solid rgba(15, 118, 110, 0.24);
      flex: 0 0 auto;
    }

    .brand-logo img {
      width: 100%;
      height: 100%;
      object-fit: cover;
      display: block;
    }

    .brand-title {
      margin: 0;
      font-family: "Space Grotesk", "Noto Sans SC", sans-serif;
      font-size: 18px;
      font-weight: 700;
      letter-spacing: 0.02em;
    }

    .brand-subtitle {
      margin: 2px 0 0;
      color: var(--ink-1);
      font-size: 12px;
      line-height: 1.4;
    }

    .menu {
      display: grid;
      gap: 10px;
    }

    .menu-group,
    .menu-module-list,
    .menu-subgroup {
      display: grid;
      gap: 10px;
    }

    .menu-group-head,
    .menu-subgroup-title {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 10px;
    }

    .menu-group-title,
    .menu-subgroup-title {
      margin: 0;
      font-size: 11px;
      letter-spacing: 0.16em;
      text-transform: uppercase;
      color: var(--ink-1);
    }

    .menu-group-meta,
    .menu-subgroup-count {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-width: 34px;
      padding: 4px 9px;
      border-radius: 999px;
      border: 1px solid var(--line);
      font-size: 11px;
      font-family: var(--font-display);
      letter-spacing: 0.08em;
      white-space: nowrap;
    }

    .menu-filter-field {
      gap: 5px;
    }

    .menu-btn {
      border: 1px solid transparent;
      border-radius: 12px;
      padding: 11px 12px;
      text-align: left;
      font-size: 14px;
      color: var(--ink-0);
      background: #ffffff;
      cursor: pointer;
      transition: all 0.2s ease;
      display: flex;
      align-items: center;
      justify-content: space-between;
    }

    .menu-btn:hover {
      border-color: rgba(15, 118, 110, 0.35);
      transform: translateY(-1px);
    }

    .menu-btn.active {
      background: linear-gradient(120deg, rgba(15, 118, 110, 0.14), rgba(15, 118, 110, 0.05));
      border-color: rgba(15, 118, 110, 0.45);
      color: var(--primary-strong);
      font-weight: 600;
    }

    .menu-btn.menu-btn-compact {
      align-items: flex-start;
      gap: 10px;
      padding: 10px 11px;
    }

    .menu-btn.menu-btn-compact .menu-btn-main {
      gap: 3px;
    }

    .menu-btn.menu-btn-compact .menu-btn-title {
      font-size: 13px;
    }

    .menu-btn.menu-btn-compact .menu-btn-desc {
      font-size: 11px;
      line-height: 1.45;
    }

    .theme-switcher {
      position: fixed;
      top: 18px;
      right: 18px;
      z-index: 120;
      display: inline-flex;
      align-items: center;
      gap: 8px;
      padding: 8px;
      border-radius: 999px;
      border: 1px solid rgba(94, 119, 158, 0.24);
      background: rgba(8, 15, 28, 0.84);
      backdrop-filter: blur(18px);
      box-shadow: 0 18px 44px rgba(2, 6, 23, 0.26);
    }

    .theme-switcher-btn {
      border: 0;
      border-radius: 999px;
      padding: 8px 14px;
      background: transparent;
      color: inherit;
      font-size: 12px;
      font-weight: 600;
      cursor: pointer;
      transition: background 0.18s ease, color 0.18s ease, transform 0.18s ease;
    }

    .theme-switcher-btn:hover {
      transform: translateY(-1px);
    }

    .theme-switcher-btn.active {
      background: rgba(34, 197, 94, 0.16);
      color: #22c55e;
    }

    .meta {
      margin-top: auto;
      border: 1px dashed rgba(15, 118, 110, 0.3);
      border-radius: 12px;
      padding: 12px;
      background: rgba(15, 118, 110, 0.04);
      font-size: 12px;
      color: var(--ink-1);
      line-height: 1.6;
    }

    .content {
      padding: 18px;
      display: flex;
      flex-direction: column;
      gap: 14px;
      overflow: hidden;
    }

    .toolbar {
      display: grid;
      grid-template-columns: repeat(12, minmax(0, 1fr));
      gap: 10px;
      align-items: end;
      padding-bottom: 4px;
      border-bottom: 1px solid var(--line);
    }

    .field {
      display: flex;
      flex-direction: column;
      gap: 6px;
    }

    .field label {
      font-size: 12px;
      color: var(--ink-1);
    }

    .field input,
    .field select,
    .field textarea {
      width: 100%;
      border: 1px solid rgba(15, 118, 110, 0.26);
      border-radius: 10px;
      padding: 10px 11px;
      font-size: 14px;
      background: #fff;
      color: var(--ink-0);
      transition: border-color 0.2s ease, box-shadow 0.2s ease;
      font-family: "Noto Sans SC", sans-serif;
    }

    .field textarea {
      min-height: 120px;
      resize: vertical;
    }

    .field input:focus,
    .field select:focus,
    .field textarea:focus {
      border-color: var(--primary);
      box-shadow: 0 0 0 3px rgba(15, 118, 110, 0.12);
      outline: none;
    }

    .span-12 {
      grid-column: span 12;
    }

    .span-6 {
      grid-column: span 6;
    }

    .span-4 {
      grid-column: span 4;
    }

    .span-3 {
      grid-column: span 3;
    }

    .actions {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      align-items: center;
    }

    .btn {
      border: 1px solid transparent;
      border-radius: 10px;
      padding: 9px 12px;
      font-size: 13px;
      font-weight: 600;
      cursor: pointer;
      transition: transform 0.14s ease, filter 0.2s ease, border-color 0.2s ease;
      font-family: "Space Grotesk", "Noto Sans SC", sans-serif;
      letter-spacing: 0.01em;
    }

    .btn:hover {
      transform: translateY(-1px);
      filter: saturate(1.05);
    }

    .btn.primary {
      background: linear-gradient(135deg, var(--primary), #1f9d92);
      color: #fff;
    }

    .btn.ghost {
      background: #fff;
      color: var(--ink-0);
      border-color: rgba(15, 118, 110, 0.22);
    }

    .btn.warn {
      background: #fff8ec;
      color: var(--warn);
      border-color: rgba(180, 83, 9, 0.24);
    }

    .btn.danger {
      background: #fff2f1;
      color: var(--bad);
      border-color: rgba(180, 35, 24, 0.25);
    }

    .badge {
      display: inline-flex;
      align-items: center;
      gap: 4px;
      border-radius: 999px;
      padding: 4px 10px;
      font-size: 12px;
      font-weight: 600;
      border: 1px solid;
    }

    .badge.ok {
      color: var(--ok);
      background: #eefaf3;
      border-color: rgba(21, 115, 71, 0.25);
    }

    .badge.warn {
      color: var(--warn);
      background: #fff8ec;
      border-color: rgba(180, 83, 9, 0.24);
    }

    .panels {
      position: relative;
      flex: 1;
      min-height: 0;
    }

    .panel {
      display: none;
      height: 100%;
      overflow: auto;
      animation: fadeIn 0.24s ease;
    }

    .panel.active {
      display: block;
    }

    @keyframes fadeIn {
      from {
        opacity: 0;
        transform: translateY(8px);
      }

      to {
        opacity: 1;
        transform: translateY(0);
      }
    }

    .cards {
      display: grid;
      gap: 12px;
      grid-template-columns: repeat(4, minmax(0, 1fr));
      margin-top: 4px;
    }

    .card {
      border-radius: var(--radius-md);
      border: 1px solid var(--line);
      background: #fff;
      padding: 14px;
      display: grid;
      gap: 6px;
      min-height: 108px;
    }

    .card h3 {
      margin: 0;
      font-size: 13px;
      color: var(--ink-1);
      font-weight: 500;
    }

    .card .value {
      margin: 0;
      font-family: "Space Grotesk", "Noto Sans SC", sans-serif;
      font-size: 25px;
      font-weight: 700;
      letter-spacing: 0.02em;
      color: var(--ink-0);
    }

    .card .hint {
      margin: 0;
      font-size: 12px;
      color: var(--ink-1);
    }

    .panel-box {
      margin-top: 12px;
      border: 1px solid var(--line);
      border-radius: var(--radius-md);
      background: #fff;
      padding: 14px;
      display: grid;
      gap: 12px;
    }

    .command-center-shell {
      --command-bg: #03131b;
      --command-panel: rgba(6, 28, 40, 0.78);
      --command-panel-strong: rgba(9, 34, 48, 0.9);
      --command-line: rgba(129, 230, 217, 0.16);
      --command-text: #e6fffb;
      --command-muted: rgba(214, 251, 246, 0.68);
      --command-cyan: #45e0d1;
      --command-blue: #6ea8ff;
      --command-violet: #a995ff;
      --command-emerald: #4ade80;
      --command-amber: #fbbf24;
      --command-rose: #fb7185;
      --command-orange: #fb923c;
      position: relative;
      overflow: hidden;
      padding: 20px;
      border-radius: 24px;
      border: 1px solid rgba(15, 118, 110, 0.14);
      background:
        radial-gradient(720px 420px at 0% 0%, rgba(69, 224, 209, 0.16), transparent 55%),
        radial-gradient(680px 420px at 100% 0%, rgba(239, 111, 62, 0.14), transparent 55%),
        radial-gradient(560px 280px at 50% 100%, rgba(110, 168, 255, 0.12), transparent 58%),
        linear-gradient(180deg, rgba(2, 15, 22, 0.98), rgba(4, 20, 28, 0.98));
      box-shadow: 0 28px 60px rgba(2, 15, 22, 0.34);
      color: var(--command-text);
      display: grid;
      gap: 18px;
    }

    .command-center-shell::before {
      content: "";
      position: absolute;
      inset: 0;
      pointer-events: none;
      background:
        linear-gradient(rgba(214, 251, 246, 0.05) 1px, transparent 1px),
        linear-gradient(90deg, rgba(214, 251, 246, 0.05) 1px, transparent 1px);
      background-size: 34px 34px;
      mask-image: radial-gradient(circle at center, rgba(0, 0, 0, 0.94), transparent 90%);
    }

    .command-center-shell::after {
      content: "";
      position: absolute;
      inset: -45% -15%;
      pointer-events: none;
      background: linear-gradient(110deg, transparent 36%, rgba(69, 224, 209, 0.12) 50%, transparent 64%);
      transform: translateX(-42%);
      animation: commandSweep 16s linear infinite;
    }

    .command-center-shell > * {
      position: relative;
      z-index: 1;
    }

    .command-toolbar {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      gap: 18px;
    }

    .command-toolbar h1 {
      margin: 8px 0 10px;
      font-family: var(--font-display);
      font-size: clamp(30px, 4vw, 48px);
      line-height: 0.98;
      letter-spacing: -0.04em;
      color: #f0fdfa;
    }

    .command-toolbar p {
      margin: 0;
      max-width: 780px;
      color: rgba(230, 255, 251, 0.78);
      line-height: 1.7;
      font-size: 13px;
    }

    .command-toolbar .btn {
      border-color: rgba(129, 230, 217, 0.2);
    }

    .command-toolbar .btn.ghost {
      background: rgba(8, 27, 38, 0.72);
      color: var(--command-text);
      border-color: rgba(129, 230, 217, 0.16);
    }

    .command-toolbar .btn.primary {
      background: linear-gradient(135deg, rgba(15, 118, 110, 0.96), rgba(239, 111, 62, 0.82));
      color: #fff;
      border-color: transparent;
    }

    .command-toolbar-actions {
      display: flex;
      gap: 10px;
      flex-wrap: wrap;
      justify-content: flex-end;
    }

    .command-kicker {
      font-size: 11px;
      letter-spacing: 0.28em;
      text-transform: uppercase;
      color: rgba(230, 255, 251, 0.58);
    }

    .command-status-strip {
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: 14px;
    }

    .command-strip-item {
      border-radius: 18px;
      border: 1px solid var(--command-line);
      background: rgba(8, 27, 38, 0.68);
      padding: 14px 16px;
      backdrop-filter: blur(12px);
    }

    .command-strip-item span {
      display: block;
      font-size: 11px;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      color: var(--command-muted);
    }

    .command-strip-item strong {
      display: block;
      margin-top: 8px;
      color: #f0fdfa;
      font-family: var(--font-mono);
      font-size: 18px;
    }

    #commandCenterStatusText[data-tone="ok"] {
      color: #86efac;
    }

    #commandCenterStatusText[data-tone="bad"] {
      color: #fda4af;
    }

    #commandCenterStatusText[data-tone="neutral"] {
      color: #fef3c7;
    }

    .command-center-content {
      display: grid;
      gap: 16px;
    }

    .command-loading {
      min-height: 280px;
      border-radius: 24px;
      border: 1px solid var(--command-line);
      background: rgba(8, 27, 38, 0.66);
      display: grid;
      place-items: center;
      gap: 12px;
      color: rgba(230, 255, 251, 0.78);
    }

    .command-loading-ring {
      width: 54px;
      height: 54px;
      border-radius: 999px;
      border: 3px solid rgba(214, 251, 246, 0.14);
      border-top-color: rgba(69, 224, 209, 0.9);
      animation: commandSpin 1s linear infinite;
    }

    .command-hero-panel {
      border-radius: 26px;
      border: 1px solid rgba(69, 224, 209, 0.14);
      background:
        radial-gradient(circle at top left, rgba(69, 224, 209, 0.12), transparent 48%),
        linear-gradient(135deg, rgba(6, 25, 37, 0.96), rgba(3, 16, 24, 0.92));
      padding: 22px;
      display: grid;
      grid-template-columns: 1.4fr 0.9fr;
      gap: 18px;
      box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.02), 0 18px 44px rgba(2, 15, 22, 0.26);
    }

    .command-hero-panel h2 {
      margin: 8px 0 10px;
      font-family: var(--font-display);
      font-size: clamp(28px, 3.4vw, 42px);
      line-height: 0.98;
      letter-spacing: -0.04em;
      color: #f0fdfa;
    }

    .command-hero-panel p {
      margin: 0;
      color: rgba(230, 255, 251, 0.76);
      line-height: 1.7;
      max-width: 720px;
    }

    .command-hero-badges,
    .command-node-badges,
    .command-inline-stats {
      display: flex;
      gap: 10px;
      flex-wrap: wrap;
    }

    .command-focus-grid {
      display: grid;
      gap: 12px;
    }

    .command-focus-card,
    .command-inline-stat {
      padding: 18px;
      border-radius: 22px;
      border: 1px solid rgba(214, 251, 246, 0.12);
      background: linear-gradient(180deg, rgba(10, 33, 46, 0.82), rgba(6, 22, 31, 0.84));
    }

    .command-focus-card span,
    .command-inline-stat span {
      display: block;
      color: var(--command-muted);
      font-size: 11px;
      letter-spacing: 0.14em;
      text-transform: uppercase;
    }

    .command-focus-card strong {
      display: block;
      margin: 10px 0 6px;
      color: #f0fdfa;
      font-family: var(--font-display);
      font-size: clamp(28px, 4vw, 40px);
      line-height: 1;
    }

    .command-focus-card small,
    .command-inline-stat strong {
      display: block;
      color: rgba(230, 255, 251, 0.74);
    }

    .command-inline-stat {
      min-width: 130px;
      padding: 12px 14px;
      border-radius: 16px;
      background: rgba(10, 33, 46, 0.72);
    }

    .command-inline-stat strong {
      margin-top: 8px;
      color: #f0fdfa;
    }

    .command-badge {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      padding: 6px 12px;
      border-radius: 999px;
      border: 1px solid rgba(214, 251, 246, 0.16);
      background: rgba(8, 27, 38, 0.74);
      color: #d6fbf6;
      font-size: 12px;
    }

    .command-badge.ok {
      color: #bbf7d0;
      border-color: rgba(74, 222, 128, 0.24);
    }

    .command-badge.warn,
    .command-badge.orange,
    .command-badge.amber {
      color: #fde68a;
      border-color: rgba(251, 191, 36, 0.24);
    }

    .command-badge.bad,
    .command-badge.rose {
      color: #fecdd3;
      border-color: rgba(251, 113, 133, 0.24);
    }

    .command-badge.violet {
      color: #ddd6fe;
      border-color: rgba(169, 149, 255, 0.24);
    }

    .command-metrics-grid {
      display: grid;
      grid-template-columns: repeat(4, minmax(0, 1fr));
      gap: 14px;
    }

    .command-metric-card {
      padding: 18px;
      border-radius: 22px;
      border: 1px solid rgba(214, 251, 246, 0.12);
      background: linear-gradient(180deg, rgba(7, 28, 40, 0.86), rgba(5, 20, 30, 0.86));
      position: relative;
      overflow: hidden;
    }

    .command-metric-card::after {
      content: "";
      position: absolute;
      inset: auto -10% -55% -10%;
      height: 120px;
      background: radial-gradient(circle, rgba(69, 224, 209, 0.18), transparent 68%);
      pointer-events: none;
    }

    .command-metric-card span,
    .command-metric-card small {
      display: block;
    }

    .command-metric-card span {
      color: var(--command-muted);
      font-size: 11px;
      letter-spacing: 0.12em;
      text-transform: uppercase;
    }

    .command-metric-card strong {
      display: block;
      margin: 12px 0 8px;
      color: #f0fdfa;
      font-family: var(--font-display);
      font-size: clamp(28px, 3vw, 38px);
      line-height: 0.96;
    }

    .command-metric-card small {
      color: rgba(230, 255, 251, 0.74);
    }

    .command-metric-card.cyan {
      box-shadow: inset 0 0 0 1px rgba(69, 224, 209, 0.06);
    }

    .command-metric-card.blue {
      box-shadow: inset 0 0 0 1px rgba(110, 168, 255, 0.06);
    }

    .command-metric-card.emerald {
      box-shadow: inset 0 0 0 1px rgba(74, 222, 128, 0.06);
    }

    .command-metric-card.amber {
      box-shadow: inset 0 0 0 1px rgba(251, 191, 36, 0.06);
    }

    .command-metric-card.rose {
      box-shadow: inset 0 0 0 1px rgba(251, 113, 133, 0.06);
    }

    .command-metric-card.violet {
      box-shadow: inset 0 0 0 1px rgba(169, 149, 255, 0.06);
    }

    .command-metric-card.orange {
      box-shadow: inset 0 0 0 1px rgba(251, 146, 60, 0.06);
    }

    .command-metric-card.teal {
      box-shadow: inset 0 0 0 1px rgba(45, 212, 191, 0.06);
    }

    .command-layout {
      display: grid;
      grid-template-columns: repeat(12, minmax(0, 1fr));
      gap: 14px;
    }

    .command-span-12 {
      grid-column: span 12;
    }

    .command-span-8 {
      grid-column: span 8;
    }

    .command-span-6 {
      grid-column: span 6;
    }

    .command-span-4 {
      grid-column: span 4;
    }

    .command-panel {
      border-radius: 24px;
      border: 1px solid var(--command-line);
      background: linear-gradient(180deg, rgba(7, 28, 40, 0.84), rgba(5, 20, 30, 0.86));
      padding: 18px;
      box-shadow: 0 18px 44px rgba(2, 15, 22, 0.24);
    }

    .command-panel-head,
    .command-node-head,
    .command-node-footer,
    .command-feed-top {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      gap: 12px;
    }

    .command-panel-head {
      margin-bottom: 14px;
    }

    .command-panel-kicker {
      color: var(--command-muted);
      font-size: 11px;
      letter-spacing: 0.18em;
      text-transform: uppercase;
    }

    .command-panel h3,
    .command-panel h4 {
      margin: 4px 0 0;
      color: #f0fdfa;
      font-family: var(--font-display);
    }

    .command-chart-shell,
    .command-node-spark {
      border-radius: 22px;
      border: 1px solid rgba(214, 251, 246, 0.12);
      overflow: hidden;
      background: rgba(5, 20, 30, 0.72);
    }

    .command-traffic-svg,
    .command-node-sparkline {
      width: 100%;
      height: auto;
      display: block;
    }

    .command-health-list,
    .command-user-list,
    .command-bars {
      display: grid;
      gap: 10px;
    }

    .command-health-item,
    .command-feed-item,
    .command-user-row {
      padding: 14px;
      border-radius: 18px;
      border: 1px solid rgba(214, 251, 246, 0.12);
      background: rgba(6, 23, 33, 0.68);
    }

    .command-health-item {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      gap: 12px;
    }

    .command-health-item strong,
    .command-feed-top strong,
    .command-user-identity strong,
    .command-user-metrics strong {
      display: block;
      color: #f0fdfa;
    }

    .command-health-item span,
    .command-feed-meta,
    .command-feed-sub,
    .command-user-identity span,
    .command-user-metrics span,
    .command-node-subline,
    .command-node-owner,
    .command-node-meter-bottom,
    .command-node-alert,
    .command-spark-empty {
      color: var(--command-muted);
      font-size: 13px;
      line-height: 1.55;
    }

    .command-log-grid,
    .command-node-stats {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 10px;
      margin-top: 12px;
    }

    .command-log-grid > div,
    .command-node-stats > div {
      padding: 12px;
      border-radius: 16px;
      background: rgba(8, 27, 38, 0.76);
      border: 1px solid rgba(214, 251, 246, 0.12);
    }

    .command-log-grid span,
    .command-node-stats span {
      display: block;
      color: var(--command-muted);
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }

    .command-log-grid strong,
    .command-node-stats strong {
      display: block;
      margin-top: 8px;
      color: #f0fdfa;
      font-size: 17px;
    }

    .command-user-row {
      display: grid;
      grid-template-columns: 56px minmax(0, 1.2fr) minmax(0, 1fr) minmax(140px, 0.9fr);
      gap: 12px;
      align-items: center;
    }

    .command-rank {
      color: rgba(69, 224, 209, 0.94);
      font-family: var(--font-mono);
      font-size: 14px;
    }

    .command-user-bar,
    .command-bar-track,
    .command-meter {
      height: 10px;
      border-radius: 999px;
      background: rgba(3, 16, 24, 0.96);
      overflow: hidden;
    }

    .command-user-bar-fill,
    .command-bar-fill.cyan {
      height: 100%;
      border-radius: inherit;
      background: linear-gradient(90deg, rgba(69, 224, 209, 0.9), rgba(110, 168, 255, 0.94));
      box-shadow: 0 0 22px rgba(69, 224, 209, 0.2);
    }

    .command-bar-fill.violet {
      height: 100%;
      border-radius: inherit;
      background: linear-gradient(90deg, rgba(169, 149, 255, 0.9), rgba(239, 111, 62, 0.9));
    }

    .command-split-grid,
    .command-node-grid {
      display: grid;
      gap: 16px;
    }

    .command-split-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .command-node-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .command-bar-row {
      display: grid;
      grid-template-columns: minmax(0, 1fr) minmax(120px, 1fr) auto;
      gap: 12px;
      align-items: center;
    }

    .command-split-grid h4 {
      margin: 0 0 10px;
      color: #f0fdfa;
      font-family: var(--font-display);
    }

    .command-bar-copy strong,
    .command-bar-value {
      color: #f0fdfa;
    }

    .command-bar-copy span {
      display: block;
      color: var(--command-muted);
      font-size: 12px;
      margin-top: 2px;
    }

    .command-node-card {
      border-radius: 22px;
      border: 1px solid rgba(214, 251, 246, 0.12);
      background:
        radial-gradient(260px 160px at top right, rgba(69, 224, 209, 0.08), transparent 58%),
        linear-gradient(180deg, rgba(8, 27, 38, 0.84), rgba(5, 20, 30, 0.86));
      padding: 16px;
    }

    .command-node-title {
      display: flex;
      gap: 10px;
      align-items: center;
      flex-wrap: wrap;
    }

    .command-node-title h4 {
      margin: 0;
      font-size: 22px;
    }

    .command-node-owner,
    .command-node-footer {
      margin-top: 12px;
    }

    .command-node-spark {
      margin-top: 12px;
      padding: 10px;
    }

    .command-node-meter {
      margin-top: 14px;
    }

    .command-node-meter-top {
      display: flex;
      justify-content: space-between;
      gap: 10px;
      align-items: flex-start;
    }

    .command-node-meter-top span {
      color: var(--command-muted);
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }

    .command-node-meter-top strong {
      color: #f0fdfa;
    }

    .command-meter-fill {
      height: 100%;
      border-radius: inherit;
      background: linear-gradient(90deg, rgba(74, 222, 128, 0.9), rgba(45, 212, 191, 0.92));
    }

    .command-meter.warn .command-meter-fill {
      background: linear-gradient(90deg, rgba(251, 191, 36, 0.92), rgba(251, 146, 60, 0.92));
    }

    .command-meter.danger .command-meter-fill {
      background: linear-gradient(90deg, rgba(251, 113, 133, 0.92), rgba(248, 113, 113, 0.92));
    }

    .command-empty {
      padding: 16px;
      border-radius: 18px;
      border: 1px dashed rgba(214, 251, 246, 0.18);
      color: var(--command-muted);
      text-align: center;
      background: rgba(8, 27, 38, 0.54);
    }

    .command-feed-item + .command-feed-item {
      margin-top: 10px;
    }

    .command-shell-empty {
      padding: 0;
      color: rgba(230, 255, 251, 0.82);
    }

    @keyframes commandSweep {
      from {
        transform: translateX(-42%);
      }

      to {
        transform: translateX(42%);
      }
    }

    @keyframes commandSpin {
      from {
        transform: rotate(0deg);
      }

      to {
        transform: rotate(360deg);
      }
    }

    .box-title {
      margin: 0;
      font-size: 15px;
      font-weight: 700;
      color: var(--ink-0);
      font-family: "Space Grotesk", "Noto Sans SC", sans-serif;
    }

    .table-wrap {
      overflow: auto;
      border: 1px solid rgba(15, 118, 110, 0.18);
      border-radius: 10px;
    }

    table {
      width: 100%;
      border-collapse: collapse;
      min-width: 760px;
    }

    thead th {
      text-align: left;
      font-size: 12px;
      color: var(--ink-1);
      background: rgba(15, 118, 110, 0.06);
      border-bottom: 1px solid rgba(15, 118, 110, 0.18);
      padding: 10px;
      font-weight: 600;
    }

    tbody td {
      border-bottom: 1px solid rgba(15, 118, 110, 0.1);
      padding: 10px;
      font-size: 13px;
      vertical-align: top;
    }

    tbody tr:hover {
      background: rgba(15, 118, 110, 0.04);
    }

    .theme-grid {
      display: grid;
      gap: 12px;
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }

    .theme-card {
      border: 1px solid rgba(15, 118, 110, 0.2);
      border-radius: 12px;
      background: #fff;
      padding: 12px;
      display: grid;
      gap: 8px;
    }

    .theme-card.active {
      border-color: rgba(15, 118, 110, 0.5);
      box-shadow: 0 0 0 3px rgba(15, 118, 110, 0.1);
    }

    .theme-name {
      margin: 0;
      font-family: "Space Grotesk", "Noto Sans SC", sans-serif;
      font-size: 16px;
      font-weight: 700;
    }

    .theme-meta {
      margin: 0;
      font-size: 12px;
      color: var(--ink-1);
      line-height: 1.6;
    }

    pre {
      margin: 0;
      background: #10191d;
      color: #e7f6f4;
      border-radius: 10px;
      padding: 12px;
      font-size: 12px;
      line-height: 1.5;
      overflow: auto;
      max-height: 420px;
    }

    .toast {
      position: fixed;
      right: 22px;
      bottom: 20px;
      padding: 10px 12px;
      border-radius: 10px;
      border: 1px solid;
      background: #fff;
      box-shadow: 0 12px 24px rgba(12, 31, 28, 0.18);
      font-size: 13px;
      opacity: 0;
      transform: translateY(12px);
      transition: all 0.25s ease;
      z-index: 99;
      pointer-events: none;
      max-width: 420px;
    }

    .toast.show {
      opacity: 1;
      transform: translateY(0);
    }

    .toast.info {
      color: #175967;
      border-color: rgba(23, 89, 103, 0.26);
    }

    .toast.success {
      color: var(--ok);
      border-color: rgba(21, 115, 71, 0.26);
    }

    .toast.error {
      color: var(--bad);
      border-color: rgba(180, 35, 24, 0.26);
    }

    .empty {
      font-size: 13px;
      color: var(--ink-1);
      padding: 6px 2px;
    }

    .ops-shell {
      display: block;
      min-height: 640px;
    }

    .ops-main {
      border: 1px solid rgba(15, 118, 110, 0.2);
      border-radius: 12px;
      background: #fff;
      padding: 12px;
      display: grid;
      gap: 12px;
      align-content: start;
    }

    .ops-main-head {
      border-bottom: 1px dashed rgba(15, 118, 110, 0.24);
      padding-bottom: 10px;
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      justify-content: space-between;
      gap: 10px;
    }

    .ops-main-title {
      margin: 0;
      font-family: "Space Grotesk", "Noto Sans SC", sans-serif;
      font-size: 17px;
      font-weight: 700;
    }

    .ops-main-subtitle {
      margin: 4px 0 0;
      color: var(--ink-1);
      font-size: 12px;
      line-height: 1.55;
    }

    .ops-workspace {
      display: grid;
      gap: 12px;
      align-content: start;
    }

    .ops-card {
      border: 1px solid rgba(15, 118, 110, 0.18);
      border-radius: 11px;
      padding: 12px;
      display: grid;
      gap: 10px;
      background: #fcfefe;
    }

    .ops-card-title {
      margin: 0;
      font-size: 14px;
      font-weight: 700;
      color: var(--ink-0);
      font-family: "Space Grotesk", "Noto Sans SC", sans-serif;
    }

    .ops-grid-2 {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 12px;
    }

    .ops-grid-3 {
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: 12px;
    }

    .ops-kpi {
      border: 1px solid rgba(15, 118, 110, 0.18);
      border-radius: 10px;
      background: #fff;
      padding: 10px 12px;
      display: grid;
      gap: 5px;
    }

    .ops-kpi .label {
      font-size: 12px;
      color: var(--ink-1);
    }

    .ops-kpi .num {
      font-family: "Space Grotesk", "Noto Sans SC", sans-serif;
      font-size: 24px;
      font-weight: 700;
      line-height: 1;
      color: var(--ink-0);
    }

    .hint-text {
      margin: 0;
      font-size: 12px;
      color: var(--ink-1);
      line-height: 1.6;
    }

    .muted-code {
      display: inline-block;
      padding: 2px 7px;
      border-radius: 999px;
      border: 1px solid rgba(15, 118, 110, 0.2);
      background: rgba(15, 118, 110, 0.06);
      font-size: 12px;
      font-family: var(--font-mono);
      color: var(--ink-1);
    }

    body:not(.command-center-only)::before {
      content: "";
      position: fixed;
      inset: 0;
      pointer-events: none;
      background:
        linear-gradient(rgba(148, 163, 184, 0.04) 1px, transparent 1px),
        linear-gradient(90deg, rgba(148, 163, 184, 0.04) 1px, transparent 1px);
      background-size: 32px 32px;
      mask-image: linear-gradient(180deg, rgba(255, 255, 255, 0.95), transparent 88%);
      opacity: 0.5;
    }

    body:not(.command-center-only) .layout {
      position: relative;
      z-index: 1;
      grid-template-columns: 320px minmax(0, 1fr);
      gap: 18px;
      padding: 18px;
    }

    body:not(.command-center-only) .sidebar,
    body:not(.command-center-only) .content {
      border-color: rgba(71, 85, 105, 0.34);
      background:
        linear-gradient(180deg, rgba(9, 15, 28, 0.94), rgba(5, 10, 20, 0.9)),
        radial-gradient(circle at top, rgba(34, 197, 94, 0.08), transparent 36%);
      box-shadow:
        inset 0 1px 0 rgba(255, 255, 255, 0.04),
        0 24px 64px rgba(2, 6, 23, 0.34);
      backdrop-filter: blur(18px);
    }

    body:not(.command-center-only) .sidebar {
      padding: 24px 18px;
      gap: 18px;
    }

    body:not(.command-center-only) .brand {
      padding: 14px;
      border: 1px solid rgba(71, 85, 105, 0.34);
      background:
        radial-gradient(circle at top left, rgba(34, 197, 94, 0.18), transparent 42%),
        linear-gradient(135deg, rgba(15, 23, 42, 0.95), rgba(8, 15, 28, 0.92));
    }

    body:not(.command-center-only) .brand-logo {
      background: rgba(255, 255, 255, 0.94);
      border-color: rgba(34, 197, 94, 0.24);
      box-shadow: 0 10px 24px rgba(2, 6, 23, 0.24);
    }

    body:not(.command-center-only) .brand-title {
      font-family: var(--font-display);
      color: #f8fafc;
    }

    body:not(.command-center-only) .brand-subtitle {
      color: rgba(191, 219, 254, 0.78);
    }

    .sidebar-section-label {
      margin: 0;
      color: rgba(148, 163, 184, 0.82);
      font-size: 11px;
      letter-spacing: 0.18em;
      text-transform: uppercase;
    }

    body:not(.command-center-only) .menu {
      gap: 12px;
    }

    body:not(.command-center-only) .menu-btn {
      border-color: rgba(71, 85, 105, 0.34);
      border-radius: 16px;
      padding: 14px;
      background: linear-gradient(180deg, rgba(15, 23, 42, 0.84), rgba(10, 17, 32, 0.92));
      color: #e2e8f0;
      gap: 14px;
      align-items: flex-start;
    }

    body:not(.command-center-only) .menu-btn:hover {
      border-color: rgba(56, 189, 248, 0.4);
      background: linear-gradient(180deg, rgba(15, 23, 42, 0.96), rgba(8, 15, 28, 0.98));
      box-shadow: 0 14px 32px rgba(2, 6, 23, 0.26);
    }

    body:not(.command-center-only) .menu-btn.active {
      border-color: rgba(34, 197, 94, 0.42);
      background:
        radial-gradient(circle at left top, rgba(34, 197, 94, 0.18), transparent 42%),
        linear-gradient(135deg, rgba(15, 23, 42, 0.98), rgba(10, 19, 32, 0.98));
      color: #f8fafc;
      box-shadow:
        inset 0 0 0 1px rgba(34, 197, 94, 0.08),
        0 18px 32px rgba(2, 6, 23, 0.24);
    }

    .menu-btn-copy {
      display: grid;
      gap: 4px;
      flex: 1;
    }

    .menu-btn-copy strong {
      display: block;
      color: inherit;
      font-size: 14px;
      font-weight: 700;
    }

    .menu-btn-copy small {
      display: block;
      color: rgba(191, 219, 254, 0.74);
      font-size: 12px;
      line-height: 1.5;
    }

    .menu-btn-index {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-width: 38px;
      padding: 4px 10px;
      border-radius: 999px;
      border: 1px solid rgba(71, 85, 105, 0.46);
      background: rgba(5, 10, 20, 0.74);
      color: rgba(226, 232, 240, 0.88);
      font-family: var(--font-display);
      font-size: 12px;
    }

    .sidebar-status-card {
      border: 1px solid rgba(71, 85, 105, 0.34);
      border-radius: 18px;
      background:
        radial-gradient(circle at top right, rgba(56, 189, 248, 0.16), transparent 40%),
        linear-gradient(180deg, rgba(15, 23, 42, 0.86), rgba(7, 12, 24, 0.92));
      padding: 14px;
      display: grid;
      gap: 12px;
    }

    .sidebar-status-grid {
      display: grid;
      gap: 10px;
    }

    .sidebar-status-grid span {
      display: block;
      font-size: 11px;
      letter-spacing: 0.12em;
      text-transform: uppercase;
      color: rgba(148, 163, 184, 0.86);
    }

    .sidebar-status-grid strong {
      display: block;
      margin-top: 6px;
      font-family: var(--font-display);
      color: #f8fafc;
      font-size: 13px;
      word-break: break-all;
    }

    .sidebar-note {
      margin: 0;
      color: rgba(191, 219, 254, 0.76);
      font-size: 12px;
      line-height: 1.65;
    }

    body:not(.command-center-only) .meta {
      margin-top: 0;
      border-style: solid;
      border-color: rgba(71, 85, 105, 0.34);
      border-radius: 18px;
      background:
        radial-gradient(circle at bottom right, rgba(34, 197, 94, 0.12), transparent 42%),
        linear-gradient(180deg, rgba(8, 15, 28, 0.84), rgba(5, 10, 20, 0.94));
      color: rgba(191, 219, 254, 0.78);
      padding: 14px;
    }

    body:not(.command-center-only) .content {
      padding: 18px;
      gap: 16px;
    }

    body:not(.command-center-only) .session-shell {
      padding: 0;
      border: 0;
      gap: 14px;
      align-items: stretch;
      border-bottom: 0;
    }

    .session-title-block,
    body:not(.command-center-only) .session-shell .field {
      border: 1px solid rgba(71, 85, 105, 0.34);
      border-radius: 20px;
      background:
        linear-gradient(180deg, rgba(15, 23, 42, 0.88), rgba(8, 15, 28, 0.92)),
        radial-gradient(circle at top left, rgba(56, 189, 248, 0.1), transparent 36%);
      box-shadow: 0 18px 40px rgba(2, 6, 23, 0.22);
      padding: 16px;
      min-height: 100%;
    }

    .session-title-block {
      display: grid;
      gap: 8px;
      align-content: center;
    }

    .session-eyebrow {
      color: rgba(103, 232, 249, 0.92);
      font-size: 11px;
      letter-spacing: 0.22em;
      text-transform: uppercase;
    }

    .session-title-block h2 {
      margin: 0;
      font-family: var(--font-display);
      font-size: clamp(26px, 3vw, 34px);
      line-height: 1.05;
      color: #f8fafc;
    }

    .session-title-block p {
      margin: 0;
      color: rgba(191, 219, 254, 0.82);
      line-height: 1.65;
      font-size: 13px;
      max-width: 660px;
    }

    body:not(.command-center-only) .field label {
      color: rgba(148, 163, 184, 0.9);
      letter-spacing: 0.04em;
      text-transform: uppercase;
      font-size: 11px;
    }

    body:not(.command-center-only) .field input,
    body:not(.command-center-only) .field select,
    body:not(.command-center-only) .field textarea {
      border-color: rgba(71, 85, 105, 0.46);
      background: rgba(5, 10, 20, 0.88);
      color: #f8fafc;
      font-family: var(--font-sans);
    }

    body:not(.command-center-only) .field input:disabled {
      color: rgba(226, 232, 240, 0.86);
      background: rgba(15, 23, 42, 0.78);
    }

    body:not(.command-center-only) .field input:focus,
    body:not(.command-center-only) .field select:focus,
    body:not(.command-center-only) .field textarea:focus {
      border-color: rgba(56, 189, 248, 0.72);
      box-shadow: 0 0 0 3px rgba(56, 189, 248, 0.14);
    }

    .session-chip-grid {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      align-items: center;
    }

    body:not(.command-center-only) .btn.primary {
      background: linear-gradient(135deg, rgba(34, 197, 94, 0.98), rgba(22, 163, 74, 0.92));
      color: #f8fafc;
      box-shadow: 0 14px 28px rgba(20, 83, 45, 0.28);
    }

    body:not(.command-center-only) .btn.ghost {
      background: rgba(15, 23, 42, 0.84);
      color: #e2e8f0;
      border-color: rgba(71, 85, 105, 0.44);
    }

    body:not(.command-center-only) .btn.warn {
      background: rgba(120, 53, 15, 0.28);
      color: #fdba74;
      border-color: rgba(245, 158, 11, 0.3);
    }

    body:not(.command-center-only) .btn.danger {
      background: rgba(127, 29, 29, 0.28);
      color: #fda4af;
      border-color: rgba(248, 113, 113, 0.32);
    }

    body:not(.command-center-only) .badge {
      background: rgba(15, 23, 42, 0.84);
      border-color: rgba(71, 85, 105, 0.44);
      color: #cbd5e1;
    }

    body:not(.command-center-only) .badge.ok {
      color: #86efac;
      background: rgba(20, 83, 45, 0.26);
      border-color: rgba(34, 197, 94, 0.3);
    }

    body:not(.command-center-only) .badge.warn {
      color: #fdba74;
      background: rgba(120, 53, 15, 0.26);
      border-color: rgba(245, 158, 11, 0.3);
    }

    body:not(.command-center-only) .panel-box {
      margin-top: 0;
      border-color: rgba(71, 85, 105, 0.34);
      border-radius: 24px;
      background:
        linear-gradient(180deg, rgba(9, 15, 28, 0.94), rgba(5, 10, 20, 0.94)),
        radial-gradient(circle at top right, rgba(56, 189, 248, 0.08), transparent 34%);
      padding: 18px;
      box-shadow:
        inset 0 1px 0 rgba(255, 255, 255, 0.04),
        0 20px 48px rgba(2, 6, 23, 0.24);
    }

    .panel-box-head {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      gap: 16px;
      margin-bottom: 16px;
    }

    .panel-box-copy {
      display: grid;
      gap: 8px;
    }

    .section-kicker,
    .ops-section-kicker {
      margin: 0;
      color: rgba(103, 232, 249, 0.92);
      font-size: 11px;
      letter-spacing: 0.22em;
      text-transform: uppercase;
    }

    body:not(.command-center-only) .box-title,
    body:not(.command-center-only) .ops-card-title,
    body:not(.command-center-only) .ops-main-title {
      color: #f8fafc;
      font-family: var(--font-display);
    }

    .box-subtitle,
    .ops-card-subtitle {
      margin: 0;
      color: rgba(191, 219, 254, 0.78);
      font-size: 13px;
      line-height: 1.65;
    }

    body:not(.command-center-only) .empty {
      color: rgba(191, 219, 254, 0.76);
    }

    body:not(.command-center-only) .table-wrap {
      border-color: rgba(71, 85, 105, 0.34);
      border-radius: 18px;
      background: rgba(2, 6, 23, 0.56);
    }

    body:not(.command-center-only) thead th {
      color: rgba(191, 219, 254, 0.82);
      background: rgba(15, 23, 42, 0.9);
      border-bottom-color: rgba(71, 85, 105, 0.34);
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }

    body:not(.command-center-only) tbody td {
      border-bottom-color: rgba(71, 85, 105, 0.2);
      color: #e2e8f0;
    }

    body:not(.command-center-only) tbody tr:hover {
      background: rgba(15, 23, 42, 0.48);
    }

    body:not(.command-center-only) pre {
      border: 1px solid rgba(71, 85, 105, 0.34);
      border-radius: 18px;
      background: linear-gradient(180deg, rgba(2, 6, 23, 0.92), rgba(8, 15, 28, 0.96));
      color: #dbeafe;
    }

    body:not(.command-center-only) .toast {
      background: rgba(9, 15, 28, 0.96);
      border-color: rgba(71, 85, 105, 0.44);
      color: #e2e8f0;
    }

    body:not(.command-center-only) .ops-shell {
      min-height: 700px;
    }

    body:not(.command-center-only) .ops-main,
    body:not(.command-center-only) .ops-card,
    body:not(.command-center-only) .ops-kpi {
      border-color: rgba(71, 85, 105, 0.34);
      background:
        linear-gradient(180deg, rgba(15, 23, 42, 0.88), rgba(8, 15, 28, 0.92)),
        radial-gradient(circle at top left, rgba(34, 197, 94, 0.08), transparent 34%);
      box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
    }

    body:not(.command-center-only) .ops-main {
      padding: 18px;
      border-radius: 24px;
      gap: 16px;
    }

    body:not(.command-center-only) .ops-main-head {
      border-bottom-color: rgba(71, 85, 105, 0.34);
      padding-bottom: 14px;
    }

    .ops-main-copy {
      display: grid;
      gap: 8px;
    }

    .ops-module-tag {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      width: fit-content;
      border-radius: 999px;
      border: 1px solid rgba(56, 189, 248, 0.3);
      background: rgba(8, 47, 73, 0.28);
      color: #67e8f9;
      padding: 5px 10px;
      font-size: 11px;
      letter-spacing: 0.08em;
      text-transform: uppercase;
    }

    body:not(.command-center-only) .ops-main-subtitle {
      color: rgba(191, 219, 254, 0.78);
      font-size: 13px;
    }

    body:not(.command-center-only) .ops-card {
      border-radius: 20px;
      padding: 16px;
      gap: 12px;
    }

    .ops-grid-4 {
      display: grid;
      grid-template-columns: repeat(4, minmax(0, 1fr));
      gap: 12px;
    }

    body:not(.command-center-only) .ops-kpi {
      border-radius: 18px;
      padding: 14px;
    }

    body:not(.command-center-only) .ops-kpi .label {
      color: rgba(148, 163, 184, 0.88);
      text-transform: uppercase;
      letter-spacing: 0.08em;
      font-size: 11px;
    }

    body:not(.command-center-only) .ops-kpi .num {
      color: #f8fafc;
      font-family: var(--font-display);
      font-size: 28px;
    }

    .ops-hero {
      border: 1px solid rgba(71, 85, 105, 0.34);
      border-radius: 22px;
      background:
        radial-gradient(circle at top right, rgba(34, 197, 94, 0.16), transparent 42%),
        linear-gradient(135deg, rgba(15, 23, 42, 0.92), rgba(8, 15, 28, 0.96));
      padding: 18px;
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      gap: 16px;
    }

    .ops-hero-title {
      margin: 6px 0 0;
      color: #f8fafc;
      font-family: var(--font-display);
      font-size: clamp(24px, 3vw, 30px);
      line-height: 1.08;
    }

    .ops-hero-meta {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      align-items: center;
    }

    .ops-two-column,
    .ops-split-shell {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 14px;
    }

    .ops-stack {
      display: grid;
      gap: 14px;
      align-content: start;
    }

    .ops-table-header {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      gap: 14px;
      flex-wrap: wrap;
    }

    .ops-table-caption {
      display: grid;
      gap: 6px;
    }

    .ops-form-grid {
      gap: 12px;
    }

    .ops-helper-list {
      display: grid;
      gap: 10px;
    }

    .ops-helper-item {
      border: 1px solid rgba(71, 85, 105, 0.3);
      border-radius: 16px;
      background: rgba(2, 6, 23, 0.38);
      padding: 12px 14px;
    }

    .ops-helper-item strong {
      display: block;
      color: #f8fafc;
      margin-bottom: 4px;
      font-size: 13px;
    }

    .ops-helper-item p {
      margin: 0;
      color: rgba(191, 219, 254, 0.76);
      font-size: 12px;
      line-height: 1.6;
    }

    .ops-note {
      margin: 0;
      color: rgba(148, 163, 184, 0.9);
      font-size: 12px;
      line-height: 1.65;
    }

    .overview-grid {
      display: grid;
      grid-template-columns: 1.15fr 0.85fr;
      gap: 14px;
    }

    .overview-summary-grid {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 12px;
      margin-top: 14px;
    }

    .overview-summary-card,
    .overview-rule-item,
    .overview-quick-card {
      border: 1px solid rgba(71, 85, 105, 0.3);
      border-radius: 18px;
      background:
        linear-gradient(180deg, rgba(15, 23, 42, 0.82), rgba(8, 15, 28, 0.92)),
        radial-gradient(circle at top left, rgba(56, 189, 248, 0.08), transparent 34%);
      padding: 14px;
    }

    .overview-summary-card span {
      display: block;
      color: rgba(148, 163, 184, 0.86);
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }

    .overview-summary-card strong {
      display: block;
      margin: 10px 0 6px;
      color: #f8fafc;
      font-family: var(--font-display);
      font-size: 26px;
      line-height: 1;
    }

    .overview-summary-card small,
    .overview-rule-item p {
      display: block;
      color: rgba(191, 219, 254, 0.76);
      font-size: 12px;
      line-height: 1.6;
    }

    .overview-rule-list,
    .overview-quick-grid {
      display: grid;
      gap: 12px;
      margin-top: 14px;
    }

    .overview-rule-item strong,
    .overview-quick-card strong {
      display: block;
      color: #f8fafc;
      font-size: 14px;
      margin-bottom: 6px;
    }

    .overview-rule-item p {
      margin: 0;
    }

    .overview-quick-card {
      width: 100%;
      text-align: left;
      cursor: pointer;
      transition: transform 0.18s ease, border-color 0.18s ease, box-shadow 0.18s ease;
    }

    .overview-quick-card:hover {
      transform: translateY(-2px);
      border-color: rgba(34, 197, 94, 0.34);
      box-shadow: 0 18px 34px rgba(2, 6, 23, 0.22);
    }

    .overview-quick-card span {
      display: block;
      color: rgba(103, 232, 249, 0.9);
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.12em;
      margin-bottom: 8px;
    }

    .overview-quick-card p {
      margin: 0;
      color: rgba(191, 219, 254, 0.76);
      font-size: 12px;
      line-height: 1.6;
    }

    button:focus-visible,
    input:focus-visible,
    select:focus-visible,
    textarea:focus-visible,
    a:focus-visible {
      outline: 2px solid rgba(103, 232, 249, 0.86);
      outline-offset: 2px;
    }

    @media (prefers-reduced-motion: reduce) {
      *,
      *::before,
      *::after {
        animation-duration: 0.01ms !important;
        animation-iteration-count: 1 !important;
        transition-duration: 0.01ms !important;
        scroll-behavior: auto !important;
      }
    }

    @media (max-width: 1200px) {
      .cards {
        grid-template-columns: repeat(2, minmax(0, 1fr));
      }

      .theme-grid {
        grid-template-columns: repeat(2, minmax(0, 1fr));
      }

      .command-metrics-grid,
      .command-node-grid {
        grid-template-columns: repeat(2, minmax(0, 1fr));
      }

      .command-layout {
        grid-template-columns: repeat(6, minmax(0, 1fr));
      }

      .command-span-12,
      .command-span-8,
      .command-span-6 {
        grid-column: span 6;
      }

              .command-span-4 {
        grid-column: span 3;
      }

      .ops-grid-4,
      .overview-summary-grid {
        grid-template-columns: repeat(2, minmax(0, 1fr));
      }

      .overview-grid {
        grid-template-columns: 1fr;
      }

      .span-3,
      .span-4,
      .span-6 {
        grid-column: span 6;
      }
    }

    @media (max-width: 960px) {
      .layout {
        grid-template-columns: 1fr;
      }

      .sidebar {
        position: static;
        max-height: none;
      }

      .theme-grid,
      .cards {
        grid-template-columns: 1fr;
      }

      .command-toolbar,
      .command-hero-panel,
      .command-status-strip,
      .command-metrics-grid,
      .command-layout,
      .command-split-grid,
      .command-node-grid,
      .command-log-grid,
      .command-node-stats {
        grid-template-columns: 1fr;
      }

      .command-panel-head,
      .command-node-head,
      .command-node-footer,
      .command-feed-top {
        flex-direction: column;
      }

      .command-user-row,
      .command-bar-row {
        grid-template-columns: 1fr;
      }

      .ops-grid-4,
      .overview-grid,
      .overview-summary-grid {
        grid-template-columns: 1fr;
      }

      .command-span-12,
      .command-span-8,
      .command-span-6,
      .command-span-4 {
        grid-column: span 1;
      }

      .ops-shell {
        grid-template-columns: 1fr;
        min-height: 0;
      }

      .ops-grid-2,
      .ops-grid-3 {
        grid-template-columns: 1fr;
      }

      .span-3,
      .span-4,
      .span-6,
      .span-12 {
        grid-column: span 12;
      }
    }

    body.command-center-only {
      color: #d6fbf6;
      background:
        radial-gradient(circle at 10% 8%, rgba(34, 211, 238, 0.16) 0%, transparent 24%),
        radial-gradient(circle at 84% 12%, rgba(251, 146, 60, 0.14) 0%, transparent 20%),
        radial-gradient(circle at 50% 100%, rgba(56, 189, 248, 0.12) 0%, transparent 32%),
        linear-gradient(180deg, #020617 0%, #030b16 44%, #07131f 100%);
    }

    body.command-center-only::before {
      content: "";
      position: fixed;
      inset: 0;
      pointer-events: none;
      opacity: 0.22;
      background:
        linear-gradient(rgba(56, 189, 248, 0.06) 1px, transparent 1px),
        linear-gradient(90deg, rgba(56, 189, 248, 0.04) 1px, transparent 1px);
      background-size: 28px 28px;
      mask-image: linear-gradient(180deg, rgba(255, 255, 255, 0.9), transparent 85%);
    }

    body.command-center-only .sidebar {
      display: none;
    }

    body.command-center-only .layout {
      grid-template-columns: 1fr;
      padding: 12px;
    }

    body.command-center-only .content {
      padding: 0;
      border: 1px solid rgba(71, 85, 105, 0.34);
      background:
        linear-gradient(180deg, rgba(2, 6, 23, 0.92), rgba(3, 12, 26, 0.96)),
        radial-gradient(circle at top, rgba(14, 165, 233, 0.14), transparent 46%);
      box-shadow:
        0 24px 90px rgba(2, 6, 23, 0.66),
        inset 0 1px 0 rgba(148, 163, 184, 0.08);
      backdrop-filter: blur(16px);
    }

    body.command-center-only .toolbar {
      position: sticky;
      top: 0;
      z-index: 12;
      margin: 0;
      padding: 18px 22px 14px;
      border-radius: 0;
      border-left: 0;
      border-right: 0;
      border-top: 0;
      border-bottom: 1px solid rgba(71, 85, 105, 0.34);
      background:
        linear-gradient(135deg, rgba(15, 23, 42, 0.95), rgba(8, 15, 28, 0.92)),
        radial-gradient(circle at left top, rgba(34, 211, 238, 0.16), transparent 32%);
      box-shadow: 0 18px 40px rgba(2, 6, 23, 0.34);
    }

    body.command-center-only .toolbar label,
    body.command-center-only .field label {
      color: rgba(148, 163, 184, 0.92);
    }

    body.command-center-only .toolbar input {
      color: #f8fafc;
      border-color: rgba(71, 85, 105, 0.5);
      background: rgba(8, 15, 28, 0.9);
    }

    body.command-center-only .toolbar .badge.ok,
    body.command-center-only .toolbar .badge.warn {
      border-color: rgba(56, 189, 248, 0.34);
      background: rgba(8, 47, 73, 0.54);
      color: #67e8f9;
    }

    body.command-center-only .panels {
      padding: 0 22px 22px;
    }

    body.command-center-only .panel {
      border: 0;
      padding: 0;
      background: transparent;
      box-shadow: none;
    }

    body.command-center-only .command-center-shell {
      gap: 18px;
    }

    body.command-center-only .command-toolbar {
      padding: 24px 26px;
      border: 1px solid rgba(71, 85, 105, 0.34);
      background:
        linear-gradient(120deg, rgba(8, 15, 28, 0.96), rgba(7, 20, 32, 0.94)),
        radial-gradient(circle at top left, rgba(34, 211, 238, 0.14), transparent 32%);
      box-shadow:
        inset 0 1px 0 rgba(148, 163, 184, 0.08),
        0 22px 54px rgba(2, 6, 23, 0.4);
    }

    body.command-center-only .command-kicker,
    body.command-center-only .command-panel-kicker {
      color: rgba(103, 232, 249, 0.92);
      letter-spacing: 0.24em;
    }

    body.command-center-only .command-toolbar h1,
    body.command-center-only .command-hero-copy h2,
    body.command-center-only .command-panel h3 {
      color: #f8fafc;
    }

    body.command-center-only .command-toolbar p,
    body.command-center-only .command-hero-copy p,
    body.command-center-only .command-empty,
    body.command-center-only .command-feed-sub,
    body.command-center-only .command-feed-meta,
    body.command-center-only .command-node-owner,
    body.command-center-only .command-node-subline {
      color: rgba(191, 219, 254, 0.82);
    }

    body.command-center-only .command-panel,
    body.command-center-only .command-status-strip,
    body.command-center-only .command-metric-card,
    body.command-center-only .command-focus-card,
    body.command-center-only .command-node-card,
    body.command-center-only .command-feed-item {
      border-color: rgba(71, 85, 105, 0.34);
      background:
        linear-gradient(180deg, rgba(6, 14, 27, 0.95), rgba(7, 17, 30, 0.9)),
        radial-gradient(circle at top, rgba(34, 211, 238, 0.08), transparent 34%);
      box-shadow:
        inset 0 1px 0 rgba(148, 163, 184, 0.05),
        0 18px 38px rgba(2, 6, 23, 0.26);
    }

    body.command-center-only .command-strip-item strong,
    body.command-center-only .command-metric-card strong,
    body.command-center-only .command-focus-card strong,
    body.command-center-only .command-health-item strong,
    body.command-center-only .command-log-grid strong,
    body.command-center-only .command-user-metrics strong,
    body.command-center-only .command-node-stats strong,
    body.command-center-only .command-inline-stat strong {
      color: #f8fafc;
    }

    body.command-center-only .command-metric-card small,
    body.command-center-only .command-focus-card small,
    body.command-center-only .command-health-item span,
    body.command-center-only .command-log-grid span,
    body.command-center-only .command-user-metrics span,
    body.command-center-only .command-node-stats span,
    body.command-center-only .command-inline-stat span {
      color: rgba(148, 163, 184, 0.92);
    }

    body.command-center-only .command-badge.neutral {
      border-color: rgba(71, 85, 105, 0.5);
      background: rgba(15, 23, 42, 0.82);
      color: #cbd5e1;
    }

    body.command-center-only .command-badge.ok {
      border-color: rgba(16, 185, 129, 0.34);
      background: rgba(6, 78, 59, 0.28);
      color: #6ee7b7;
    }

    body.command-center-only .command-badge.warn {
      border-color: rgba(245, 158, 11, 0.34);
      background: rgba(120, 53, 15, 0.26);
      color: #fdba74;
    }

    body.command-center-only .command-badge.bad,
    body.command-center-only .command-badge.rose {
      border-color: rgba(248, 113, 113, 0.34);
      background: rgba(127, 29, 29, 0.28);
      color: #fda4af;
    }

    body.command-center-only .command-badge.violet {
      border-color: rgba(167, 139, 250, 0.34);
      background: rgba(76, 29, 149, 0.26);
      color: #c4b5fd;
    }

    body.command-center-only .command-badge.orange,
    body.command-center-only .command-badge.amber {
      border-color: rgba(251, 146, 60, 0.34);
      background: rgba(124, 45, 18, 0.28);
      color: #fdba74;
    }

    body.command-center-only .command-badge.cyan,
    body.command-center-only .command-badge.teal {
      border-color: rgba(34, 211, 238, 0.34);
      background: rgba(8, 47, 73, 0.34);
      color: #67e8f9;
    }

    body:not(.command-center-only) {
      color: #dce6f4;
    }

    body:not(.command-center-only) .layout {
      gap: 18px;
      padding: 18px;
    }

    body:not(.command-center-only) .sidebar,
    body:not(.command-center-only) .content {
      border-color: rgba(94, 119, 158, 0.24);
      background:
        linear-gradient(180deg, rgba(6, 12, 24, 0.94), rgba(10, 18, 32, 0.92)),
        radial-gradient(circle at top, rgba(34, 197, 94, 0.08), transparent 36%);
      box-shadow:
        inset 0 1px 0 rgba(226, 232, 240, 0.05),
        0 24px 64px rgba(2, 6, 23, 0.34);
      backdrop-filter: blur(18px);
    }

    body:not(.command-center-only) .sidebar {
      gap: 18px;
      padding: 20px 18px;
    }

    body:not(.command-center-only) .brand {
      padding: 14px;
      border: 1px solid rgba(94, 119, 158, 0.18);
      background:
        radial-gradient(circle at top left, rgba(34, 197, 94, 0.16), transparent 48%),
        linear-gradient(135deg, rgba(8, 15, 28, 0.94), rgba(9, 17, 31, 0.92));
    }

    body:not(.command-center-only) .brand-logo {
      border-color: rgba(34, 197, 94, 0.28);
      background: rgba(15, 23, 42, 0.86);
      box-shadow: 0 14px 28px rgba(2, 6, 23, 0.24);
    }

    body:not(.command-center-only) .brand-title,
    body:not(.command-center-only) .login-title,
    body:not(.command-center-only) .box-title,
    body:not(.command-center-only) .ops-main-title,
    body:not(.command-center-only) .ops-card-title {
      font-family: var(--font-display);
      letter-spacing: -0.02em;
      color: #f8fafc;
    }

    body:not(.command-center-only) .brand-subtitle,
    body:not(.command-center-only) .login-subtitle,
    body:not(.command-center-only) .empty,
    body:not(.command-center-only) .ops-main-subtitle,
    body:not(.command-center-only) .hint-text {
      color: #8ea4bf;
    }

    body:not(.command-center-only) .sidebar-section {
      border: 1px solid rgba(94, 119, 158, 0.18);
      border-radius: 16px;
      padding: 14px;
      background: rgba(10, 18, 32, 0.82);
      display: grid;
      gap: 12px;
    }

    body:not(.command-center-only) .sidebar-section-title {
      margin: 0;
      font-size: 11px;
      letter-spacing: 0.18em;
      text-transform: uppercase;
      color: #67e8f9;
    }

    body:not(.command-center-only) .sidebar-mini-grid {
      display: grid;
      gap: 10px;
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    body:not(.command-center-only) .sidebar-mini-card {
      border-radius: 14px;
      padding: 12px;
      border: 1px solid rgba(94, 119, 158, 0.18);
      background: rgba(6, 12, 24, 0.88);
      display: grid;
      gap: 6px;
    }

    body:not(.command-center-only) .sidebar-mini-card span {
      font-size: 11px;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      color: #8ea4bf;
    }

    body:not(.command-center-only) .sidebar-mini-card strong {
      font-size: 16px;
      color: #f8fafc;
      font-family: var(--font-display);
    }

    body:not(.command-center-only) .menu {
      gap: 12px;
    }

    body:not(.command-center-only) .menu-btn {
      padding: 14px;
      border-radius: 16px;
      border-color: rgba(94, 119, 158, 0.18);
      background: rgba(8, 15, 28, 0.92);
      color: #dce6f4;
      align-items: flex-start;
      gap: 12px;
      box-shadow: inset 0 1px 0 rgba(226, 232, 240, 0.03);
    }

    body:not(.command-center-only) .menu-btn:hover {
      border-color: rgba(56, 189, 248, 0.36);
      background: rgba(10, 18, 32, 0.96);
      transform: translateY(-1px);
    }

    body:not(.command-center-only) .menu-btn.active {
      border-color: rgba(34, 197, 94, 0.34);
      color: #f8fafc;
      background:
        radial-gradient(circle at left top, rgba(34, 197, 94, 0.2), transparent 52%),
        linear-gradient(135deg, rgba(9, 17, 31, 0.98), rgba(8, 15, 28, 0.96));
      box-shadow: 0 18px 36px rgba(2, 6, 23, 0.18);
    }

    body:not(.command-center-only) .menu-btn-main {
      display: grid;
      gap: 4px;
      min-width: 0;
      flex: 1;
    }

    body:not(.command-center-only) .menu-btn-title {
      font-size: 15px;
      font-weight: 600;
      color: inherit;
    }

    body:not(.command-center-only) .menu-btn-desc {
      font-size: 12px;
      line-height: 1.5;
      color: #8ea4bf;
    }

    body:not(.command-center-only) .menu-btn.active .menu-btn-desc {
      color: #c7d5e7;
    }

    body:not(.command-center-only) .menu-btn-index {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-width: 42px;
      padding: 5px 10px;
      border-radius: 999px;
      border: 1px solid rgba(94, 119, 158, 0.2);
      background: rgba(15, 23, 42, 0.94);
      color: #67e8f9;
      font-size: 11px;
      font-family: var(--font-display);
      letter-spacing: 0.14em;
      text-transform: uppercase;
    }

    body:not(.command-center-only) .meta {
      border-style: solid;
      border-color: rgba(94, 119, 158, 0.18);
      border-radius: 16px;
      padding: 14px;
      background:
        radial-gradient(circle at top right, rgba(56, 189, 248, 0.1), transparent 46%),
        rgba(10, 18, 32, 0.82);
      color: #8ea4bf;
    }

    body:not(.command-center-only) .content {
      padding: 20px;
      gap: 16px;
    }

    body:not(.command-center-only) .session-toolbar {
      grid-template-columns: minmax(0, 1.2fr) repeat(3, minmax(0, 0.4fr));
      align-items: stretch;
      gap: 14px;
      padding-bottom: 0;
      border-bottom: 0;
    }

    body:not(.command-center-only) .toolbar-head {
      border: 1px solid rgba(94, 119, 158, 0.22);
      border-radius: 22px;
      padding: 18px 20px;
      background:
        radial-gradient(circle at left top, rgba(34, 197, 94, 0.18), transparent 42%),
        linear-gradient(135deg, rgba(7, 13, 25, 0.98), rgba(9, 17, 31, 0.94));
      display: grid;
      gap: 8px;
    }

    body:not(.command-center-only) .toolbar-heading {
      margin: 0;
      font-family: var(--font-display);
      font-size: clamp(26px, 2.8vw, 36px);
      line-height: 1;
      color: #f8fafc;
    }

    body:not(.command-center-only) .toolbar-subheading {
      margin: 0;
      max-width: 720px;
      color: #8ea4bf;
      line-height: 1.7;
      font-size: 13px;
    }

    body:not(.command-center-only) .toolbar-chip-row {
      display: flex;
      flex-wrap: wrap;
      gap: 10px;
    }

    body:not(.command-center-only) .toolbar-chip {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      padding: 7px 12px;
      border-radius: 999px;
      border: 1px solid rgba(94, 119, 158, 0.2);
      background: rgba(8, 15, 28, 0.92);
      color: #c7d5e7;
      font-size: 12px;
    }

    body:not(.command-center-only) .toolbar-chip strong {
      color: #f8fafc;
      font-family: var(--font-display);
      font-size: 12px;
    }

    body:not(.command-center-only) .toolbar-stat {
      border: 1px solid rgba(94, 119, 158, 0.18);
      border-radius: 22px;
      padding: 16px;
      background: rgba(8, 15, 28, 0.92);
      display: grid;
      gap: 8px;
    }

    body:not(.command-center-only) .toolbar-stat .field {
      gap: 8px;
    }

    body:not(.command-center-only) .toolbar-stat .field label {
      color: #8ea4bf;
      letter-spacing: 0.08em;
      text-transform: uppercase;
    }

    body:not(.command-center-only) .field input,
    body:not(.command-center-only) .field select,
    body:not(.command-center-only) .field textarea {
      border-color: rgba(94, 119, 158, 0.26);
      background: rgba(15, 23, 42, 0.84);
      color: #f8fafc;
      font-family: var(--font-sans);
    }

    body:not(.command-center-only) .field input:disabled {
      color: #f8fafc;
      background: rgba(7, 13, 25, 0.92);
      opacity: 1;
    }

    body:not(.command-center-only) .field input:focus,
    body:not(.command-center-only) .field select:focus,
    body:not(.command-center-only) .field textarea:focus {
      border-color: rgba(34, 197, 94, 0.48);
      box-shadow: 0 0 0 3px rgba(34, 197, 94, 0.12);
    }

    body:not(.command-center-only) .btn {
      border-radius: 12px;
      padding: 10px 14px;
      font-family: var(--font-display);
    }

    body:not(.command-center-only) .btn.primary {
      background: linear-gradient(135deg, rgba(34, 197, 94, 0.98), rgba(22, 163, 74, 0.92));
      color: #04110b;
      box-shadow: 0 10px 24px rgba(22, 163, 74, 0.24);
    }

    body:not(.command-center-only) .btn.ghost {
      background: rgba(15, 23, 42, 0.9);
      color: #dce6f4;
      border-color: rgba(94, 119, 158, 0.22);
    }

    body:not(.command-center-only) .btn.warn {
      background: rgba(120, 53, 15, 0.26);
      color: #fdba74;
      border-color: rgba(245, 158, 11, 0.28);
    }

    body:not(.command-center-only) .btn.danger {
      background: rgba(127, 29, 29, 0.26);
      color: #fecaca;
      border-color: rgba(248, 113, 113, 0.28);
    }

    body:not(.command-center-only) .btn[disabled] {
      opacity: 0.5;
      cursor: not-allowed;
      transform: none;
      filter: none;
    }

    body:not(.command-center-only) .badge {
      font-family: var(--font-display);
      letter-spacing: 0.04em;
    }

    body:not(.command-center-only) .badge.ok {
      color: #86efac;
      background: rgba(6, 78, 59, 0.28);
      border-color: rgba(34, 197, 94, 0.28);
    }

    body:not(.command-center-only) .badge.warn {
      color: #fdba74;
      background: rgba(120, 53, 15, 0.26);
      border-color: rgba(245, 158, 11, 0.28);
    }

    body:not(.command-center-only) .panel {
      padding-right: 2px;
    }

    body:not(.command-center-only) .panel-shell,
    body:not(.command-center-only) .panel-box {
      margin-top: 0;
      border-color: rgba(94, 119, 158, 0.2);
      border-radius: 24px;
      background:
        linear-gradient(180deg, rgba(8, 15, 28, 0.96), rgba(9, 17, 31, 0.92)),
        radial-gradient(circle at top, rgba(56, 189, 248, 0.06), transparent 30%);
      padding: 18px;
      gap: 14px;
      box-shadow: inset 0 1px 0 rgba(226, 232, 240, 0.04);
    }

    body:not(.command-center-only) .section-eyebrow {
      margin: 0;
      color: #67e8f9;
      font-size: 11px;
      letter-spacing: 0.24em;
      text-transform: uppercase;
    }

    body:not(.command-center-only) .section-heading {
      margin: 6px 0 0;
      color: #f8fafc;
      font-family: var(--font-display);
      font-size: clamp(24px, 2.6vw, 34px);
      line-height: 1;
    }

    body:not(.command-center-only) .section-copy {
      margin: 8px 0 0;
      max-width: 760px;
      color: #8ea4bf;
      line-height: 1.7;
      font-size: 13px;
    }

    body:not(.command-center-only) .overview-head,
    body:not(.command-center-only) .users-head,
    body:not(.command-center-only) .api-head {
      display: flex;
      justify-content: space-between;
      gap: 18px;
      align-items: flex-start;
      flex-wrap: wrap;
    }

    body:not(.command-center-only) .overview-head-actions {
      display: flex;
      gap: 10px;
      flex-wrap: wrap;
    }

    body:not(.command-center-only) .overview-card-grid,
    body:not(.command-center-only) .users-meta-grid,
    body:not(.command-center-only) .api-note-grid {
      display: grid;
      gap: 14px;
      grid-template-columns: repeat(4, minmax(0, 1fr));
    }

    body:not(.command-center-only) .overview-kpi-card,
    body:not(.command-center-only) .users-meta-card,
    body:not(.command-center-only) .api-note-card {
      border-radius: 18px;
      border: 1px solid rgba(94, 119, 158, 0.18);
      background: rgba(10, 18, 32, 0.82);
      padding: 16px;
      display: grid;
      gap: 8px;
    }

    body:not(.command-center-only) .overview-kpi-card span,
    body:not(.command-center-only) .users-meta-card span,
    body:not(.command-center-only) .api-note-card span {
      font-size: 11px;
      letter-spacing: 0.12em;
      text-transform: uppercase;
      color: #8ea4bf;
    }

    body:not(.command-center-only) .overview-kpi-card strong,
    body:not(.command-center-only) .users-meta-card strong,
    body:not(.command-center-only) .api-note-card strong {
      font-size: clamp(24px, 2.2vw, 32px);
      line-height: 1;
      color: #f8fafc;
      font-family: var(--font-display);
    }

    body:not(.command-center-only) .overview-kpi-card small,
    body:not(.command-center-only) .users-meta-card small,
    body:not(.command-center-only) .api-note-card small {
      color: #8ea4bf;
      line-height: 1.55;
    }

    body:not(.command-center-only) .table-wrap {
      border-color: rgba(94, 119, 158, 0.2);
      border-radius: 18px;
      background: rgba(4, 9, 18, 0.62);
    }

    body:not(.command-center-only) thead th {
      color: #8ea4bf;
      background: rgba(15, 23, 42, 0.94);
      border-bottom-color: rgba(94, 119, 158, 0.2);
    }

    body:not(.command-center-only) tbody td {
      border-bottom-color: rgba(94, 119, 158, 0.14);
      color: #dce6f4;
    }

    body:not(.command-center-only) tbody tr:hover {
      background: rgba(34, 197, 94, 0.06);
    }

    body:not(.command-center-only) pre {
      border: 1px solid rgba(94, 119, 158, 0.2);
      border-radius: 18px;
      background: rgba(4, 9, 18, 0.92);
      color: #dce6f4;
      padding: 14px;
    }

    body:not(.command-center-only) .ops-main {
      border-color: rgba(94, 119, 158, 0.2);
      border-radius: 24px;
      padding: 18px;
      gap: 16px;
      background:
        linear-gradient(180deg, rgba(8, 15, 28, 0.98), rgba(9, 17, 31, 0.94)),
        radial-gradient(circle at top, rgba(34, 197, 94, 0.08), transparent 30%);
    }

    body:not(.command-center-only) .ops-main-head {
      border-bottom-color: rgba(94, 119, 158, 0.18);
      padding-bottom: 14px;
      gap: 14px;
    }

    body:not(.command-center-only) .ops-main-head-meta {
      display: flex;
      gap: 10px;
      flex-wrap: wrap;
      margin-top: 8px;
    }

    body:not(.command-center-only) .ops-module-chip {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      padding: 6px 12px;
      border-radius: 999px;
      border: 1px solid rgba(94, 119, 158, 0.18);
      background: rgba(15, 23, 42, 0.88);
      color: #c7d5e7;
      font-size: 12px;
    }

    body:not(.command-center-only) .ops-module-chip strong {
      color: #67e8f9;
      font-family: var(--font-display);
      font-size: 12px;
    }

    body:not(.command-center-only) .ops-workspace {
      gap: 16px;
    }

    body:not(.command-center-only) .ops-card {
      border-color: rgba(94, 119, 158, 0.18);
      border-radius: 20px;
      padding: 16px;
      background:
        linear-gradient(180deg, rgba(10, 18, 32, 0.9), rgba(7, 13, 25, 0.94)),
        radial-gradient(circle at top right, rgba(56, 189, 248, 0.06), transparent 34%);
      gap: 12px;
    }

    body:not(.command-center-only) .ops-kpi {
      border-color: rgba(94, 119, 158, 0.18);
      border-radius: 18px;
      background: rgba(10, 18, 32, 0.88);
      padding: 16px;
      gap: 8px;
    }

    body:not(.command-center-only) .ops-kpi .num {
      color: #f8fafc;
      font-size: 28px;
    }

    body:not(.command-center-only) .ops-kpi .label {
      color: #8ea4bf;
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }

    body:not(.command-center-only) .ops-inline-note {
      margin: 0;
      font-size: 12px;
      line-height: 1.6;
      color: #8ea4bf;
    }

    body:not(.command-center-only) .ops-highlight-grid,
    body:not(.command-center-only) .ops-split-grid,
    body:not(.command-center-only) .ops-summary-grid {
      display: grid;
      gap: 14px;
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    body:not(.command-center-only) .ops-highlight-card,
    body:not(.command-center-only) .ops-summary-card {
      border-radius: 18px;
      border: 1px solid rgba(94, 119, 158, 0.18);
      background: rgba(6, 12, 24, 0.92);
      padding: 16px;
      display: grid;
      gap: 8px;
    }

    body:not(.command-center-only) .ops-highlight-card span,
    body:not(.command-center-only) .ops-summary-card span {
      font-size: 11px;
      letter-spacing: 0.12em;
      text-transform: uppercase;
      color: #8ea4bf;
    }

    body:not(.command-center-only) .ops-highlight-card strong,
    body:not(.command-center-only) .ops-summary-card strong {
      color: #f8fafc;
      font-family: var(--font-display);
      font-size: 26px;
      line-height: 1;
    }

    body:not(.command-center-only) .ops-highlight-card small,
    body:not(.command-center-only) .ops-summary-card small {
      color: #8ea4bf;
      line-height: 1.55;
    }

    body:not(.command-center-only) .ops-review-table td,
    body:not(.command-center-only) .ops-review-table th,
    body:not(.command-center-only) .ops-plan-table td,
    body:not(.command-center-only) .ops-plan-table th {
      white-space: normal;
    }

    body:not(.command-center-only) .ops-cell-stack {
      display: grid;
      gap: 4px;
    }

    body:not(.command-center-only) .ops-cell-stack strong {
      color: #f8fafc;
    }

    body:not(.command-center-only) .ops-cell-stack span,
    body:not(.command-center-only) .ops-cell-stack small {
      color: #8ea4bf;
      line-height: 1.45;
    }

    body:not(.command-center-only) .ops-plan-meter {
      height: 8px;
      border-radius: 999px;
      overflow: hidden;
      background: rgba(15, 23, 42, 0.94);
    }

    body:not(.command-center-only) .ops-plan-meter-fill {
      height: 100%;
      border-radius: inherit;
      background: linear-gradient(90deg, rgba(34, 197, 94, 0.96), rgba(56, 189, 248, 0.96));
    }

    body:not(.command-center-only) .ops-paged-actions {
      justify-content: space-between;
    }

    body:not(.command-center-only) .login-card {
      width: min(520px, 100%);
      padding: 24px;
      border-color: rgba(94, 119, 158, 0.22);
      background:
        radial-gradient(circle at left top, rgba(34, 197, 94, 0.16), transparent 38%),
        linear-gradient(180deg, rgba(7, 13, 25, 0.98), rgba(10, 18, 32, 0.94));
      box-shadow:
        inset 0 1px 0 rgba(226, 232, 240, 0.05),
        0 30px 80px rgba(2, 6, 23, 0.4);
    }

    body:not(.command-center-only) .login-title {
      font-size: 26px;
    }

    body:not(.command-center-only) .login-error {
      color: #fca5a5;
    }

    body:not(.command-center-only) .toast {
      background: rgba(8, 15, 28, 0.96);
      border-color: rgba(94, 119, 158, 0.22);
      box-shadow: 0 18px 44px rgba(2, 6, 23, 0.38);
    }

    body:not(.command-center-only) .menu-group + .menu-group {
      padding-top: 14px;
      border-top: 1px solid rgba(94, 119, 158, 0.16);
    }

    body:not(.command-center-only) .menu-group-title,
    body:not(.command-center-only) .menu-subgroup-title {
      color: #67e8f9;
    }

    body:not(.command-center-only) .menu-group-meta,
    body:not(.command-center-only) .menu-subgroup-count {
      border-color: rgba(94, 119, 158, 0.2);
      background: rgba(15, 23, 42, 0.9);
      color: #c7d5e7;
    }

    body:not(.command-center-only) .menu-subgroup + .menu-subgroup {
      padding-top: 10px;
      border-top: 1px dashed rgba(94, 119, 158, 0.14);
    }

    body:not(.command-center-only) .menu-filter-field input {
      background: rgba(15, 23, 42, 0.9);
      border-color: rgba(94, 119, 158, 0.2);
    }

    body:not(.command-center-only) .theme-switcher {
      color: #dce6f4;
      border-color: rgba(94, 119, 158, 0.2);
      background: rgba(8, 15, 28, 0.86);
    }

    body:not(.command-center-only) .theme-switcher-btn.active {
      background: rgba(34, 197, 94, 0.18);
      color: #86efac;
    }

    body:not(.command-center-only)[data-admin-theme="light"] {
      color: #122033;
      background:
        radial-gradient(circle at 10% 12%, rgba(16, 185, 129, 0.16) 0%, transparent 28%),
        radial-gradient(circle at 88% 0%, rgba(14, 165, 233, 0.14) 0%, transparent 26%),
        linear-gradient(180deg, #eff6ff 0%, #f8fafc 50%, #e2e8f0 100%);
    }

    body:not(.command-center-only)[data-admin-theme="light"]::before {
      opacity: 0.1;
      background:
        linear-gradient(rgba(37, 99, 235, 0.08) 1px, transparent 1px),
        linear-gradient(90deg, rgba(37, 99, 235, 0.06) 1px, transparent 1px);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .sidebar,
    body:not(.command-center-only)[data-admin-theme="light"] .content {
      border-color: rgba(148, 163, 184, 0.26);
      background:
        linear-gradient(180deg, rgba(255, 255, 255, 0.96), rgba(248, 250, 252, 0.94)),
        radial-gradient(circle at top, rgba(14, 165, 233, 0.08), transparent 34%);
      box-shadow:
        inset 0 1px 0 rgba(255, 255, 255, 0.75),
        0 24px 60px rgba(148, 163, 184, 0.18);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .brand {
      border-color: rgba(148, 163, 184, 0.22);
      background:
        radial-gradient(circle at top left, rgba(16, 185, 129, 0.14), transparent 46%),
        linear-gradient(135deg, rgba(255, 255, 255, 0.98), rgba(241, 245, 249, 0.96));
    }

    body:not(.command-center-only)[data-admin-theme="light"] .brand-logo {
      background: #ffffff;
      border-color: rgba(16, 185, 129, 0.2);
      box-shadow: 0 10px 20px rgba(148, 163, 184, 0.16);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .brand-title,
    body:not(.command-center-only)[data-admin-theme="light"] .login-title,
    body:not(.command-center-only)[data-admin-theme="light"] .box-title,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-card-title,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-main-title,
    body:not(.command-center-only)[data-admin-theme="light"] .section-heading,
    body:not(.command-center-only)[data-admin-theme="light"] .toolbar-heading {
      color: #0f172a;
    }

    body:not(.command-center-only)[data-admin-theme="light"] .brand-subtitle,
    body:not(.command-center-only)[data-admin-theme="light"] .login-subtitle,
    body:not(.command-center-only)[data-admin-theme="light"] .empty,
    body:not(.command-center-only)[data-admin-theme="light"] .card .hint,
    body:not(.command-center-only)[data-admin-theme="light"] .section-copy,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-main-subtitle,
    body:not(.command-center-only)[data-admin-theme="light"] .toolbar-subheading,
    body:not(.command-center-only)[data-admin-theme="light"] .hint-text,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-inline-note,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-cell-stack span,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-cell-stack small,
    body:not(.command-center-only)[data-admin-theme="light"] .overview-kpi-card small,
    body:not(.command-center-only)[data-admin-theme="light"] .users-meta-card small,
    body:not(.command-center-only)[data-admin-theme="light"] .api-note-card small,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-highlight-card small,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-summary-card small,
    body:not(.command-center-only)[data-admin-theme="light"] .toolbar-stat .field label,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-helper-item p {
      color: #475569;
    }

    body:not(.command-center-only)[data-admin-theme="light"] .sidebar-section,
    body:not(.command-center-only)[data-admin-theme="light"] .sidebar-mini-card,
    body:not(.command-center-only)[data-admin-theme="light"] .toolbar-head,
    body:not(.command-center-only)[data-admin-theme="light"] .toolbar-stat,
    body:not(.command-center-only)[data-admin-theme="light"] .panel-shell,
    body:not(.command-center-only)[data-admin-theme="light"] .panel-box,
    body:not(.command-center-only)[data-admin-theme="light"] .overview-kpi-card,
    body:not(.command-center-only)[data-admin-theme="light"] .users-meta-card,
    body:not(.command-center-only)[data-admin-theme="light"] .api-note-card,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-main,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-card,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-kpi,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-highlight-card,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-summary-card,
    body:not(.command-center-only)[data-admin-theme="light"] .overview-summary-card,
    body:not(.command-center-only)[data-admin-theme="light"] .overview-rule-item,
    body:not(.command-center-only)[data-admin-theme="light"] .overview-quick-card,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-helper-item {
      border-color: rgba(148, 163, 184, 0.22);
      background:
        linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(248, 250, 252, 0.94)),
        radial-gradient(circle at top right, rgba(14, 165, 233, 0.05), transparent 34%);
      box-shadow:
        inset 0 1px 0 rgba(255, 255, 255, 0.82),
        0 16px 36px rgba(148, 163, 184, 0.14);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .sidebar-section-title,
    body:not(.command-center-only)[data-admin-theme="light"] .section-eyebrow,
    body:not(.command-center-only)[data-admin-theme="light"] .menu-group-title,
    body:not(.command-center-only)[data-admin-theme="light"] .menu-subgroup-title {
      color: #0284c7;
    }

    body:not(.command-center-only)[data-admin-theme="light"] .menu-group-meta,
    body:not(.command-center-only)[data-admin-theme="light"] .menu-subgroup-count {
      border-color: rgba(148, 163, 184, 0.22);
      background: rgba(241, 245, 249, 0.92);
      color: #0369a1;
    }

    body:not(.command-center-only)[data-admin-theme="light"] .menu-group + .menu-group,
    body:not(.command-center-only)[data-admin-theme="light"] .menu-subgroup + .menu-subgroup,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-main-head {
      border-color: rgba(148, 163, 184, 0.22);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .menu-btn {
      color: #1e293b;
      border-color: rgba(148, 163, 184, 0.22);
      background: rgba(255, 255, 255, 0.94);
      box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.9);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .menu-btn:hover {
      border-color: rgba(14, 165, 233, 0.34);
      background: rgba(255, 255, 255, 1);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .menu-btn.active {
      border-color: rgba(16, 185, 129, 0.34);
      background:
        radial-gradient(circle at left top, rgba(16, 185, 129, 0.16), transparent 46%),
        linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(240, 253, 250, 0.96));
      color: #0f172a;
    }

    body:not(.command-center-only)[data-admin-theme="light"] .menu-btn-desc,
    body:not(.command-center-only)[data-admin-theme="light"] .sidebar-mini-card span,
    body:not(.command-center-only)[data-admin-theme="light"] .toolbar-chip,
    body:not(.command-center-only)[data-admin-theme="light"] .overview-kpi-card span,
    body:not(.command-center-only)[data-admin-theme="light"] .users-meta-card span,
    body:not(.command-center-only)[data-admin-theme="light"] .api-note-card span,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-kpi .label,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-highlight-card span,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-summary-card span {
      color: #475569;
    }

    body:not(.command-center-only)[data-admin-theme="light"] .menu-btn-index,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-module-chip,
    body:not(.command-center-only)[data-admin-theme="light"] .toolbar-chip,
    body:not(.command-center-only)[data-admin-theme="light"] .muted-code {
      border-color: rgba(148, 163, 184, 0.22);
      background: rgba(241, 245, 249, 0.92);
      color: #0369a1;
    }

    body:not(.command-center-only)[data-admin-theme="light"] .toolbar-chip strong,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-module-chip strong,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-helper-item strong {
      color: #0f172a;
    }

    body:not(.command-center-only)[data-admin-theme="light"] .field label {
      color: #334155;
    }

    body:not(.command-center-only)[data-admin-theme="light"] .field input,
    body:not(.command-center-only)[data-admin-theme="light"] .field select,
    body:not(.command-center-only)[data-admin-theme="light"] .field textarea {
      border-color: rgba(148, 163, 184, 0.24);
      background: rgba(255, 255, 255, 0.96);
      color: #0f172a;
    }

    body:not(.command-center-only)[data-admin-theme="light"] .field input:disabled {
      background: rgba(241, 245, 249, 0.94);
      color: #0f172a;
    }

    body:not(.command-center-only)[data-admin-theme="light"] .btn.primary {
      background: linear-gradient(135deg, #0f766e, #14b8a6);
      color: #f8fafc;
      box-shadow: 0 10px 24px rgba(20, 184, 166, 0.22);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .btn.ghost {
      background: rgba(255, 255, 255, 0.98);
      color: #0f172a;
      border-color: rgba(148, 163, 184, 0.24);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .btn.warn {
      background: rgba(255, 247, 237, 0.96);
      color: #b45309;
      border-color: rgba(245, 158, 11, 0.24);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .btn.danger {
      background: rgba(254, 242, 242, 0.96);
      color: #b91c1c;
      border-color: rgba(248, 113, 113, 0.24);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .badge.ok {
      color: #047857;
      background: rgba(236, 253, 245, 0.96);
      border-color: rgba(16, 185, 129, 0.24);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .badge.warn {
      color: #b45309;
      background: rgba(255, 247, 237, 0.96);
      border-color: rgba(245, 158, 11, 0.24);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .command-center-shell {
      color: #1e293b;
      border-color: rgba(148, 163, 184, 0.22);
      background:
        radial-gradient(720px 420px at 0% 0%, rgba(14, 165, 233, 0.12), transparent 55%),
        radial-gradient(680px 420px at 100% 0%, rgba(16, 185, 129, 0.1), transparent 55%),
        linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(248, 250, 252, 0.96));
      box-shadow: 0 24px 60px rgba(148, 163, 184, 0.18);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .command-center-shell::before,
    body:not(.command-center-only)[data-admin-theme="light"] .command-center-shell::after {
      opacity: 0.45;
    }

    body:not(.command-center-only)[data-admin-theme="light"] .command-panel,
    body:not(.command-center-only)[data-admin-theme="light"] .command-strip-item,
    body:not(.command-center-only)[data-admin-theme="light"] .command-metric-card,
    body:not(.command-center-only)[data-admin-theme="light"] .command-feed-item {
      border-color: rgba(148, 163, 184, 0.22);
      background:
        linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(248, 250, 252, 0.96)),
        radial-gradient(circle at top right, rgba(14, 165, 233, 0.06), transparent 34%);
      box-shadow: 0 14px 34px rgba(148, 163, 184, 0.14);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .command-hero-panel,
    body:not(.command-center-only)[data-admin-theme="light"] .command-focus-card,
    body:not(.command-center-only)[data-admin-theme="light"] .command-inline-stat,
    body:not(.command-center-only)[data-admin-theme="light"] .command-health-item,
    body:not(.command-center-only)[data-admin-theme="light"] .command-user-row,
    body:not(.command-center-only)[data-admin-theme="light"] .command-log-grid > div,
    body:not(.command-center-only)[data-admin-theme="light"] .command-node-stats > div,
    body:not(.command-center-only)[data-admin-theme="light"] .command-node-card,
    body:not(.command-center-only)[data-admin-theme="light"] .command-chart-shell,
    body:not(.command-center-only)[data-admin-theme="light"] .command-node-spark {
      border-color: rgba(148, 163, 184, 0.22);
      background:
        linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(248, 250, 252, 0.96)),
        radial-gradient(circle at top right, rgba(14, 165, 233, 0.06), transparent 34%);
      box-shadow:
        inset 0 1px 0 rgba(255, 255, 255, 0.86),
        0 14px 34px rgba(148, 163, 184, 0.14);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .command-toolbar h1,
    body:not(.command-center-only)[data-admin-theme="light"] .command-panel h3,
    body:not(.command-center-only)[data-admin-theme="light"] .command-feed-top strong,
    body:not(.command-center-only)[data-admin-theme="light"] .command-strip-item strong,
    body:not(.command-center-only)[data-admin-theme="light"] .command-metric-card strong,
    body:not(.command-center-only)[data-admin-theme="light"] .overview-summary-card strong,
    body:not(.command-center-only)[data-admin-theme="light"] .overview-rule-item strong,
    body:not(.command-center-only)[data-admin-theme="light"] .overview-quick-card strong,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-kpi .num,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-highlight-card strong,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-summary-card strong,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-cell-stack strong,
    body:not(.command-center-only)[data-admin-theme="light"] .command-focus-card strong,
    body:not(.command-center-only)[data-admin-theme="light"] .command-inline-stat strong,
    body:not(.command-center-only)[data-admin-theme="light"] .command-health-item strong,
    body:not(.command-center-only)[data-admin-theme="light"] .command-user-identity strong,
    body:not(.command-center-only)[data-admin-theme="light"] .command-user-metrics strong,
    body:not(.command-center-only)[data-admin-theme="light"] .command-log-grid strong,
    body:not(.command-center-only)[data-admin-theme="light"] .command-node-stats strong,
    body:not(.command-center-only)[data-admin-theme="light"] .command-node-title h4,
    body:not(.command-center-only)[data-admin-theme="light"] .command-node-meter-top strong,
    body:not(.command-center-only)[data-admin-theme="light"] .command-bar-copy strong,
    body:not(.command-center-only)[data-admin-theme="light"] .command-bar-value {
      color: #0f172a;
    }

    body:not(.command-center-only)[data-admin-theme="light"] .command-toolbar p,
    body:not(.command-center-only)[data-admin-theme="light"] .command-panel-kicker,
    body:not(.command-center-only)[data-admin-theme="light"] .command-feed-meta,
    body:not(.command-center-only)[data-admin-theme="light"] .command-feed-sub,
    body:not(.command-center-only)[data-admin-theme="light"] .overview-summary-card small,
    body:not(.command-center-only)[data-admin-theme="light"] .overview-rule-item p,
    body:not(.command-center-only)[data-admin-theme="light"] .overview-quick-card p,
    body:not(.command-center-only)[data-admin-theme="light"] .ops-card-subtitle,
    body:not(.command-center-only)[data-admin-theme="light"] .command-focus-card small,
    body:not(.command-center-only)[data-admin-theme="light"] .command-health-item span,
    body:not(.command-center-only)[data-admin-theme="light"] .command-user-identity span,
    body:not(.command-center-only)[data-admin-theme="light"] .command-user-metrics span,
    body:not(.command-center-only)[data-admin-theme="light"] .command-log-grid span,
    body:not(.command-center-only)[data-admin-theme="light"] .command-node-stats span,
    body:not(.command-center-only)[data-admin-theme="light"] .command-node-subline,
    body:not(.command-center-only)[data-admin-theme="light"] .command-node-owner,
    body:not(.command-center-only)[data-admin-theme="light"] .command-node-meter-bottom,
    body:not(.command-center-only)[data-admin-theme="light"] .command-node-alert,
    body:not(.command-center-only)[data-admin-theme="light"] .command-spark-empty,
    body:not(.command-center-only)[data-admin-theme="light"] .command-node-meter-top span,
    body:not(.command-center-only)[data-admin-theme="light"] .command-bar-copy span {
      color: #334155;
    }

    body:not(.command-center-only)[data-admin-theme="light"] .command-user-bar,
    body:not(.command-center-only)[data-admin-theme="light"] .command-bar-track,
    body:not(.command-center-only)[data-admin-theme="light"] .command-meter {
      background: rgba(226, 232, 240, 0.88);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .command-badge {
      border-color: rgba(148, 163, 184, 0.22);
      background: rgba(248, 250, 252, 0.98);
      color: #334155;
    }

    body:not(.command-center-only)[data-admin-theme="light"] .command-badge.ok {
      color: #047857;
      border-color: rgba(16, 185, 129, 0.24);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .command-badge.cyan,
    body:not(.command-center-only)[data-admin-theme="light"] .command-badge.teal {
      color: #0369a1;
      border-color: rgba(14, 165, 233, 0.22);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .table-wrap {
      border-color: rgba(148, 163, 184, 0.24);
      background: rgba(255, 255, 255, 0.98);
    }

    body:not(.command-center-only)[data-admin-theme="light"] thead th {
      color: #334155;
      background: rgba(241, 245, 249, 0.94);
      border-bottom-color: rgba(148, 163, 184, 0.22);
    }

    body:not(.command-center-only)[data-admin-theme="light"] tbody td {
      color: #1e293b;
      border-bottom-color: rgba(148, 163, 184, 0.16);
    }

    body:not(.command-center-only)[data-admin-theme="light"] tbody tr:hover {
      background: rgba(236, 253, 245, 0.7);
    }

    body:not(.command-center-only)[data-admin-theme="light"] pre,
    body:not(.command-center-only)[data-admin-theme="light"] .toast,
    body:not(.command-center-only)[data-admin-theme="light"] .theme-switcher,
    body:not(.command-center-only)[data-admin-theme="light"] .meta,
    body:not(.command-center-only)[data-admin-theme="light"] .login-card {
      border-color: rgba(148, 163, 184, 0.24);
      background:
        linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(248, 250, 252, 0.96)),
        radial-gradient(circle at top right, rgba(14, 165, 233, 0.06), transparent 38%);
      color: #1e293b;
      box-shadow:
        inset 0 1px 0 rgba(255, 255, 255, 0.84),
        0 18px 44px rgba(148, 163, 184, 0.18);
    }

    body:not(.command-center-only)[data-admin-theme="light"] .login-error {
      color: #b91c1c;
    }

    body:not(.command-center-only)[data-admin-theme="light"] .theme-switcher-btn.active {
      background: rgba(14, 165, 233, 0.14);
      color: #0369a1;
    }

    @media (max-width: 1200px) {
      body:not(.command-center-only) .session-toolbar,
      body:not(.command-center-only) .overview-card-grid,
      body:not(.command-center-only) .users-meta-grid,
      body:not(.command-center-only) .api-note-grid {
        grid-template-columns: repeat(2, minmax(0, 1fr));
      }

      body:not(.command-center-only) .ops-shell {
        min-height: 680px;
      }
    }

    @media (max-width: 960px) {
      body:not(.command-center-only) .session-toolbar,
      body:not(.command-center-only) .overview-card-grid,
      body:not(.command-center-only) .users-meta-grid,
      body:not(.command-center-only) .api-note-grid,
      body:not(.command-center-only) .ops-highlight-grid,
      body:not(.command-center-only) .ops-split-grid,
      body:not(.command-center-only) .ops-summary-grid,
      body:not(.command-center-only) .sidebar-mini-grid {
        grid-template-columns: 1fr;
      }

      body:not(.command-center-only) .overview-head,
      body:not(.command-center-only) .users-head,
      body:not(.command-center-only) .api-head {
        flex-direction: column;
      }
    }
  </style>
</head>

<body class="{{ !empty($command_center_only) ? 'command-center-only' : '' }}" data-admin-theme="dark">
  <div class="theme-switcher {{ !empty($command_center_only) ? 'hidden' : '' }}" id="themeSwitcher" aria-label="管理员主题切换">
    <button type="button" class="theme-switcher-btn active" id="themeDarkBtn" data-theme="dark" aria-pressed="true">暗黑</button>
    <button type="button" class="theme-switcher-btn" id="themeLightBtn" data-theme="light" aria-pressed="false">明亮</button>
  </div>
  <section class="login-view" id="loginView">
    <div class="login-card">
      <h1 class="login-title">{{ $title }} 管理员登录</h1>
      <p class="login-subtitle">未授权访问控制面板将自动返回到登录界面。登录成功后才会加载后台数据。</p>
      <div class="field">
        <label for="loginEmailInput">管理员邮箱</label>
        <input id="loginEmailInput" type="email" placeholder="admin@example.com" autocomplete="username" />
      </div>
      <div class="field">
        <label for="loginPasswordInput">密码</label>
        <input id="loginPasswordInput" type="password" placeholder="至少 8 位" autocomplete="current-password" />
      </div>
      <div class="field hidden" id="loginCaptchaField">
        <label>验证码</label>
        <div id="loginCaptchaWidget"></div>
        <div class="login-subtitle" id="loginCaptchaHint" style="margin-top: 8px;"></div>
      </div>
      <p class="login-subtitle hidden" id="loginPowHint"></p>
      <div class="actions">
        <button class="btn primary" id="loginSubmitBtn">登录控制台</button>
      </div>
      <p class="login-error" id="loginErrorText"></p>
    </div>
  </section>

  <div class="layout hidden" id="adminLayout">
    <aside class="sidebar">
      <div class="brand">
        <div class="brand-logo">
          <img id="brandLogo" alt="logo" src="{{ $logo ?: '' }}" />
        </div>
        <div>
          <h1 class="brand-title">{{ $title }}</h1>
          <p class="brand-subtitle">Admin Control Center · v{{ $version }}</p>
        </div>
      </div>

      <section class="sidebar-section">
        <p class="sidebar-section-title">Super Admin Workspace</p>
        <div class="sidebar-mini-grid">
          <div class="sidebar-mini-card">
            <span>后台入口</span>
            <strong id="securePathText">/{{ $secure_path }}</strong>
          </div>
          <div class="sidebar-mini-card">
            <span>监控大屏</span>
            <strong>/{{ $secure_path }}/command-center</strong>
          </div>
        </div>
      </section>

      <nav class="menu" id="menuTabs">
        <section class="menu-group">
          <div class="menu-group-head">
            <p class="menu-group-title">核心入口</p>
            <span class="menu-group-meta">3 个入口</span>
          </div>
          <button class="menu-btn active" data-tab="overview">
            <span class="menu-btn-main">
              <span class="menu-btn-title">指挥台</span>
              <span class="menu-btn-desc">今日工作流、基础 KPI 和独立监控入口</span>
            </span>
            <span class="menu-btn-index">01</span>
          </button>
          <button class="menu-btn" data-tab="users">
            <span class="menu-btn-main">
              <span class="menu-btn-title">用户与封禁</span>
              <span class="menu-btn-desc">处理风险用户、封禁原因与订阅密钥</span>
            </span>
            <span class="menu-btn-index">02</span>
          </button>
          <button class="menu-btn" data-tab="api">
            <span class="menu-btn-main">
              <span class="menu-btn-title">高级请求工具</span>
              <span class="menu-btn-desc">直接调试 API、配置与系统状态</span>
            </span>
            <span class="menu-btn-index">03</span>
          </button>
        </section>

        <section class="menu-group" id="primaryOpsMenuGroup">
          <div class="menu-group-head">
            <p class="menu-group-title">全功能模块</p>
            <span class="menu-group-meta" id="primaryOpsMenuCount" role="status" aria-live="polite">14 个模块</span>
          </div>
          <div class="field menu-filter-field">
            <label for="primaryOpsMenuSearch">模块检索</label>
            <input id="primaryOpsMenuSearch" type="search" placeholder="搜索风险、套餐、支付、工单..." />
          </div>
          <div class="menu-module-list" id="primaryOpsMenuList">
            <p class="empty">正在加载一级模块菜单...</p>
          </div>
        </section>
      </nav>

      <div class="meta">
        <div><strong>控制台说明</strong></div>
        <div style="margin-top: 6px;">此管理台直接调用后端接口，不依赖原前端加密资源。适合超管做日常运营、风控和配置维护。</div>
        <div style="margin-top: 8px;"><strong>登录态</strong>：同域下可与独立监控大屏共享，避免双后台重复登录。</div>
      </div>
    </aside>

    <main class="content">
      <section class="toolbar session-toolbar" id="sessionToolbar">
        <div class="toolbar-head">
          <p class="section-eyebrow">SUPER ADMIN WORKSPACE</p>
          <h2 class="toolbar-heading">高级管理员控制台</h2>
          <p class="toolbar-subheading">把入口安全、用户封禁、风险审查、套餐运维、支付与系统治理收敛到一个可快速执行的深色操作面中。</p>
          <div class="toolbar-chip-row">
            <span class="toolbar-chip"><strong>Secure Path</strong> /{{ $secure_path }}</span>
            <span class="toolbar-chip"><strong>Mode</strong> Native Admin</span>
            <span class="toolbar-chip"><strong>Deploy</strong> Docker Runtime</span>
          </div>
        </div>
        <div class="toolbar-stat">
          <div class="field">
            <label>会话状态</label>
            <div class="actions">
              <span id="authState" class="badge warn">未认证</span>
              <span class="badge ok">Secure Console</span>
            </div>
          </div>
        </div>
        <div class="toolbar-stat">
          <div class="field">
            <label>当前管理员</label>
            <input id="currentAdminText" value="-" disabled />
          </div>
        </div>
        <div class="toolbar-stat">
          <div class="field">
            <label>指挥动作</label>
            <div class="actions">
              <button class="btn ghost" id="quickUsersBtn">用户面板</button>
              <a class="btn ghost" id="openCommandCenterBtn" href="/{{ $secure_path }}/command-center" target="_blank" rel="noreferrer">监控大屏</a>
              <button class="btn primary" id="refreshOverviewBtn">同步总览</button>
              <button class="btn warn" id="logoutBtn">退出登录</button>
            </div>
          </div>
        </div>
      </section>

      <section class="panels">
        <div class="panel active" data-panel="overview" id="panelOverview">
          <section class="command-center-shell">
            <div class="command-toolbar">
              <div>
                <div class="command-kicker">SUPER ADMIN OVERVIEW</div>
                <h1>Command Deck</h1>
                <p>这里保留后台运营总览和快捷入口，完整的超级管理员监控大屏仍然独立出去，避免公开概览与超管监控视图耦合。</p>
              </div>
              <div class="command-hero-badges">
                <span class="command-badge neutral">后台 /{{ $secure_path }}</span>
                <span class="command-badge cyan">原生登录态同步</span>
                <span class="command-badge ok">Docked Control Surface</span>
              </div>
            </div>

            <div class="command-status-strip">
              <div class="command-strip-item">
                <span>自动刷新</span>
                <strong id="commandCenterCountdown">--</strong>
              </div>
              <div class="command-strip-item">
                <span>最近快照</span>
                <strong id="commandCenterUpdatedAt">-</strong>
              </div>
              <div class="command-strip-item">
                <span>监控状态</span>
                <strong id="commandCenterStatusText" data-tone="neutral">准备同步</strong>
              </div>
            </div>

            <div class="command-metrics-grid" id="overviewCards">
              <article class="command-metric-card teal">
                <span>当月收入</span>
                <strong id="kpiMonthIncome">-</strong>
                <small>单位：系统货币</small>
              </article>
              <article class="command-metric-card cyan">
                <span>当月新增用户</span>
                <strong id="kpiMonthUsers">-</strong>
                <small>注册统计</small>
              </article>
              <article class="command-metric-card amber">
                <span>待处理工单</span>
                <strong id="kpiPendingTicket">-</strong>
                <small>需要管理员跟进</small>
              </article>
              <article class="command-metric-card blue">
                <span>在线用户</span>
                <strong id="kpiOnlineUsers">-</strong>
                <small>最近 10 分钟活跃</small>
              </article>
            </div>

            <div id="commandCenterDashboard" class="command-center-content">
              <div class="command-loading">
                <div class="command-loading-ring"></div>
                <div>正在载入后台 overview 统计...</div>
              </div>
            </div>

            <p class="empty command-shell-empty" id="overviewHint">登录后会自动同步后台 overview 统计，独立监控大屏请从上方入口打开。</p>
          </section>
        </div>

        <div class="panel" data-panel="users" id="panelUsers">
          <div class="panel-box panel-shell">
            <div class="users-head">
              <div>
                <p class="section-eyebrow">USER OPS</p>
                <h2 class="section-heading">用户与封禁操作台</h2>
                <p class="section-copy">在一个页面里完成用户查阅、封禁/解封、封禁原因记录与订阅密钥重置，适合快速处理风控与客服场景。</p>
              </div>
              <div class="actions">
                <button class="btn primary" id="loadUsersBtn">刷新用户列表</button>
                <button class="btn ghost" id="prevUsersBtn">上一页</button>
                <button class="btn ghost" id="nextUsersBtn">下一页</button>
                <span class="badge ok" id="usersPageInfo">页码 1</span>
              </div>
            </div>

            <div class="users-meta-grid">
              <article class="users-meta-card">
                <span>处理目标</span>
                <strong>封禁与复核</strong>
                <small>封禁必须填写原因，便于后续人工回溯。</small>
              </article>
              <article class="users-meta-card">
                <span>高频动作</span>
                <strong>订阅密钥重置</strong>
                <small>在风控、换密和泄漏处置时直接执行。</small>
              </article>
              <article class="users-meta-card">
                <span>当前页</span>
                <strong>分页巡检</strong>
                <small>保持短列表节奏，适合快速翻页和人工逐条处理。</small>
              </article>
              <article class="users-meta-card">
                <span>审查配合</span>
                <strong>风险模块联动</strong>
                <small>可先在风控页得出建议，再回到这里做最终操作。</small>
              </article>
            </div>

            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>邮箱</th>
                    <th>套餐</th>
                    <th>余额</th>
                    <th>状态</th>
                    <th>封禁原因</th>
                    <th>管理员</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="usersTableBody">
                  <tr>
                    <td colspan="8" class="empty">暂无数据，请点击“刷新列表”。</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <div class="panel" data-panel="api" id="panelApi">
          <div class="panel-box panel-shell">
            <div class="api-head">
              <div>
                <p class="section-eyebrow">ADVANCED REQUEST TOOL</p>
                <h2 class="section-heading">高级请求工具</h2>
                <p class="section-copy">直接构造管理接口请求，适合验证新接口、排查权限问题和调试灰度功能。</p>
              </div>
              <div class="actions">
                <button class="btn primary" id="sendApiBtn">发送请求</button>
                <button class="btn ghost" id="presetThemeBtn">系统状态示例</button>
                <button class="btn ghost" id="presetLimitBtn">并发 IP 示例</button>
              </div>
            </div>

            <div class="api-note-grid">
              <article class="api-note-card">
                <span>路径策略</span>
                <strong>V2 / V1 / Custom</strong>
                <small>既支持新版后台，也保留兼容路由做精细排错。</small>
              </article>
              <article class="api-note-card">
                <span>调试方式</span>
                <strong>原始 JSON</strong>
                <small>不隐藏字段，方便直接比对返回载荷。</small>
              </article>
              <article class="api-note-card">
                <span>适用场景</span>
                <strong>接口验收</strong>
                <small>适合上线前检查权限、参数校验和响应结构。</small>
              </article>
              <article class="api-note-card">
                <span>风险控制</span>
                <strong>同账号直连</strong>
                <small>避免切换多个后台导致状态不一致。</small>
              </article>
            </div>

            <div class="toolbar">
              <div class="field span-3">
                <label for="apiVersionSelect">请求路径类型</label>
                <select id="apiVersionSelect">
                  <option value="v2">/api/v2（新版）</option>
                  <option value="v1">/api/v1（兼容版）</option>
                  <option value="custom">完整地址（自定义）</option>
                </select>
              </div>
              <div class="field span-2">
                <label for="apiMethodSelect">请求方法</label>
                <select id="apiMethodSelect">
                  <option value="GET">GET</option>
                  <option value="POST">POST</option>
                  <option value="PUT">PUT</option>
                  <option value="DELETE">DELETE</option>
                </select>
              </div>
              <div class="field span-7">
                <label for="apiEndpointInput">请求地址</label>
                <input id="apiEndpointInput" placeholder="例如：system/getSystemStatus 或 admin/users/1/concurrent-ip-limit" />
              </div>
              <div class="field span-12">
                <label for="apiBodyInput">请求内容（可选，按示例填写）</label>
                <textarea id="apiBodyInput" placeholder='例如：{"concurrent_ip_limit":2}'></textarea>
              </div>
            </div>
            <pre id="apiResponseOutput">等待请求...</pre>
          </div>
        </div>

        <div class="panel" data-panel="ops" id="panelOps">
          <div class="panel-box panel-shell">
            <p class="section-eyebrow">OPERATIONS WORKSPACE</p>
            <h2 class="section-heading">全功能模块工作台</h2>
            <p class="section-copy">将站点基座、风控、套餐、支付、通知和系统治理拆成独立模块。模块切换已经收敛到左侧一级菜单，当前区域只保留当前模块的执行界面。</p>
            <div class="ops-shell">
              <section class="ops-main">
                <div class="ops-main-head">
                  <div>
                    <div class="ops-main-head-meta">
                      <span class="ops-module-chip" id="opsModuleTag"><strong>MODULE</strong> 站点基座</span>
                    </div>
                    <h3 class="ops-main-title" id="opsModuleTitle">安全设置</h3>
                    <p class="ops-main-subtitle" id="opsModuleDesc">登录保护、验证方式和后台地址设置。</p>
                  </div>
                  <div class="actions">
                    <button class="btn ghost" id="opsRefreshBtn">刷新当前模块</button>
                    <button class="btn ghost" id="opsOpenApiBtn">打开高级请求工具</button>
                  </div>
                </div>
                <div class="ops-workspace" id="opsWorkspace">
                  <p class="empty">请选择模块后进行操作。</p>
                </div>
              </section>
            </div>
          </div>
        </div>
      </section>
    </main>
  </div>

  <div id="toast" class="toast info"></div>

  <script>
    (() => {
      const settings = window.settings || {};
      const state = {
        token: "",
        currentAdmin: "",
        activeTab: "overview",
        adminTheme: "dark",
        loginCommConfig: null,
        loginCommConfigPromise: null,
        loginCaptchaProvider: "",
        loginCaptchaToken: "",
        loginCaptchaWidgetId: null,
        loginCaptchaScriptPromise: null,
        loginCaptchaScriptSrc: "",
        loginCaptchaV3Ready: false,
        loginPowProof: null,
        loginPowPromise: null,
        usersPage: 1,
        usersLastPage: 1,
        users: [],
        overviewRefreshTimer: null,
        overviewCountdownTimer: null,
        overviewNextRefreshIn: 0,
        overviewRefreshIntervalSeconds: 20,
        activeOpsModule: "security",
        opsCache: {
          config: null,
          plans: [],
          groups: [],
          payments: [],
          paymentMethods: [],
          notices: [],
          tickets: [],
          coupons: [],
          giftTemplates: [],
          giftCodes: [],
          giftTypes: {},
          giftStats: null,
          plugins: [],
          systemStatus: null,
          queueStats: null,
          systemLogs: null,
          failedJobs: null,
          riskReviews: [],
          riskReviewPagination: null,
          riskReviewPage: 1,
          banRecords: [],
          banRecordPagination: null,
          banRecordPage: 1,
          trafficStats: null,
          trafficLogs: [],
          trafficPagination: null,
          trafficDays: 30
        }
      };

      const dom = {
        loginView: document.getElementById("loginView"),
        adminLayout: document.getElementById("adminLayout"),
        loginEmailInput: document.getElementById("loginEmailInput"),
        loginPasswordInput: document.getElementById("loginPasswordInput"),
        loginCaptchaField: document.getElementById("loginCaptchaField"),
        loginCaptchaWidget: document.getElementById("loginCaptchaWidget"),
        loginCaptchaHint: document.getElementById("loginCaptchaHint"),
        loginPowHint: document.getElementById("loginPowHint"),
        loginSubmitBtn: document.getElementById("loginSubmitBtn"),
        loginErrorText: document.getElementById("loginErrorText"),
        themeSwitcher: document.getElementById("themeSwitcher"),
        themeDarkBtn: document.getElementById("themeDarkBtn"),
        themeLightBtn: document.getElementById("themeLightBtn"),
        menuTabs: document.getElementById("menuTabs"),
        primaryOpsMenuCount: document.getElementById("primaryOpsMenuCount"),
        primaryOpsMenuList: document.getElementById("primaryOpsMenuList"),
        primaryOpsMenuSearch: document.getElementById("primaryOpsMenuSearch"),
        panels: Array.from(document.querySelectorAll(".panel")),
        logoutBtn: document.getElementById("logoutBtn"),
        currentAdminText: document.getElementById("currentAdminText"),
        authState: document.getElementById("authState"),
        usersTableBody: document.getElementById("usersTableBody"),
        usersPageInfo: document.getElementById("usersPageInfo"),
        loadUsersBtn: document.getElementById("loadUsersBtn"),
        prevUsersBtn: document.getElementById("prevUsersBtn"),
        nextUsersBtn: document.getElementById("nextUsersBtn"),
        refreshOverviewBtn: document.getElementById("refreshOverviewBtn"),
        quickUsersBtn: document.getElementById("quickUsersBtn"),
        openCommandCenterBtn: document.getElementById("openCommandCenterBtn"),
        commandCenterDashboard: document.getElementById("commandCenterDashboard"),
        commandCenterCountdown: document.getElementById("commandCenterCountdown"),
        commandCenterUpdatedAt: document.getElementById("commandCenterUpdatedAt"),
        commandCenterStatusText: document.getElementById("commandCenterStatusText"),
        overviewHint: document.getElementById("overviewHint"),
        reloadAppBtn: document.getElementById("reloadAppBtn"),
        apiVersionSelect: document.getElementById("apiVersionSelect"),
        apiMethodSelect: document.getElementById("apiMethodSelect"),
        apiEndpointInput: document.getElementById("apiEndpointInput"),
        apiBodyInput: document.getElementById("apiBodyInput"),
        sendApiBtn: document.getElementById("sendApiBtn"),
        presetThemeBtn: document.getElementById("presetThemeBtn"),
        presetLimitBtn: document.getElementById("presetLimitBtn"),
        apiResponseOutput: document.getElementById("apiResponseOutput"),
        opsModuleTag: document.getElementById("opsModuleTag"),
        opsModuleTitle: document.getElementById("opsModuleTitle"),
        opsModuleDesc: document.getElementById("opsModuleDesc"),
        opsWorkspace: document.getElementById("opsWorkspace"),
        opsRefreshBtn: document.getElementById("opsRefreshBtn"),
        opsOpenApiBtn: document.getElementById("opsOpenApiBtn"),
        toast: document.getElementById("toast")
      };

      const securePath = (settings.secure_path || "").replace(/^\/+|\/+$/g, "");
      const commandCenterOnly = Boolean(settings.command_center_only);
      const SHARED_AUTH_KEY = "auth_data";
      const SHARED_LEGACY_TOKEN_KEY = "token";
      const SHARED_PROFILE_KEY = "me";
      const sharedAuthKeys = ["auth_data", "PORTAL_ACCESS_TOKEN", "Portal_access_token", "access_token"];

      function normalizeBearer(token) {
        const clean = String(token || "").trim();
        if (!clean) return "";
        return clean.toLowerCase().startsWith("bearer ") ? clean : `Bearer ${clean}`;
      }

      function readSharedAuthToken() {
        for (const key of [...sharedAuthKeys, SHARED_LEGACY_TOKEN_KEY]) {
          const value = normalizeBearer(localStorage.getItem(key) || "");
          if (value) {
            return value;
          }
        }
        return "";
      }

      function persistSharedAuthToken(token) {
        const value = normalizeBearer(token);
        sharedAuthKeys.forEach((key) => {
          if (value) {
            localStorage.setItem(key, value);
          } else {
            localStorage.removeItem(key);
          }
        });
      }

      function persistSharedLegacyToken(token) {
        const value = String(token || "").trim();
        if (value) {
          localStorage.setItem(SHARED_LEGACY_TOKEN_KEY, value);
          return;
        }
        localStorage.removeItem(SHARED_LEGACY_TOKEN_KEY);
      }

      function persistSharedAdminProfile(profile = null) {
        if (!profile) {
          localStorage.removeItem("me");
          return;
        }

        localStorage.setItem("me", JSON.stringify({
          id: profile.id,
          email: profile.email,
          is_admin: !!profile.is_admin,
          is_super_admin: !!profile.is_super_admin,
          secure_path: securePath || profile.secure_path || null
        }));
      }

      function showToast(message, type = "info") {
        dom.toast.className = `toast ${type}`;
        dom.toast.textContent = message;
        dom.toast.classList.add("show");
        window.clearTimeout(showToast._timer);
        showToast._timer = window.setTimeout(() => dom.toast.classList.remove("show"), 2600);
      }

      function setAuthState(ok, label) {
        dom.authState.className = `badge ${ok ? "ok" : "warn"}`;
        dom.authState.textContent = label;
      }

      function updateToken(token, persist = true) {
        state.token = normalizeBearer(token);
        if (persist) {
          persistSharedAuthToken(state.token);
        }
      }

      function clearToken(clearSharedAuth = false) {
        clearOverviewRefresh();
        state.token = "";
        state.currentAdmin = "";
        if (clearSharedAuth) {
          persistSharedAuthToken("");
          persistSharedLegacyToken("");
          persistSharedAdminProfile(null);
        }
        if (dom.currentAdminText) {
          dom.currentAdminText.value = "-";
        }
        setAuthState(false, "未认证");
      }

      function setLoginError(message = "") {
        dom.loginErrorText.textContent = message;
      }

      function showLoginView(message = "", options = {}) {
        const clearSharedAuth = Boolean(
          typeof options === "boolean" ? options : options?.clearSharedAuth
        );
        clearToken(clearSharedAuth);
        setLoginError(message);
        dom.adminLayout.classList.add("hidden");
        dom.loginView.classList.remove("hidden");
        prepareLoginSecurity().catch(() => {});
      }

      function showConsoleView() {
        setLoginError("");
        dom.loginView.classList.add("hidden");
        dom.adminLayout.classList.remove("hidden");
      }

      function headers(withJson = true) {
        const h = {
          Accept: "application/json"
        };
        if (withJson) {
          h["Content-Type"] = "application/json";
        }
        if (state.token) {
          h.Authorization = state.token;
        }
        return h;
      }

      function buildV2(endpoint) {
        const cleaned = String(endpoint || "").replace(/^\/+/, "");
        return `/api/v2/${securePath}/${cleaned}`;
      }

      function buildV1(endpoint) {
        const cleaned = String(endpoint || "").replace(/^\/+/, "");
        return `/api/v1/${cleaned}`;
      }

      async function syncAdminSession() {
        if (!state.token) {
          return null;
        }

        const body = await request({ method: "GET", url: buildV1("user/me") });
        const me = toData(body) || {};

        if (!me.is_admin || !me.is_super_admin) {
          const error = new Error("当前账号没有超级管理员权限");
          error.code = "not_super_admin";
          throw error;
        }

        state.currentAdmin = String(me.email || "");
        if (dom.currentAdminText) {
          dom.currentAdminText.value = state.currentAdmin || "-";
        }
        setAuthState(true, "已认证");
        persistSharedAdminProfile(me);
        return me;
      }

      async function request({ method = "GET", url, data, formData }) {
        const init = { method };
        if (formData) {
          init.headers = headers(false);
          delete init.headers["Content-Type"];
          init.body = formData;
        } else if (data !== undefined && method !== "GET") {
          init.headers = headers(true);
          init.body = JSON.stringify(data);
        } else {
          init.headers = headers(false);
        }

        const res = await fetch(url, init);
        const contentType = res.headers.get("content-type") || "";
        const body = contentType.includes("application/json") ? await res.json() : await res.text();

        if (res.status === 401) {
          const error = new Error("未授权访问，请重新登录");
          error.status = 401;
          showLoginView(error.message, { clearSharedAuth: true });
          throw error;
        }

        if (!res.ok) {
          const message = extractError(body) || `请求失败 (${res.status})`;
          throw new Error(message);
        }

        return body;
      }

      function extractError(body) {
        if (!body) return "";
        if (typeof body === "string") return body.slice(0, 160);
        return body.message || body.error || (body.data && body.data.message) || "";
      }

      function toData(body) {
        if (!body || typeof body !== "object") return body;
        if (Object.prototype.hasOwnProperty.call(body, "status")) {
          return body.data;
        }
        return body;
      }

      const LOGIN_COMM_CONFIG_URL = "/api/v1/guest/comm/config";
      const POW_CHALLENGE_URL = "/api/v1/passport/auth/pow-challenge";
      const TURNSTILE_SCRIPT_ID = "admin_turnstile_script";
      const RECAPTCHA_SCRIPT_ID = "admin_recaptcha_script";

      function isCaptchaEnabled(config) {
        return Number(config?.is_captcha || 0) === 1;
      }

      function getCaptchaProvider(config) {
        const type = String(config?.captcha_type || "recaptcha").toLowerCase();
        if (type === "turnstile") return "turnstile";
        if (type === "recaptcha-v3") return "recaptcha-v3";
        return "recaptcha";
      }

      function isPowEnabled(config) {
        return Number(config?.pow_enable || 0) === 1;
      }

      function setLoginPowHint(message = "", isError = false) {
        if (!dom.loginPowHint) return;
        const text = String(message || "").trim();
        dom.loginPowHint.classList.toggle("hidden", !text);
        dom.loginPowHint.classList.toggle("error", Boolean(isError));
        dom.loginPowHint.textContent = text;
      }

      function setLoginCaptchaHint(message = "") {
        if (!dom.loginCaptchaHint) return;
        dom.loginCaptchaHint.textContent = String(message || "").trim();
      }

      async function loadLoginCommConfig(force = false) {
        if (!force && state.loginCommConfig) return state.loginCommConfig;
        if (!force && state.loginCommConfigPromise) return state.loginCommConfigPromise;

        state.loginCommConfigPromise = request({ method: "GET", url: LOGIN_COMM_CONFIG_URL })
          .then((body) => {
            const cfg = toData(body);
            state.loginCommConfig = (cfg && typeof cfg === "object") ? cfg : {};
            return state.loginCommConfig;
          })
          .finally(() => {
            state.loginCommConfigPromise = null;
          });

        return state.loginCommConfigPromise;
      }

      function updateLoginSubmitAvailability() {
        const cfg = state.loginCommConfig || {};
        const captchaEnabled = isCaptchaEnabled(cfg);
        const powEnabled = isPowEnabled(cfg);
        const provider = state.loginCaptchaProvider || getCaptchaProvider(cfg);

        let captchaReady = true;
        if (captchaEnabled) {
          if (provider === "recaptcha-v3") {
            captchaReady = Boolean(state.loginCaptchaV3Ready);
          } else {
            captchaReady = Boolean(state.loginCaptchaToken);
          }
        }

        const powReady = !powEnabled || Boolean(state.loginPowProof);
        dom.loginSubmitBtn.disabled = !(captchaReady && powReady);
      }

      function waitFor(predicate, { timeoutMs = 8000, intervalMs = 80 } = {}) {
        return new Promise((resolve, reject) => {
          const start = Date.now();
          const timer = window.setInterval(() => {
            try {
              if (predicate()) {
                window.clearInterval(timer);
                resolve(true);
                return;
              }
              if (Date.now() - start >= timeoutMs) {
                window.clearInterval(timer);
                reject(new Error("脚本加载超时"));
              }
            } catch (err) {
              window.clearInterval(timer);
              reject(err);
            }
          }, intervalMs);
        });
      }

      function ensureScriptLoaded(id, src) {
        const desired = String(src || "").trim();
        if (!desired) return Promise.reject(new Error("脚本地址为空"));

        const existed = document.getElementById(id);
        if (existed) {
          const existedSrc = existed.getAttribute("src") || "";
          if (existedSrc === desired) {
            return Promise.resolve();
          }
          existed.remove();
        }

        return new Promise((resolve, reject) => {
          const script = document.createElement("script");
          script.id = id;
          script.src = desired;
          script.async = true;
          script.defer = true;
          script.onload = () => resolve();
          script.onerror = () => reject(new Error("脚本加载失败"));
          document.head.appendChild(script);
        });
      }

      async function ensureTurnstileLoaded() {
        await ensureScriptLoaded(TURNSTILE_SCRIPT_ID, "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit");
        await waitFor(() => window.turnstile && typeof window.turnstile.render === "function", { timeoutMs: 12000 });
      }

      function resolveRecaptchaScriptSrc(provider, siteKey) {
        const p = String(provider || "recaptcha").toLowerCase();
        if (p === "recaptcha-v3") {
          const key = String(siteKey || "").trim();
          return `https://www.recaptcha.net/recaptcha/api.js?render=${encodeURIComponent(key)}`;
        }
        return "https://www.recaptcha.net/recaptcha/api.js?render=explicit";
      }

      async function ensureRecaptchaLoaded(provider, siteKey) {
        const src = resolveRecaptchaScriptSrc(provider, siteKey);
        await ensureScriptLoaded(RECAPTCHA_SCRIPT_ID, src);
        if (String(provider || "").toLowerCase() === "recaptcha-v3") {
          await waitFor(() => window.grecaptcha && typeof window.grecaptcha.execute === "function" && typeof window.grecaptcha.ready === "function", { timeoutMs: 12000 });
          return;
        }
        await waitFor(() => window.grecaptcha && typeof window.grecaptcha.render === "function", { timeoutMs: 12000 });
      }

      function resetLoginCaptchaWidget() {
        const provider = String(state.loginCaptchaProvider || "").toLowerCase();
        const widgetId = state.loginCaptchaWidgetId;

        state.loginCaptchaToken = "";
        state.loginCaptchaWidgetId = null;
        state.loginCaptchaV3Ready = false;

        if (provider === "turnstile" && window.turnstile && widgetId != null) {
          try { window.turnstile.reset(widgetId); } catch (_) {}
        }
        if (provider === "recaptcha" && window.grecaptcha && widgetId != null) {
          try { window.grecaptcha.reset(widgetId); } catch (_) {}
        }

        if (dom.loginCaptchaWidget) {
          dom.loginCaptchaWidget.innerHTML = "";
        }
      }

      async function prewarmLoginCaptcha() {
        const cfg = await loadLoginCommConfig().catch(() => ({}));
        const enabled = isCaptchaEnabled(cfg);
        if (!dom.loginCaptchaField) return;

        resetLoginCaptchaWidget();
        dom.loginCaptchaField.classList.toggle("hidden", !enabled);

        if (!enabled) {
          state.loginCaptchaProvider = "";
          setLoginCaptchaHint("");
          updateLoginSubmitAvailability();
          return;
        }

        const provider = getCaptchaProvider(cfg);
        state.loginCaptchaProvider = provider;

        if (provider === "turnstile") {
          const siteKey = String(cfg.turnstile_site_key || "").trim();
          if (!siteKey) {
            setLoginCaptchaHint("Turnstile 站点密钥未配置");
            updateLoginSubmitAvailability();
            return;
          }
          setLoginCaptchaHint("请完成 Turnstile 人机验证后登录。");
          await ensureTurnstileLoaded();
          const wid = window.turnstile.render(dom.loginCaptchaWidget, {
            sitekey: siteKey,
            callback: (token) => {
              state.loginCaptchaToken = String(token || "").trim();
              updateLoginSubmitAvailability();
            },
            "expired-callback": () => {
              state.loginCaptchaToken = "";
              updateLoginSubmitAvailability();
            },
            "error-callback": () => {
              state.loginCaptchaToken = "";
              updateLoginSubmitAvailability();
            }
          });
          state.loginCaptchaWidgetId = wid;
          updateLoginSubmitAvailability();
          return;
        }

        if (provider === "recaptcha") {
          const siteKey = String(cfg.recaptcha_site_key || "").trim();
          if (!siteKey) {
            setLoginCaptchaHint("reCAPTCHA 站点密钥未配置");
            updateLoginSubmitAvailability();
            return;
          }
          setLoginCaptchaHint("请完成 reCAPTCHA 人机验证后登录。");
          await ensureRecaptchaLoaded("recaptcha", siteKey);
          const wid = window.grecaptcha.render(dom.loginCaptchaWidget, {
            sitekey: siteKey,
            callback: (token) => {
              state.loginCaptchaToken = String(token || "").trim();
              updateLoginSubmitAvailability();
            },
            "expired-callback": () => {
              state.loginCaptchaToken = "";
              updateLoginSubmitAvailability();
            },
            "error-callback": () => {
              state.loginCaptchaToken = "";
              updateLoginSubmitAvailability();
            }
          });
          state.loginCaptchaWidgetId = wid;
          updateLoginSubmitAvailability();
          return;
        }

        if (provider === "recaptcha-v3") {
          const siteKey = String(cfg.recaptcha_v3_site_key || "").trim();
          if (!siteKey) {
            setLoginCaptchaHint("reCAPTCHA v3 站点密钥未配置");
            updateLoginSubmitAvailability();
            return;
          }
          setLoginCaptchaHint("当前启用 reCAPTCHA v3（无感验证），点击登录将自动完成校验。");
          await ensureRecaptchaLoaded("recaptcha-v3", siteKey);
          state.loginCaptchaV3Ready = true;
          updateLoginSubmitAvailability();
          return;
        }

        setLoginCaptchaHint("未知验证码服务类型");
        updateLoginSubmitAvailability();
      }

      async function sha256Hex(message) {
        if (!window.crypto || !window.crypto.subtle || !window.TextEncoder) {
          throw new Error("浏览器不支持 WebCrypto，无法计算 PoW");
        }
        const data = new TextEncoder().encode(String(message || ""));
        const digest = await window.crypto.subtle.digest("SHA-256", data);
        return Array.from(new Uint8Array(digest))
          .map((b) => b.toString(16).padStart(2, "0"))
          .join("");
      }

      function yieldControl() {
        return new Promise((resolve) => window.setTimeout(resolve, 0));
      }

      async function loadPowChallenge() {
        const res = await fetch(POW_CHALLENGE_URL, { credentials: "same-origin" });
        const text = await res.text();
        let json = null;
        try { json = text ? JSON.parse(text) : null; } catch (_) { json = null; }
        if (!res.ok) {
          throw new Error(json?.message || json?.error || `HTTP ${res.status}`);
        }
        return json?.data || null;
      }

      async function computePowProof(challenge) {
        if (!challenge || typeof challenge !== "object") return null;
        const difficulty = Number(challenge.difficulty || 4);
        const prefix = "0".repeat(Math.max(1, difficulty));
        let nonce = 0;

        while (true) {
          const input = `${challenge.seed}|${challenge.base}|${challenge.token}|${nonce}`;
          const hash = await sha256Hex(input);
          if (hash.startsWith(prefix)) {
            return {
              id: challenge.challenge_id,
              nonce: String(nonce),
              hash,
              token: challenge.token,
              expires_at: challenge.expires_at,
            };
          }
          nonce += 1;
          if (nonce % 250 === 0) {
            await yieldControl();
          }
        }
      }

      async function ensureLoginPowProof() {
        const cfg = state.loginCommConfig || await loadLoginCommConfig().catch(() => ({}));
        if (!isPowEnabled(cfg)) {
          state.loginPowProof = null;
          setLoginPowHint("");
          updateLoginSubmitAvailability();
          return null;
        }

        const current = state.loginPowProof;
        const now = Math.floor(Date.now() / 1000);
        if (current?.expires_at && Number(current.expires_at) > now + 5) {
          return current;
        }

        if (state.loginPowPromise) {
          return state.loginPowPromise;
        }

        setLoginPowHint(`正在准备防刷验证，当前难度 ${Number(cfg?.pow_effective_difficulty || cfg?.pow_difficulty || 4)}，请稍候...`);
        updateLoginSubmitAvailability();

        state.loginPowPromise = (async () => {
          const challenge = await loadPowChallenge();
          const proof = await computePowProof(challenge);
          state.loginPowProof = proof;
          setLoginPowHint(`防刷验证已准备完成，当前难度 ${Number(cfg?.pow_effective_difficulty || cfg?.pow_difficulty || 4)}。`);
          return proof;
        })()
          .catch((err) => {
            state.loginPowProof = null;
            setLoginPowHint(err.message || "防刷验证初始化失败", true);
            throw err;
          })
          .finally(() => {
            state.loginPowPromise = null;
            updateLoginSubmitAvailability();
          });

        return state.loginPowPromise;
      }

      async function prepareLoginSecurity() {
        await loadLoginCommConfig().catch(() => ({}));
        await prewarmLoginCaptcha().catch((err) => {
          setLoginCaptchaHint(err.message || "验证码加载失败");
        });
        await ensureLoginPowProof().catch(() => {});
        updateLoginSubmitAvailability();
      }

      function money(v) {
        const n = Number(v || 0);
        return Number.isFinite(n) ? n.toLocaleString("zh-CN", { maximumFractionDigits: 2 }) : "0";
      }

      function escapeHtml(v) {
        return String(v == null ? "" : v)
          .replace(/&/g, "&amp;")
          .replace(/</g, "&lt;")
          .replace(/>/g, "&gt;")
          .replace(/"/g, "&quot;")
          .replace(/'/g, "&#39;");
      }

      const OPS_NAV_GROUPS = [
        { key: "foundation", name: "站点基座" },
        { key: "business", name: "业务运营" },
        { key: "governance", name: "风控治理" }
      ];

      const OPS_GROUP_MAP = OPS_NAV_GROUPS.reduce((acc, item) => {
        acc[item.key] = item;
        return acc;
      }, {});

      const OPS_MODULES = [
        { key: "security", name: "安全设置", desc: "登录保护、验证码和后台地址保护", group: "foundation", tag: "ACCESS" },
        { key: "oauth", name: "第三方登录", desc: "Linux DO 登录参数、回调地址和开关", group: "foundation", tag: "OAUTH" },
        { key: "site", name: "站点设置", desc: "站点名称、主题、注册和订阅地址", group: "foundation", tag: "SITE" },
        { key: "telegram", name: "Telegram 通知", desc: "Bot 配置与通知开关", group: "foundation", tag: "BOT" },
        { key: "plans", name: "套餐管理", desc: "套餐列表、价格和上下架", group: "business", tag: "PLANS" },
        { key: "payments", name: "支付管理", desc: "支付方式新增、修改、启用和停用", group: "business", tag: "BILLING" },
        { key: "notices", name: "公告管理", desc: "公告发布、显示开关和内容编辑", group: "business", tag: "CONTENT" },
        { key: "tickets", name: "工单管理", desc: "工单查看、回复和关闭", group: "business", tag: "SUPPORT" },
        { key: "coupons", name: "优惠券", desc: "优惠券生成、显示开关和删除", group: "business", tag: "PROMO" },
        { key: "giftcards", name: "礼品卡", desc: "模板、兑换码、统计和批量生成", group: "business", tag: "GIFT" },
        { key: "riskreview", name: "风险审查", desc: "共享 IP 风险识别、LLM 预审查和人工复核入口", group: "governance", tag: "RISK" },
        { key: "plugins", name: "扩展功能", desc: "扩展安装、启用、停用和设置", group: "governance", tag: "PLUGIN" },
        { key: "system", name: "系统状态", desc: "运行状态、任务队列、日志和失败任务", group: "governance", tag: "SYSTEM" },
        { key: "traffic", name: "流量重置", desc: "重置统计、日志和手动重置记录", group: "governance", tag: "TRAFFIC" }
      ];

      const OPS_MODULE_MAP = OPS_MODULES.reduce((acc, item) => {
        acc[item.key] = item;
        return acc;
      }, {});

      const ADMIN_THEME_STORAGE_KEY = "notxboard_admin_console_theme";

      function normalizeAdminTheme(value) {
        return String(value || "").toLowerCase() === "light" ? "light" : "dark";
      }

      function syncThemeButtons() {
        if (dom.themeDarkBtn) {
          dom.themeDarkBtn.classList.toggle("active", state.adminTheme === "dark");
          dom.themeDarkBtn.setAttribute("aria-pressed", state.adminTheme === "dark" ? "true" : "false");
        }
        if (dom.themeLightBtn) {
          dom.themeLightBtn.classList.toggle("active", state.adminTheme === "light");
          dom.themeLightBtn.setAttribute("aria-pressed", state.adminTheme === "light" ? "true" : "false");
        }
      }

      function moduleMatchesSearch(module, keyword = "") {
        if (!module) return false;
        const query = String(keyword || "").trim().toLowerCase();
        if (!query) return true;
        const groupName = OPS_GROUP_MAP[module.group]?.name || "";
        return [module.name, module.desc, module.tag, groupName]
          .filter(Boolean)
          .some((text) => String(text).toLowerCase().includes(query));
      }

      function applyAdminTheme(theme, persist = true) {
        const nextTheme = commandCenterOnly ? "dark" : normalizeAdminTheme(theme);
        state.adminTheme = nextTheme;
        document.body.setAttribute("data-admin-theme", nextTheme);
        syncThemeButtons();
        if (!commandCenterOnly && persist) {
          localStorage.setItem(ADMIN_THEME_STORAGE_KEY, nextTheme);
        }
      }

      function renderPrimaryOpsMenu(keyword = "") {
        if (!dom.primaryOpsMenuList) return;
        const query = String(keyword || "").trim().toLowerCase();
        let totalVisible = 0;
        const sections = OPS_NAV_GROUPS.map((group) => {
          const modules = OPS_MODULES.filter((module) => {
            if (module.group !== group.key) return false;
            return moduleMatchesSearch(module, query);
          });
          if (!modules.length) return "";
          totalVisible += modules.length;
          return `
            <section class="menu-subgroup">
              <div class="menu-subgroup-title">
                <span>${escapeHtml(group.name)}</span>
                <span class="menu-subgroup-count">${modules.length}</span>
              </div>
              ${modules.map((module) => `
                <button class="menu-btn menu-btn-compact ${state.activeTab === "ops" && state.activeOpsModule === module.key ? "active" : ""}" data-tab="ops" data-ops-module="${module.key}">
                  <span class="menu-btn-main">
                    <span class="menu-btn-title">${escapeHtml(module.name)}</span>
                    <span class="menu-btn-desc">${escapeHtml(module.desc)}</span>
                  </span>
                  <span class="menu-btn-index">${escapeHtml(module.tag || "MOD")}</span>
                </button>
              `).join("")}
            </section>
          `;
        }).filter(Boolean);

        if (dom.primaryOpsMenuCount) {
          dom.primaryOpsMenuCount.textContent = query
            ? `显示 ${totalVisible} / ${OPS_MODULES.length}`
            : `${OPS_MODULES.length} 个模块`;
        }

        dom.primaryOpsMenuList.innerHTML = sections.length
          ? sections.join("")
          : '<p class="empty">没有匹配的模块，请调整检索关键字。</p>';
      }

      function setActiveMenuButton(tab, moduleKey = "") {
        const nextTab = String(tab || "overview");
        const nextModule = String(moduleKey || "");
        document.querySelectorAll(".menu-btn").forEach((btn) => {
          const btnTab = btn.dataset.tab || "";
          const btnModule = btn.dataset.opsModule || "";
          const active = nextTab === "ops"
            ? (btnTab === "ops" && btnModule === nextModule)
            : (btnTab === nextTab && !btnModule);
          btn.classList.toggle("active", active);
          if (active) {
            btn.setAttribute("aria-current", "page");
          } else {
            btn.removeAttribute("aria-current");
          }
        });
      }

      function isOn(v) {
        return v === true || Number(v) === 1 || String(v) === "1";
      }

      function yesNo(v) {
        return isOn(v) ? "开启" : "关闭";
      }

      function readValue(id, fallback = "") {
        const el = document.getElementById(id);
        if (!el) return fallback;
        return el.value != null ? el.value : fallback;
      }

      function readInt(id, fallback = 0, nullable = false) {
        const raw = String(readValue(id, "")).trim();
        if (!raw) return nullable ? null : fallback;
        const n = Number(raw);
        if (!Number.isFinite(n)) return nullable ? null : fallback;
        return Math.trunc(n);
      }

      function readFloat(id, fallback = 0, nullable = false) {
        const raw = String(readValue(id, "")).trim();
        if (!raw) return nullable ? null : fallback;
        const n = Number(raw);
        return Number.isFinite(n) ? n : (nullable ? null : fallback);
      }

      function readBoolFromSelect(id) {
        return isOn(readValue(id, "0"));
      }

      function setInputValue(id, value) {
        const el = document.getElementById(id);
        if (!el) return;
        el.value = value == null ? "" : String(value);
      }

      function parseJsonText(raw, fallback = {}) {
        const text = String(raw || "").trim();
        if (!text) return fallback;
        return JSON.parse(text);
      }

      function parseJsonInput(id, fallback = {}) {
        return parseJsonText(readValue(id, ""), fallback);
      }

      function splitLines(raw) {
        return String(raw || "")
          .split(/\r?\n|,/)
          .map((s) => s.trim())
          .filter(Boolean);
      }

      function splitCsv(raw) {
        return String(raw || "")
          .split(",")
          .map((s) => s.trim())
          .filter(Boolean);
      }

      function toUnixFromInput(raw) {
        if (!raw) return 0;
        const ms = new Date(raw).getTime();
        return Number.isFinite(ms) ? Math.floor(ms / 1000) : 0;
      }

      function toInputDatetime(ts) {
        const n = Number(ts || 0);
        if (!Number.isFinite(n) || n <= 0) return "";
        const d = new Date(n * 1000);
        const pad = (v) => String(v).padStart(2, "0");
        return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
      }

      function formatDateTime(ts) {
        const n = Number(ts || 0);
        if (!Number.isFinite(n) || n <= 0) return "-";
        return new Date(n * 1000).toLocaleString("zh-CN", { hour12: false });
      }

      function formatTrafficKb(kb) {
        let value = Number(kb) || 0;
        if (!Number.isFinite(value) || value < 0) value = 0;
        const gb = value / 1024 / 1024;
        const digits = gb >= 100 ? 0 : (gb >= 10 ? 1 : 2);
        return `${gb.toFixed(digits)} GB`;
      }

      function formatCompactNumber(value) {
        const num = Number(value);
        if (!Number.isFinite(num)) return "0";
        if (typeof Intl !== "undefined" && Intl.NumberFormat) {
          return new Intl.NumberFormat("zh-CN", { notation: "compact", maximumFractionDigits: 1 }).format(num);
        }
        return String(Math.round(num));
      }

      function formatMoneyCent(value) {
        const amount = Number(value);
        if (!Number.isFinite(amount)) return "¥0";
        const yuan = amount / 100;
        return `¥${yuan.toLocaleString("zh-CN", {
          minimumFractionDigits: Math.abs(amount % 100) > 0 ? 2 : 0,
          maximumFractionDigits: 2,
        })}`;
      }

      function formatRatePerSecond(value) {
        let num = Number(value);
        if (!Number.isFinite(num) || num <= 0) return "0 B/s";
        const units = ["B/s", "KB/s", "MB/s", "GB/s"];
        let idx = 0;
        while (num >= 1024 && idx < units.length - 1) {
          num /= 1024;
          idx += 1;
        }
        const digits = num >= 100 ? 0 : (num >= 10 ? 1 : 2);
        return `${num.toFixed(digits)} ${units[idx]}`;
      }

      function formatDurationShort(seconds) {
        const total = Math.max(0, Math.floor(Number(seconds) || 0));
        if (total < 60) return `${total}s`;
        if (total < 3600) return `${Math.floor(total / 60)}m ${total % 60}s`;
        if (total < 86400) return `${Math.floor(total / 3600)}h ${Math.floor((total % 3600) / 60)}m`;
        return `${Math.floor(total / 86400)}d ${Math.floor((total % 86400) / 3600)}h`;
      }

      function formatPercent(value) {
        const num = Number(value);
        if (!Number.isFinite(num)) return "-";
        if (Math.abs(num) >= 100) return `${Math.round(num)}%`;
        if (Math.abs(num) >= 10) return `${num.toFixed(1)}%`;
        return `${num.toFixed(2)}%`;
      }

      function formatLatencyMs(value) {
        if (value === null || value === undefined || value === "") return "-";
        const latency = Number(value);
        if (!Number.isFinite(latency) || latency < 0) return "-";
        return `${Math.round(latency)} ms`;
      }

      function formatAnyTimestamp(rawValue, fallback = "-") {
        if (rawValue === null || rawValue === undefined || rawValue === "") return fallback;
        if (typeof rawValue === "number" && Number.isFinite(rawValue)) return formatDateTime(rawValue);

        const text = String(rawValue).trim();
        if (!text) return fallback;

        const asInt = Number(text);
        if (Number.isFinite(asInt) && /^\d+$/.test(text)) {
          return formatDateTime(asInt);
        }

        const parsed = new Date(text);
        if (!Number.isNaN(parsed.getTime())) {
          return parsed.toLocaleString("zh-CN", { hour12: false });
        }

        return text;
      }

      function buildCommandCenterEmpty(message) {
        return `<div class="command-empty">${escapeHtml(message || "暂无数据")}</div>`;
      }

      function buildOverviewLanding(meta = {}) {
        const monitorPath = meta.securePath ? `/${meta.securePath}/command-center` : "/command-center";
        const stats = meta.stats || {};
        const cards = [
          { label: "当月收入", value: money(stats.month_income || 0), hint: "后台 overview 原始统计" },
          { label: "新增注册", value: formatCompactNumber(stats.month_register_total || 0), hint: "用于快速感知转化趋势" },
          { label: "待处理工单", value: formatCompactNumber(stats.ticket_pending_total || 0), hint: "需要人工尽快跟进" },
          { label: "在线用户", value: formatCompactNumber(stats.online_users || 0), hint: "最近 10 分钟活跃窗口" },
        ];
        return `
          <section class="overview-grid">
            <article class="command-panel">
              <div class="command-panel-head">
                <div>
                  <div class="command-panel-kicker">TODAY'S WORKBENCH</div>
                  <h3>今日指挥面板</h3>
                </div>
                ${buildCommandCenterStatusBadge(meta.adminEmail || "-", "ok")}
              </div>
              <div class="overview-summary-grid">
                ${cards.map((card) => `
                  <article class="overview-summary-card">
                    <span>${escapeHtml(card.label)}</span>
                    <strong>${escapeHtml(card.value)}</strong>
                    <small>${escapeHtml(card.hint)}</small>
                  </article>
                `).join("")}
              </div>
              <div class="overview-rule-list">
                <article class="overview-rule-item">
                  <strong>后台指挥与独立监控已拆分</strong>
                  <p>超管后台负责配置、风控和操作执行，完整监控大屏继续放在独立页面，避免与公开总览混淆。</p>
                </article>
                <article class="overview-rule-item">
                  <strong>同域登录态自动复用</strong>
                  <p>当前控制台与独立监控页共用同域登录态，减少多后台切换和重复认证。</p>
                </article>
              </div>
            </article>

            <article class="command-panel">
              <div class="command-panel-head">
                <div>
                  <div class="command-panel-kicker">QUICK ROUTES</div>
                  <h3>高频操作捷径</h3>
                </div>
                ${buildCommandCenterStatusBadge(monitorPath, "cyan")}
              </div>
              <div class="overview-quick-grid">
                <button type="button" class="overview-quick-card" data-overview-jump="users">
                  <span>User Ops</span>
                  <strong>进入用户与封禁面板</strong>
                  <p>快速处理封禁、解封、封禁原因补录和订阅密钥重置。</p>
                </button>
                <button type="button" class="overview-quick-card" data-overview-jump="ops" data-overview-module="riskreview">
                  <span>Risk Review</span>
                  <strong>查看风险审查结果</strong>
                  <p>直接跳到共享 IP 风险识别、LLM 预审和封禁流水。</p>
                </button>
                <button type="button" class="overview-quick-card" data-overview-jump="ops" data-overview-module="plans">
                  <span>Plans</span>
                  <strong>维护套餐与价格</strong>
                  <p>进入套餐管理工作台，执行编辑、上下架和批量复核。</p>
                </button>
                <button type="button" class="overview-quick-card" data-overview-open="command-center">
                  <span>Monitor</span>
                  <strong>打开独立监控大屏</strong>
                  <p>完整监控请使用独立大屏，继续沿用当前后台登录态。</p>
                </button>
              </div>
            </article>
          </section>
        `;
      }

      function buildCommandCenterStatusBadge(label, tone = "neutral") {
        return `<span class="command-badge ${escapeHtml(tone)}">${escapeHtml(label)}</span>`;
      }

      function buildCommandCenterAreaChart(points) {
        const rows = Array.isArray(points) ? points : [];
        if (!rows.length) return buildCommandCenterEmpty("最近 7 天还没有节点流量记录");

        const width = 960;
        const height = 286;
        const left = 32;
        const right = 22;
        const top = 26;
        const bottom = 34;
        const chartWidth = width - left - right;
        const chartHeight = height - top - bottom;
        const maxValue = Math.max(1, ...rows.map((item) => Number(item?.total_kb || 0)));
        const step = rows.length > 1 ? chartWidth / (rows.length - 1) : chartWidth;
        const xFor = (idx) => left + step * idx;
        const yFor = (value) => top + (chartHeight - ((Number(value || 0) / maxValue) * chartHeight));
        const linePoints = rows.map((item, idx) => `${xFor(idx).toFixed(2)},${yFor(item?.total_kb || 0).toFixed(2)}`).join(" ");
        const areaPoints = `${left},${height - bottom} ${linePoints} ${left + chartWidth},${height - bottom}`;
        const ticks = [0, 0.33, 0.66, 1].map((ratio) => {
          const value = Math.round(maxValue * ratio);
          return { value, y: yFor(value) };
        });

        return `
          <div class="command-chart-shell">
            <svg viewBox="0 0 ${width} ${height}" class="command-traffic-svg" role="img" aria-label="最近 7 天流量趋势">
              <defs>
                <linearGradient id="commandCenterTrafficGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stop-color="rgba(69,224,209,0.42)"></stop>
                  <stop offset="100%" stop-color="rgba(15,118,110,0.03)"></stop>
                </linearGradient>
                <linearGradient id="commandCenterTrafficStroke" x1="0" y1="0" x2="1" y2="0">
                  <stop offset="0%" stop-color="#45e0d1"></stop>
                  <stop offset="100%" stop-color="#fb923c"></stop>
                </linearGradient>
              </defs>
              <rect x="0" y="0" width="${width}" height="${height}" rx="22" fill="rgba(3, 16, 24, 0.9)"></rect>
              ${ticks.map((tick) => `
                <g>
                  <line x1="${left}" y1="${tick.y}" x2="${width - right}" y2="${tick.y}" stroke="rgba(214,251,246,0.14)" stroke-dasharray="6 8"></line>
                  <text x="4" y="${tick.y + 4}" fill="rgba(214,251,246,0.72)" font-size="11">${escapeHtml(formatTrafficKb(tick.value))}</text>
                </g>
              `).join("")}
              <line x1="${left}" y1="${top}" x2="${left}" y2="${height - bottom}" stroke="rgba(214,251,246,0.18)"></line>
              <line x1="${left}" y1="${height - bottom}" x2="${width - right}" y2="${height - bottom}" stroke="rgba(214,251,246,0.18)"></line>
              <polygon points="${areaPoints}" fill="url(#commandCenterTrafficGradient)"></polygon>
              <polyline points="${linePoints}" fill="none" stroke="url(#commandCenterTrafficStroke)" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"></polyline>
              ${rows.map((item, idx) => `
                <g>
                  <circle cx="${xFor(idx).toFixed(2)}" cy="${yFor(item?.total_kb || 0).toFixed(2)}" r="4.5" fill="#45e0d1" stroke="rgba(3, 16, 24, 0.96)" stroke-width="2"></circle>
                  <text x="${xFor(idx).toFixed(2)}" y="${height - 10}" text-anchor="middle" fill="rgba(214,251,246,0.72)" font-size="11">${escapeHtml(String(item?.date || "").slice(5))}</text>
                </g>
              `).join("")}
            </svg>
          </div>
        `;
      }

      function buildCommandCenterSparkline(samples, width = 260, height = 74) {
        const rows = Array.isArray(samples) ? samples : [];
        if (!rows.length) return '<div class="command-spark-empty">暂无 TCPing 采样</div>';

        const left = 8;
        const right = 8;
        const top = 10;
        const bottom = 14;
        const chartWidth = width - left - right;
        const chartHeight = height - top - bottom;
        const reachable = rows
          .map((item, idx) => ({ item, idx }))
          .filter(({ item }) => item?.is_reachable && Number.isFinite(Number(item?.latency_ms)));
        const maxLatency = Math.max(40, ...reachable.map(({ item }) => Number(item?.latency_ms || 0)));
        const step = rows.length > 1 ? chartWidth / (rows.length - 1) : chartWidth;
        const xFor = (idx) => left + step * idx;
        const yFor = (value) => top + (chartHeight - ((Number(value || 0) / maxLatency) * chartHeight));
        const linePoints = reachable.map(({ item, idx }) => `${xFor(idx).toFixed(2)},${yFor(item.latency_ms).toFixed(2)}`).join(" ");
        const offlineMarkers = rows
          .map((item, idx) => (!item?.is_reachable ? { x: xFor(idx), y: height - 10 } : null))
          .filter(Boolean);

        return `
          <svg viewBox="0 0 ${width} ${height}" class="command-node-sparkline" role="img" aria-label="TCPing latency sparkline">
            <line x1="${left}" y1="${height - bottom}" x2="${width - right}" y2="${height - bottom}" stroke="rgba(214,251,246,0.16)"></line>
            <line x1="${left}" y1="${top}" x2="${left}" y2="${height - bottom}" stroke="rgba(214,251,246,0.1)"></line>
            ${linePoints ? `<polyline points="${linePoints}" fill="none" stroke="rgba(69,224,209,0.94)" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"></polyline>` : ""}
            ${reachable.map(({ item, idx }) => `<circle cx="${xFor(idx).toFixed(2)}" cy="${yFor(item.latency_ms).toFixed(2)}" r="2.5" fill="rgba(103,232,249,0.88)"></circle>`).join("")}
            ${offlineMarkers.map((dot) => `<circle cx="${dot.x.toFixed(2)}" cy="${dot.y.toFixed(2)}" r="3" fill="rgba(248,113,113,0.92)"></circle>`).join("")}
          </svg>
        `;
      }

      function buildCommandCenterBars(items, {
        valueKey = "value",
        labelKey = "label",
        emptyText = "暂无数据",
        accent = "cyan",
        subtitle = null,
        valueFormatter = (value) => formatCompactNumber(value),
      } = {}) {
        const rows = Array.isArray(items) ? items : [];
        if (!rows.length) return buildCommandCenterEmpty(emptyText);

        const maxValue = Math.max(1, ...rows.map((item) => Number(item?.[valueKey] || 0)));
        return `
          <div class="command-bars">
            ${rows.map((item) => {
              const value = Number(item?.[valueKey] || 0);
              const width = Math.max(4, (value / maxValue) * 100);
              const label = item?.[labelKey] ?? item?.label ?? item?.name ?? "-";
              const sub = typeof subtitle === "function" ? subtitle(item) : "";
              return `
                <div class="command-bar-row">
                  <div class="command-bar-copy">
                    <strong>${escapeHtml(label)}</strong>
                    ${sub ? `<span>${escapeHtml(sub)}</span>` : ""}
                  </div>
                  <div class="command-bar-track">
                    <div class="command-bar-fill ${escapeHtml(accent)}" style="width:${width.toFixed(2)}%"></div>
                  </div>
                  <div class="command-bar-value">${escapeHtml(valueFormatter(value, item))}</div>
                </div>
              `;
            }).join("")}
          </div>
        `;
      }

      function buildCommandCenterDashboard(payload = {}, meta = {}) {
        const overview = payload?.overview || {};
        const system = payload?.system || {};
        const trend = Array.isArray(payload?.traffic_trend) ? payload.traffic_trend : [];
        const topUsers = Array.isArray(payload?.top_users) ? payload.top_users : [];
        const watchlist = Array.isArray(payload?.node_watchlist) ? payload.node_watchlist : [];
        const protocolDistribution = Array.isArray(payload?.protocol_distribution) ? payload.protocol_distribution : [];
        const regionDistribution = Array.isArray(payload?.region_distribution) ? payload.region_distribution : [];
        const tickets = Array.isArray(payload?.tickets) ? payload.tickets : [];
        const refunds = Array.isArray(payload?.refunds) ? payload.refunds : [];
        const tcpingAgents = Array.isArray(payload?.tcping_agents) ? payload.tcping_agents : [];
        const tcpingAlerts = Array.isArray(payload?.tcping_alerts) ? payload.tcping_alerts : [];
        const auditStream = Array.isArray(payload?.audit_stream) ? payload.audit_stream : [];
        const totalNodes = Number(overview?.total_nodes || 0);
        const onlineNodes = Number(overview?.online_nodes || 0);
        const nodeOnlineRatio = totalNodes > 0 ? (onlineNodes / totalNodes) * 100 : 0;
        const alertCount = Number(overview?.tcping_alerts_active || 0);
        const schedulerTone = system?.schedule_ok ? "ok" : "bad";
        const horizonTone = system?.horizon?.available ? (system?.horizon?.ok ? "ok" : "bad") : "neutral";
        const alertTone = alertCount > 0 ? "warn" : "ok";
        const logTone = Number(system?.logs?.errors_last_24h || 0) > 0 ? "warn" : "ok";
        const userMaxTraffic = Math.max(1, ...topUsers.map((item) => Number(item?.traffic_kb || 0)));
        const metrics = [
          {
            label: "用户总量",
            value: formatCompactNumber(overview?.total_users || 0),
            meta: `实时活跃 ${formatCompactNumber(overview?.live_users || 0)}`,
            tone: "cyan",
          },
          {
            label: "活跃订阅",
            value: formatCompactNumber(overview?.active_subscriptions || 0),
            meta: `今日使用 ${formatCompactNumber(overview?.traffic_today_unique_users || 0)} 用户`,
            tone: "blue",
          },
          {
            label: "在线节点",
            value: `${onlineNodes}/${totalNodes || 0}`,
            meta: `在线率 ${formatPercent(nodeOnlineRatio)}`,
            tone: "emerald",
          },
          {
            label: "TCPing 告警",
            value: formatCompactNumber(alertCount),
            meta: `监控覆盖 ${formatCompactNumber(overview?.tcping_enabled_nodes || 0)} 节点`,
            tone: "rose",
          },
          {
            label: "探针在线",
            value: `${formatCompactNumber(overview?.tcping_agents_online || 0)}/${formatCompactNumber(overview?.tcping_agents_total || 0)}`,
            meta: "独立于 V2bX 的探活体系",
            tone: "violet",
          },
          {
            label: "待处理事务",
            value: `${formatCompactNumber(overview?.open_tickets || 0)} / ${formatCompactNumber(overview?.pending_refunds || 0)}`,
            meta: "工单 / 退款",
            tone: "orange",
          },
          {
            label: "高压节点",
            value: formatCompactNumber(overview?.traffic_hot_nodes || 0),
            meta: `维护中 ${formatCompactNumber(overview?.maintenance_nodes || 0)}`,
            tone: "amber",
          },
          {
            label: "今日流量",
            value: formatTrafficKb(overview?.traffic_today_kb || 0),
            meta: `24h 收入 ${formatMoneyCent(overview?.revenue_24h_amount || 0)}`,
            tone: "teal",
          },
        ];

        return `
          <section class="command-hero-panel">
            <div class="command-hero-copy">
              <div class="command-kicker">RECOVERED INTO NATIVE ADMIN OVERVIEW</div>
              <h2>Super Admin Pulse Matrix</h2>
              <p>这套监控大屏现在直接挂在原生超管后台页面里，路径、菜单和交互体系与原后台保持一致，同时把节点在线、TCPing 探测、真实用户流量、退款、工单与审计流集中到同一视图。</p>
              <div class="command-hero-badges">
                ${buildCommandCenterStatusBadge(system?.schedule_ok ? "Scheduler 正常" : "Scheduler 异常", schedulerTone)}
                ${buildCommandCenterStatusBadge(system?.horizon?.available ? (system?.horizon?.ok ? "Horizon 正常" : "Horizon 暂停") : "Horizon 未启用", horizonTone)}
                ${buildCommandCenterStatusBadge(alertCount > 0 ? `活跃告警 ${alertCount}` : "无活跃告警", alertTone)}
                ${buildCommandCenterStatusBadge(`当前账号 ${meta?.adminEmail || "-"}`, "neutral")}
                ${buildCommandCenterStatusBadge(`后台 /${meta?.securePath || "-"}`, "violet")}
              </div>
            </div>
            <div class="command-focus-grid">
              <div class="command-focus-card">
                <span>NODE UPTIME</span>
                <strong>${formatPercent(nodeOnlineRatio)}</strong>
                <small>${onlineNodes} / ${totalNodes || 0} 节点在线</small>
              </div>
              <div class="command-focus-card">
                <span>LIVE THROUGHPUT</span>
                <strong>${escapeHtml(formatRatePerSecond(overview?.throughput_download_bps || 0))}</strong>
                <small>上传 ${escapeHtml(formatRatePerSecond(overview?.throughput_upload_bps || 0))}</small>
              </div>
              <div class="command-focus-card">
                <span>USER FOOTPRINT</span>
                <strong>${escapeHtml(formatCompactNumber(overview?.traffic_today_unique_users || 0))}</strong>
                <small>今天真实使用节点的订阅用户</small>
              </div>
            </div>
          </section>

          <section class="command-metrics-grid">
            ${metrics.map((metric) => `
              <article class="command-metric-card ${escapeHtml(metric.tone)}">
                <span>${escapeHtml(metric.label)}</span>
                <strong>${escapeHtml(metric.value)}</strong>
                <small>${escapeHtml(metric.meta)}</small>
              </article>
            `).join("")}
          </section>

          <section class="command-layout">
            <article class="command-panel command-span-8">
              <div class="command-panel-head">
                <div>
                  <div class="command-panel-kicker">NETWORK FLOW</div>
                  <h3>最近 7 天流量热区</h3>
                </div>
                <div class="command-inline-stats">
                  <div class="command-inline-stat">
                    <span>今日流量</span>
                    <strong>${escapeHtml(formatTrafficKb(overview?.traffic_today_kb || 0))}</strong>
                  </div>
                  <div class="command-inline-stat">
                    <span>24h 完成订单</span>
                    <strong>${escapeHtml(formatCompactNumber(overview?.completed_orders_24h || 0))}</strong>
                  </div>
                  <div class="command-inline-stat">
                    <span>实时下行</span>
                    <strong>${escapeHtml(formatRatePerSecond(overview?.throughput_download_bps || 0))}</strong>
                  </div>
                </div>
              </div>
              ${buildCommandCenterAreaChart(trend)}
            </article>

            <article class="command-panel command-span-4">
              <div class="command-panel-head">
                <div>
                  <div class="command-panel-kicker">SYSTEM HEALTH</div>
                  <h3>运行栈体征</h3>
                </div>
              </div>
              <div class="command-health-list">
                <div class="command-health-item">
                  <div>
                    <strong>计划任务</strong>
                    <span>最近运行 ${escapeHtml(formatAnyTimestamp(system?.schedule_last_runtime))}</span>
                  </div>
                  ${buildCommandCenterStatusBadge(system?.schedule_ok ? "正常" : "异常", schedulerTone)}
                </div>
                <div class="command-health-item">
                  <div>
                    <strong>Horizon</strong>
                    <span>${system?.horizon?.available ? `Master ${formatCompactNumber(system?.horizon?.master_count || 0)} / 暂停 ${formatCompactNumber(system?.horizon?.paused_masters || 0)}` : "当前环境未启用"}</span>
                  </div>
                  ${buildCommandCenterStatusBadge(system?.horizon?.available ? (system?.horizon?.ok ? "运行中" : "告警") : "N/A", horizonTone)}
                </div>
                <div class="command-health-item">
                  <div>
                    <strong>日志压力</strong>
                    <span>Info ${formatCompactNumber(system?.logs?.info || 0)} / Warn ${formatCompactNumber(system?.logs?.warning || 0)} / Error ${formatCompactNumber(system?.logs?.error || 0)}</span>
                  </div>
                  ${buildCommandCenterStatusBadge(Number(system?.logs?.errors_last_24h || 0) > 0 ? "需要关注" : "平稳", logTone)}
                </div>
              </div>
              <div class="command-log-grid">
                <div><span>24h 错误</span><strong>${escapeHtml(formatCompactNumber(system?.logs?.errors_last_24h || 0))}</strong></div>
                <div><span>24h 警告</span><strong>${escapeHtml(formatCompactNumber(system?.logs?.warnings_last_24h || 0))}</strong></div>
                <div><span>总日志</span><strong>${escapeHtml(formatCompactNumber(system?.logs?.total || 0))}</strong></div>
                <div><span>当前账号</span><strong>${escapeHtml(meta?.adminEmail || "-")}</strong></div>
              </div>
            </article>

            <article class="command-panel command-span-6">
              <div class="command-panel-head">
                <div>
                  <div class="command-panel-kicker">USER PRESSURE</div>
                  <h3>高流量用户榜</h3>
                </div>
              </div>
              ${topUsers.length ? `
                <div class="command-user-list">
                  ${topUsers.map((user, index) => {
                    const width = Math.max(8, ((Number(user?.traffic_kb || 0) / userMaxTraffic) * 100));
                    return `
                      <div class="command-user-row">
                        <div class="command-rank">#${String(index + 1).padStart(2, "0")}</div>
                        <div class="command-user-identity">
                          <strong>${escapeHtml(user?.display_name || user?.email || "-")}</strong>
                          <span>${escapeHtml(user?.email || "-")}</span>
                        </div>
                        <div class="command-user-bar">
                          <div class="command-user-bar-fill" style="width:${width.toFixed(2)}%"></div>
                        </div>
                        <div class="command-user-metrics">
                          <strong>${escapeHtml(formatTrafficKb(user?.traffic_kb || 0))}</strong>
                          <span>${escapeHtml(user?.plan_name || "-")}${user?.subscription_expired_at ? ` · 到期 ${escapeHtml(formatAnyTimestamp(user.subscription_expired_at))}` : ""}</span>
                        </div>
                      </div>
                    `;
                  }).join("")}
                </div>
              ` : buildCommandCenterEmpty("当前窗口暂无用户流量排行")}
            </article>

            <article class="command-panel command-span-6">
              <div class="command-panel-head">
                <div>
                  <div class="command-panel-kicker">TOPOLOGY MIX</div>
                  <h3>协议与地区分布</h3>
                </div>
              </div>
              <div class="command-split-grid">
                <div>
                  <h4>协议占比</h4>
                  ${buildCommandCenterBars(protocolDistribution.map((item) => ({
                    label: item?.label || item?.protocol || "-",
                    value: item?.total || 0,
                    online: item?.online || 0,
                  })), {
                    accent: "cyan",
                    emptyText: "暂无节点协议数据",
                    subtitle: (item) => `在线 ${formatCompactNumber(item?.online || 0)}`,
                    valueFormatter: (value) => `${formatCompactNumber(value)} 个`,
                  })}
                </div>
                <div>
                  <h4>地区热度</h4>
                  ${buildCommandCenterBars(regionDistribution.map((item) => ({
                    label: item?.location_name || item?.location_code || "-",
                    value: item?.total || 0,
                    online: item?.online || 0,
                  })), {
                    accent: "violet",
                    emptyText: "暂无节点地区数据",
                    subtitle: (item) => `在线 ${formatCompactNumber(item?.online || 0)}`,
                    valueFormatter: (value) => `${formatCompactNumber(value)} 个`,
                  })}
                </div>
              </div>
            </article>

            <article class="command-panel command-span-12">
              <div class="command-panel-head">
                <div>
                  <div class="command-panel-kicker">NODE WATCHLIST</div>
                  <h3>节点关注列表</h3>
                </div>
              </div>
              ${watchlist.length ? `
                <div class="command-node-grid">
                  ${watchlist.map((node) => {
                    const progress = Math.max(0, Math.min(100, Number(node?.traffic_usage_percentage || 0)));
                    const meterClass = progress >= 90 ? "danger" : (progress >= 70 ? "warn" : "ok");
                    return `
                      <article class="command-node-card">
                        <div class="command-node-head">
                          <div>
                            <div class="command-node-title">
                              <h4>${escapeHtml(node?.name || "-")}</h4>
                              ${buildCommandCenterStatusBadge(node?.protocol_label || node?.protocol || "-", "neutral")}
                            </div>
                            <div class="command-node-subline">${escapeHtml(node?.host || "-")} : ${escapeHtml(node?.port || "-")} · ${escapeHtml(node?.location_name || "-")}</div>
                          </div>
                          <div class="command-node-badges">
                            ${buildCommandCenterStatusBadge(node?.online_status === "online" ? "上报在线" : "上报离线", node?.online_status === "online" ? "ok" : "bad")}
                            ${buildCommandCenterStatusBadge(node?.tcping_status === "unsupported"
                              ? "不支持 UDP 探测"
                              : (node?.tcping_enabled ? (node?.tcping_status === "offline" ? "TCPing 异常" : (node?.tcping_status === "online" ? "TCPing 正常" : "TCPing 等待")) : "未启用 TCPing"),
                            node?.tcping_status === "unsupported"
                              ? "neutral"
                              : (node?.tcping_enabled ? (node?.tcping_status === "offline" ? "warn" : "ok") : "neutral"))}
                            ${node?.active_alert ? buildCommandCenterStatusBadge(`告警 ${node.active_alert.count || 1}`, "rose") : ""}
                          </div>
                        </div>
                        <div class="command-node-owner">${escapeHtml(node?.owner_name || "-")} · ${escapeHtml(node?.owner_email || "-")}</div>
                        <div class="command-node-spark">${buildCommandCenterSparkline(node?.tcping_samples)}</div>
                        <div class="command-node-meter">
                          <div class="command-node-meter-top">
                            <span>节点总流量</span>
                            <strong>${node?.traffic_limit_kb > 0 ? `${escapeHtml(formatTrafficKb(node?.traffic_used_kb || 0))} / ${escapeHtml(formatTrafficKb(node?.traffic_limit_kb || 0))}` : `${escapeHtml(formatTrafficKb(node?.traffic_used_kb || 0))} / 无限`}</strong>
                          </div>
                          <div class="command-meter ${meterClass}">
                            <div class="command-meter-fill" style="width:${node?.traffic_limit_kb > 0 ? progress.toFixed(2) : 100}%"></div>
                          </div>
                          <div class="command-node-meter-bottom">${node?.traffic_limit_kb > 0 ? `压力 ${escapeHtml(formatPercent(progress))}` : "未设置节点总流量上限"}</div>
                        </div>
                        <div class="command-node-stats">
                          <div><span>在线用户</span><strong>${escapeHtml(formatCompactNumber(node?.online_users || 0))}</strong></div>
                          <div><span>活跃连接</span><strong>${escapeHtml(formatCompactNumber(node?.active_connections || 0))}</strong></div>
                          <div><span>7 天流量</span><strong>${escapeHtml(formatTrafficKb(node?.weekly_traffic_kb || 0))}</strong></div>
                          <div><span>最近延迟</span><strong>${escapeHtml(formatLatencyMs(node?.tcping_last_latency_ms))}</strong></div>
                        </div>
                        <div class="command-node-footer">
                          <div class="command-node-alert">${node?.active_alert?.latest_error ? escapeHtml(node.active_alert.latest_error) : (node?.tcping_last_error ? escapeHtml(node.tcping_last_error) : "未检测到最新异常说明")}</div>
                          <div class="command-node-alert">最近采样 ${escapeHtml(formatAnyTimestamp(node?.tcping_last_sampled_at))}</div>
                        </div>
                      </article>
                    `;
                  }).join("")}
                </div>
              ` : buildCommandCenterEmpty("当前没有节点可展示")}
            </article>

            <article class="command-panel command-span-4">
              <div class="command-panel-head">
                <div>
                  <div class="command-panel-kicker">TCPING ALERTS</div>
                  <h3>活跃告警</h3>
                </div>
              </div>
              ${tcpingAlerts.length ? tcpingAlerts.map((alert) => `
                <div class="command-feed-item">
                  <div class="command-feed-top">
                    <strong>${escapeHtml(alert?.node_name || "-")}</strong>
                    ${buildCommandCenterStatusBadge(alert?.status || "active", "rose")}
                  </div>
                  <div class="command-feed-meta">${escapeHtml(alert?.user_email || "-")} · ${escapeHtml(alert?.node_location_name || "-")} · ${escapeHtml(alert?.node_protocol || "-")}</div>
                  <div class="command-feed-sub">${escapeHtml(formatDurationShort(alert?.duration_seconds || 0))} · ${escapeHtml(alert?.latest_error || "无错误信息")}</div>
                </div>
              `).join("") : buildCommandCenterEmpty("暂无活跃 TCPing 告警")}
            </article>

            <article class="command-panel command-span-4">
              <div class="command-panel-head">
                <div>
                  <div class="command-panel-kicker">PROBES</div>
                  <h3>探针健康</h3>
                </div>
              </div>
              ${tcpingAgents.length ? tcpingAgents.map((agent) => `
                <div class="command-feed-item">
                  <div class="command-feed-top">
                    <strong>${escapeHtml(agent?.name || "-")}</strong>
                    ${buildCommandCenterStatusBadge(agent?.is_online ? "在线" : "离线", agent?.is_online ? "ok" : "warn")}
                  </div>
                  <div class="command-feed-meta">${escapeHtml(agent?.owner_name || "-")} · ${escapeHtml(agent?.owner_email || "-")}</div>
                  <div class="command-feed-sub">心跳 ${escapeHtml(formatAnyTimestamp(agent?.last_heartbeat_at))} · 同步 ${escapeHtml(formatAnyTimestamp(agent?.last_sync_at))}</div>
                </div>
              `).join("") : buildCommandCenterEmpty("暂无探针数据")}
            </article>

            <article class="command-panel command-span-4">
              <div class="command-panel-head">
                <div>
                  <div class="command-panel-kicker">SUPPORT QUEUE</div>
                  <h3>工单池</h3>
                </div>
              </div>
              ${tickets.length ? tickets.map((ticket) => `
                <div class="command-feed-item">
                  <div class="command-feed-top">
                    <strong>${escapeHtml(ticket?.subject || "-")}</strong>
                    ${buildCommandCenterStatusBadge(Number(ticket?.status || 0) === 0 ? "OPEN" : "CLOSED", Number(ticket?.status || 0) === 0 ? "warn" : "neutral")}
                  </div>
                  <div class="command-feed-meta">${escapeHtml(ticket?.user_email || "-")} · ${escapeHtml(ticket?.node_name || "-")}</div>
                  <div class="command-feed-sub">更新 ${escapeHtml(formatAnyTimestamp(ticket?.updated_at))} · 负责人 ${escapeHtml(ticket?.assigned_admin_email || "-")}</div>
                </div>
              `).join("") : buildCommandCenterEmpty("当前没有开启工单")}
            </article>

            <article class="command-panel command-span-6">
              <div class="command-panel-head">
                <div>
                  <div class="command-panel-kicker">REFUND FLOW</div>
                  <h3>退款流程</h3>
                </div>
              </div>
              ${refunds.length ? refunds.map((refund) => `
                <div class="command-feed-item">
                  <div class="command-feed-top">
                    <strong>${escapeHtml(refund?.user_email || "-")}</strong>
                    ${buildCommandCenterStatusBadge(refund?.status || "pending", refund?.status === "voting" ? "warn" : "orange")}
                  </div>
                  <div class="command-feed-meta">${escapeHtml(refund?.plan_name || "-")} · trade_no ${escapeHtml(refund?.trade_no || "-")}</div>
                  <div class="command-feed-sub">申请 ${escapeHtml(formatMoneyCent(refund?.refund_amount || 0))} / 原金额 ${escapeHtml(formatMoneyCent(refund?.gateway_amount || 0))} · 更新 ${escapeHtml(formatAnyTimestamp(refund?.updated_at))}</div>
                </div>
              `).join("") : buildCommandCenterEmpty("当前没有待处理退款")}
            </article>

            <article class="command-panel command-span-6">
              <div class="command-panel-head">
                <div>
                  <div class="command-panel-kicker">AUDIT STREAM</div>
                  <h3>审计事件流</h3>
                </div>
              </div>
              ${auditStream.length ? auditStream.map((log) => `
                <div class="command-feed-item">
                  <div class="command-feed-top">
                    <strong>${escapeHtml(log?.user_email || "-")}</strong>
                    ${buildCommandCenterStatusBadge(log?.action_taken || "logged", log?.action_taken === "blocked" ? "bad" : (log?.action_taken === "allowed" ? "ok" : "neutral"))}
                  </div>
                  <div class="command-feed-meta">${escapeHtml(log?.node_name || "-")} · ${escapeHtml(log?.node_location_name || "-")} · ${escapeHtml(log?.ip_address || "-")}</div>
                  <div class="command-feed-sub">${escapeHtml(log?.target_domain || log?.target_protocol || "未提供目标")} · ${escapeHtml(formatAnyTimestamp(log?.created_at))}</div>
                </div>
              `).join("") : buildCommandCenterEmpty("当前没有审计事件")}
            </article>
          </section>
        `;
      }

      function clearOverviewRefresh() {
        if (state.overviewRefreshTimer) {
          window.clearTimeout(state.overviewRefreshTimer);
          state.overviewRefreshTimer = null;
        }
        if (state.overviewCountdownTimer) {
          window.clearInterval(state.overviewCountdownTimer);
          state.overviewCountdownTimer = null;
        }
        state.overviewNextRefreshIn = 0;
      }

      function setOverviewStatus(text, tone = "neutral") {
        if (!dom.commandCenterStatusText) return;
        dom.commandCenterStatusText.textContent = text;
        dom.commandCenterStatusText.setAttribute("data-tone", tone);
      }

      function updateOverviewCountdown() {
        if (!dom.commandCenterCountdown) return;
        const remaining = Math.max(0, Math.floor(state.overviewNextRefreshIn || 0));
        const minutes = Math.floor(remaining / 60);
        const seconds = remaining % 60;
        dom.commandCenterCountdown.textContent = `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
      }

      function scheduleOverviewRefresh(seconds) {
        clearOverviewRefresh();
        if (!state.token) {
          updateOverviewCountdown();
          return;
        }
        state.overviewRefreshIntervalSeconds = Math.max(10, Number(seconds) || 20);
        state.overviewNextRefreshIn = state.overviewRefreshIntervalSeconds;
        updateOverviewCountdown();
        state.overviewCountdownTimer = window.setInterval(() => {
          if (!state.token) {
            clearOverviewRefresh();
            return;
          }
          state.overviewNextRefreshIn = Math.max(0, state.overviewNextRefreshIn - 1);
          updateOverviewCountdown();
        }, 1000);
        state.overviewRefreshTimer = window.setTimeout(() => {
          loadOverview(false).catch((err) => {
            showToast(err.message || "监控刷新失败", "error");
          });
        }, state.overviewRefreshIntervalSeconds * 1000);
      }

      function randomSecurePath() {
        return `admin-${Math.random().toString(36).slice(2, 10)}${Date.now().toString(36).slice(-4)}`;
      }

      function normalizeList(body) {
        const data = toData(body);
        if (Array.isArray(data)) return data;
        if (data && Array.isArray(data.data)) return data.data;
        if (body && Array.isArray(body.data)) return body.data;
        return [];
      }

      function normalizeObject(body) {
        const data = toData(body);
        if (data && typeof data === "object") return data;
        if (body && typeof body === "object") return body;
        return {};
      }

      function normalizeConfigValue(v, fallback = "") {
        return v == null ? fallback : v;
      }

      function opsBoolField(id, label, value, span = 3) {
        return `
          <div class="field span-${span}">
            <label for="${id}">${escapeHtml(label)}</label>
            <select id="${id}">
              <option value="1" ${isOn(value) ? "selected" : ""}>开启</option>
              <option value="0" ${isOn(value) ? "" : "selected"}>关闭</option>
            </select>
          </div>
        `;
      }

      function setPreOutput(id, value) {
        const el = document.getElementById(id);
        if (!el) return;
        el.textContent = typeof value === "string" ? value : JSON.stringify(value, null, 2);
      }

      function setOpsModuleHeader(moduleKey) {
        const module = OPS_MODULE_MAP[moduleKey] || OPS_MODULES[0];
        dom.opsModuleTitle.textContent = module.name;
        dom.opsModuleDesc.textContent = module.desc;
        if (dom.opsModuleTag) {
          const groupLabel = OPS_GROUP_MAP[module.group]?.name || "模块";
          dom.opsModuleTag.innerHTML = `<strong>${escapeHtml(module.tag || "MODULE")}</strong> ${escapeHtml(groupLabel)}`;
        }
      }

      async function ensureOpsConfig(force = false) {
        if (!force && state.opsCache.config) {
          return state.opsCache.config;
        }
        const body = await request({ method: "GET", url: buildV2("config/fetch") });
        state.opsCache.config = normalizeObject(body);
        return state.opsCache.config;
      }

      async function loadOpsModuleData(moduleKey, force = false) {
        if (moduleKey === "security" || moduleKey === "oauth" || moduleKey === "site" || moduleKey === "telegram") {
          await ensureOpsConfig(force);
          return;
        }

        if (moduleKey === "riskreview") {
          if (!force && state.opsCache.config && state.opsCache.riskReviewPagination && state.opsCache.banRecordPagination) return;
          const riskReviewPage = Math.max(1, Number(state.opsCache.riskReviewPage || 1));
          const banRecordPage = Math.max(1, Number(state.opsCache.banRecordPage || 1));
          const [configBody, reviewsBody, banRecordsBody] = await Promise.all([
            ensureOpsConfig(force),
            request({ method: "GET", url: `${buildV2("risk-review/fetch")}?current=${riskReviewPage}&pageSize=20` }),
            request({ method: "GET", url: `${buildV2("user/ban-records")}?current=${banRecordPage}&pageSize=20` }),
          ]);
          state.opsCache.config = normalizeObject(configBody);
          const reviewsObj = normalizeObject(reviewsBody);
          const banRecordsObj = normalizeObject(banRecordsBody);
          state.opsCache.riskReviews = Array.isArray(reviewsObj.data) ? reviewsObj.data : [];
          state.opsCache.riskReviewPagination = {
            current_page: Number(reviewsObj.current_page || 1),
            last_page: Number(reviewsObj.last_page || 1),
            total: Number(reviewsObj.total || 0),
          };
          state.opsCache.banRecords = Array.isArray(banRecordsObj.data) ? banRecordsObj.data : [];
          state.opsCache.banRecordPagination = {
            current_page: Number(banRecordsObj.current_page || 1),
            last_page: Number(banRecordsObj.last_page || 1),
            total: Number(banRecordsObj.total || 0),
          };
          state.opsCache.riskReviewPage = state.opsCache.riskReviewPagination.current_page || 1;
          state.opsCache.banRecordPage = state.opsCache.banRecordPagination.current_page || 1;
          return;
        }

        if (moduleKey === "plans") {
          if (!force && state.opsCache.plans.length) return;
          const [planBody, groupBody] = await Promise.all([
            request({ method: "GET", url: buildV2("plan/fetch") }),
            request({ method: "GET", url: buildV2("server/group/fetch") })
          ]);
          state.opsCache.plans = normalizeList(planBody);
          state.opsCache.groups = normalizeList(groupBody);
          return;
        }

        if (moduleKey === "payments") {
          if (!force && state.opsCache.payments.length) return;
          const [paymentBody, methodsBody] = await Promise.all([
            request({ method: "GET", url: buildV2("payment/fetch") }),
            request({ method: "GET", url: buildV2("payment/getPaymentMethods") })
          ]);
          state.opsCache.payments = normalizeList(paymentBody);
          state.opsCache.paymentMethods = normalizeList(methodsBody);
          return;
        }

        if (moduleKey === "notices") {
          if (!force && state.opsCache.notices.length) return;
          const body = await request({ method: "GET", url: buildV2("notice/fetch") });
          state.opsCache.notices = normalizeList(body);
          return;
        }

        if (moduleKey === "tickets") {
          if (!force && state.opsCache.tickets.length) return;
          const body = await request({ method: "GET", url: `${buildV2("ticket/fetch")}?current=1&pageSize=20` });
          state.opsCache.tickets = normalizeList(body);
          return;
        }

        if (moduleKey === "coupons") {
          if (!force && state.opsCache.coupons.length) return;
          const body = await request({ method: "GET", url: `${buildV2("coupon/fetch")}?current=1&pageSize=20` });
          state.opsCache.coupons = normalizeList(body);
          return;
        }

        if (moduleKey === "giftcards") {
          if (!force && state.opsCache.giftTemplates.length) return;
          const [templateBody, typeBody, statBody, codeBody] = await Promise.all([
            request({ method: "GET", url: `${buildV2("gift-card/templates")}?per_page=20&page=1` }),
            request({ method: "GET", url: buildV2("gift-card/types") }),
            request({ method: "GET", url: buildV2("gift-card/statistics") }),
            request({ method: "GET", url: `${buildV2("gift-card/codes")}?per_page=20&page=1` })
          ]);
          state.opsCache.giftTemplates = normalizeList(templateBody);
          state.opsCache.giftTypes = normalizeObject(typeBody);
          state.opsCache.giftStats = normalizeObject(statBody);
          state.opsCache.giftCodes = normalizeList(codeBody);
          return;
        }

        if (moduleKey === "plugins") {
          if (!force && state.opsCache.plugins.length) return;
          const body = await request({ method: "GET", url: buildV2("plugin/getPlugins") });
          state.opsCache.plugins = normalizeList(body);
          return;
        }

        if (moduleKey === "system") {
          if (!force && state.opsCache.systemStatus && state.opsCache.queueStats) return;
          const [statusRs, queueRs, logRs, failedRs] = await Promise.allSettled([
            request({ method: "GET", url: buildV2("system/getSystemStatus") }),
            request({ method: "GET", url: buildV2("system/getQueueStats") }),
            request({ method: "GET", url: `${buildV2("system/getSystemLog")}?current=1&page_size=20` }),
            request({ method: "GET", url: `${buildV2("system/getHorizonFailedJobs")}?current=1&page_size=20` })
          ]);

          state.opsCache.systemStatus = statusRs.status === "fulfilled"
            ? normalizeObject(statusRs.value)
            : { _error: statusRs.reason?.message || "读取失败" };
          state.opsCache.queueStats = queueRs.status === "fulfilled"
            ? normalizeObject(queueRs.value)
            : { _error: queueRs.reason?.message || "读取失败" };
          state.opsCache.systemLogs = logRs.status === "fulfilled"
            ? normalizeObject(logRs.value)
            : { data: [], total: 0, _error: logRs.reason?.message || "读取失败" };
          state.opsCache.failedJobs = failedRs.status === "fulfilled"
            ? normalizeObject(failedRs.value)
            : { data: [], total: 0, _error: failedRs.reason?.message || "读取失败" };
          return;
        }

        if (moduleKey === "traffic") {
          const days = Number(state.opsCache.trafficDays || 30);
          if (!force && state.opsCache.trafficStats && state.opsCache.trafficLogs.length) return;
          const [statBody, logsBody] = await Promise.all([
            request({ method: "GET", url: `${buildV2("traffic-reset/stats")}?days=${days}` }),
            request({ method: "GET", url: `${buildV2("traffic-reset/logs")}?per_page=20&page=1` })
          ]);
          state.opsCache.trafficStats = normalizeObject(statBody);
          const logsObj = normalizeObject(logsBody);
          state.opsCache.trafficLogs = Array.isArray(logsObj.data) ? logsObj.data : [];
          state.opsCache.trafficPagination = logsObj.pagination || null;
          return;
        }
      }

      function renderOpsSecurity() {
        const config = state.opsCache.config || {};
        const safe = config.safe || {};
        const system = config.system || {};
        const whitelist = Array.isArray(safe.email_whitelist_suffix)
          ? safe.email_whitelist_suffix.join("\n")
          : "";
        const registerMode = String(normalizeConfigValue(safe.register_mode, "all"));

        return `
          <div class="ops-grid-3">
            <article class="ops-kpi">
              <span class="label">安全模式</span>
              <span class="num">${yesNo(safe.safe_mode_enable)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">验证码</span>
              <span class="num">${yesNo(safe.captcha_enable)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">当前 PoW 难度</span>
              <span class="num">${escapeHtml(normalizeConfigValue(safe.pow_effective_difficulty, safe.pow_difficulty || 4))}</span>
            </article>
          </div>

          <section class="ops-card">
            <h4 class="ops-card-title">访问安全策略</h4>
            <div class="toolbar">
              ${opsBoolField("safe_mode_enable", "安全模式", safe.safe_mode_enable, 3)}
              ${opsBoolField("safe_email_verify", "注册邮箱验证", safe.email_verify, 3)}
              <div class="field span-3">
                <label for="safe_register_mode">注册方式</label>
                <select id="safe_register_mode">
                  <option value="all" ${registerMode === "all" ? "selected" : ""}>邮箱 + OAuth</option>
                  <option value="email_only" ${registerMode === "email_only" ? "selected" : ""}>仅邮箱注册</option>
                  <option value="oauth_only" ${registerMode === "oauth_only" ? "selected" : ""}>仅 OAuth 注册</option>
                  <option value="closed" ${registerMode === "closed" ? "selected" : ""}>关闭注册</option>
                </select>
              </div>
              ${opsBoolField("safe_email_whitelist_enable", "邮箱后缀白名单", safe.email_whitelist_enable, 3)}
              ${opsBoolField("safe_email_gmail_limit_enable", "Gmail 邮箱规则限制", safe.email_gmail_limit_enable, 3)}
              <div class="field span-6">
                <label for="safe_secure_path">后台登录地址</label>
                <input id="safe_secure_path" value="${escapeHtml(normalizeConfigValue(safe.secure_path, securePath))}" />
              </div>
              <div class="field span-6">
                <label for="safe_email_whitelist_suffix">允许后缀（每行一个）</label>
                <textarea id="safe_email_whitelist_suffix" placeholder="gmail.com&#10;qq.com">${escapeHtml(whitelist)}</textarea>
              </div>
            </div>
            <div class="actions">
              <button class="btn ghost" id="opsGenerateSecurePathBtn">一键生成随机地址</button>
              <button class="btn primary" id="opsSecuritySaveBtn">保存安全设置</button>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">登录与注册防护</h4>
            <div class="toolbar">
              ${opsBoolField("safe_register_limit_by_ip_enable", "按网络地址限制注册", safe.register_limit_by_ip_enable, 3)}
              <div class="field span-3">
                <label for="safe_register_limit_count">注册次数上限</label>
                <input id="safe_register_limit_count" type="number" min="1" value="${escapeHtml(normalizeConfigValue(safe.register_limit_count, 3))}" />
              </div>
              <div class="field span-3">
                <label for="safe_register_limit_expire">统计时长(分钟)</label>
                <input id="safe_register_limit_expire" type="number" min="1" value="${escapeHtml(normalizeConfigValue(safe.register_limit_expire, 60))}" />
              </div>
              ${opsBoolField("safe_password_limit_enable", "密码错误限制", safe.password_limit_enable, 3)}
              <div class="field span-3">
                <label for="safe_password_limit_count">密码错误上限</label>
                <input id="safe_password_limit_count" type="number" min="1" value="${escapeHtml(normalizeConfigValue(safe.password_limit_count, 5))}" />
              </div>
              <div class="field span-3">
                <label for="safe_password_limit_expire">锁定时长(分钟)</label>
                <input id="safe_password_limit_expire" type="number" min="1" value="${escapeHtml(normalizeConfigValue(safe.password_limit_expire, 60))}" />
              </div>
              <div class="field span-3">
                <label for="safe_login_token_expire_days">登录态有效期(天，0=永不过期)</label>
                <input id="safe_login_token_expire_days" type="number" min="0" max="3650" value="${escapeHtml(normalizeConfigValue(safe.login_token_expire_days, 365))}" />
              </div>
            </div>
          </section>

	          <section class="ops-card">
	            <h4 class="ops-card-title">验证码与防刷验证</h4>
	            <div class="toolbar">
              ${opsBoolField("safe_captcha_enable", "启用验证码", safe.captcha_enable, 3)}
              <div class="field span-3">
                <label for="safe_captcha_type">验证码类型</label>
                <select id="safe_captcha_type">
                  <option value="recaptcha" ${String(safe.captcha_type) === "recaptcha" ? "selected" : ""}>Google 验证（常规）</option>
                  <option value="recaptcha-v3" ${String(safe.captcha_type) === "recaptcha-v3" ? "selected" : ""}>Google 验证（无感）</option>
                  <option value="turnstile" ${String(safe.captcha_type) === "turnstile" ? "selected" : ""}>Cloudflare 验证</option>
                </select>
              </div>
              ${opsBoolField("safe_pow_enable", "启用防刷验证", safe.pow_enable, 3)}
              ${opsBoolField("safe_pow_require_ja3", "设备指纹校验", safe.pow_require_ja3, 3)}
              ${opsBoolField("safe_pow_auto_scale_enable", "自动调节难度", safe.pow_auto_scale_enable, 3)}
              <div class="field span-3">
                <label for="safe_pow_difficulty">防刷难度</label>
                <input id="safe_pow_difficulty" type="number" min="1" max="8" value="${escapeHtml(normalizeConfigValue(safe.pow_difficulty, 4))}" />
              </div>
              <div class="field span-3">
                <label for="safe_pow_ttl">有效时间(秒)</label>
                <input id="safe_pow_ttl" type="number" min="30" max="600" value="${escapeHtml(normalizeConfigValue(safe.pow_ttl, 120))}" />
              </div>
              <div class="field span-3">
                <label for="safe_recap_score">无感验证分数线</label>
                <input id="safe_recap_score" type="number" min="0" max="1" step="0.01" value="${escapeHtml(normalizeConfigValue(safe.recaptcha_v3_score_threshold, 0.5))}" />
              </div>
              <div class="field span-3">
                <label for="safe_pow_auto_max_difficulty">自动调节上限</label>
                <input id="safe_pow_auto_max_difficulty" type="number" min="1" max="8" value="${escapeHtml(normalizeConfigValue(safe.pow_auto_max_difficulty, 7))}" />
              </div>
              <div class="field span-3">
                <label for="safe_pow_effective_difficulty">当前生效难度</label>
                <input id="safe_pow_effective_difficulty" value="${escapeHtml(normalizeConfigValue(safe.pow_effective_difficulty, safe.pow_difficulty || 4))}" disabled />
              </div>
              <div class="field span-6">
                <label for="safe_pow_seed_salt">防刷密钥</label>
                <input id="safe_pow_seed_salt" value="${escapeHtml(normalizeConfigValue(safe.pow_seed_salt, ""))}" />
              </div>
              <div class="field span-6">
                <label for="safe_pow_base_value">防刷基础值</label>
                <input id="safe_pow_base_value" value="${escapeHtml(normalizeConfigValue(safe.pow_base_value, "portal"))}" />
              </div>
              <div class="field span-6">
                <label for="safe_recaptcha_key">Google 验证服务端密钥</label>
                <input id="safe_recaptcha_key" value="${escapeHtml(normalizeConfigValue(safe.recaptcha_key, ""))}" />
              </div>
              <div class="field span-6">
                <label for="safe_recaptcha_site_key">Google 验证网页端密钥</label>
                <input id="safe_recaptcha_site_key" value="${escapeHtml(normalizeConfigValue(safe.recaptcha_site_key, ""))}" />
              </div>
              <div class="field span-6">
                <label for="safe_recaptcha_v3_secret_key">Google 无感服务端密钥</label>
                <input id="safe_recaptcha_v3_secret_key" value="${escapeHtml(normalizeConfigValue(safe.recaptcha_v3_secret_key, ""))}" />
              </div>
              <div class="field span-6">
                <label for="safe_recaptcha_v3_site_key">Google 无感网页端密钥</label>
                <input id="safe_recaptcha_v3_site_key" value="${escapeHtml(normalizeConfigValue(safe.recaptcha_v3_site_key, ""))}" />
              </div>
              <div class="field span-6">
                <label for="safe_turnstile_secret_key">Cloudflare 服务端密钥</label>
                <input id="safe_turnstile_secret_key" value="${escapeHtml(normalizeConfigValue(safe.turnstile_secret_key, ""))}" />
              </div>
              <div class="field span-6">
                <label for="safe_turnstile_site_key">Cloudflare 网页端密钥</label>
                <input id="safe_turnstile_site_key" value="${escapeHtml(normalizeConfigValue(safe.turnstile_site_key, ""))}" />
              </div>
	            </div>
	            <p class="hint-text">提示：如果修改后台登录地址，请先确认新地址可打开，再关闭当前页面。</p>
	          </section>

	          <section class="ops-card">
	            <h4 class="ops-card-title">争议退款投票</h4>
	            <div class="toolbar">
	              ${opsBoolField("sys_refund_dispute_enable", "启用争议退款投票", system.refund_dispute_enable, 3)}
	            </div>
	            <p class="hint-text">关闭后用户侧不再渲染争议投票入口与投票页面；已发起的争议投票也会被接口层拒绝访问。</p>
	          </section>

	          <pre id="opsSecurityResult">等待保存...</pre>
	        `;
	      }

      function renderOpsOAuth() {
        const config = state.opsCache.config || {};
        const oauth = config.oauth || {};
        const site = config.site || {};
        const appUrl = String(normalizeConfigValue(site.app_url, settings.base_url || "")).replace(/\/+$/, "");
        const callbackPreview = appUrl
          ? `${appUrl}/api/v1/passport/oauth2/linux-do/callback`
          : "/api/v1/passport/oauth2/linux-do/callback";

        return `
          <div class="ops-grid-3">
            <article class="ops-kpi">
              <span class="label">第三方登录</span>
              <span class="num">${yesNo(oauth.oauth_linux_do_enable)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">客户端编号</span>
              <span class="num">${oauth.oauth_linux_do_client_id ? "已填写" : "未填写"}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">客户端密钥</span>
              <span class="num">${oauth.oauth_linux_do_client_secret ? "已填写" : "未填写"}</span>
            </article>
          </div>

          <section class="ops-card">
            <h4 class="ops-card-title">Linux DO 登录设置</h4>
            <div class="toolbar">
              ${opsBoolField("oauth_linux_do_enable", "启用 Linux DO 登录", oauth.oauth_linux_do_enable, 4)}
              <div class="field span-8">
                <label for="oauth_linux_do_client_id">客户端编号（Client ID）</label>
                <input id="oauth_linux_do_client_id" value="${escapeHtml(normalizeConfigValue(oauth.oauth_linux_do_client_id, ""))}" placeholder="请填写 Linux DO 应用 Client ID" />
              </div>
              <div class="field span-12">
                <label for="oauth_linux_do_client_secret">客户端密钥（Client Secret）</label>
                <input id="oauth_linux_do_client_secret" type="password" value="${escapeHtml(normalizeConfigValue(oauth.oauth_linux_do_client_secret, ""))}" placeholder="请填写 Linux DO 应用 Client Secret" />
              </div>
              <div class="field span-12">
                <label for="oauth_linux_do_redirect_uri">回调地址（必须和 Linux DO 应用后台一致）</label>
                <input id="oauth_linux_do_redirect_uri" value="${escapeHtml(normalizeConfigValue(oauth.oauth_linux_do_redirect_uri, callbackPreview))}" />
              </div>
              <div class="field span-12">
                <label>系统推荐回调地址（根据站点地址自动生成）</label>
                <input value="${escapeHtml(callbackPreview)}" disabled />
              </div>
            </div>
            <div class="actions">
              <button class="btn ghost" id="opsOAuthUsePreviewBtn">使用推荐回调地址</button>
              <button class="btn primary" id="opsOAuthSaveBtn">保存第三方登录设置</button>
            </div>
            <p class="hint-text">普通用户会显示第三方登录得到的用户名，建议先配置完整后再开放登录入口。</p>
          </section>

          <pre id="opsOAuthResult">等待保存...</pre>
        `;
      }

      function renderOpsSite() {
        const config = state.opsCache.config || {};
        const site = config.site || {};
        const frontend = config.frontend || {};
        const subscribe = config.subscribe || {};
        const registerMode = String(normalizeConfigValue(site.register_mode, "all"));
        const currentTheme = normalizeConfigValue(frontend.frontend_theme, "Maintainable");

        return `
          <section class="ops-card">
            <h4 class="ops-card-title">站点基础信息</h4>
            <div class="toolbar">
              <div class="field span-6">
                <label for="site_app_name">站点名称</label>
                <input id="site_app_name" value="${escapeHtml(normalizeConfigValue(site.app_name, "Portal"))}" />
              </div>
              <div class="field span-6">
                <label for="site_app_description">站点描述</label>
                <input id="site_app_description" value="${escapeHtml(normalizeConfigValue(site.app_description, ""))}" />
              </div>
              <div class="field span-6">
                <label for="site_app_url">站点地址</label>
                <input id="site_app_url" value="${escapeHtml(normalizeConfigValue(site.app_url, ""))}" placeholder="https://example.com" />
              </div>
              <div class="field span-6">
                <label for="site_subscribe_url">订阅域名（可空）</label>
                <input id="site_subscribe_url" value="${escapeHtml(normalizeConfigValue(site.subscribe_url, ""))}" placeholder="https://sub.example.com" />
              </div>
              <div class="field span-6">
                <label for="site_register_mode">注册方式</label>
                <select id="site_register_mode">
                  <option value="all" ${registerMode === "all" ? "selected" : ""}>邮箱 + OAuth</option>
                  <option value="email_only" ${registerMode === "email_only" ? "selected" : ""}>仅邮箱注册</option>
                  <option value="oauth_only" ${registerMode === "oauth_only" ? "selected" : ""}>仅 OAuth 注册</option>
                  <option value="closed" ${registerMode === "closed" ? "selected" : ""}>关闭注册</option>
                </select>
              </div>
              <div class="field span-6">
                <label for="site_logo">Logo 地址</label>
                <input id="site_logo" value="${escapeHtml(normalizeConfigValue(site.logo, ""))}" />
              </div>
              <div class="field span-6">
                <label for="site_tos_url">服务条款地址</label>
                <input id="site_tos_url" value="${escapeHtml(normalizeConfigValue(site.tos_url, ""))}" />
              </div>
              ${opsBoolField("site_force_https", "强制 HTTPS", site.force_https, 3)}
              <div class="field span-3">
                <label for="site_currency">货币代码</label>
                <input id="site_currency" value="${escapeHtml(normalizeConfigValue(site.currency, "CNY"))}" />
              </div>
              <div class="field span-3">
                <label for="site_currency_symbol">货币符号</label>
                <input id="site_currency_symbol" value="${escapeHtml(normalizeConfigValue(site.currency_symbol, "¥"))}" />
              </div>
              <div class="field span-3">
                <label for="site_try_out_plan_id">试用套餐编号</label>
                <input id="site_try_out_plan_id" type="number" min="0" value="${escapeHtml(normalizeConfigValue(site.try_out_plan_id, 0))}" />
              </div>
              <div class="field span-3">
                <label for="site_try_out_hour">试用时长(小时)</label>
                <input id="site_try_out_hour" type="number" min="1" value="${escapeHtml(normalizeConfigValue(site.try_out_hour, 1))}" />
              </div>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">前端与订阅行为</h4>
            <div class="toolbar">
              <div class="field span-4">
                <label for="site_frontend_theme_fixed">前端主题（固定）</label>
                <input id="site_frontend_theme_fixed" value="${escapeHtml(currentTheme)}" disabled />
              </div>
              <div class="field span-4">
                <label for="site_frontend_sidebar">侧边栏风格</label>
                <select id="site_frontend_sidebar">
                  <option value="light" ${String(frontend.frontend_theme_sidebar) === "light" ? "selected" : ""}>light</option>
                  <option value="dark" ${String(frontend.frontend_theme_sidebar) === "dark" ? "selected" : ""}>dark</option>
                </select>
              </div>
              <div class="field span-4">
                <label for="site_frontend_header">顶部风格</label>
                <select id="site_frontend_header">
                  <option value="dark" ${String(frontend.frontend_theme_header) === "dark" ? "selected" : ""}>dark</option>
                  <option value="light" ${String(frontend.frontend_theme_header) === "light" ? "selected" : ""}>light</option>
                </select>
              </div>
              <div class="field span-4">
                <label for="site_frontend_color">主题配色</label>
                <select id="site_frontend_color">
                  <option value="default" ${String(frontend.frontend_theme_color) === "default" ? "selected" : ""}>default</option>
                  <option value="darkblue" ${String(frontend.frontend_theme_color) === "darkblue" ? "selected" : ""}>darkblue</option>
                  <option value="black" ${String(frontend.frontend_theme_color) === "black" ? "selected" : ""}>black</option>
                  <option value="green" ${String(frontend.frontend_theme_color) === "green" ? "selected" : ""}>green</option>
                </select>
              </div>
              <div class="field span-4">
                <label for="site_subscribe_path">订阅路径</label>
                <input id="site_subscribe_path" value="${escapeHtml(normalizeConfigValue(subscribe.subscribe_path, "s"))}" />
              </div>
              <div class="field span-12">
                <label for="site_subscribe_root_domains">订阅根域名（每行一个，用于随机子域名）</label>
                <textarea id="site_subscribe_root_domains" placeholder="example.com&#10;sub.example.net">${escapeHtml(normalizeConfigValue(site.subscribe_root_domains, ""))}</textarea>
              </div>
              ${opsBoolField("site_plan_change_enable", "允许变更套餐", subscribe.plan_change_enable, 4)}
              ${opsBoolField("site_surplus_enable", "保留剩余价值", subscribe.surplus_enable, 4)}
              <div class="field span-12">
                <label for="site_frontend_background_url">背景图地址（可空）</label>
                <input id="site_frontend_background_url" value="${escapeHtml(normalizeConfigValue(frontend.frontend_background_url, ""))}" />
              </div>
            </div>
            <div class="actions">
              <button class="btn primary" id="opsSiteSaveBtn">保存站点设置</button>
            </div>
          </section>

          <pre id="opsSiteResult">等待保存...</pre>
        `;
      }

      function renderOpsTelegram() {
        const config = state.opsCache.config || {};
        const telegram = config.telegram || {};

        return `
          <section class="ops-card">
            <h4 class="ops-card-title">Bot 基础配置</h4>
            <div class="toolbar">
              ${opsBoolField("telegram_bot_enable", "启用 Telegram Bot", telegram.telegram_bot_enable, 3)}
              <div class="field span-6">
                <label for="telegram_bot_token">Bot Token</label>
                <input id="telegram_bot_token" type="password" autocomplete="new-password" value="${escapeHtml(normalizeConfigValue(telegram.telegram_bot_token, ""))}" placeholder="Bot Token" />
              </div>
              <div class="field span-6">
                <label for="telegram_discuss_link">讨论群/频道链接</label>
                <input id="telegram_discuss_link" value="${escapeHtml(normalizeConfigValue(telegram.telegram_discuss_link, ""))}" placeholder="https://t.me/..." />
              </div>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">功能与通知</h4>
            <div class="toolbar">
              ${opsBoolField("telegram_user_ticket_enable", "允许用户在 Telegram 处理工单", telegram.telegram_user_ticket_enable, 3)}
              ${opsBoolField("telegram_notify_ticket_created", "工单创建通知", telegram.telegram_notify_ticket_created, 3)}
              ${opsBoolField("telegram_notify_ticket_replied", "工单回复通知", telegram.telegram_notify_ticket_replied, 3)}
              ${opsBoolField("telegram_notify_ticket_closed", "工单关闭通知", telegram.telegram_notify_ticket_closed, 3)}
              ${opsBoolField("telegram_notify_payment_success", "支付成功通知", telegram.telegram_notify_payment_success, 3)}
              ${opsBoolField("telegram_notify_notice_published", "公告发布通知", telegram.telegram_notify_notice_published, 3)}
              ${opsBoolField("telegram_notify_tcping_alert", "TCPing 告警通知", telegram.telegram_notify_tcping_alert, 3)}
              ${opsBoolField("telegram_notify_tcping_recover", "TCPing 恢复通知", telegram.telegram_notify_tcping_recover, 3)}
              ${opsBoolField("telegram_notify_refund_vote", "争议投票通知", telegram.telegram_notify_refund_vote, 3)}
              ${opsBoolField("telegram_notify_refund_status", "争议状态变更通知", telegram.telegram_notify_refund_status, 3)}
              ${opsBoolField("telegram_notify_user_risk_detected", "用户风险审查通知", telegram.telegram_notify_user_risk_detected, 3)}
              ${opsBoolField("telegram_notify_user_banned", "用户封禁通知", telegram.telegram_notify_user_banned, 3)}
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">操作</h4>
            <div class="toolbar">
              <div class="field span-12">
                <div class="actions">
                  <button class="btn ghost" id="opsRegisterTelegramWebhookBtn">注册/刷新 Webhook</button>
                  <button class="btn primary" id="opsTelegramSaveBtn">保存 Telegram 设置</button>
                </div>
              </div>
            </div>
            <pre id="opsTelegramResult">等待操作...</pre>
          </section>
        `;
      }

      function renderOpsRiskReview() {
        const config = state.opsCache.config || {};
        const risk = config.risk_review || {};
        const telegram = config.telegram || {};
        const reviews = Array.isArray(state.opsCache.riskReviews) ? state.opsCache.riskReviews : [];
        const pagination = state.opsCache.riskReviewPagination || {};
        const banRecords = Array.isArray(state.opsCache.banRecords) ? state.opsCache.banRecords : [];
        const banPagination = state.opsCache.banRecordPagination || {};
        const reviewTotal = Number(pagination.total || reviews.length || 0);
        const banTotal = Number(banPagination.total || banRecords.length || 0);

        return `
          <section class="ops-card">
            <p class="ops-section-kicker">RISK REVIEW</p>
            <h4 class="ops-card-title">共享 IP 风险识别与封禁流水</h4>
            <p class="ops-card-subtitle">当单 IP 命中多个用户时先进入待审队列，LLM 只做预审建议，最终封禁动作保留给超级管理员人工确认。</p>
            <div class="ops-main-head-meta">
              <span class="ops-module-chip"><strong>Review</strong> ${escapeHtml(yesNo(risk.user_risk_review_enable))}</span>
              <span class="ops-module-chip"><strong>LLM</strong> ${escapeHtml(yesNo(risk.user_risk_review_llm_enable))}</span>
              <span class="ops-module-chip"><strong>Cooldown</strong> ${escapeHtml(String(normalizeConfigValue(risk.user_risk_review_notify_cooldown_minutes, 60)))} 分钟</span>
            </div>
          </section>

          <div class="ops-highlight-grid ops-grid-4">
            <article class="ops-kpi">
              <span class="label">审查开关</span>
              <span class="num">${yesNo(risk.user_risk_review_enable)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">LLM 预审查</span>
              <span class="num">${yesNo(risk.user_risk_review_llm_enable)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">最近审查记录</span>
              <span class="num">${formatCompactNumber(reviewTotal)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">封禁记录总数</span>
              <span class="num">${formatCompactNumber(banTotal)}</span>
            </article>
          </div>

          <div class="ops-split-grid">
            <section class="ops-card">
              <div class="ops-table-header">
                <div class="ops-table-caption">
                  <h4 class="ops-card-title">共享 IP 风险识别</h4>
                  <p class="ops-card-subtitle">控制扫描窗口、命中人数阈值、提醒冷却和 Telegram 通知节奏。</p>
                </div>
              </div>
              <div class="toolbar">
                ${opsBoolField("risk_user_risk_review_enable", "启用定期审查", risk.user_risk_review_enable, 3)}
                <div class="field span-3">
                  <label for="risk_user_risk_review_schedule_minutes">审查间隔(分钟)</label>
                  <input id="risk_user_risk_review_schedule_minutes" type="number" min="5" max="1440" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_schedule_minutes, 30))}" />
                </div>
                <div class="field span-3">
                  <label for="risk_user_risk_review_time_window_minutes">共享 IP 统计窗口(分钟)</label>
                  <input id="risk_user_risk_review_time_window_minutes" type="number" min="5" max="1440" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_time_window_minutes, 60))}" />
                </div>
                <div class="field span-3">
                  <label for="risk_user_risk_review_context_hours">上下文分析时长(小时)</label>
                  <input id="risk_user_risk_review_context_hours" type="number" min="1" max="168" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_context_hours, 24))}" />
                </div>
                <div class="field span-3">
                  <label for="risk_user_risk_review_min_shared_ip_users">最少命中用户数</label>
                  <input id="risk_user_risk_review_min_shared_ip_users" type="number" min="2" max="50" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_min_shared_ip_users, 2))}" />
                </div>
                <div class="field span-3">
                  <label for="risk_user_risk_review_scan_limit">单次最大扫描 IP 数</label>
                  <input id="risk_user_risk_review_scan_limit" type="number" min="1" max="200" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_scan_limit, 20))}" />
                </div>
                <div class="field span-3">
                  <label for="risk_user_risk_review_notify_cooldown_minutes">提醒冷却(分钟)</label>
                  <input id="risk_user_risk_review_notify_cooldown_minutes" type="number" min="5" max="10080" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_notify_cooldown_minutes, 60))}" />
                </div>
                ${opsBoolField("risk_telegram_notify_user_risk_detected", "TG 风险提醒", telegram.telegram_notify_user_risk_detected, 3)}
                ${opsBoolField("risk_telegram_notify_user_banned", "TG 封禁通知", telegram.telegram_notify_user_banned, 3)}
              </div>
            </section>

            <section class="ops-card">
              <div class="ops-table-header">
                <div class="ops-table-caption">
                  <h4 class="ops-card-title">LLM 预审查与人工执行</h4>
                  <p class="ops-card-subtitle">模型只给出滥用建议与封禁意见，实际封禁仍需人工确认并填写原因。</p>
                </div>
                <div class="actions">
                  <button class="btn primary" id="opsRiskReviewSaveBtn">保存设置</button>
                  <button class="btn ghost" id="opsRiskReviewRunBtn">立即执行一次审查</button>
                  <button class="btn ghost" id="opsRiskReviewReloadBtn">刷新审查记录</button>
                </div>
              </div>
              <div class="toolbar">
                ${opsBoolField("risk_user_risk_review_llm_enable", "启用 LLM 预审", risk.user_risk_review_llm_enable, 3)}
                <div class="field span-6">
                  <label for="risk_user_risk_review_llm_base_url">OpenAI Compatible Base URL</label>
                  <input id="risk_user_risk_review_llm_base_url" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_llm_base_url, ""))}" placeholder="https://api.openai.com/v1 或 https://api.openai.com/v1/responses" />
                </div>
                <div class="field span-6">
                  <label for="risk_user_risk_review_llm_api_key">API Key</label>
                  <input id="risk_user_risk_review_llm_api_key" type="password" autocomplete="new-password" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_llm_api_key, ""))}" placeholder="sk-..." />
                </div>
                <p class="hint-text span-12">支持直接填写根路径、完整 <code>/chat/completions</code> 地址，或完整 <code>/responses</code> 地址，系统会自动识别端点类型。</p>
                <div class="field span-6">
                  <label for="risk_user_risk_review_llm_model">模型名称</label>
                  <input id="risk_user_risk_review_llm_model" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_llm_model, ""))}" placeholder="gpt-4.1-mini / qwen-plus / deepseek-chat" />
                </div>
                <div class="field span-3">
                  <label for="risk_user_risk_review_llm_timeout_seconds">超时(秒)</label>
                  <input id="risk_user_risk_review_llm_timeout_seconds" type="number" min="5" max="120" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_llm_timeout_seconds, 20))}" />
                </div>
                <div class="field span-3">
                  <label for="risk_user_risk_review_llm_temperature">Temperature</label>
                  <input id="risk_user_risk_review_llm_temperature" type="number" min="0" max="1" step="0.1" value="${escapeHtml(normalizeConfigValue(risk.user_risk_review_llm_temperature, 0.2))}" />
                </div>
              </div>
              <div class="ops-helper-list">
                <div class="ops-helper-item">
                  <strong>人工复核优先级</strong>
                  <p>单 IP 命中多用户会直接进入中风险，建议先看共享 IP、匹配用户和模型摘要，再决定是否封禁。</p>
                </div>
                <div class="ops-helper-item">
                  <strong>通知策略</strong>
                  <p>Telegram 仅负责提醒与封禁结果同步，避免相同用户在冷却期内重复打扰。</p>
                </div>
              </div>
            </section>
          </div>

          <section class="ops-card">
            <div class="ops-table-header">
              <div class="ops-table-caption">
                <h4 class="ops-card-title">最近审查结果</h4>
                <p class="ops-card-subtitle">当前页 ${pagination.current_page || 1} / ${pagination.last_page || 1}，支持直接从建议结果进入封禁动作。</p>
              </div>
              <span class="badge ok">共 ${formatCompactNumber(reviewTotal)} 条审查记录</span>
            </div>
            <div class="table-wrap">
              <table class="ops-review-table">
                <thead>
                  <tr>
                    <th>时间</th>
                    <th>用户</th>
                    <th>共享 IP</th>
                    <th>命中用户</th>
                    <th>风险</th>
                    <th>状态</th>
                    <th>模型</th>
                    <th>意见摘要</th>
                    <th>处理建议</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody>
                  ${reviews.length ? reviews.map((review) => `
                    <tr>
                      <td>${formatAnyTimestamp(review.reviewed_at)}</td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong>#${review.user_id}</strong>
                          <span>${escapeHtml(review.user_email || "-")}</span>
                        </div>
                      </td>
                      <td>${escapeHtml(review.shared_ip || "-")}</td>
                      <td>${escapeHtml((review.matched_users || []).map((item) => item.email || `#${item.id}`).join(", ") || String(review.matched_user_count || 0))}</td>
                      <td><span class="badge ${String(review.risk_level) === "high" ? "warn" : "ok"}">${escapeHtml(review.risk_level || "medium")} / ${escapeHtml(review.suspicion_score || 0)}</span></td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong><span class="badge ${review.user_banned ? "warn" : "ok"}">${review.user_banned ? "已封禁" : "正常"}</span></strong>
                          <span>${escapeHtml((review.user_ban_reason || "").trim() || "-")}</span>
                        </div>
                      </td>
                      <td>${escapeHtml(review.llm_model || "heuristic")}</td>
                      <td>${escapeHtml(review.summary || "-")}</td>
                      <td>${escapeHtml(review.recommendation || "-")}</td>
                      <td>
                        ${review.user_banned ? '<span class="badge ok">已处理</span>' : `<button class="btn warn risk-review-ban-btn" data-user-id="${review.user_id}" data-user-email="${escapeHtml(review.user_email || "")}" data-ban-reason="${escapeHtml(((review.recommendation || review.summary || "").trim()))}">封禁用户</button>`}
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="10" class="empty">暂无风险审查记录</td></tr>'}
                </tbody>
              </table>
            </div>
            <div class="actions ops-paged-actions">
              <button class="btn ghost" id="opsRiskReviewPrevBtn" ${Number(pagination.current_page || 1) <= 1 ? "disabled" : ""}>上一页</button>
              <button class="btn ghost" id="opsRiskReviewNextBtn" ${Number(pagination.current_page || 1) >= Number(pagination.last_page || 1) ? "disabled" : ""}>下一页</button>
            </div>
          </section>

          <section class="ops-card">
            <div class="ops-table-header">
              <div class="ops-table-caption">
                <h4 class="ops-card-title">最近封禁记录</h4>
                <p class="ops-card-subtitle">当前页 ${banPagination.current_page || 1} / ${banPagination.last_page || 1}，记录封禁和解封原因，便于后续追溯。</p>
              </div>
              <span class="badge ok">共 ${formatCompactNumber(banTotal)} 条封禁流水</span>
            </div>
            <div class="table-wrap">
              <table class="ops-review-table">
                <thead>
                  <tr>
                    <th>时间</th>
                    <th>用户</th>
                    <th>操作人</th>
                    <th>动作</th>
                    <th>来源</th>
                    <th>原因</th>
                  </tr>
                </thead>
                <tbody>
                  ${banRecords.length ? banRecords.map((record) => `
                    <tr>
                      <td>${formatAnyTimestamp(record.created_at)}</td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong>#${record.user_id}</strong>
                          <span>${escapeHtml(record.user_email || "-")}</span>
                        </div>
                      </td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong>${record.admin_id ? `#${record.admin_id}` : "-"}</strong>
                          <span>${escapeHtml(record.admin_email || "system")}</span>
                        </div>
                      </td>
                      <td><span class="badge ${String(record.action) === "ban" ? "warn" : "ok"}">${escapeHtml(record.action || "-")}</span></td>
                      <td>${escapeHtml(record.source || "-")}</td>
                      <td>${escapeHtml(record.reason || "-")}</td>
                    </tr>
                  `).join("") : '<tr><td colspan="6" class="empty">暂无封禁记录</td></tr>'}
                </tbody>
              </table>
            </div>
            <div class="actions ops-paged-actions">
              <button class="btn ghost" id="opsBanRecordPrevBtn" ${Number(banPagination.current_page || 1) <= 1 ? "disabled" : ""}>上一页</button>
              <button class="btn ghost" id="opsBanRecordNextBtn" ${Number(banPagination.current_page || 1) >= Number(banPagination.last_page || 1) ? "disabled" : ""}>下一页</button>
            </div>
          </section>

          <pre id="opsRiskReviewResult">等待操作...</pre>
        `;
      }

      function renderOpsPlans() {
        const plans = state.opsCache.plans || [];
        const groups = state.opsCache.groups || [];
        const visiblePlans = plans.filter((item) => isOn(item.show)).length;
        const sellPlans = plans.filter((item) => isOn(item.sell)).length;
        const groupCount = new Set(plans.map((item) => item.group_id).filter(Boolean)).size;
        const groupOptions = [`<option value="">未指定</option>`]
          .concat(groups.map((g) => `<option value="${g.id}">${escapeHtml(g.name || `Group-${g.id}`)}</option>`))
          .join("");

        return `
          <section class="ops-card">
            <p class="ops-section-kicker">PLAN MANAGEMENT</p>
            <h4 class="ops-card-title">套餐管理工作台</h4>
            <p class="ops-card-subtitle">把套餐列表、上下架和编辑器放到一个连续流程里，减少超管维护价格和销售状态时的来回切换。</p>
            <div class="ops-main-head-meta">
              <span class="ops-module-chip"><strong>Total</strong> ${formatCompactNumber(plans.length)} 套餐</span>
              <span class="ops-module-chip"><strong>Visible</strong> ${formatCompactNumber(visiblePlans)}</span>
              <span class="ops-module-chip"><strong>Selling</strong> ${formatCompactNumber(sellPlans)}</span>
            </div>
          </section>

          <div class="ops-summary-grid">
            <article class="ops-summary-card">
              <span>套餐总数</span>
              <strong>${formatCompactNumber(plans.length)}</strong>
              <small>当前已创建的套餐条目总量。</small>
            </article>
            <article class="ops-summary-card">
              <span>展示中</span>
              <strong>${formatCompactNumber(visiblePlans)}</strong>
              <small>前端仍然对用户显示的套餐数。</small>
            </article>
            <article class="ops-summary-card">
              <span>在售套餐</span>
              <strong>${formatCompactNumber(sellPlans)}</strong>
              <small>允许新购或续费的套餐数。</small>
            </article>
            <article class="ops-summary-card">
              <span>关联分组</span>
              <strong>${formatCompactNumber(groupCount)}</strong>
              <small>已在套餐中实际使用的节点分组数量。</small>
            </article>
          </div>

          <div class="ops-split-grid">
          <section class="ops-card">
            <div class="ops-table-header">
              <div class="ops-table-caption">
                <h4 class="ops-card-title">套餐列表</h4>
                <p class="ops-card-subtitle">左侧快速浏览套餐当前展示与销售状态，点击编辑可直接回填右侧编辑器。</p>
              </div>
              <div class="actions">
                <button class="btn ghost" id="opsPlansReloadBtn">刷新套餐</button>
                <span class="badge ok">共 ${plans.length} 个套餐</span>
              </div>
            </div>
            <div class="table-wrap">
              <table class="ops-plan-table">
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>名称</th>
                    <th>分组</th>
                    <th>流量配额</th>
                    <th>限速</th>
                    <th>状态</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="opsPlansTableBody">
                  ${plans.length ? plans.map((p) => `
                    <tr>
                      <td>${p.id}</td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong>${escapeHtml(p.name || "-")}</strong>
                          <span>${Array.isArray(p.tags) && p.tags.length ? escapeHtml(p.tags.join(", ")) : "无标签"}</span>
                        </div>
                      </td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong>${escapeHtml((p.group && p.group.name) || "-")}</strong>
                          <small>group_id: ${escapeHtml(String(p.group_id || "-"))}</small>
                        </div>
                      </td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong>${escapeHtml(String(p.transfer_enable ?? "-"))}</strong>
                          <small>原始配额值</small>
                        </div>
                      </td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong>${escapeHtml(String(p.speed_limit || 0))} Mbps</strong>
                          <small>设备 ${escapeHtml(String(p.device_limit || 0))} / 容量 ${escapeHtml(String(p.capacity_limit || 0))}</small>
                        </div>
                      </td>
                      <td>
                        <div class="ops-cell-stack">
                          <strong><span class="badge ${isOn(p.show) ? "ok" : "warn"}">${isOn(p.show) ? "展示" : "隐藏"}</span></strong>
                          <small>${isOn(p.sell) ? "在售中" : "停售中"}</small>
                          <div class="ops-plan-meter">
                            <div class="ops-plan-meter-fill" style="width:${(isOn(p.show) ? 50 : 0) + (isOn(p.sell) ? 50 : 0)}%"></div>
                          </div>
                        </div>
                      </td>
                      <td>
                        <div class="actions">
                          <button class="btn ghost" data-plan-action="edit" data-id="${p.id}">编辑</button>
                          <button class="btn ghost" data-plan-action="toggle-show" data-id="${p.id}">${isOn(p.show) ? "隐藏" : "展示"}</button>
                          <button class="btn ghost" data-plan-action="toggle-sell" data-id="${p.id}">${isOn(p.sell) ? "停售" : "上架"}</button>
                          <button class="btn danger" data-plan-action="drop" data-id="${p.id}">删除</button>
                        </div>
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="7" class="empty">暂无套餐</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <section class="ops-card">
            <div class="ops-table-header">
              <div class="ops-table-caption">
                <h4 class="ops-card-title">套餐编辑器</h4>
                <p class="ops-card-subtitle">支持新建、回填编辑、价格 JSON 调整和基础销售参数维护。</p>
              </div>
              <div class="actions">
                <button class="btn primary" id="opsPlanSaveBtn">保存套餐</button>
                <button class="btn ghost" id="opsPlanClearBtn">清空编辑器</button>
              </div>
            </div>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsPlanId">套餐编号（新建留空）</label>
                <input id="opsPlanId" type="number" min="1" placeholder="自动创建" />
              </div>
              <div class="field span-6">
                <label for="opsPlanName">套餐名称</label>
                <input id="opsPlanName" placeholder="例如：旗舰套餐" />
              </div>
              <div class="field span-3">
                <label for="opsPlanGroupId">分组</label>
                <select id="opsPlanGroupId">${groupOptions}</select>
              </div>
              <div class="field span-3">
                <label for="opsPlanTransfer">流量配额(GB)</label>
                <input id="opsPlanTransfer" type="number" min="1" value="100" />
              </div>
              <div class="field span-3">
                <label for="opsPlanSpeedLimit">速度限制(Mbps)</label>
                <input id="opsPlanSpeedLimit" type="number" min="0" value="0" />
              </div>
              <div class="field span-3">
                <label for="opsPlanDeviceLimit">设备限制</label>
                <input id="opsPlanDeviceLimit" type="number" min="0" value="0" />
              </div>
              <div class="field span-3">
                <label for="opsPlanCapacityLimit">容量限制(人数)</label>
                <input id="opsPlanCapacityLimit" type="number" min="0" value="0" />
              </div>
              <div class="field span-4">
                <label for="opsPlanResetMethod">重置策略</label>
                <input id="opsPlanResetMethod" type="number" min="0" value="0" />
              </div>
              <div class="field span-8">
                <label for="opsPlanTags">标签（逗号分隔）</label>
                <input id="opsPlanTags" placeholder="热门,推荐" />
              </div>
              <div class="field span-12">
                <label for="opsPlanContent">套餐描述</label>
                <textarea id="opsPlanContent" placeholder="可写套餐文案说明"></textarea>
              </div>
              <div class="field span-12">
                <label for="opsPlanPrices">价格设置（按示例填写，单位分）</label>
                <textarea id="opsPlanPrices">{}</textarea>
              </div>
            </div>
            <div class="ops-helper-list">
              <div class="ops-helper-item">
                <strong>编辑流程</strong>
                <p>先从左侧点击“编辑”回填已有套餐，再统一保存，避免手动抄写字段造成误差。</p>
              </div>
              <div class="ops-helper-item">
                <strong>价格字段</strong>
                <p>prices 保持 JSON 结构即可，适合一次性批量调整月付、年付和重置流量价格。</p>
              </div>
            </div>
            <pre id="opsPlanResult">等待操作...</pre>
          </section>
          </div>
        `;
      }

      function renderOpsPayments() {
        const payments = state.opsCache.payments || [];
        const methods = state.opsCache.paymentMethods || [];
        const methodOptions = methods.length
          ? methods.map((m) => `<option value="${escapeHtml(m)}">${escapeHtml(m)}</option>`).join("")
          : '<option value="">暂无可用支付方式</option>';

        return `
          <section class="ops-card">
            <h4 class="ops-card-title">支付方式列表</h4>
            <div class="actions">
              <button class="btn ghost" id="opsPaymentsReloadBtn">刷新支付方式</button>
              <span class="badge ok">共 ${payments.length} 项</span>
            </div>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>名称</th>
                    <th>支付标识</th>
                    <th>状态</th>
                    <th>手续费</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="opsPaymentsTableBody">
                  ${payments.length ? payments.map((p) => `
                    <tr>
                      <td>${p.id}</td>
                      <td>${escapeHtml(p.name || "-")}</td>
                      <td>${escapeHtml(p.payment || "-")}</td>
                      <td><span class="badge ${isOn(p.enable) ? "ok" : "warn"}">${isOn(p.enable) ? "启用" : "停用"}</span></td>
                      <td>${escapeHtml(p.handling_fee_fixed || 0)} / ${escapeHtml(p.handling_fee_percent || 0)}%</td>
                      <td>
                        <div class="actions">
                          <button class="btn ghost" data-payment-action="edit" data-id="${p.id}">编辑</button>
                          <button class="btn ghost" data-payment-action="toggle" data-id="${p.id}">${isOn(p.enable) ? "停用" : "启用"}</button>
                          <button class="btn danger" data-payment-action="drop" data-id="${p.id}">删除</button>
                        </div>
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="6" class="empty">暂无支付方式</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">支付方式编辑器</h4>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsPaymentId">编号（新建留空）</label>
                <input id="opsPaymentId" type="number" min="1" placeholder="自动创建" />
              </div>
              <div class="field span-5">
                <label for="opsPaymentName">显示名称</label>
                <input id="opsPaymentName" placeholder="例如：支付宝" />
              </div>
              <div class="field span-4">
                <label for="opsPaymentGateway">支付方式标识</label>
                <select id="opsPaymentGateway">${methodOptions}</select>
              </div>
              <div class="field span-4">
                <label for="opsPaymentIcon">图标</label>
                <input id="opsPaymentIcon" placeholder="alipay" />
              </div>
              <div class="field span-4">
                <label for="opsPaymentNotifyDomain">回调域名</label>
                <input id="opsPaymentNotifyDomain" placeholder="https://example.com" />
              </div>
              <div class="field span-2">
                <label for="opsPaymentFeeFixed">固定手续费</label>
                <input id="opsPaymentFeeFixed" type="number" min="0" value="0" />
              </div>
              <div class="field span-2">
                <label for="opsPaymentFeePercent">百分比手续费</label>
                <input id="opsPaymentFeePercent" type="number" min="0" max="100" step="0.01" value="0" />
              </div>
              <div class="field span-12">
                <label for="opsPaymentConfig">详细设置（按示例填写）</label>
                <textarea id="opsPaymentConfig">{}</textarea>
              </div>
            </div>
            <div class="actions">
              <button class="btn primary" id="opsPaymentSaveBtn">保存支付方式</button>
              <button class="btn ghost" id="opsPaymentClearBtn">清空编辑器</button>
            </div>
          </section>

          <pre id="opsPaymentResult">等待操作...</pre>
        `;
      }

      function renderOpsNotices() {
        const notices = state.opsCache.notices || [];
        return `
          <section class="ops-card">
            <h4 class="ops-card-title">公告列表</h4>
            <div class="actions">
              <button class="btn ghost" id="opsNoticesReloadBtn">刷新公告</button>
              <span class="badge ok">共 ${notices.length} 条</span>
            </div>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>标题</th>
                    <th>显示</th>
                    <th>弹窗</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="opsNoticesTableBody">
                  ${notices.length ? notices.map((n) => `
                    <tr>
                      <td>${n.id}</td>
                      <td>${escapeHtml(n.title || "-")}</td>
                      <td>${yesNo(n.show)}</td>
                      <td>${yesNo(n.popup)}</td>
                      <td>
                        <div class="actions">
                          <button class="btn ghost" data-notice-action="edit" data-id="${n.id}">编辑</button>
                          <button class="btn ghost" data-notice-action="toggle" data-id="${n.id}">${isOn(n.show) ? "隐藏" : "展示"}</button>
                          <button class="btn danger" data-notice-action="drop" data-id="${n.id}">删除</button>
                        </div>
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="5" class="empty">暂无公告</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">公告编辑器</h4>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsNoticeId">编号（新建留空）</label>
                <input id="opsNoticeId" type="number" min="1" />
              </div>
              <div class="field span-9">
                <label for="opsNoticeTitle">标题</label>
                <input id="opsNoticeTitle" />
              </div>
              <div class="field span-6">
                <label for="opsNoticeImgUrl">图片地址</label>
                <input id="opsNoticeImgUrl" />
              </div>
              <div class="field span-6">
                <label for="opsNoticeTags">标签（逗号分隔）</label>
                <input id="opsNoticeTags" placeholder="系统,公告" />
              </div>
              ${opsBoolField("opsNoticeShow", "显示公告", true, 3)}
              ${opsBoolField("opsNoticePopup", "弹窗展示", false, 3)}
              <div class="field span-12">
                <label for="opsNoticeContent">内容</label>
                <textarea id="opsNoticeContent"></textarea>
              </div>
            </div>
            <div class="actions">
              <button class="btn primary" id="opsNoticeSaveBtn">保存公告</button>
              <button class="btn ghost" id="opsNoticeClearBtn">清空编辑器</button>
            </div>
          </section>

          <pre id="opsNoticeResult">等待操作...</pre>
        `;
      }

      function renderOpsTickets() {
        const tickets = state.opsCache.tickets || [];
        return `
          <section class="ops-card">
            <h4 class="ops-card-title">工单列表</h4>
            <div class="actions">
              <button class="btn ghost" id="opsTicketsReloadBtn">刷新工单</button>
              <span class="badge ok">共 ${tickets.length} 条</span>
            </div>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>用户</th>
                    <th>状态</th>
                    <th>回复状态</th>
                    <th>更新时间</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="opsTicketsTableBody">
                  ${tickets.length ? tickets.map((t) => `
                    <tr>
                      <td>${t.id}</td>
                      <td>${escapeHtml((t.user && t.user.email) || "-")}</td>
                      <td>${escapeHtml(t.status)}</td>
                      <td>${escapeHtml(t.reply_status)}</td>
                      <td>${escapeHtml(t.updated_at || "-")}</td>
                      <td>
                        <div class="actions">
                          <button class="btn ghost" data-ticket-action="detail" data-id="${t.id}">详情</button>
                          <button class="btn ghost" data-ticket-action="reply" data-id="${t.id}">填写回复</button>
                          <button class="btn warn" data-ticket-action="close" data-id="${t.id}">关闭</button>
                        </div>
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="6" class="empty">暂无工单</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">工单处理器</h4>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsTicketId">工单编号</label>
                <input id="opsTicketId" type="number" min="1" />
              </div>
              <div class="field span-9">
                <label for="opsTicketReply">回复内容</label>
                <input id="opsTicketReply" placeholder="输入回复内容" />
              </div>
            </div>
            <div class="actions">
              <button class="btn primary" id="opsTicketReplyBtn">发送回复</button>
              <button class="btn warn" id="opsTicketCloseBtn">关闭工单</button>
              <button class="btn ghost" id="opsTicketDetailBtn">读取详情</button>
            </div>
          </section>

          <pre id="opsTicketResult">等待操作...</pre>
          <pre id="opsTicketDetail">工单详情输出...</pre>
        `;
      }

      function renderOpsCoupons() {
        const coupons = state.opsCache.coupons || [];
        const now = Math.floor(Date.now() / 1000);
        return `
          <section class="ops-card">
            <h4 class="ops-card-title">优惠券列表</h4>
            <div class="actions">
              <button class="btn ghost" id="opsCouponsReloadBtn">刷新优惠券</button>
              <span class="badge ok">共 ${coupons.length} 条</span>
            </div>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>名称</th>
                    <th>券码</th>
                    <th>类型</th>
                    <th>值</th>
                    <th>显示</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="opsCouponsTableBody">
                  ${coupons.length ? coupons.map((c) => `
                    <tr>
                      <td>${c.id}</td>
                      <td>${escapeHtml(c.name || "-")}</td>
                      <td>${escapeHtml(c.code || "-")}</td>
                      <td>${Number(c.type) === 2 ? "比例" : "金额"}</td>
                      <td>${escapeHtml(c.value || 0)}</td>
                      <td>${yesNo(c.show)}</td>
                      <td>
                        <div class="actions">
                          <button class="btn ghost" data-coupon-action="toggle" data-id="${c.id}">${isOn(c.show) ? "隐藏" : "展示"}</button>
                          <button class="btn danger" data-coupon-action="drop" data-id="${c.id}">删除</button>
                        </div>
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="7" class="empty">暂无优惠券</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">优惠券生成器</h4>
            <div class="toolbar">
              <div class="field span-4">
                <label for="opsCouponName">名称</label>
                <input id="opsCouponName" value="活动券" />
              </div>
              <div class="field span-2">
                <label for="opsCouponType">类型</label>
                <select id="opsCouponType">
                  <option value="1">金额</option>
                  <option value="2">比例</option>
                </select>
              </div>
              <div class="field span-2">
                <label for="opsCouponValue">值</label>
                <input id="opsCouponValue" type="number" value="100" />
              </div>
              <div class="field span-2">
                <label for="opsCouponGenerateCount">生成数量</label>
                <input id="opsCouponGenerateCount" type="number" min="1" max="500" value="1" />
              </div>
              <div class="field span-2">
                <label for="opsCouponCode">固定券码(可空)</label>
                <input id="opsCouponCode" />
              </div>
              <div class="field span-4">
                <label for="opsCouponStartedAt">开始时间</label>
                <input id="opsCouponStartedAt" type="datetime-local" value="${toInputDatetime(now)}" />
              </div>
              <div class="field span-4">
                <label for="opsCouponEndedAt">结束时间</label>
                <input id="opsCouponEndedAt" type="datetime-local" value="${toInputDatetime(now + 86400 * 7)}" />
              </div>
              <div class="field span-2">
                <label for="opsCouponLimitUse">总可用次数</label>
                <input id="opsCouponLimitUse" type="number" min="1" />
              </div>
              <div class="field span-2">
                <label for="opsCouponLimitUseWithUser">单用户次数</label>
                <input id="opsCouponLimitUseWithUser" type="number" min="1" />
              </div>
              <div class="field span-6">
                <label for="opsCouponLimitPlanIds">限制套餐编号（逗号分隔）</label>
                <input id="opsCouponLimitPlanIds" placeholder="1,2,3" />
              </div>
              <div class="field span-6">
                <label for="opsCouponLimitPeriod">限制周期（逗号分隔）</label>
                <input id="opsCouponLimitPeriod" placeholder="month_price,year_price" />
              </div>
            </div>
            <div class="actions">
              <button class="btn primary" id="opsCouponGenerateBtn">生成优惠券</button>
            </div>
          </section>

          <pre id="opsCouponResult">等待操作...</pre>
        `;
      }

      function renderOpsGiftCards() {
        const templates = state.opsCache.giftTemplates || [];
        const codes = state.opsCache.giftCodes || [];
        const types = state.opsCache.giftTypes || {};
        const typeData = normalizeObject(types.data || types);
        const stats = normalizeObject((state.opsCache.giftStats && state.opsCache.giftStats.data) || state.opsCache.giftStats || {});
        const totalStats = stats.total_stats || {};

        const typeOptions = Object.keys(typeData).length
          ? Object.keys(typeData).map((k) => `<option value="${escapeHtml(k)}">${escapeHtml(typeData[k])}</option>`).join("")
          : '<option value="1">类型 1</option>';

        return `
          <div class="ops-grid-3">
            <article class="ops-kpi">
              <span class="label">模板数量</span>
              <span class="num">${money(totalStats.templates_count || 0)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">兑换码总数</span>
              <span class="num">${money(totalStats.codes_count || 0)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">已使用兑换码</span>
              <span class="num">${money(totalStats.used_codes_count || 0)}</span>
            </article>
          </div>

          <section class="ops-card">
            <h4 class="ops-card-title">模板管理</h4>
            <div class="actions">
              <button class="btn ghost" id="opsGiftReloadBtn">刷新礼品卡数据</button>
              <span class="badge ok">模板 ${templates.length}</span>
            </div>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>名称</th>
                    <th>类型</th>
                    <th>状态</th>
                    <th>兑换码</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="opsGiftTemplateTableBody">
                  ${templates.length ? templates.map((t) => `
                    <tr>
                      <td>${t.id}</td>
                      <td>${escapeHtml(t.name || "-")}</td>
                      <td>${escapeHtml(t.type_name || t.type || "-")}</td>
                      <td>${isOn(t.status) ? "启用" : "停用"}</td>
                      <td>${escapeHtml(t.codes_count || 0)}</td>
                      <td>
                        <div class="actions">
                          <button class="btn ghost" data-gift-template-action="edit" data-id="${t.id}">编辑</button>
                          <button class="btn danger" data-gift-template-action="drop" data-id="${t.id}">删除</button>
                        </div>
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="6" class="empty">暂无模板</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">模板编辑器</h4>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsGiftTemplateId">模板编号（新建留空）</label>
                <input id="opsGiftTemplateId" type="number" min="1" />
              </div>
              <div class="field span-5">
                <label for="opsGiftTemplateName">模板名称</label>
                <input id="opsGiftTemplateName" />
              </div>
              <div class="field span-2">
                <label for="opsGiftTemplateType">类型</label>
                <select id="opsGiftTemplateType">${typeOptions}</select>
              </div>
              ${opsBoolField("opsGiftTemplateStatus", "启用状态", true, 2)}
              <div class="field span-12">
                <label for="opsGiftTemplateDesc">描述</label>
                <textarea id="opsGiftTemplateDesc"></textarea>
              </div>
              <div class="field span-4">
                <label for="opsGiftTemplateThemeColor">主题色</label>
                <input id="opsGiftTemplateThemeColor" value="#1890ff" />
              </div>
              <div class="field span-8">
                <label for="opsGiftTemplateRewards">奖励设置（按示例填写，必填）</label>
                <textarea id="opsGiftTemplateRewards">{"balance":1000}</textarea>
              </div>
              <div class="field span-6">
                <label for="opsGiftTemplateConditions">使用条件（按示例填写）</label>
                <textarea id="opsGiftTemplateConditions">{}</textarea>
              </div>
              <div class="field span-6">
                <label for="opsGiftTemplateLimits">使用限制（按示例填写）</label>
                <textarea id="opsGiftTemplateLimits">{}</textarea>
              </div>
            </div>
            <div class="actions">
              <button class="btn primary" id="opsGiftTemplateSaveBtn">保存模板</button>
              <button class="btn ghost" id="opsGiftTemplateClearBtn">清空编辑器</button>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">兑换码管理</h4>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsGiftCodeTemplateId">模板编号</label>
                <input id="opsGiftCodeTemplateId" type="number" min="1" />
              </div>
              <div class="field span-3">
                <label for="opsGiftCodeCount">生成数量</label>
                <input id="opsGiftCodeCount" type="number" min="1" max="10000" value="10" />
              </div>
              <div class="field span-2">
                <label for="opsGiftCodePrefix">前缀</label>
                <input id="opsGiftCodePrefix" value="GC" />
              </div>
              <div class="field span-2">
                <label for="opsGiftCodeHours">有效小时</label>
                <input id="opsGiftCodeHours" type="number" min="1" />
              </div>
              <div class="field span-2">
                <label for="opsGiftCodeMaxUsage">最大次数</label>
                <input id="opsGiftCodeMaxUsage" type="number" min="1" value="1" />
              </div>
            </div>
            <div class="actions">
              <button class="btn primary" id="opsGiftCodeGenerateBtn">生成兑换码</button>
            </div>

            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>兑换码</th>
                    <th>模板</th>
                    <th>状态</th>
                    <th>使用/上限</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="opsGiftCodeTableBody">
                  ${codes.length ? codes.map((c) => `
                    <tr>
                      <td>${c.id}</td>
                      <td>${escapeHtml(c.code)}</td>
                      <td>${escapeHtml(c.template_name || c.template_id)}</td>
                      <td>${escapeHtml(c.status_name || c.status)}</td>
                      <td>${escapeHtml(c.usage_count || 0)} / ${escapeHtml(c.max_usage || 1)}</td>
                      <td>
                        <div class="actions">
                          <button class="btn ghost" data-gift-code-action="toggle" data-id="${c.id}" data-status="${escapeHtml(c.status)}">${Number(c.status) === 3 ? "启用" : "禁用"}</button>
                          <button class="btn danger" data-gift-code-action="drop" data-id="${c.id}">删除</button>
                        </div>
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="6" class="empty">暂无兑换码</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <pre id="opsGiftResult">等待操作...</pre>
        `;
      }

      function renderOpsPlugins() {
        const plugins = state.opsCache.plugins || [];
        return `
          <section class="ops-card">
            <h4 class="ops-card-title">扩展列表</h4>
            <div class="actions">
              <button class="btn ghost" id="opsPluginsReloadBtn">刷新扩展</button>
              <span class="badge ok">共 ${plugins.length} 个扩展</span>
            </div>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>标识</th>
                    <th>名称</th>
                    <th>版本</th>
                    <th>类型</th>
                    <th>安装</th>
                    <th>启用</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody id="opsPluginsTableBody">
                  ${plugins.length ? plugins.map((p) => `
                    <tr>
                      <td>${escapeHtml(p.code)}</td>
                      <td>${escapeHtml(p.name || "-")}</td>
                      <td>${escapeHtml(p.version || "-")}</td>
                      <td>${escapeHtml(p.type || "-")}</td>
                      <td>${yesNo(p.is_installed)}</td>
                      <td>${yesNo(p.is_enabled)}</td>
                      <td>
                        <div class="actions">
                          ${!p.is_installed ? `<button class="btn ghost" data-plugin-action="install" data-code="${escapeHtml(p.code)}">安装</button>` : ""}
                          ${p.is_installed ? `<button class="btn ghost" data-plugin-action="${p.is_enabled ? "disable" : "enable"}" data-code="${escapeHtml(p.code)}">${p.is_enabled ? "禁用" : "启用"}</button>` : ""}
                          ${p.is_installed ? `<button class="btn ghost" data-plugin-action="uninstall" data-code="${escapeHtml(p.code)}">卸载</button>` : ""}
                          ${p.need_upgrade ? `<button class="btn ghost" data-plugin-action="upgrade" data-code="${escapeHtml(p.code)}">升级</button>` : ""}
                          ${p.can_be_deleted ? `<button class="btn danger" data-plugin-action="delete" data-code="${escapeHtml(p.code)}">删除</button>` : ""}
                          <button class="btn ghost" data-plugin-action="config" data-code="${escapeHtml(p.code)}">配置</button>
                        </div>
                      </td>
                    </tr>
                  `).join("") : '<tr><td colspan="7" class="empty">暂无扩展</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">扩展设置编辑器</h4>
            <div class="toolbar">
              <div class="field span-4">
                <label for="opsPluginCode">扩展标识</label>
                <input id="opsPluginCode" placeholder="plugin-code" />
              </div>
              <div class="field span-8">
                <label for="opsPluginConfig">扩展设置（按示例填写）</label>
                <textarea id="opsPluginConfig">{}</textarea>
              </div>
            </div>
            <div class="actions">
              <button class="btn primary" id="opsPluginSaveConfigBtn">保存扩展设置</button>
            </div>
          </section>

          <pre id="opsPluginResult">等待操作...</pre>
        `;
      }

      function renderOpsSystem() {
        const status = state.opsCache.systemStatus || {};
        const queue = state.opsCache.queueStats || {};
        const logsObj = state.opsCache.systemLogs || {};
        const failedObj = state.opsCache.failedJobs || {};
        const logs = Array.isArray(logsObj.data) ? logsObj.data : [];
        const failed = Array.isArray(failedObj.data) ? failedObj.data : [];

        return `
          <div class="ops-grid-3">
            <article class="ops-kpi">
              <span class="label">调度状态</span>
              <span class="num">${status._error ? "ERR" : (isOn(status.schedule) ? "ON" : "OFF")}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">队列状态</span>
              <span class="num">${queue._error ? "ERR" : (isOn(queue.status) ? "ON" : "OFF")}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">失败任务</span>
              <span class="num">${money(queue.failedJobs || failedObj.total || 0)}</span>
            </article>
          </div>

          <section class="ops-card">
            <h4 class="ops-card-title">系统状态</h4>
            <div class="actions">
              <button class="btn ghost" id="opsSystemReloadBtn">刷新系统数据</button>
            </div>
            <pre id="opsSystemStatusOutput">${escapeHtml(JSON.stringify({ status, queue }, null, 2))}</pre>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">系统日志</h4>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsSystemLogLevel">级别</label>
                <select id="opsSystemLogLevel">
                  <option value="">全部</option>
                  <option value="info">info</option>
                  <option value="warning">warning</option>
                  <option value="error">error</option>
                </select>
              </div>
              <div class="field span-6">
                <label for="opsSystemLogKeyword">关键字</label>
                <input id="opsSystemLogKeyword" placeholder="URI / title / data" />
              </div>
              <div class="field span-3">
                <label>操作</label>
                <div class="actions">
                  <button class="btn primary" id="opsSystemLoadLogsBtn">查询日志</button>
                </div>
              </div>
            </div>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>级别</th>
                    <th>标题</th>
                    <th>时间</th>
                  </tr>
                </thead>
                <tbody>
                  ${logs.length ? logs.slice(0, 50).map((l) => `
                    <tr>
                      <td>${l.id || "-"}</td>
                      <td>${escapeHtml(l.level || "-")}</td>
                      <td>${escapeHtml(l.title || l.uri || "-")}</td>
                      <td>${formatDateTime(l.created_at)}</td>
                    </tr>
                  `).join("") : '<tr><td colspan="4" class="empty">暂无日志</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">日志清理</h4>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsSystemClearDays">清理多少天前</label>
                <input id="opsSystemClearDays" type="number" min="0" max="365" value="30" />
              </div>
              <div class="field span-3">
                <label for="opsSystemClearLevel">级别</label>
                <select id="opsSystemClearLevel">
                  <option value="all">all</option>
                  <option value="info">info</option>
                  <option value="warning">warning</option>
                  <option value="error">error</option>
                </select>
              </div>
              <div class="field span-3">
                <label for="opsSystemClearLimit">单次清理数量</label>
                <input id="opsSystemClearLimit" type="number" min="100" max="10000" value="1000" />
              </div>
              <div class="field span-3">
                <label>操作</label>
                <div class="actions">
                  <button class="btn warn" id="opsSystemClearBtn">执行清理</button>
                </div>
              </div>
            </div>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">失败任务</h4>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>连接</th>
                    <th>队列</th>
                    <th>失败时间</th>
                  </tr>
                </thead>
                <tbody>
                  ${failed.length ? failed.slice(0, 30).map((j) => `
                    <tr>
                      <td>${escapeHtml(j.id || "-")}</td>
                      <td>${escapeHtml(j.connection || "-")}</td>
                      <td>${escapeHtml(j.queue || "-")}</td>
                      <td>${escapeHtml(j.failed_at || "-")}</td>
                    </tr>
                  `).join("") : '<tr><td colspan="4" class="empty">暂无失败任务</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <pre id="opsSystemResult">等待操作...</pre>
        `;
      }

      function renderOpsTraffic() {
        const statObj = normalizeObject(state.opsCache.trafficStats || {});
        const stats = normalizeObject(statObj.data || statObj);
        const logs = state.opsCache.trafficLogs || [];
        const pagination = state.opsCache.trafficPagination || {};
        const days = Number(state.opsCache.trafficDays || 30);

        return `
          <div class="ops-grid-3">
            <article class="ops-kpi">
              <span class="label">${days}天总重置</span>
              <span class="num">${money(stats.total_resets || 0)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">手动重置</span>
              <span class="num">${money(stats.manual_resets || 0)}</span>
            </article>
            <article class="ops-kpi">
              <span class="label">自动重置</span>
              <span class="num">${money(stats.auto_resets || 0)}</span>
            </article>
          </div>

          <section class="ops-card">
            <h4 class="ops-card-title">流量重置控制</h4>
            <div class="toolbar">
              <div class="field span-3">
                <label for="opsTrafficDays">统计天数</label>
                <input id="opsTrafficDays" type="number" min="1" max="365" value="${days}" />
              </div>
              <div class="field span-3">
                <label for="opsTrafficUserId">用户编号</label>
                <input id="opsTrafficUserId" type="number" min="1" />
              </div>
              <div class="field span-6">
                <label for="opsTrafficReason">手动重置原因</label>
                <input id="opsTrafficReason" placeholder="例如：申诉处理" />
              </div>
              <div class="field span-3">
                <label>重置操作</label>
                <div class="actions">
                  <button class="btn warn" id="opsTrafficResetBtn">立即重置用户</button>
                </div>
              </div>
              <div class="field span-3">
                <label>用户历史</label>
                <div class="actions">
                  <button class="btn ghost" id="opsTrafficHistoryBtn">读取重置历史</button>
                </div>
              </div>
              <div class="field span-3">
                <label>刷新统计</label>
                <div class="actions">
                  <button class="btn ghost" id="opsTrafficReloadBtn">刷新流量数据</button>
                </div>
              </div>
            </div>
            <pre id="opsTrafficHistory">用户历史输出...</pre>
          </section>

          <section class="ops-card">
            <h4 class="ops-card-title">重置日志（当前页 ${pagination.current_page || 1} / ${pagination.last_page || 1}）</h4>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>用户</th>
                    <th>类型</th>
                    <th>来源</th>
                    <th>重置时间</th>
                    <th>旧流量</th>
                    <th>新流量</th>
                  </tr>
                </thead>
                <tbody>
                  ${logs.length ? logs.map((log) => `
                    <tr>
                      <td>${log.id}</td>
                      <td>${escapeHtml(log.user_email || "-")}</td>
                      <td>${escapeHtml(log.reset_type_name || log.reset_type || "-")}</td>
                      <td>${escapeHtml(log.trigger_source_name || log.trigger_source || "-")}</td>
                      <td>${escapeHtml(log.reset_time || "-")}</td>
                      <td>${escapeHtml((log.old_traffic && log.old_traffic.formatted) || "-")}</td>
                      <td>${escapeHtml((log.new_traffic && log.new_traffic.formatted) || "-")}</td>
                    </tr>
                  `).join("") : '<tr><td colspan="7" class="empty">暂无重置日志</td></tr>'}
                </tbody>
              </table>
            </div>
          </section>

          <pre id="opsTrafficResult">等待操作...</pre>
        `;
      }

      function renderOpsModule(moduleKey) {
        if (moduleKey === "security") return renderOpsSecurity();
        if (moduleKey === "oauth") return renderOpsOAuth();
        if (moduleKey === "site") return renderOpsSite();
        if (moduleKey === "telegram") return renderOpsTelegram();
        if (moduleKey === "riskreview") return renderOpsRiskReview();
        if (moduleKey === "plans") return renderOpsPlans();
        if (moduleKey === "payments") return renderOpsPayments();
        if (moduleKey === "notices") return renderOpsNotices();
        if (moduleKey === "tickets") return renderOpsTickets();
        if (moduleKey === "coupons") return renderOpsCoupons();
        if (moduleKey === "giftcards") return renderOpsGiftCards();
        if (moduleKey === "plugins") return renderOpsPlugins();
        if (moduleKey === "system") return renderOpsSystem();
        if (moduleKey === "traffic") return renderOpsTraffic();
        return '<p class="empty">未实现模块</p>';
      }

      function bindOpsSecurity() {
        const genBtn = document.getElementById("opsGenerateSecurePathBtn");
        const saveBtn = document.getElementById("opsSecuritySaveBtn");

        if (genBtn) {
          genBtn.addEventListener("click", () => {
            setInputValue("safe_secure_path", randomSecurePath());
          });
        }

        if (saveBtn) {
          saveBtn.addEventListener("click", async () => {
            try {
              const payload = {
                safe_mode_enable: readBoolFromSelect("safe_mode_enable"),
                email_verify: readBoolFromSelect("safe_email_verify"),
                register_mode: readValue("safe_register_mode", "all"),
                email_whitelist_enable: readBoolFromSelect("safe_email_whitelist_enable"),
                email_whitelist_suffix: splitLines(readValue("safe_email_whitelist_suffix")),
	                email_gmail_limit_enable: readBoolFromSelect("safe_email_gmail_limit_enable"),
	                secure_path: readValue("safe_secure_path").trim(),
	                login_token_expire_days: Math.max(0, Math.min(3650, readInt("safe_login_token_expire_days", 365))),
	                register_limit_by_ip_enable: readBoolFromSelect("safe_register_limit_by_ip_enable"),
	                register_limit_count: readInt("safe_register_limit_count", 3),
	                register_limit_expire: readInt("safe_register_limit_expire", 60),
	                password_limit_enable: readBoolFromSelect("safe_password_limit_enable"),
	                password_limit_count: readInt("safe_password_limit_count", 5),
	                password_limit_expire: readInt("safe_password_limit_expire", 60),
	                refund_dispute_enable: readBoolFromSelect("sys_refund_dispute_enable"),
	                captcha_enable: readBoolFromSelect("safe_captcha_enable"),
	                captcha_type: readValue("safe_captcha_type", "recaptcha"),
	                recaptcha_key: readValue("safe_recaptcha_key"),
	                recaptcha_site_key: readValue("safe_recaptcha_site_key"),
	                recaptcha_v3_secret_key: readValue("safe_recaptcha_v3_secret_key"),
                recaptcha_v3_site_key: readValue("safe_recaptcha_v3_site_key"),
                recaptcha_v3_score_threshold: readFloat("safe_recap_score", 0.5),
                turnstile_secret_key: readValue("safe_turnstile_secret_key"),
                turnstile_site_key: readValue("safe_turnstile_site_key"),
                pow_enable: readBoolFromSelect("safe_pow_enable"),
                pow_auto_scale_enable: readBoolFromSelect("safe_pow_auto_scale_enable"),
                pow_difficulty: readInt("safe_pow_difficulty", 4),
                pow_auto_max_difficulty: readInt("safe_pow_auto_max_difficulty", 7),
                pow_ttl: readInt("safe_pow_ttl", 120),
                pow_seed_salt: readValue("safe_pow_seed_salt"),
                pow_base_value: readValue("safe_pow_base_value"),
                pow_require_ja3: readBoolFromSelect("safe_pow_require_ja3")
              };

              const resp = await request({ method: "POST", url: buildV2("config/save"), data: payload });
              state.opsCache.config = null;
              setPreOutput("opsSecurityResult", resp);
              showToast("安全设置已保存", "success");
              await openOpsModule("security", { force: true });
            } catch (err) {
              setPreOutput("opsSecurityResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }
      }

      function bindOpsOAuth() {
        const previewBtn = document.getElementById("opsOAuthUsePreviewBtn");
        const saveBtn = document.getElementById("opsOAuthSaveBtn");

        if (previewBtn) {
          previewBtn.addEventListener("click", () => {
            const config = state.opsCache.config || {};
            const site = config.site || {};
            const appUrl = String(normalizeConfigValue(site.app_url, settings.base_url || "")).replace(/\/+$/, "");
            const callbackPreview = appUrl
              ? `${appUrl}/api/v1/passport/oauth2/linux-do/callback`
              : "/api/v1/passport/oauth2/linux-do/callback";
            setInputValue("oauth_linux_do_redirect_uri", callbackPreview);
          });
        }

        if (saveBtn) {
          saveBtn.addEventListener("click", async () => {
            try {
              const payload = {
                oauth_linux_do_enable: readBoolFromSelect("oauth_linux_do_enable"),
                oauth_linux_do_client_id: readValue("oauth_linux_do_client_id").trim(),
                oauth_linux_do_client_secret: readValue("oauth_linux_do_client_secret").trim(),
                oauth_linux_do_redirect_uri: readValue("oauth_linux_do_redirect_uri").trim()
              };

              const resp = await request({ method: "POST", url: buildV2("config/save"), data: payload });
              state.opsCache.config = null;
              setPreOutput("opsOAuthResult", resp);
              showToast("第三方登录设置已保存", "success");
              await openOpsModule("oauth", { force: true });
            } catch (err) {
              setPreOutput("opsOAuthResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }
      }

      function bindOpsSite() {
        const saveBtn = document.getElementById("opsSiteSaveBtn");
        if (!saveBtn) return;
        saveBtn.addEventListener("click", async () => {
          try {
            const payload = {
              app_name: readValue("site_app_name"),
              app_description: readValue("site_app_description"),
              app_url: readValue("site_app_url"),
              subscribe_url: readValue("site_subscribe_url"),
              subscribe_root_domains: readValue("site_subscribe_root_domains"),
              logo: readValue("site_logo"),
              tos_url: readValue("site_tos_url"),
              force_https: readBoolFromSelect("site_force_https"),
              register_mode: readValue("site_register_mode", "all"),
              currency: readValue("site_currency"),
              currency_symbol: readValue("site_currency_symbol"),
              try_out_plan_id: readInt("site_try_out_plan_id", 0),
              try_out_hour: readInt("site_try_out_hour", 1),
              frontend_theme_sidebar: readValue("site_frontend_sidebar"),
              frontend_theme_header: readValue("site_frontend_header"),
              frontend_theme_color: readValue("site_frontend_color"),
              frontend_background_url: readValue("site_frontend_background_url"),
              subscribe_path: readValue("site_subscribe_path"),
              plan_change_enable: readBoolFromSelect("site_plan_change_enable"),
              surplus_enable: readBoolFromSelect("site_surplus_enable")
            };
            const resp = await request({ method: "POST", url: buildV2("config/save"), data: payload });
            state.opsCache.config = null;
            setPreOutput("opsSiteResult", resp);
          showToast("站点设置已保存", "success");
          await openOpsModule("site", { force: true });
        } catch (err) {
          setPreOutput("opsSiteResult", err.message || "保存失败");
          showToast(err.message || "保存失败", "error");
        }
      });
    }

      function bindOpsTelegram() {
        const saveBtn = document.getElementById("opsTelegramSaveBtn");
        const webhookBtn = document.getElementById("opsRegisterTelegramWebhookBtn");

        if (saveBtn) {
          saveBtn.addEventListener("click", async () => {
            try {
              const payload = {
                telegram_bot_enable: readBoolFromSelect("telegram_bot_enable"),
                telegram_bot_token: readValue("telegram_bot_token").trim(),
                telegram_discuss_link: readValue("telegram_discuss_link"),
                telegram_user_ticket_enable: readBoolFromSelect("telegram_user_ticket_enable"),
                telegram_notify_ticket_created: readBoolFromSelect("telegram_notify_ticket_created"),
                telegram_notify_ticket_replied: readBoolFromSelect("telegram_notify_ticket_replied"),
                telegram_notify_ticket_closed: readBoolFromSelect("telegram_notify_ticket_closed"),
                telegram_notify_payment_success: readBoolFromSelect("telegram_notify_payment_success"),
                telegram_notify_notice_published: readBoolFromSelect("telegram_notify_notice_published"),
                telegram_notify_tcping_alert: readBoolFromSelect("telegram_notify_tcping_alert"),
                telegram_notify_tcping_recover: readBoolFromSelect("telegram_notify_tcping_recover"),
                telegram_notify_refund_vote: readBoolFromSelect("telegram_notify_refund_vote"),
                telegram_notify_refund_status: readBoolFromSelect("telegram_notify_refund_status"),
                telegram_notify_user_risk_detected: readBoolFromSelect("telegram_notify_user_risk_detected"),
                telegram_notify_user_banned: readBoolFromSelect("telegram_notify_user_banned")
              };

              const resp = await request({ method: "POST", url: buildV2("config/save"), data: payload });
              state.opsCache.config = null;
              setPreOutput("opsTelegramResult", resp);
              showToast("Telegram 设置已保存", "success");
              await openOpsModule("telegram", { force: true });
            } catch (err) {
              setPreOutput("opsTelegramResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }

        if (webhookBtn) {
          webhookBtn.addEventListener("click", async () => {
            try {
              const resp = await request({ method: "POST", url: buildV2("config/setTelegramWebhook") });
              setPreOutput("opsTelegramResult", resp);
              showToast("Webhook 已注册/刷新", "success");
            } catch (err) {
              setPreOutput("opsTelegramResult", err.message || "Webhook 注册失败");
              showToast(err.message || "Webhook 注册失败", "error");
            }
          });
        }
      }

      function bindOpsRiskReview() {
        const saveBtn = document.getElementById("opsRiskReviewSaveBtn");
        const runBtn = document.getElementById("opsRiskReviewRunBtn");
        const reloadBtn = document.getElementById("opsRiskReviewReloadBtn");
        const prevBtn = document.getElementById("opsRiskReviewPrevBtn");
        const nextBtn = document.getElementById("opsRiskReviewNextBtn");
        const banPrevBtn = document.getElementById("opsBanRecordPrevBtn");
        const banNextBtn = document.getElementById("opsBanRecordNextBtn");
        const banButtons = Array.from(document.querySelectorAll(".risk-review-ban-btn"));

        if (saveBtn) {
          saveBtn.addEventListener("click", async () => {
            try {
              const payload = {
                user_risk_review_enable: readBoolFromSelect("risk_user_risk_review_enable"),
                user_risk_review_schedule_minutes: readInt("risk_user_risk_review_schedule_minutes", 30),
                user_risk_review_time_window_minutes: readInt("risk_user_risk_review_time_window_minutes", 60),
                user_risk_review_context_hours: readInt("risk_user_risk_review_context_hours", 24),
                user_risk_review_min_shared_ip_users: readInt("risk_user_risk_review_min_shared_ip_users", 2),
                user_risk_review_scan_limit: readInt("risk_user_risk_review_scan_limit", 20),
                user_risk_review_notify_cooldown_minutes: readInt("risk_user_risk_review_notify_cooldown_minutes", 60),
                user_risk_review_llm_enable: readBoolFromSelect("risk_user_risk_review_llm_enable"),
                user_risk_review_llm_base_url: readValue("risk_user_risk_review_llm_base_url").trim(),
                user_risk_review_llm_api_key: readValue("risk_user_risk_review_llm_api_key").trim(),
                user_risk_review_llm_model: readValue("risk_user_risk_review_llm_model").trim(),
                user_risk_review_llm_timeout_seconds: readInt("risk_user_risk_review_llm_timeout_seconds", 20),
                user_risk_review_llm_temperature: readFloat("risk_user_risk_review_llm_temperature", 0.2),
                telegram_notify_user_risk_detected: readBoolFromSelect("risk_telegram_notify_user_risk_detected"),
                telegram_notify_user_banned: readBoolFromSelect("risk_telegram_notify_user_banned"),
              };

              const resp = await request({ method: "POST", url: buildV2("config/save"), data: payload });
              state.opsCache.config = null;
              setPreOutput("opsRiskReviewResult", resp);
              showToast("风险审查设置已保存", "success");
              await openOpsModule("riskreview", { force: true });
            } catch (err) {
              setPreOutput("opsRiskReviewResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }

        if (runBtn) {
          runBtn.addEventListener("click", async () => {
            try {
              const resp = await request({ method: "POST", url: buildV2("risk-review/run"), data: { limit: readInt("risk_user_risk_review_scan_limit", 20) } });
              setPreOutput("opsRiskReviewResult", resp);
              showToast("风险审查已执行", "success");
              await openOpsModule("riskreview", { force: true });
            } catch (err) {
              setPreOutput("opsRiskReviewResult", err.message || "执行失败");
              showToast(err.message || "执行失败", "error");
            }
          });
        }

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("riskreview", { force: true }));
        }

        if (prevBtn) {
          prevBtn.addEventListener("click", async () => {
            state.opsCache.riskReviewPage = Math.max(1, Number(state.opsCache.riskReviewPage || 1) - 1);
            await openOpsModule("riskreview", { force: true });
          });
        }

        if (nextBtn) {
          nextBtn.addEventListener("click", async () => {
            const lastPage = Number((state.opsCache.riskReviewPagination || {}).last_page || 1);
            state.opsCache.riskReviewPage = Math.min(lastPage, Number(state.opsCache.riskReviewPage || 1) + 1);
            await openOpsModule("riskreview", { force: true });
          });
        }

        if (banPrevBtn) {
          banPrevBtn.addEventListener("click", async () => {
            state.opsCache.banRecordPage = Math.max(1, Number(state.opsCache.banRecordPage || 1) - 1);
            await openOpsModule("riskreview", { force: true });
          });
        }

        if (banNextBtn) {
          banNextBtn.addEventListener("click", async () => {
            const lastPage = Number((state.opsCache.banRecordPagination || {}).last_page || 1);
            state.opsCache.banRecordPage = Math.min(lastPage, Number(state.opsCache.banRecordPage || 1) + 1);
            await openOpsModule("riskreview", { force: true });
          });
        }

        banButtons.forEach((btn) => {
          btn.addEventListener("click", async () => {
            try {
              const userId = Number(btn.dataset.userId || 0);
              const userEmail = String(btn.dataset.userEmail || "");
              const defaultReason = String(btn.dataset.banReason || "");
              if (!userId) throw new Error("缺少用户编号");
              const reason = window.prompt(`请输入封禁原因（${userEmail || `#${userId}`}）`, defaultReason);
              if (reason === null) return;
              if (!String(reason).trim()) {
                throw new Error("封禁原因不能为空");
              }
              await request({
                method: "POST",
                url: buildV2("user/update"),
                data: {
                  id: userId,
                  banned: 1,
                  ban_reason: String(reason).trim(),
                }
              });
              showToast("用户已封禁", "success");
              await loadUsers().catch(() => {});
              await openOpsModule("riskreview", { force: true });
            } catch (err) {
              showToast(err.message || "封禁失败", "error");
            }
          });
        });
      }

      function bindOpsPlans() {
        const plans = state.opsCache.plans || [];
        const saveBtn = document.getElementById("opsPlanSaveBtn");
        const clearBtn = document.getElementById("opsPlanClearBtn");
        const reloadBtn = document.getElementById("opsPlansReloadBtn");
        const tableBody = document.getElementById("opsPlansTableBody");

        const clearForm = () => {
          setInputValue("opsPlanId", "");
          setInputValue("opsPlanName", "");
          setInputValue("opsPlanGroupId", "");
          setInputValue("opsPlanTransfer", "100");
          setInputValue("opsPlanSpeedLimit", "0");
          setInputValue("opsPlanDeviceLimit", "0");
          setInputValue("opsPlanCapacityLimit", "0");
          setInputValue("opsPlanResetMethod", "0");
          setInputValue("opsPlanTags", "");
          setInputValue("opsPlanContent", "");
          setInputValue("opsPlanPrices", "{}");
        };

        const fillFromPlan = (plan) => {
          if (!plan) return;
          setInputValue("opsPlanId", plan.id);
          setInputValue("opsPlanName", plan.name || "");
          setInputValue("opsPlanGroupId", plan.group_id || "");
          setInputValue("opsPlanTransfer", plan.transfer_enable || 100);
          setInputValue("opsPlanSpeedLimit", plan.speed_limit || 0);
          setInputValue("opsPlanDeviceLimit", plan.device_limit || 0);
          setInputValue("opsPlanCapacityLimit", plan.capacity_limit || 0);
          setInputValue("opsPlanResetMethod", plan.reset_traffic_method || 0);
          setInputValue("opsPlanTags", Array.isArray(plan.tags) ? plan.tags.join(",") : "");
          setInputValue("opsPlanContent", plan.content || "");

          let prices = {};
          if (plan.prices && typeof plan.prices === "object") {
            prices = plan.prices;
          } else {
            const fallbackKeys = ["month_price", "quarter_price", "half_year_price", "year_price", "two_year_price", "three_year_price", "onetime_price", "reset_price"];
            fallbackKeys.forEach((key) => {
              if (plan[key] != null) prices[key] = plan[key];
            });
          }
          setInputValue("opsPlanPrices", JSON.stringify(prices, null, 2));
        };

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("plans", { force: true }));
        }

        if (clearBtn) {
          clearBtn.addEventListener("click", clearForm);
        }

        if (saveBtn) {
          saveBtn.addEventListener("click", async () => {
            try {
              const id = readInt("opsPlanId", 0, true);
              const payload = {
                name: readValue("opsPlanName").trim(),
                transfer_enable: readInt("opsPlanTransfer", 100),
                group_id: readInt("opsPlanGroupId", 0, true),
                speed_limit: readInt("opsPlanSpeedLimit", 0, true),
                device_limit: readInt("opsPlanDeviceLimit", 0, true),
                capacity_limit: readInt("opsPlanCapacityLimit", 0, true),
                reset_traffic_method: readInt("opsPlanResetMethod", 0, true),
                tags: splitCsv(readValue("opsPlanTags")),
                content: readValue("opsPlanContent"),
                prices: parseJsonInput("opsPlanPrices", {})
              };
              if (!payload.name) {
                throw new Error("套餐名称不能为空");
              }
              if (id) payload.id = id;
              const resp = await request({ method: "POST", url: buildV2("plan/save"), data: payload });
              setPreOutput("opsPlanResult", resp);
              showToast("套餐已保存", "success");
              await openOpsModule("plans", { force: true });
            } catch (err) {
              setPreOutput("opsPlanResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }

        if (tableBody) {
          tableBody.addEventListener("click", async (e) => {
            const btn = e.target.closest("button[data-plan-action]");
            if (!btn) return;
            const action = btn.dataset.planAction;
            const id = Number(btn.dataset.id);
            const plan = plans.find((item) => Number(item.id) === id);
            if (!id || !plan) return;

            try {
              if (action === "edit") {
                fillFromPlan(plan);
                showToast(`已载入套餐 ${id}`, "info");
                return;
              }

              if (action === "drop") {
                if (!window.confirm(`确认删除套餐 #${id}？`)) return;
                const resp = await request({ method: "POST", url: buildV2("plan/drop"), data: { id } });
                setPreOutput("opsPlanResult", resp);
                showToast("套餐已删除", "success");
                await openOpsModule("plans", { force: true });
                return;
              }

              if (action === "toggle-show" || action === "toggle-sell") {
                const payload = {
                  id,
                  show: Number(plan.show) || 0,
                  sell: Number(plan.sell) || 0,
                  renew: Number(plan.renew) || 0
                };
                if (action === "toggle-show") payload.show = payload.show ? 0 : 1;
                if (action === "toggle-sell") payload.sell = payload.sell ? 0 : 1;
                const resp = await request({ method: "POST", url: buildV2("plan/update"), data: payload });
                setPreOutput("opsPlanResult", resp);
                showToast("套餐状态已更新", "success");
                await openOpsModule("plans", { force: true });
              }
            } catch (err) {
              setPreOutput("opsPlanResult", err.message || "操作失败");
              showToast(err.message || "操作失败", "error");
            }
          });
        }
      }

      function bindOpsPayments() {
        const payments = state.opsCache.payments || [];
        const saveBtn = document.getElementById("opsPaymentSaveBtn");
        const clearBtn = document.getElementById("opsPaymentClearBtn");
        const reloadBtn = document.getElementById("opsPaymentsReloadBtn");
        const tableBody = document.getElementById("opsPaymentsTableBody");

        const clearForm = () => {
          setInputValue("opsPaymentId", "");
          setInputValue("opsPaymentName", "");
          setInputValue("opsPaymentGateway", "");
          setInputValue("opsPaymentIcon", "");
          setInputValue("opsPaymentNotifyDomain", "");
          setInputValue("opsPaymentFeeFixed", "0");
          setInputValue("opsPaymentFeePercent", "0");
          setInputValue("opsPaymentConfig", "{}");
        };

        const fillFromPayment = (payment) => {
          if (!payment) return;
          setInputValue("opsPaymentId", payment.id);
          setInputValue("opsPaymentName", payment.name || "");
          setInputValue("opsPaymentGateway", payment.payment || "");
          setInputValue("opsPaymentIcon", payment.icon || "");
          setInputValue("opsPaymentNotifyDomain", payment.notify_domain || "");
          setInputValue("opsPaymentFeeFixed", payment.handling_fee_fixed || 0);
          setInputValue("opsPaymentFeePercent", payment.handling_fee_percent || 0);
          const config = typeof payment.config === "object"
            ? payment.config
            : parseJsonText(payment.config, {});
          setInputValue("opsPaymentConfig", JSON.stringify(config, null, 2));
        };

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("payments", { force: true }));
        }

        if (clearBtn) {
          clearBtn.addEventListener("click", clearForm);
        }

        if (saveBtn) {
          saveBtn.addEventListener("click", async () => {
            try {
              const id = readInt("opsPaymentId", 0, true);
              const payload = {
                name: readValue("opsPaymentName").trim(),
                payment: readValue("opsPaymentGateway").trim(),
                icon: readValue("opsPaymentIcon").trim(),
                config: parseJsonInput("opsPaymentConfig", {}),
                notify_domain: readValue("opsPaymentNotifyDomain").trim() || null,
                handling_fee_fixed: readInt("opsPaymentFeeFixed", 0, true),
                handling_fee_percent: readFloat("opsPaymentFeePercent", 0, true)
              };
              if (!payload.name || !payload.payment) {
                throw new Error("名称和支付方式标识不能为空");
              }
              if (id) payload.id = id;
              const resp = await request({ method: "POST", url: buildV2("payment/save"), data: payload });
              setPreOutput("opsPaymentResult", resp);
              showToast("支付方式已保存", "success");
              await openOpsModule("payments", { force: true });
            } catch (err) {
              setPreOutput("opsPaymentResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }

        if (tableBody) {
          tableBody.addEventListener("click", async (e) => {
            const btn = e.target.closest("button[data-payment-action]");
            if (!btn) return;
            const action = btn.dataset.paymentAction;
            const id = Number(btn.dataset.id);
            const payment = payments.find((item) => Number(item.id) === id);
            if (!id || !payment) return;
            try {
              if (action === "edit") {
                fillFromPayment(payment);
                showToast(`已载入支付方式 #${id}`, "info");
                return;
              }
              if (action === "toggle") {
                const resp = await request({ method: "POST", url: buildV2("payment/show"), data: { id } });
                setPreOutput("opsPaymentResult", resp);
                showToast("支付方式状态已切换", "success");
                await openOpsModule("payments", { force: true });
                return;
              }
              if (action === "drop") {
                if (!window.confirm(`确认删除支付方式 #${id}？`)) return;
                const resp = await request({ method: "POST", url: buildV2("payment/drop"), data: { id } });
                setPreOutput("opsPaymentResult", resp);
                showToast("支付方式已删除", "success");
                await openOpsModule("payments", { force: true });
              }
            } catch (err) {
              setPreOutput("opsPaymentResult", err.message || "操作失败");
              showToast(err.message || "操作失败", "error");
            }
          });
        }
      }

      function bindOpsNotices() {
        const notices = state.opsCache.notices || [];
        const saveBtn = document.getElementById("opsNoticeSaveBtn");
        const clearBtn = document.getElementById("opsNoticeClearBtn");
        const reloadBtn = document.getElementById("opsNoticesReloadBtn");
        const tableBody = document.getElementById("opsNoticesTableBody");

        const clearForm = () => {
          setInputValue("opsNoticeId", "");
          setInputValue("opsNoticeTitle", "");
          setInputValue("opsNoticeContent", "");
          setInputValue("opsNoticeImgUrl", "");
          setInputValue("opsNoticeTags", "");
          setInputValue("opsNoticeShow", "1");
          setInputValue("opsNoticePopup", "0");
        };

        const fillForm = (notice) => {
          if (!notice) return;
          setInputValue("opsNoticeId", notice.id);
          setInputValue("opsNoticeTitle", notice.title || "");
          setInputValue("opsNoticeContent", notice.content || "");
          setInputValue("opsNoticeImgUrl", notice.img_url || "");
          setInputValue("opsNoticeTags", Array.isArray(notice.tags) ? notice.tags.join(",") : "");
          setInputValue("opsNoticeShow", isOn(notice.show) ? "1" : "0");
          setInputValue("opsNoticePopup", isOn(notice.popup) ? "1" : "0");
        };

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("notices", { force: true }));
        }

        if (clearBtn) {
          clearBtn.addEventListener("click", clearForm);
        }

        if (saveBtn) {
          saveBtn.addEventListener("click", async () => {
            try {
              const id = readInt("opsNoticeId", 0, true);
              const payload = {
                title: readValue("opsNoticeTitle").trim(),
                content: readValue("opsNoticeContent").trim(),
                img_url: readValue("opsNoticeImgUrl").trim() || null,
                tags: splitCsv(readValue("opsNoticeTags")),
                show: readBoolFromSelect("opsNoticeShow"),
                popup: readBoolFromSelect("opsNoticePopup")
              };
              if (!payload.title || !payload.content) {
                throw new Error("标题和内容不能为空");
              }
              if (id) payload.id = id;
              const resp = await request({ method: "POST", url: buildV2("notice/save"), data: payload });
              setPreOutput("opsNoticeResult", resp);
              showToast("公告已保存", "success");
              await openOpsModule("notices", { force: true });
            } catch (err) {
              setPreOutput("opsNoticeResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }

        if (tableBody) {
          tableBody.addEventListener("click", async (e) => {
            const btn = e.target.closest("button[data-notice-action]");
            if (!btn) return;
            const action = btn.dataset.noticeAction;
            const id = Number(btn.dataset.id);
            const notice = notices.find((n) => Number(n.id) === id);
            if (!id || !notice) return;

            try {
              if (action === "edit") {
                fillForm(notice);
                return;
              }
              if (action === "toggle") {
                const resp = await request({ method: "POST", url: buildV2("notice/show"), data: { id } });
                setPreOutput("opsNoticeResult", resp);
                showToast("公告显示状态已切换", "success");
                await openOpsModule("notices", { force: true });
                return;
              }
              if (action === "drop") {
                if (!window.confirm(`确认删除公告 #${id}？`)) return;
                const resp = await request({ method: "POST", url: buildV2("notice/drop"), data: { id } });
                setPreOutput("opsNoticeResult", resp);
                showToast("公告已删除", "success");
                await openOpsModule("notices", { force: true });
              }
            } catch (err) {
              setPreOutput("opsNoticeResult", err.message || "操作失败");
              showToast(err.message || "操作失败", "error");
            }
          });
        }
      }

      function bindOpsTickets() {
        const reloadBtn = document.getElementById("opsTicketsReloadBtn");
        const tableBody = document.getElementById("opsTicketsTableBody");
        const replyBtn = document.getElementById("opsTicketReplyBtn");
        const closeBtn = document.getElementById("opsTicketCloseBtn");
        const detailBtn = document.getElementById("opsTicketDetailBtn");

        const fetchDetail = async () => {
          const id = readInt("opsTicketId", 0);
          if (!id) throw new Error("请输入正确的工单编号");
          const detail = await request({ method: "GET", url: `${buildV2("ticket/fetch")}?id=${id}` });
          setPreOutput("opsTicketDetail", detail);
        };

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("tickets", { force: true }));
        }

        if (tableBody) {
          tableBody.addEventListener("click", async (e) => {
            const btn = e.target.closest("button[data-ticket-action]");
            if (!btn) return;
            const action = btn.dataset.ticketAction;
            const id = Number(btn.dataset.id);
            if (!id) return;

            try {
              if (action === "detail") {
                setInputValue("opsTicketId", id);
                await fetchDetail();
                return;
              }
              if (action === "reply") {
                setInputValue("opsTicketId", id);
                showToast(`已选择工单 #${id}`, "info");
                return;
              }
              if (action === "close") {
                if (!window.confirm(`确认关闭工单 #${id}？`)) return;
                const resp = await request({ method: "POST", url: buildV2("ticket/close"), data: { id } });
                setPreOutput("opsTicketResult", resp);
                showToast("工单已关闭", "success");
                await openOpsModule("tickets", { force: true });
              }
            } catch (err) {
              setPreOutput("opsTicketResult", err.message || "操作失败");
              showToast(err.message || "操作失败", "error");
            }
          });
        }

        if (replyBtn) {
          replyBtn.addEventListener("click", async () => {
            try {
              const id = readInt("opsTicketId", 0);
              const message = readValue("opsTicketReply").trim();
              if (!id || !message) {
                throw new Error("工单编号和回复内容不能为空");
              }
              const resp = await request({ method: "POST", url: buildV2("ticket/reply"), data: { id, message } });
              setPreOutput("opsTicketResult", resp);
              showToast("工单回复已发送", "success");
              await openOpsModule("tickets", { force: true });
            } catch (err) {
              setPreOutput("opsTicketResult", err.message || "回复失败");
              showToast(err.message || "回复失败", "error");
            }
          });
        }

        if (closeBtn) {
          closeBtn.addEventListener("click", async () => {
            try {
              const id = readInt("opsTicketId", 0);
              if (!id) throw new Error("请输入正确的工单编号");
              if (!window.confirm(`确认关闭工单 #${id}？`)) return;
              const resp = await request({ method: "POST", url: buildV2("ticket/close"), data: { id } });
              setPreOutput("opsTicketResult", resp);
              showToast("工单已关闭", "success");
              await openOpsModule("tickets", { force: true });
            } catch (err) {
              setPreOutput("opsTicketResult", err.message || "关闭失败");
              showToast(err.message || "关闭失败", "error");
            }
          });
        }

        if (detailBtn) {
          detailBtn.addEventListener("click", async () => {
            try {
              await fetchDetail();
            } catch (err) {
              setPreOutput("opsTicketDetail", err.message || "读取失败");
              showToast(err.message || "读取失败", "error");
            }
          });
        }
      }

      function bindOpsCoupons() {
        const reloadBtn = document.getElementById("opsCouponsReloadBtn");
        const generateBtn = document.getElementById("opsCouponGenerateBtn");
        const tableBody = document.getElementById("opsCouponsTableBody");

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("coupons", { force: true }));
        }

        if (generateBtn) {
          generateBtn.addEventListener("click", async () => {
            try {
              const payload = {
                generate_count: readInt("opsCouponGenerateCount", 1),
                name: readValue("opsCouponName").trim(),
                type: readValue("opsCouponType"),
                value: readInt("opsCouponValue", 0),
                started_at: toUnixFromInput(readValue("opsCouponStartedAt")),
                ended_at: toUnixFromInput(readValue("opsCouponEndedAt")),
                limit_use: readInt("opsCouponLimitUse", 0, true),
                limit_use_with_user: readInt("opsCouponLimitUseWithUser", 0, true),
                limit_plan_ids: splitCsv(readValue("opsCouponLimitPlanIds")).map((v) => Number(v)).filter((v) => Number.isFinite(v) && v > 0),
                limit_period: splitCsv(readValue("opsCouponLimitPeriod"))
              };
              const fixedCode = readValue("opsCouponCode").trim();
              if (fixedCode) payload.code = fixedCode;
              if (!payload.name || !payload.type || !payload.value || !payload.started_at || !payload.ended_at) {
                throw new Error("请完整填写名称、类型、值、开始和结束时间");
              }
              const resp = await request({ method: "POST", url: buildV2("coupon/generate"), data: payload });
              setPreOutput("opsCouponResult", resp);
              showToast("优惠券生成成功", "success");
              await openOpsModule("coupons", { force: true });
            } catch (err) {
              setPreOutput("opsCouponResult", err.message || "生成失败");
              showToast(err.message || "生成失败", "error");
            }
          });
        }

        if (tableBody) {
          tableBody.addEventListener("click", async (e) => {
            const btn = e.target.closest("button[data-coupon-action]");
            if (!btn) return;
            const action = btn.dataset.couponAction;
            const id = Number(btn.dataset.id);
            if (!id) return;

            try {
              if (action === "toggle") {
                const resp = await request({ method: "POST", url: buildV2("coupon/show"), data: { id } });
                setPreOutput("opsCouponResult", resp);
                showToast("优惠券显示状态已更新", "success");
                await openOpsModule("coupons", { force: true });
                return;
              }
              if (action === "drop") {
                if (!window.confirm(`确认删除优惠券 #${id}？`)) return;
                const resp = await request({ method: "POST", url: buildV2("coupon/drop"), data: { id } });
                setPreOutput("opsCouponResult", resp);
                showToast("优惠券已删除", "success");
                await openOpsModule("coupons", { force: true });
              }
            } catch (err) {
              setPreOutput("opsCouponResult", err.message || "操作失败");
              showToast(err.message || "操作失败", "error");
            }
          });
        }
      }

      function bindOpsGiftCards() {
        const templates = state.opsCache.giftTemplates || [];
        const saveTemplateBtn = document.getElementById("opsGiftTemplateSaveBtn");
        const clearTemplateBtn = document.getElementById("opsGiftTemplateClearBtn");
        const generateCodeBtn = document.getElementById("opsGiftCodeGenerateBtn");
        const reloadBtn = document.getElementById("opsGiftReloadBtn");
        const templateBody = document.getElementById("opsGiftTemplateTableBody");
        const codeBody = document.getElementById("opsGiftCodeTableBody");

        const clearTemplateForm = () => {
          setInputValue("opsGiftTemplateId", "");
          setInputValue("opsGiftTemplateName", "");
          setInputValue("opsGiftTemplateDesc", "");
          setInputValue("opsGiftTemplateThemeColor", "#1890ff");
          setInputValue("opsGiftTemplateRewards", '{"balance":1000}');
          setInputValue("opsGiftTemplateConditions", "{}");
          setInputValue("opsGiftTemplateLimits", "{}");
          setInputValue("opsGiftTemplateStatus", "1");
        };

        const fillTemplateForm = (template) => {
          if (!template) return;
          setInputValue("opsGiftTemplateId", template.id);
          setInputValue("opsGiftTemplateName", template.name || "");
          setInputValue("opsGiftTemplateType", template.type || "");
          setInputValue("opsGiftTemplateStatus", isOn(template.status) ? "1" : "0");
          setInputValue("opsGiftTemplateDesc", template.description || "");
          setInputValue("opsGiftTemplateThemeColor", template.theme_color || "#1890ff");
          setInputValue("opsGiftTemplateRewards", JSON.stringify(template.rewards || {}, null, 2));
          setInputValue("opsGiftTemplateConditions", JSON.stringify(template.conditions || {}, null, 2));
          setInputValue("opsGiftTemplateLimits", JSON.stringify(template.limits || {}, null, 2));
        };

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("giftcards", { force: true }));
        }

        if (clearTemplateBtn) {
          clearTemplateBtn.addEventListener("click", clearTemplateForm);
        }

        if (saveTemplateBtn) {
          saveTemplateBtn.addEventListener("click", async () => {
            try {
              const id = readInt("opsGiftTemplateId", 0, true);
              const payload = {
                name: readValue("opsGiftTemplateName").trim(),
                type: readInt("opsGiftTemplateType", 1),
                status: readBoolFromSelect("opsGiftTemplateStatus"),
                description: readValue("opsGiftTemplateDesc"),
                theme_color: readValue("opsGiftTemplateThemeColor") || "#1890ff",
                rewards: parseJsonInput("opsGiftTemplateRewards", {}),
                conditions: parseJsonInput("opsGiftTemplateConditions", {}),
                limits: parseJsonInput("opsGiftTemplateLimits", {})
              };
              if (!payload.name) throw new Error("模板名称不能为空");
              let resp;
              if (id) {
                payload.id = id;
                resp = await request({ method: "POST", url: buildV2("gift-card/update-template"), data: payload });
              } else {
                resp = await request({ method: "POST", url: buildV2("gift-card/create-template"), data: payload });
              }
              setPreOutput("opsGiftResult", resp);
              showToast("礼品卡模板已保存", "success");
              await openOpsModule("giftcards", { force: true });
            } catch (err) {
              setPreOutput("opsGiftResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }

        if (generateCodeBtn) {
          generateCodeBtn.addEventListener("click", async () => {
            try {
              const payload = {
                template_id: readInt("opsGiftCodeTemplateId", 0),
                count: readInt("opsGiftCodeCount", 10),
                prefix: readValue("opsGiftCodePrefix", "GC").trim() || "GC",
                max_usage: readInt("opsGiftCodeMaxUsage", 1)
              };
              const expiresHours = readInt("opsGiftCodeHours", 0, true);
              if (expiresHours) payload.expires_hours = expiresHours;
              if (!payload.template_id || !payload.count) throw new Error("模板编号和数量不能为空");
              const resp = await request({ method: "POST", url: buildV2("gift-card/generate-codes"), data: payload });
              setPreOutput("opsGiftResult", resp);
              showToast("兑换码生成成功", "success");
              await openOpsModule("giftcards", { force: true });
            } catch (err) {
              setPreOutput("opsGiftResult", err.message || "生成失败");
              showToast(err.message || "生成失败", "error");
            }
          });
        }

        if (templateBody) {
          templateBody.addEventListener("click", async (e) => {
            const btn = e.target.closest("button[data-gift-template-action]");
            if (!btn) return;
            const action = btn.dataset.giftTemplateAction;
            const id = Number(btn.dataset.id);
            const template = templates.find((t) => Number(t.id) === id);
            if (!id || !template) return;
            try {
              if (action === "edit") {
                fillTemplateForm(template);
                return;
              }
              if (action === "drop") {
                if (!window.confirm(`确认删除模板 #${id}？`)) return;
                const resp = await request({ method: "POST", url: buildV2("gift-card/delete-template"), data: { id } });
                setPreOutput("opsGiftResult", resp);
                showToast("模板已删除", "success");
                await openOpsModule("giftcards", { force: true });
              }
            } catch (err) {
              setPreOutput("opsGiftResult", err.message || "操作失败");
              showToast(err.message || "操作失败", "error");
            }
          });
        }

        if (codeBody) {
          codeBody.addEventListener("click", async (e) => {
            const btn = e.target.closest("button[data-gift-code-action]");
            if (!btn) return;
            const action = btn.dataset.giftCodeAction;
            const id = Number(btn.dataset.id);
            if (!id) return;
            try {
              if (action === "toggle") {
                const currentStatus = Number(btn.dataset.status);
                const toggleAction = currentStatus === 3 ? "enable" : "disable";
                const resp = await request({ method: "POST", url: buildV2("gift-card/toggle-code"), data: { id, action: toggleAction } });
                setPreOutput("opsGiftResult", resp);
                showToast("兑换码状态已更新", "success");
                await openOpsModule("giftcards", { force: true });
                return;
              }
              if (action === "drop") {
                if (!window.confirm(`确认删除兑换码 #${id}？`)) return;
                const resp = await request({ method: "POST", url: buildV2("gift-card/delete-code"), data: { id } });
                setPreOutput("opsGiftResult", resp);
                showToast("兑换码已删除", "success");
                await openOpsModule("giftcards", { force: true });
              }
            } catch (err) {
              setPreOutput("opsGiftResult", err.message || "操作失败");
              showToast(err.message || "操作失败", "error");
            }
          });
        }
      }

      function bindOpsPlugins() {
        const plugins = state.opsCache.plugins || [];
        const reloadBtn = document.getElementById("opsPluginsReloadBtn");
        const tableBody = document.getElementById("opsPluginsTableBody");
        const saveConfigBtn = document.getElementById("opsPluginSaveConfigBtn");

        const actionEndpointMap = {
          install: "plugin/install",
          uninstall: "plugin/uninstall",
          enable: "plugin/enable",
          disable: "plugin/disable",
          upgrade: "plugin/upgrade",
          delete: "plugin/delete"
        };

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("plugins", { force: true }));
        }

        if (saveConfigBtn) {
          saveConfigBtn.addEventListener("click", async () => {
            try {
              const code = readValue("opsPluginCode").trim();
              if (!code) throw new Error("请输入扩展标识");
              const config = parseJsonInput("opsPluginConfig", {});
              const resp = await request({ method: "POST", url: buildV2("plugin/config"), data: { code, config } });
              setPreOutput("opsPluginResult", resp);
              showToast("扩展设置已保存", "success");
              await openOpsModule("plugins", { force: true });
            } catch (err) {
              setPreOutput("opsPluginResult", err.message || "保存失败");
              showToast(err.message || "保存失败", "error");
            }
          });
        }

        if (tableBody) {
          tableBody.addEventListener("click", async (e) => {
            const btn = e.target.closest("button[data-plugin-action]");
            if (!btn) return;
            const action = btn.dataset.pluginAction;
            const code = btn.dataset.code;
            if (!code) return;

            try {
              if (action === "config") {
                const plugin = plugins.find((p) => String(p.code) === String(code));
                setInputValue("opsPluginCode", code);
                setInputValue("opsPluginConfig", JSON.stringify((plugin && plugin.config) || {}, null, 2));
                showToast(`已加载 ${code} 配置`, "info");
                return;
              }

              const endpoint = actionEndpointMap[action];
              if (!endpoint) return;
              const resp = await request({ method: "POST", url: buildV2(endpoint), data: { code } });
              setPreOutput("opsPluginResult", resp);
              showToast(`扩展 ${code} 操作成功`, "success");
              await openOpsModule("plugins", { force: true });
            } catch (err) {
              setPreOutput("opsPluginResult", err.message || "操作失败");
              showToast(err.message || "操作失败", "error");
            }
          });
        }
      }

      function bindOpsSystem() {
        const reloadBtn = document.getElementById("opsSystemReloadBtn");
        const loadLogsBtn = document.getElementById("opsSystemLoadLogsBtn");
        const clearBtn = document.getElementById("opsSystemClearBtn");

        if (reloadBtn) {
          reloadBtn.addEventListener("click", () => openOpsModule("system", { force: true }));
        }

        if (loadLogsBtn) {
          loadLogsBtn.addEventListener("click", async () => {
            try {
              const level = readValue("opsSystemLogLevel");
              const keyword = readValue("opsSystemLogKeyword").trim();
              const query = new URLSearchParams({ current: "1", page_size: "20" });
              if (level) query.set("level", level);
              if (keyword) query.set("keyword", keyword);
              const body = await request({ method: "GET", url: `${buildV2("system/getSystemLog")}?${query.toString()}` });
              state.opsCache.systemLogs = normalizeObject(body);
              dom.opsWorkspace.innerHTML = renderOpsSystem();
              bindOpsSystem();
            } catch (err) {
              setPreOutput("opsSystemResult", err.message || "读取失败");
              showToast(err.message || "读取失败", "error");
            }
          });
        }

        if (clearBtn) {
          clearBtn.addEventListener("click", async () => {
            try {
              if (!window.confirm("确认执行日志清理？")) return;
              const payload = {
                days: readInt("opsSystemClearDays", 30),
                level: readValue("opsSystemClearLevel", "all"),
                limit: readInt("opsSystemClearLimit", 1000)
              };
              const resp = await request({ method: "POST", url: buildV2("system/clearSystemLog"), data: payload });
              setPreOutput("opsSystemResult", resp);
              showToast("日志清理完成", "success");
              await openOpsModule("system", { force: true });
            } catch (err) {
              setPreOutput("opsSystemResult", err.message || "清理失败");
              showToast(err.message || "清理失败", "error");
            }
          });
        }
      }

      function bindOpsTraffic() {
        const reloadBtn = document.getElementById("opsTrafficReloadBtn");
        const resetBtn = document.getElementById("opsTrafficResetBtn");
        const historyBtn = document.getElementById("opsTrafficHistoryBtn");

        if (reloadBtn) {
          reloadBtn.addEventListener("click", async () => {
            state.opsCache.trafficDays = readInt("opsTrafficDays", 30);
            await openOpsModule("traffic", { force: true });
          });
        }

        if (resetBtn) {
          resetBtn.addEventListener("click", async () => {
            try {
              const userId = readInt("opsTrafficUserId", 0);
              const reason = readValue("opsTrafficReason").trim();
              if (!userId) throw new Error("请输入用户编号");
              const payload = { user_id: userId };
              if (reason) payload.reason = reason;
              const resp = await request({ method: "POST", url: buildV2("traffic-reset/reset-user"), data: payload });
              setPreOutput("opsTrafficResult", resp);
              showToast("用户流量已重置", "success");
              await openOpsModule("traffic", { force: true });
            } catch (err) {
              setPreOutput("opsTrafficResult", err.message || "重置失败");
              showToast(err.message || "重置失败", "error");
            }
          });
        }

        if (historyBtn) {
          historyBtn.addEventListener("click", async () => {
            try {
              const userId = readInt("opsTrafficUserId", 0);
              if (!userId) throw new Error("请输入用户编号");
              const history = await request({ method: "GET", url: `${buildV2(`traffic-reset/user/${userId}/history`)}?limit=20` });
              setPreOutput("opsTrafficHistory", history);
              showToast("历史记录已加载", "success");
            } catch (err) {
              setPreOutput("opsTrafficHistory", err.message || "读取失败");
              showToast(err.message || "读取失败", "error");
            }
          });
        }
      }

      function bindOpsModule(moduleKey) {
        if (moduleKey === "security") return bindOpsSecurity();
        if (moduleKey === "oauth") return bindOpsOAuth();
        if (moduleKey === "site") return bindOpsSite();
        if (moduleKey === "telegram") return bindOpsTelegram();
        if (moduleKey === "riskreview") return bindOpsRiskReview();
        if (moduleKey === "plans") return bindOpsPlans();
        if (moduleKey === "payments") return bindOpsPayments();
        if (moduleKey === "notices") return bindOpsNotices();
        if (moduleKey === "tickets") return bindOpsTickets();
        if (moduleKey === "coupons") return bindOpsCoupons();
        if (moduleKey === "giftcards") return bindOpsGiftCards();
        if (moduleKey === "plugins") return bindOpsPlugins();
        if (moduleKey === "system") return bindOpsSystem();
        if (moduleKey === "traffic") return bindOpsTraffic();
      }

      async function openOpsModule(moduleKey, options = {}) {
        const key = OPS_MODULE_MAP[moduleKey] ? moduleKey : OPS_MODULES[0].key;
        state.activeOpsModule = key;
        const currentMenuQuery = readValue("primaryOpsMenuSearch", "");
        if (currentMenuQuery && !moduleMatchesSearch(OPS_MODULE_MAP[key], currentMenuQuery) && dom.primaryOpsMenuSearch) {
          dom.primaryOpsMenuSearch.value = "";
        }
        renderPrimaryOpsMenu(readValue("primaryOpsMenuSearch", ""));
        setOpsModuleHeader(key);
        if (state.activeTab === "ops") {
          setActiveMenuButton("ops", key);
        }

        if (!state.token) {
          dom.opsWorkspace.innerHTML = '<p class="empty">请先登录管理员账号后加载模块。</p>';
          return;
        }

        dom.opsWorkspace.innerHTML = `<p class="empty">正在加载「${escapeHtml(OPS_MODULE_MAP[key].name)}」...</p>`;
        try {
          await loadOpsModuleData(key, Boolean(options.force));
          dom.opsWorkspace.innerHTML = renderOpsModule(key);
          bindOpsModule(key);
        } catch (err) {
          dom.opsWorkspace.innerHTML = `
            <section class="ops-card">
              <h4 class="ops-card-title">模块加载失败</h4>
              <p class="empty">${escapeHtml(err.message || "未知错误")}</p>
              <div class="actions">
                <button class="btn ghost" id="opsRetryModuleBtn">重试加载</button>
              </div>
            </section>
          `;
          const retryBtn = document.getElementById("opsRetryModuleBtn");
          if (retryBtn) {
            retryBtn.addEventListener("click", () => openOpsModule(key, { force: true }));
          }
          showToast(err.message || "模块加载失败", "error");
        }
      }

      function bindTabs() {
        dom.menuTabs.addEventListener("click", (e) => {
          const btn = e.target.closest(".menu-btn");
          if (!btn) return;
          const tab = btn.dataset.tab;
          const moduleKey = btn.dataset.opsModule || "";
          state.activeTab = tab || "overview";
          if (tab === "ops" && moduleKey) {
            state.activeOpsModule = moduleKey;
          }
          setActiveMenuButton(state.activeTab, state.activeTab === "ops" ? state.activeOpsModule : "");
          dom.panels.forEach((panel) => panel.classList.toggle("active", panel.dataset.panel === tab));
          if (tab === "overview" && state.token) {
            loadOverview(false).catch((err) => {
              showToast(err.message || "监控模块加载失败", "error");
            });
          } else {
            clearOverviewRefresh();
          }
          if (tab === "ops" && state.token) {
            openOpsModule(moduleKey || state.activeOpsModule || "security").catch((err) => {
              showToast(err.message || "模块加载失败", "error");
            });
          }
        });
      }

      async function loginWithPassword() {
        const email = dom.loginEmailInput.value.trim();
        const password = dom.loginPasswordInput.value;
        if (!email || !password) {
          setLoginError("请输入邮箱和密码");
          return;
        }

        try {
          dom.loginSubmitBtn.disabled = true;
          const cfg = await loadLoginCommConfig().catch(() => ({}));
          const payload = { email, password };

          if (isCaptchaEnabled(cfg)) {
            const provider = state.loginCaptchaProvider || getCaptchaProvider(cfg);
            if (provider === "turnstile") {
              if (!state.loginCaptchaToken) {
                throw new Error("请先完成 Turnstile 验证");
              }
              payload.turnstile_token = state.loginCaptchaToken;
            } else if (provider === "recaptcha") {
              if (!state.loginCaptchaToken) {
                throw new Error("请先完成 reCAPTCHA 验证");
              }
              payload.recaptcha_data = state.loginCaptchaToken;
            } else if (provider === "recaptcha-v3") {
              const siteKey = String(cfg.recaptcha_v3_site_key || "").trim();
              if (!siteKey) {
                throw new Error("reCAPTCHA v3 站点密钥未配置");
              }
              if (!state.loginCaptchaV3Ready) {
                await ensureRecaptchaLoaded("recaptcha-v3", siteKey);
                state.loginCaptchaV3Ready = true;
              }
              const token = await new Promise((resolve, reject) => {
                if (!window.grecaptcha || typeof window.grecaptcha.ready !== "function" || typeof window.grecaptcha.execute !== "function") {
                  reject(new Error("reCAPTCHA v3 未就绪"));
                  return;
                }
                window.grecaptcha.ready(() => {
                  window.grecaptcha.execute(siteKey, { action: "login" })
                    .then((t) => resolve(String(t || "").trim()))
                    .catch(() => reject(new Error("reCAPTCHA v3 执行失败")));
                });
              });
              if (!token) {
                throw new Error("reCAPTCHA v3 token 为空");
              }
              payload.recaptcha_v3_token = token;
            }
          }

          if (isPowEnabled(cfg)) {
            const proof = await ensureLoginPowProof();
            if (!proof) {
              throw new Error("防刷验证未就绪，请稍后重试");
            }
            payload.pow_id = proof.id;
            payload.pow_nonce = proof.nonce;
            payload.pow_hash = proof.hash;
            payload.pow_token = proof.token;
          }

          const body = await request({
            method: "POST",
            url: buildV1("passport/auth/login"),
            data: payload
          });
          const data = toData(body);
          const authData = data && data.auth_data;
          if (!authData) {
            throw new Error("登录返回信息不完整，请检查验证码和防刷验证设置");
          }
          updateToken(authData);
          persistSharedLegacyToken((data && data.token) || "");
          await syncAdminSession();
          showConsoleView();
          showToast("登录成功", "success");
          await loadAll();
        } catch (err) {
          setLoginError(err.message || "登录失败");
          // Captcha/PoW token is usually single-use; on failure always reset and prepare again.
          resetLoginCaptchaWidget();
          state.loginPowProof = null;
          prepareLoginSecurity().catch(() => {});
        } finally {
          updateLoginSubmitAvailability();
        }
      }

      async function loadOverview(manual = false) {
        clearOverviewRefresh();
        updateOverviewCountdown();

        if (dom.refreshOverviewBtn) {
          dom.refreshOverviewBtn.disabled = true;
          dom.refreshOverviewBtn.textContent = manual ? "刷新中..." : "同步中...";
        }
        setOverviewStatus(manual ? "手动拉取最新统计" : "正在同步后台总览", "neutral");
        if (dom.overviewHint) {
          dom.overviewHint.textContent = "正在同步后台 overview 统计...";
        }

        const body = await request({ method: "GET", url: buildV2("stat/getOverride") });
        const stats = (body && body.data) ? body.data : toData(body);
        document.getElementById("kpiMonthIncome").textContent = money(stats.month_income);
        document.getElementById("kpiMonthUsers").textContent = formatCompactNumber(stats.month_register_total);
        document.getElementById("kpiPendingTicket").textContent = formatCompactNumber(stats.ticket_pending_total);
        document.getElementById("kpiOnlineUsers").textContent = formatCompactNumber(stats.online_users);

        if (dom.commandCenterDashboard) {
          dom.commandCenterDashboard.innerHTML = buildOverviewLanding({
            adminEmail: state.currentAdmin || dom.currentAdminText.value || "-",
            securePath,
            stats,
          });
        }
        if (dom.commandCenterUpdatedAt) {
          dom.commandCenterUpdatedAt.textContent = formatAnyTimestamp(Math.floor(Date.now() / 1000));
        }
        setOverviewStatus("后台 overview 已同步", "ok");
        if (dom.overviewHint) {
          dom.overviewHint.textContent = "基础统计已同步。完整监控请使用上方“独立监控大屏”入口。";
        }
        scheduleOverviewRefresh(30);

        if (dom.refreshOverviewBtn) {
          dom.refreshOverviewBtn.disabled = false;
          dom.refreshOverviewBtn.textContent = "立即刷新";
        }
      }

      function renderUsers() {
        if (!state.users.length) {
          dom.usersTableBody.innerHTML = '<tr><td colspan="8" class="empty">当前页无用户数据。</td></tr>';
          return;
        }

        dom.usersTableBody.innerHTML = state.users.map((u) => {
          const banned = Number(u.banned) === 1;
          const admin = Number(u.is_admin) === 1;
          return `
            <tr>
              <td>${u.id}</td>
              <td>${escapeHtml(u.email)}</td>
              <td>${escapeHtml((u.plan && u.plan.name) || "-")}</td>
              <td>${money(u.balance)}</td>
              <td><span class="badge ${banned ? "warn" : "ok"}">${banned ? "已封禁" : "正常"}</span></td>
              <td>${escapeHtml((u.ban_reason || "").trim() || "-")}</td>
              <td>${admin ? "是" : "否"}</td>
              <td>
                <div class="actions">
                  <button class="btn ${banned ? "ghost" : "warn"}" data-action="ban" data-id="${u.id}" data-banned="${banned ? 1 : 0}">${banned ? "解封" : "封禁"}</button>
                  <button class="btn ghost" data-action="secret" data-id="${u.id}">重置订阅密钥</button>
                </div>
              </td>
            </tr>
          `;
        }).join("");
      }

      async function loadUsers() {
        const body = await request({
          method: "GET",
          url: `${buildV2("user/fetch")}?current=${state.usersPage}&pageSize=10`
        });

        state.users = Array.isArray(body.data) ? body.data : [];
        state.usersLastPage = Number(body.last_page || 1);
        dom.usersPageInfo.textContent = `页码 ${state.usersPage} / ${state.usersLastPage}`;
        renderUsers();
      }

      async function handleUserAction(e) {
        const btn = e.target.closest("button[data-action]");
        if (!btn) return;
        const action = btn.dataset.action;
        const id = Number(btn.dataset.id);

        try {
          if (action === "ban") {
            const banned = Number(btn.dataset.banned) === 1;
            const payload = { id, banned: banned ? 0 : 1 };

            if (!banned) {
              const reason = window.prompt("请输入封禁原因", "");
              if (reason === null) return;
              if (!String(reason).trim()) {
                throw new Error("封禁原因不能为空");
              }
              payload.ban_reason = String(reason).trim();
            } else {
              const unbanNote = window.prompt("如需记录解封说明可填写，可留空", "");
              if (unbanNote === null) return;
              if (String(unbanNote).trim()) {
                payload.ban_reason = String(unbanNote).trim();
              }
            }

            await request({ method: "POST", url: buildV2("user/update"), data: payload });
            showToast(banned ? "用户已解封" : "用户已封禁", "success");
          } else if (action === "secret") {
            await request({ method: "POST", url: buildV2("user/resetSecret"), data: { id } });
            showToast("已重置该用户订阅密钥", "success");
          }
          await loadUsers();
        } catch (err) {
          showToast(err.message || "操作失败", "error");
        }
      }

      async function sendApiRequest() {
        try {
          const version = dom.apiVersionSelect.value;
          const method = dom.apiMethodSelect.value;
          const endpoint = dom.apiEndpointInput.value.trim();
          if (!endpoint) {
            showToast("请输入请求地址", "error");
            return;
          }

          let url = endpoint;
          if (version === "v2") {
            url = buildV2(endpoint);
          } else if (version === "v1") {
            url = buildV1(endpoint);
          }

          let bodyData;
          if (dom.apiBodyInput.value.trim()) {
            try {
              bodyData = JSON.parse(dom.apiBodyInput.value);
            } catch (err) {
              throw new Error("请求内容格式不正确，请按示例填写");
            }
          }

          const resp = await request({ method, url, data: bodyData });
          dom.apiResponseOutput.textContent = JSON.stringify(resp, null, 2);
          showToast("请求完成", "success");
        } catch (err) {
          dom.apiResponseOutput.textContent = String(err.message || err);
          showToast(err.message || "请求失败", "error");
        }
      }

      async function loadAll() {
        if (commandCenterOnly) {
          await loadOverview();
          return;
        }

        const tasks = [
          loadOverview(),
          loadUsers(),
          openOpsModule(state.activeOpsModule || "security", { force: true })
        ];
        const results = await Promise.allSettled(tasks);
        const failed = results.filter((item) => item.status === "rejected");
        if (failed.length) {
          showToast(`${failed.length} 个模块加载失败，请到对应页面重试`, "error");
        }
      }

      function bindEvents() {
        bindTabs();

        if (dom.themeDarkBtn) {
          dom.themeDarkBtn.addEventListener("click", () => applyAdminTheme("dark"));
        }

        if (dom.themeLightBtn) {
          dom.themeLightBtn.addEventListener("click", () => applyAdminTheme("light"));
        }

        if (dom.primaryOpsMenuSearch) {
          dom.primaryOpsMenuSearch.addEventListener("input", (e) => {
            renderPrimaryOpsMenu(e.target.value || "");
            setActiveMenuButton(state.activeTab, state.activeTab === "ops" ? state.activeOpsModule : "");
          });
        }

        dom.loginSubmitBtn.addEventListener("click", loginWithPassword);
        dom.loginPasswordInput.addEventListener("keydown", (e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            loginWithPassword();
          }
        });
        dom.loginEmailInput.addEventListener("keydown", (e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            loginWithPassword();
          }
        });

        dom.logoutBtn.addEventListener("click", () => {
          showLoginView("已退出登录", { clearSharedAuth: true });
          showToast("已退出登录", "info");
        });

        dom.refreshOverviewBtn.addEventListener("click", async () => {
          try {
            await loadOverview(true);
            showToast("后台总览已刷新", "success");
          } catch (err) {
            showToast(err.message || "刷新失败", "error");
          }
        });

        if (dom.quickUsersBtn) {
          dom.quickUsersBtn.addEventListener("click", () => {
            document.querySelector('.menu-btn[data-tab="users"]')?.click();
          });
        }

        if (dom.openCommandCenterBtn) {
          dom.openCommandCenterBtn.addEventListener("click", () => {
            showToast("已独立打开超级管理员监控大屏", "info");
          });
        }

        if (dom.commandCenterDashboard) {
          dom.commandCenterDashboard.addEventListener("click", (e) => {
            const jumpBtn = e.target.closest("[data-overview-jump]");
            if (jumpBtn) {
              const tab = jumpBtn.dataset.overviewJump;
              const moduleKey = jumpBtn.dataset.overviewModule || "";
              const menuSelector = tab === "ops" && moduleKey
                ? `.menu-btn[data-tab="ops"][data-ops-module="${moduleKey}"]`
                : `.menu-btn[data-tab="${tab}"]:not([data-ops-module])`;
              const targetMenu = document.querySelector(menuSelector) || document.querySelector(`.menu-btn[data-tab="${tab}"]`);
              targetMenu?.click();
              return;
            }

            const openBtn = e.target.closest("[data-overview-open]");
            if (openBtn && openBtn.dataset.overviewOpen === "command-center") {
              if (dom.openCommandCenterBtn) {
                window.open(dom.openCommandCenterBtn.href, "_blank", "noopener,noreferrer");
                showToast("已独立打开超级管理员监控大屏", "info");
              }
            }
          });
        }

        dom.loadUsersBtn.addEventListener("click", async () => {
          try {
            await loadUsers();
            showToast("用户列表已更新", "success");
          } catch (err) {
            showToast(err.message || "加载失败", "error");
          }
        });

        dom.prevUsersBtn.addEventListener("click", async () => {
          if (state.usersPage <= 1) return;
          state.usersPage -= 1;
          await loadUsers();
        });

        dom.nextUsersBtn.addEventListener("click", async () => {
          if (state.usersPage >= state.usersLastPage) return;
          state.usersPage += 1;
          await loadUsers();
        });

        dom.usersTableBody.addEventListener("click", handleUserAction);

        dom.sendApiBtn.addEventListener("click", sendApiRequest);

        if (dom.opsRefreshBtn) {
          dom.opsRefreshBtn.addEventListener("click", () => {
            openOpsModule(state.activeOpsModule || "security", { force: true }).catch((err) => {
              showToast(err.message || "刷新失败", "error");
            });
          });
        }

        if (dom.opsOpenApiBtn) {
          dom.opsOpenApiBtn.addEventListener("click", () => {
            document.querySelector('.menu-btn[data-tab="api"]')?.click();
          });
        }

        dom.presetThemeBtn.addEventListener("click", () => {
          dom.apiVersionSelect.value = "v2";
          dom.apiMethodSelect.value = "GET";
          dom.apiEndpointInput.value = "system/getSystemStatus";
          dom.apiBodyInput.value = "";
          showToast("已填充系统状态请求", "info");
        });

        dom.presetLimitBtn.addEventListener("click", () => {
          dom.apiVersionSelect.value = "v1";
          dom.apiMethodSelect.value = "PUT";
          dom.apiEndpointInput.value = "admin/users/1/concurrent-ip-limit";
          dom.apiBodyInput.value = JSON.stringify({ concurrent_ip_limit: 2 }, null, 2);
          showToast("已填充并发 IP 限制请求", "info");
        });
      }

      async function init() {
        applyAdminTheme(localStorage.getItem(ADMIN_THEME_STORAGE_KEY) || "dark", false);
        renderPrimaryOpsMenu();
        bindEvents();
        setActiveMenuButton(state.activeTab || "overview");
        setOpsModuleHeader(state.activeOpsModule || "security");

        const storedToken = readSharedAuthToken();
        if (storedToken) {
          updateToken(storedToken, false);
          try {
            await syncAdminSession();
            showConsoleView();
            await loadAll();
            return;
          } catch (err) {
            showLoginView(
              err.message || "登录状态已失效，请重新登录。",
              { clearSharedAuth: Number(err?.status || 0) === 401 }
            );
            return;
          }
        }

        showLoginView();
      }

      window.addEventListener("storage", (event) => {
        if (event.key === ADMIN_THEME_STORAGE_KEY && !commandCenterOnly) {
          applyAdminTheme(event.newValue || "dark", false);
          return;
        }

        if (![...sharedAuthKeys, SHARED_LEGACY_TOKEN_KEY, SHARED_PROFILE_KEY].includes(event.key || "")) {
          return;
        }

        const nextToken = readSharedAuthToken();
        const currentToken = normalizeBearer(state.token);

        if (!nextToken) {
          if (currentToken) {
            showLoginView("登录状态已同步退出。");
          }
          return;
        }

        if (nextToken === currentToken) {
          return;
        }

        updateToken(nextToken, false);
        syncAdminSession()
          .then((me) => {
            if (!me) return;
            showConsoleView();
            return loadAll();
          })
          .catch((err) => {
            showLoginView(
              err.message || "登录状态同步失败，请重新登录。",
              { clearSharedAuth: Number(err?.status || 0) === 401 }
            );
          });
      });

      init();
    })();
  </script>
</body>

</html>
