# Linux DO Connect OAuth2 集成文档

## 概述

本文档描述了如何在 XBoard 项目中集成 Linux DO Connect OAuth2 认证系统。该集成提供了基于信任等级的用户分组、个性化 API 密钥管理、以及灵活的用户限制控制功能。

## 功能特性

### 1. OAuth2 认证
- 支持 Linux DO Connect 第三方登录
- 自动用户信息同步
- 令牌刷新机制
- 安全的状态验证

### 2. 用户管理
- 基于信任等级的自动用户分组
- 个性化 API 密钥生成
- 用户信息定期同步
- 支持传统邮箱注册用户

### 3. 限制控制
- 用户组限制配置（基于信任等级 0-4）
- 个人限制配置（优先于组限制）
- 设备数量限制
- 连接数限制
- 速度限制

### 4. 权限管理
- 超级管理员权限控制
- 细粒度权限验证
- 安全的中间件保护

## 安装配置

### 1. 环境变量配置

在 `.env` 文件中添加以下配置：

```env
# Linux DO Connect OAuth2 配置
LINUX_DO_CLIENT_ID=your_client_id
LINUX_DO_CLIENT_SECRET=your_client_secret
LINUX_DO_REDIRECT_URI=https://yourdomain.com/api/v1/passport/oauth2/linux-do/callback

# 可选配置
LINUX_DO_AUTO_SYNC=true
LINUX_DO_SYNC_INTERVAL=24
LINUX_DO_AUTO_API_KEY=true
LINUX_DO_AUTO_GENERATE_API_KEY=true
LINUX_DO_TOKEN_ENCRYPTION=true
LINUX_DO_LOG_OAUTH=true

# 默认限制配置（可选）
LINUX_DO_LIMIT_0_UP=50
LINUX_DO_LIMIT_0_DOWN=100
LINUX_DO_LIMIT_0_DEVICE=2
LINUX_DO_LIMIT_0_CONNECTION=5
```

### 2. 数据库迁移

运行数据库迁移来创建必要的表结构：

```bash
php artisan migrate
```

### 3. 数据库种子

运行种子文件来初始化默认配置：

```bash
php artisan db:seed --class=UserGroupLimitSeeder
php artisan db:seed --class=LinuxDoOAuthSeeder
```

### 4. 获取 OAuth2 凭据

