# notXboard

notXboard 是一个面向共享节点平台场景的增强版 XBoard。

它不是单纯的“机场面板二开”，而是把平台角色拆成了三层：

- 普通用户：购买套餐、使用节点、提交工单、申请退款
- 个人管理员：管理自己负责的节点、节点套餐、节点工单、节点访问控制
- 超级管理员：管理全局配置、支付方式、争议裁决、系统任务与平台运营

当前仓库已经默认转向 `Rust gateway + Docker` 运行面，默认部署、默认构建和默认运行路径都不再依赖 PHP 兼容层。

## 当前推荐架构

- 默认基础栈：`gateway + mysql + redis`
- 根级 [Dockerfile](/Volumes/移动/一些资料文档/notXboard/Dockerfile:1) 与 [rust-gateway/Dockerfile](/Volumes/移动/一些资料文档/notXboard/rust-gateway/Dockerfile:1) 都构建 Rust 网关镜像
- 默认 `docker compose` 已直接以 `rust-gateway/` 自身作为 `gateway` 构建上下文
- 默认未命中路由不会隐式回退到 PHP

## 快速开始

默认部署优先走 Docker。

1. 复制环境变量模板：

```bash
cp .env.example .env
chmod 600 .env
```

2. 至少补齐这些配置：

```env
APP_ENV=production
APP_DEBUG=false
APP_URL=https://your-domain.com
APP_KEY=base64:replace-me

DB_CONNECTION=mysql
DB_HOST=mysql
DB_PORT=3306
DB_DATABASE=xboard
DB_USERNAME=xboard
DB_PASSWORD=change-me
DB_ROOT_PASSWORD=change-me-root

REDIS_HOST=redis
REDIS_PORT=6379
REDIS_PASSWORD=

RUST_GATEWAY_OWNS_SCHEDULER=true
```

3. 启动默认栈：

```bash
docker compose up -d mysql redis gateway
docker compose ps
```

4. 首次初始化：

```bash
APP_PORT="${APP_PORT:-8000}"
curl -X POST "http://127.0.0.1:${APP_PORT}/bootstrap/full" \
  -H 'Content-Type: application/json' \
  --data "{\"app_name\":\"notXboard\",\"app_url\":\"http://127.0.0.1:${APP_PORT}\",\"admin_email\":\"admin@example.com\",\"admin_password\":\"ChangeMe123!\"}"
```

5. 反代到宿主机 `127.0.0.1:${APP_PORT:-8000}`。

如果你不是新装整套环境，而是要复用现有宿主机 MySQL / Redis，只改 `.env` 的 `DB_HOST` / `REDIS_HOST` 等连接参数，然后只启动你需要的容器即可。详细写法见 `docs/deployment-docker.md`。

## 主要能力

- Rust 网关已承接默认在线流量入口与核心初始化链路
- 共享节点体系：支持节点负责人、节点套餐、节点级访问控制与节点统计
- V2bX / UniProxy 接入：支持 `server/config`、`user`、`alive`、`push` 等链路
- Linux DO Connect OAuth2 登录
- EPay / EasyPay / CodePay / VPay 兼容支付
- 退款、争议、投票与超管最终裁决
- 可维护前端主题与 Rust 渲染页面
- Rust 原生后台 scheduler、数据库备份与系统任务入口

## 文档导航

按用途看：

- Docker 部署：[`docs/deployment-docker.md`](/Volumes/移动/一些资料文档/notXboard/docs/deployment-docker.md:1)
- EPay 兼容支付说明：[`docs/epay.md`](/Volumes/移动/一些资料文档/notXboard/docs/epay.md:1)
- Linux DO OAuth 集成：[`docs/LINUX_DO_OAUTH_INTEGRATION.md`](/Volumes/移动/一些资料文档/notXboard/docs/LINUX_DO_OAUTH_INTEGRATION.md:1)
- Rust / PHP 兼容面审计：[`docs/rust-compat-audit.md`](/Volumes/移动/一些资料文档/notXboard/docs/rust-compat-audit.md:1)
- Rust 性能与运行说明：[`docs/rust-performance-notes.md`](/Volumes/移动/一些资料文档/notXboard/docs/rust-performance-notes.md:1)
- 默认栈验证清单：[`docs/rust-verification-checklist.md`](/Volumes/移动/一些资料文档/notXboard/docs/rust-verification-checklist.md:1)
- Rust schema 基线说明：[`docs/rust-schema-baseline.md`](/Volumes/移动/一些资料文档/notXboard/docs/rust-schema-baseline.md:1)

## 开发与验证

常用命令：

```bash
# 隔离网络下验证默认 Rust 栈
bash tools/verify_rust_default_stack_isolated.sh

# 本地默认 Rust 栈验证
bash tools/verify_rust_default_stack.sh

# 查看 compose 生效配置
docker compose config
```

如果你在做默认运行面相关工作，建议先确认自己改的是哪一层：

- `rust-gateway/`：默认在线入口、默认页面、默认后台任务
- `app/` / `routes/` / `resources/views/`：仍待清理的 PHP 历史代码面
- `theme/portal/`：前端静态资源
- `docker-compose.yml`：默认 Rust 运行面部署拓扑

根 README 只保留入口信息。更细的部署参数、兼容边界、支付细节和 OAuth 说明，请直接看上面的对应文档。
