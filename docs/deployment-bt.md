# notXboard 宝塔面板超详细部署教程（PHP 兼容路径）

注意：

- 当前默认推荐部署方式已经是 Rust-first Docker
- 本文只保留给仍然需要宝塔 + PHP Octane/Horizon 的兼容部署场景
- 默认部署请优先看 `docs/deployment-docker.md`

这份教程假设你**完全不会**服务器部署。我会把每一步写得很具体，你只要照着做即可。

目标：在宝塔面板上完成 notXboard 的完整安装，并能正常访问后台、登录和使用订阅。

---

## 0. 你需要先准备什么

### 0.1 一台服务器
- 推荐配置：**2核 CPU / 2GB 内存** 起步
- 系统：CentOS / Ubuntu / Debian 均可（宝塔面板支持）

### 0.2 一个域名（可选，但强烈建议）
- 举例：`example.com`
- 在域名解析里把域名 A 记录指向你的服务器公网 IP

### 0.3 服务器安全组/防火墙放行端口
- 需要放行：**80** 和 **443**（HTTP / HTTPS）
- 如果你不知道怎么开：在服务器厂商控制台里找“安全组”或“防火墙”

---

## 1. 打开宝塔面板

1) 在浏览器输入你的宝塔面板地址（安装宝塔后会提供）
2) 用宝塔面板账号/密码登录

**你需要看到首页，左侧有菜单，例如：网站 / 数据库 / 软件商店 / 文件 / 终端**

---

## 2. 安装基础软件（软件商店）

进入：左侧菜单 **软件商店**

请安装以下软件（推荐版本）：

- **Nginx**
- **MySQL 5.7+ 或 8.x**
- **Redis 6+**
- **PHP 8.2**

> 安装时请等待完成，不要中途关闭页面。

---

## 3. 配置 PHP 8.2（关键）

进入：软件商店 → **PHP 8.2** → 设置  

### 3.1 安装/启用 PHP 扩展
点左侧“安装扩展”，确保以下扩展**已安装并启用**：

- swoole
- redis
- bcmath
- pcntl
- zip
- fileinfo
- mbstring
- pdo_mysql
- openssl

### 3.2 解除 PHP 禁用函数
点“禁用函数”，确保**以下函数没有被禁用**：

- `proc_open`
- `putenv`
- `pcntl_alarm`
- `pcntl_fork`
- `pcntl_signal`
- `pcntl_signal_dispatch`
- `pcntl_wait`
- `pcntl_waitpid`

> 如果你不解除这些函数，后面安装 Composer / Octane / Horizon 可能会失败。

### 3.3 建议调整 PHP 参数
点“配置修改”，建议修改以下参数（如果看到这些选项）：

- `memory_limit`：建议 `512M` 或 `1024M`
- `max_execution_time`：建议 `300`
- `upload_max_filesize`：建议 `50M` 或更高
- `post_max_size`：建议 `50M` 或更高

修改后点击“保存”。

---

## 4. 创建网站（站点）

进入：左侧菜单 **网站**

点击 **添加站点**：

- 域名：填你的域名（如 `example.com`）
- 根目录：`/www/wwwroot/notxboard`
- PHP 版本：选择 **PHP 8.2**
- 勾选“创建数据库”可选（后面我们也会手动创建）

创建完成后，宝塔会生成一个站点。

---

## 5. 创建数据库

进入：左侧菜单 **数据库**

点击 **添加数据库**，填写：

- 数据库名：例如 `notxboard`
- 用户名：例如 `notxboard_user`
- 密码：生成或手动填写（请保存好）

记录好这些信息，后面要用。

---

## 6. 上传或拉取项目代码

你可以用两种方式：

### 方式 A：Git 拉取（推荐）
进入左侧菜单 **终端**，执行：

```bash
cd /www/wwwroot
rm -rf notxboard
git clone <你的仓库地址> notxboard
```

### 方式 B：上传压缩包
1) 进入左侧菜单 **文件**  
2) 打开 `/www/wwwroot`  
3) 上传你的项目压缩包  
4) 解压后重命名为 `notxboard`

最终目录结构应该是：

```
/www/wwwroot/notxboard
  ├─ app
  ├─ public
  ├─ artisan
  ├─ composer.json
  └─ ...
```

---

## 7. 设置目录权限

进入左侧菜单 **终端**，执行：

```bash
chown -R www:www /www/wwwroot/notxboard
chmod -R 775 /www/wwwroot/notxboard/storage /www/wwwroot/notxboard/bootstrap/cache
```

