<!doctype html>
<html lang="zh-CN">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Linux DO Connect 登录</title>
    <style>
        body { font-family: system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif; padding: 48px; }
        .card { max-width: 520px; margin: 0 auto; border: 1px solid #e5e7eb; border-radius: 12px; padding: 24px; }
        .btn { display: inline-block; padding: 12px 16px; border-radius: 10px; background: #111827; color: #fff; text-decoration: none; }
        .muted { color: #6b7280; font-size: 14px; margin-top: 12px; }
    </style>
</head>
<body>
    <div class="card">
        <h1>使用 Linux DO Connect 登录</h1>
        <p>点击下方按钮将跳转到 Linux DO Connect 进行授权登录。</p>
        <p><a class="btn" href="/api/v1/passport/oauth2/linux-do/redirect">Linux DO Connect 登录</a></p>
        <p class="muted">授权回调将返回 JSON（用于前端应用接入）。</p>
    </div>
</body>
</html>

