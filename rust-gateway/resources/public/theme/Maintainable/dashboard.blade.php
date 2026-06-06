<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="app-name" content="{{ $title ?? 'Xboard' }}">
  <meta name="app-version" content="{{ $version ?? '' }}">
  <meta name="app-url" content="{{ config('app.url') }}">
  <title>{{ $title ?? 'Xboard' }}</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600;700&family=Sora:wght@300;400;500;600;700&family=Space+Mono:wght@400;600&display=swap" rel="stylesheet">
  @php($maintainableCssVersion = file_exists(public_path('theme/Maintainable/app.css')) ? filemtime(public_path('theme/Maintainable/app.css')) : ($asset_version ?? $version ?? 'dev'))
  @php($maintainableJsVersion = file_exists(public_path('theme/Maintainable/app.js')) ? filemtime(public_path('theme/Maintainable/app.js')) : ($asset_version ?? $version ?? 'dev'))
  <link rel="stylesheet" href="/theme/Maintainable/app.css?v={{ $maintainableCssVersion }}">
</head>
<body>
  <div id="app">
    <header class="topbar">
      <div class="brand">
        <span class="logo">{{ $title ?? 'Xboard' }}</span>
        <span class="badge">Maintainable</span>
      </div>
      <nav class="nav">
        <a href="#/dashboard" data-auth-only="1">概览</a>
        <a href="#/plans" data-auth-only="1">套餐</a>
        <a href="#/orders" data-auth-only="1">订单</a>
        <a href="#/refunds" data-auth-only="1">退款</a>
        <a href="#/refund-votes" data-auth-only="1">争议投票</a>
        <a href="#/tickets" data-auth-only="1">工单</a>
        <a href="#/nodes" data-auth-only="1">节点</a>
        <a href="#/node-plans" data-auth-only="1">节点套餐</a>
        <a href="#/node-admin" data-auth-only="1">节点管理</a>
        <a href="#/profile" data-auth-only="1">账户</a>
        <a href="#/payment" data-auth-only="1">收款配置</a>
        <a href="#/sponsor" data-auth-only="1">赞助</a>
        <a href="#/invite" data-auth-only="1">邀请</a>
        <a href="#/notices" data-auth-only="1">公告</a>
        <a href="#/knowledge" data-auth-only="1">知识库</a>
        <a href="#/downloads" data-auth-only="1">软件下载</a>
        <a href="#/tools" data-auth-only="1">工具</a>
        <a href="#/audit" data-auth-only="1">审计</a>
        <a href="#/admin" data-role="super-admin" data-auth-only="1" class="hidden" hidden>超管</a>
        <a href="#/login" id="logoutBtn">登录</a>
      </nav>
    </header>

    <main class="container">
      <div id="view"></div>
    </main>

    <footer class="footer">
      <span>{{ $title ?? 'Xboard' }} · {{ $version ?? '' }}</span>
    </footer>
  </div>

  <script>
    window.__APP__ = {
      baseUrl: "{{ config('app.url') }}",
      version: "{{ $version ?? 'dev' }}"
    };
  </script>
  <script type="module" src="/theme/Maintainable/app.js?v={{ $maintainableJsVersion }}"></script>
</body>
</html>