1. 访问 [Linux DO Connect](https://connect.linux.do/)
2. 注册你的应用程序
3. 获取 `client_id` 和 `client_secret`
4. 设置回调 URL: `https://yourdomain.com/api/v1/passport/oauth2/linux-do/callback`

## API 端点

### OAuth2 认证

#### 开始 OAuth2 登录
```
GET /api/v1/passport/oauth2/linux-do/redirect
```

#### OAuth2 回调处理
```
GET /api/v1/passport/oauth2/linux-do/callback?code={code}&state={state}
```

#### 刷新访问令牌
```
POST /api/v1/passport/oauth2/refresh
Authorization: Bearer {token}
```

#### 同步用户信息
```
POST /api/v1/passport/oauth2/sync
Authorization: Bearer {token}
```

### 用户 API 密钥管理

#### 获取 API 密钥信息
```
GET /api/v1/user/api-key
Authorization: Bearer {token}
```

#### 生成 API 密钥
```
POST /api/v1/user/api-key/generate
Authorization: Bearer {token}
```

#### 重置 API 密钥
```
POST /api/v1/user/api-key/reset
Authorization: Bearer {token}
```

#### 验证 API 密钥
```
POST /api/v1/user/api-key/validate
Authorization: Bearer {token}
Content-Type: application/json

{
    "api_key": "xb_..."
}
```

### 用户限制管理

#### 查看个人限制
```
GET /api/v1/user/limits
Authorization: Bearer {token}
```

#### 设置个人限制
```
POST /api/v1/user/limits
Authorization: Bearer {token}
Content-Type: application/json

{
    "speed_limit_up": 300,
    "speed_limit_down": 600,
    "device_limit": 8,
    "connection_limit": 30
}
```

#### 删除个人限制（恢复组限制）
```
DELETE /api/v1/user/limits
Authorization: Bearer {token}
```

### 超级管理员 API

#### 用户组限制管理

##### 获取所有用户组限制
```
GET /api/v1/admin/group-limits
Authorization: Bearer {super_admin_token}
```

##### 获取特定信任等级限制
```
GET /api/v1/admin/group-limits/{trustLevel}
Authorization: Bearer {super_admin_token}
```

##### 设置用户组限制
```
POST /api/v1/admin/group-limits
Authorization: Bearer {super_admin_token}
Content-Type: application/json

{
    "trust_level": 2,
    "speed_limit_up": 200,
    "speed_limit_down": 500,
    "device_limit": 5,
    "connection_limit": 20
}
```

##### 批量更新用户组限制
```
PUT /api/v1/admin/group-limits/batch
Authorization: Bearer {super_admin_token}
Content-Type: application/json

{
    "limits": [
        {
            "trust_level": 0,
            "speed_limit_up": 50,
            "speed_limit_down": 100,
            "device_limit": 2,
            "connection_limit": 5
        },
        {
            "trust_level": 1,
            "speed_limit_up": 100,
            "speed_limit_down": 200,
            "device_limit": 3,
            "connection_limit": 10
        }
    ]
}
```

##### 应用默认限制配置
```
POST /api/v1/admin/group-limits/defaults/apply
Authorization: Bearer {super_admin_token}
```

#### 用户个人限制管理

##### 获取用户列表
```
GET /api/v1/admin/users?search={query}&per_page={count}
Authorization: Bearer {super_admin_token}
```

##### 获取用户限制
```
GET /api/v1/admin/users/{userId}/limits
Authorization: Bearer {super_admin_token}
```

##### 设置用户限制
```
POST /api/v1/admin/users/{userId}/limits
Authorization: Bearer {super_admin_token}
Content-Type: application/json

{
    "speed_limit_up": 300,
    "speed_limit_down": 600,
    "device_limit": 8,
    "connection_limit": 30
}
```

#### API 密钥管理

##### 获取系统统计
```
GET /api/v1/admin/api-keys/stats
Authorization: Bearer {super_admin_token}
```

##### 搜索用户 API 密钥
```
GET /api/v1/admin/api-keys/search?query={search_term}
Authorization: Bearer {super_admin_token}
```

##### 批量生成 API 密钥
```
POST /api/v1/admin/api-keys/batch-generate
Authorization: Bearer {super_admin_token}
```

##### 清理无效 API 密钥
```
POST /api/v1/admin/api-keys/cleanup
Authorization: Bearer {super_admin_token}
```

## 信任等级说明

Linux DO Connect 使用信任等级来表示用户在社区中的活跃度和可信度：

- **Level 0 (New User)**: 新注册用户
- **Level 1 (Basic User)**: 基础用户
- **Level 2 (Member)**: 社区成员
- **Level 3 (Regular)**: 常规用户
- **Level 4 (Leader)**: 社区领导者

每个信任等级都有对应的默认限制配置，超级管理员可以自定义这些限制。

## 安全考虑

### 1. OAuth2 安全
- 使用 state 参数防止 CSRF 攻击
- 令牌加密存储
- 自动令牌刷新
- 严格的作用域控制

### 2. API 密钥安全
- 使用加密安全的随机生成
- 唯一性验证
- 前缀标识 (`xb_`)
- 定期清理无效密钥

### 3. 权限控制
- 超级管理员权限验证
- 中间件保护敏感端点
- 详细的操作日志记录
- 用户权限实时验证

## 故障排除

### 常见问题

#### 1. OAuth2 配置错误
```
Error: Linux DO OAuth2 credentials not configured
```
**解决方案**: 检查 `.env` 文件中的 `LINUX_DO_CLIENT_ID` 和 `LINUX_DO_CLIENT_SECRET` 配置。

#### 2. 回调 URL 不匹配
```
Error: Invalid redirect URI
```
**解决方案**: 确保 Linux DO Connect 应用配置中的回调 URL 与实际使用的 URL 一致。

#### 3. 权限不足
```
Error: Super administrator privileges required
```
**解决方案**: 确保用户的 `is_super_admin` 字段为 `true`。

#### 4. API 密钥格式错误
```
Error: Invalid API key format
```
**解决方案**: API 密钥应该以 `xb_` 开头，后跟 60 位十六进制字符。

### 日志调试

系统会记录详细的操作日志，可以通过以下方式查看：

```bash
# 查看 OAuth2 相关日志
tail -f storage/logs/laravel.log | grep "OAuth"

# 查看 API 密钥相关日志
tail -f storage/logs/laravel.log | grep "API key"

# 查看超级管理员操作日志
tail -f storage/logs/laravel.log | grep "Super admin"
```

## 测试

### 运行单元测试
```bash
php artisan test tests/Unit/Services/OAuth/
```

### 运行集成测试
```bash
php artisan test tests/Feature/OAuth2ControllerTest.php
php artisan test tests/Feature/LinuxDoOAuthIntegrationTest.php
```

### 运行所有相关测试
```bash
php artisan test --filter="OAuth|LinuxDo|ApiKey|Limit"
```

## 维护

### 定期任务

建议设置以下定期任务：

1. **清理过期会话**
```bash
# 每小时清理过期的在线会话
0 * * * * php artisan schedule:run
```

2. **同步用户信息**
```bash
# 每天同步活跃用户信息
0 2 * * * php artisan oauth:sync-users
```

3. **清理无效 API 密钥**
```bash
# 每周清理无效的 API 密钥
0 0 * * 0 php artisan api-keys:cleanup
```

### 监控指标

建议监控以下指标：

- OAuth2 登录成功率
- API 密钥使用情况
- 用户信任等级分布
- 限制配置应用情况
- 超级管理员操作频率

## 更新日志

### v1.0.0 (2024-01-01)
- 初始版本发布
- 支持 Linux DO Connect OAuth2 认证
- 用户组和个人限制管理
- API 密钥管理系统
- 超级管理员权限控制

---

如有问题或建议，请联系开发团队或提交 Issue。