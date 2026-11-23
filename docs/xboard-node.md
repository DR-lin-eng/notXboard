# Xboard-Node 对接

V2bX 已不再使用，节点后端统一为 [cedar2025/Xboard-Node](https://github.com/cedar2025/Xboard-Node)。

## 部署

- 单节点：在「我的节点」点击一键部署，复制命令到节点服务器以 root 执行。
- Machine 模式：在「Xboard-Node 机器」新增机器，绑定自己的节点，复制部署命令。一台机器运行一个 Xboard-Node 进程，同时承载所有绑定节点；增删绑定会通过 WebSocket `sync.nodes` 推送。
- 安装脚本 `/xboard-node-install.sh` 固定上游 install.sh 的 commit 和 SHA-256，命令只携带 10 分钟一次性 ticket，长期 token 由脚本兑换。

## 接口

| 接口 | 说明 |
| --- | --- |
| `POST /api/v2/server/handshake` | 返回推送/拉取间隔和 WebSocket 地址 |
| `GET /api/v1/server/UniProxy/config`、`/user` | 节点配置和用户列表，支持 ETag |
| `POST /api/v2/server/report` | 合并上报 traffic/alive/online/status |
| `POST /api/v2/server/machine/nodes`、`/status` | Machine 模式节点列表和机器负载 |
| `GET /api/v2/server/ws` | WebSocket：`auth.success`、`ping`、`sync.config`、`sync.users`、`sync.nodes`，接收 `node.status` |

鉴权：节点用 `token` + `node_id`，机器用 `token` + `machine_id`；GET 走查询参数，POST 走 JSON body。token 必须是 64 位十六进制。Machine token 只能访问绑定到该机器且属于同一 owner 的节点，共享节点授权逻辑不变。

## 设置项

| 键 | 默认 | 说明 |
| --- | --- | --- |
| `server_ws_enable` | `1` | 关闭后 handshake 返回 `enabled=false`，节点回退 REST 轮询 |
| `server_ws_url` | 空 | 自定义 WebSocket 地址；为空时由 `app_url` 推导（`https`→`wss`） |
| `server_ws_sync_interval` | `10` | WebSocket 变更检测间隔（秒，3-120） |

## 数据库

启动时自动创建 `server_machines`、`server_machine_load_history`，并给 `server_nodes` 增加 `machine_id`。节点凭据仍存放在兼容列 `v2bx_token`/`v2bx_node_id`，无需迁移数据。
