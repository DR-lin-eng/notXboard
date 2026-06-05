<!doctype html>
<html lang="zh-CN">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>登录中...</title>
    <style>
        body { font-family: system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif; padding: 48px; }
        .card { max-width: 520px; margin: 0 auto; border: 1px solid #e5e7eb; border-radius: 12px; padding: 24px; }
        .muted { color: #6b7280; font-size: 14px; margin-top: 12px; }
        .ok { color: #059669; }
        .bad { color: #dc2626; }
        code { background: #f3f4f6; padding: 2px 6px; border-radius: 6px; }
    </style>
</head>
<body>
    <div class="card">
        <h1>Linux DO 登录回调</h1>
        <div id="msg" class="muted">正在写入登录态并跳转...</div>
        <div class="muted">如果没有自动跳转，请点击：<a id="go" href="/">返回首页</a></div>
    </div>

    <script>
        (function () {
            const payload = @json($payload ?? null);
            const msg = document.getElementById('msg');

            try {
                if (!payload || payload.success === false) {
                    msg.className = 'muted bad';
                    msg.textContent = payload?.error || payload?.message || '登录回调失败，请返回重试。';
                    document.getElementById('go').href = '/app/#/login';
                    return;
                }

                if (!payload.data || !payload.data.auth || !payload.data.auth.auth_data) {
                    msg.className = 'muted bad';
                    msg.textContent = '登录回调数据不完整，请重试。';
                    return;
                }

                localStorage.setItem('auth_data', payload.data.auth.auth_data);
                localStorage.setItem('token', payload.data.auth.token || '');
                localStorage.setItem('me', JSON.stringify({
                    id: payload.data.user?.id,
                    email: payload.data.user?.email,
                    is_admin: !!payload.data.user?.is_admin,
                    is_super_admin: !!payload.data.user?.is_super_admin,
                    trust_level: payload.data.user?.trust_level ?? 0,
                    is_linux_do_user: true,
                    linux_do_username: payload.data.user?.linux_do_username ?? null,
                    linux_do_name: payload.data.user?.linux_do_name ?? null,
                    linux_do_avatar: payload.data.user?.linux_do_avatar ?? null,
                    api_key: payload.data.user?.api_key ?? null
                }));

                msg.className = 'muted ok';
                msg.textContent = '登录成功，正在跳转...';
                window.location.href = '/app/#/dashboard';
            } catch (e) {
                msg.className = 'muted bad';
                msg.textContent = '写入登录态失败：' + (e?.message || e);
            }
        })();
    </script>
</body>
</html>