> `www` 是宝塔默认用户，请不要改成 root。

---

## 8. 配置项目环境（.env）

进入终端执行：

```bash
cd /www/wwwroot/notxboard
cp .env.example .env
```

然后打开 `.env` 文件编辑（宝塔文件管理器也可以）：

```env
APP_URL=https://你的域名
APP_ENV=production
APP_DEBUG=false

DB_HOST=127.0.0.1
DB_PORT=3306
DB_DATABASE=你的数据库名
DB_USERNAME=你的数据库用户名
DB_PASSWORD=你的数据库密码

REDIS_HOST=127.0.0.1
REDIS_PASSWORD=null
REDIS_PORT=6379

QUEUE_CONNECTION=redis
CACHE_DRIVER=redis
```

如果你暂时没有域名，`APP_URL` 可以先用你的 IP：
```env
APP_URL=http://你的服务器IP
```

---

## 9. 一键初始化安装（兼容模式）

进入终端执行：

```bash
cd /www/wwwroot/notxboard
sh init.sh --php-compat
```

安装脚本会：

1) 安装 Composer 依赖  
2) 生成 APP_KEY  
3) 数据库迁移  
4) 创建管理员账号  

**请记住脚本输出的后台地址、账号和密码**。

---

## 10. 配置 Nginx（指向 public + 反代 Octane）

进入：网站 → 你的站点 → 设置 → 配置文件  
把站点根目录指向 `public`，并加入 Octane 反代（7001 端口）。

参考配置（把域名换成你自己的）：

```nginx
server {
    listen 80;
    server_name your-domain.com;
    root /www/wwwroot/notxboard/public;

    location / {
        try_files $uri @octane;
    }

    location @octane {
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_pass http://127.0.0.1:7001;
    }
}
```

保存后重载 Nginx（宝塔会提示重载）。

---

## 11. 启动常驻进程（Octane + Horizon）

进入：宝塔 → **进程守护**（或 Supervisor）

新增两个守护进程，工作目录都填：`/www/wwwroot/notxboard`

### 11.1 Octane
- 启动命令：

```bash
php artisan octane:start --host=127.0.0.1 --port=7001
```

- 运行用户：`www`
- 自动重启：开启

### 11.2 Horizon
- 启动命令：

```bash
php artisan horizon
```

- 运行用户：`www`
- 自动重启：开启

> 如果 Octane 启动失败，检查 PHP 扩展 `swoole` 是否启用。

---

## 12. 设置计划任务（必须）

进入：宝塔 → **计划任务**

新增一个任务：

- 任务类型：**Shell 脚本**
- 执行周期：**每分钟**
- 脚本内容：

```bash
cd /www/wwwroot/notxboard && php artisan schedule:run >> /www/wwwlogs/notxboard-scheduler.log 2>&1
```

保存即可。

---

## 13. 开启 HTTPS（强烈建议）

进入：网站 → 你的站点 → SSL  
1) 选择 Let’s Encrypt  
2) 申请并开启  
3) 勾选强制 HTTPS  

然后回到 `.env` 把 `APP_URL` 改成 `https://你的域名`。

---

## 14. 登录后台

安装脚本 `init.sh` 会输出后台地址和管理员账号密码。  
用浏览器访问后台地址并登录。

如果忘记地址：后台默认是一个随机安全路径，请查看安装输出或 `.env` 里的配置提示。

---

## 15. 新订阅链接说明

现在订阅链接采用**随机路径 + 随机查询参数**，更难被穷举。  
如果你修改过用户安全信息或更新系统，请让用户重新获取订阅链接。

---

## 常见问题排查（新手版）

### 1) 访问网站显示 502 / 504
原因：Octane 没启动或端口不对。  
解决：检查进程守护中 `octane` 是否运行，端口是否是 **7001**。

### 2) 访问 500 错误
原因：权限或 `.env` 配置错误。  
解决：
- 确认 `storage` 和 `bootstrap/cache` 有写权限  
- 确认数据库账号密码正确  

### 3) Composer 安装失败
原因：PHP 禁用函数或内存不足。  
解决：回到第 3 步检查禁用函数，提升 `memory_limit`。

### 4) 登录后无法获取订阅
原因：用户订阅已过期或套餐无效。  
解决：在后台检查用户套餐状态。

---

完成以上步骤后，你的站点就可以对外使用了。
