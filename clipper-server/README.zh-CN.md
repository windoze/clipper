# Clipper Server

使用 `clipper_indexer` 库管理剪贴板条目的 REST API 服务器，支持 WebSocket。

## 功能特性

- **REST API** - 剪贴板条目的增删改查操作
- **全文搜索** - 支持筛选（标签、日期范围）和分页
- **WebSocket 支持** - 实时更新通知
- **文件附件支持** - 剪贴板条目可附加文件
- **元数据管理** - 标签和备注管理
- **多源配置** - 支持命令行参数、环境变量、配置文件
- **优雅关闭** - 信号处理
- **内置 Web UI** - 支持拖放上传文件
- **TLS/HTTPS 支持** - 手动证书或自动（Let's Encrypt）证书
- **证书热重载** - 零停机证书更新
- **自动清理** - 可配置的保留策略

## 快速开始

### 配置

服务器可以通过多种来源配置（按优先级排序）：

1. **命令行参数**（最高优先级）
2. **环境变量**
3. **配置文件**（TOML）
4. **默认值**（最低优先级）

#### 命令行参数

```bash
clipper-server [选项]

选项：
  -c, --config <FILE>              配置文件路径
      --db-path <PATH>             数据库路径
      --storage-path <PATH>        文件附件存储路径
      --listen-addr <ADDR>         服务器监听地址（默认: 0.0.0.0）
  -p, --port <PORT>                服务器监听端口（默认: 3000）
      --bearer-token <TOKEN>       用于身份验证的 Bearer 令牌
      --cleanup-enabled            启用旧剪贴自动清理
      --cleanup-retention-days <DAYS>   保留天数（默认: 30）
      --cleanup-interval-hours <HOURS>  清理间隔小时数（默认: 24）
  -h, --help                       打印帮助信息
```

#### 环境变量

- `CLIPPER_CONFIG` - 配置文件路径
- `CLIPPER_DB_PATH` - 数据库目录路径（默认: `./data/db`）
- `CLIPPER_STORAGE_PATH` - 文件存储目录路径（默认: `./data/storage`）
- `CLIPPER_LISTEN_ADDR` - 服务器监听地址（默认: `0.0.0.0`）
- `PORT` - 服务器端口（默认: `3000`）
- `RUST_LOG` - 日志级别（默认: `clipper_server=debug,tower_http=debug`）
- `CLIPPER_CLEANUP_ENABLED` - 启用自动清理（默认: `false`）
- `CLIPPER_CLEANUP_RETENTION_DAYS` - 保留天数（默认: `30`）
- `CLIPPER_CLEANUP_INTERVAL_HOURS` - 清理间隔小时数（默认: `24`）
- `CLIPPER_BEARER_TOKEN` - 身份验证 Bearer 令牌（如设置，所有请求需要认证）

#### 配置文件

创建 `config.toml` 或 `clipper-server.toml` 文件：

```toml
[database]
path = "./data/db"

[storage]
path = "./data/storage"

[server]
listen_addr = "0.0.0.0"
port = 3000

[cleanup]
enabled = false
retention_days = 30
interval_hours = 24

[auth]
# bearer_token = "your-secret-token"
```

或指定自定义配置文件位置：

```bash
clipper-server --config /path/to/config.toml
```

完整示例请参阅 `config.toml.example`。

### 身份验证

启用 Bearer 令牌认证以保护 API：

```bash
# 通过命令行
clipper-server --bearer-token your-secret-token

# 通过环境变量
CLIPPER_BEARER_TOKEN=your-secret-token clipper-server

# 通过配置文件（参见 config.toml.example）
```

启用身份验证后：
- 所有 REST API 端点（除 `/health` 外）需要 `Authorization: Bearer <token>` 头
- 文件下载也支持 `?token=<token>` 查询参数
- WebSocket 连接使用基于消息的身份验证（客户端连接后发送认证消息）
- Web UI 在需要认证时会显示登录界面

认证请求示例：
```bash
curl -H "Authorization: Bearer your-secret-token" http://localhost:3000/clips
```

### TLS/HTTPS 配置

使用 TLS 特性构建以支持 HTTPS：

```bash
# 手动证书
cargo build -p clipper-server --features tls

# 自动 Let's Encrypt 证书
cargo build -p clipper-server --features acme

# 完整 TLS 支持和安全凭证存储
cargo build -p clipper-server --features full-tls
```

#### TLS 环境变量（需要 `tls` 特性）

- `CLIPPER_TLS_ENABLED` - 启用 HTTPS（默认: `false`）
- `CLIPPER_TLS_PORT` - HTTPS 端口（默认: `443`）
- `CLIPPER_TLS_CERT` - TLS 证书文件路径（PEM 格式）
- `CLIPPER_TLS_KEY` - TLS 私钥文件路径（PEM 格式）
- `CLIPPER_TLS_REDIRECT` - 将 HTTP 重定向到 HTTPS（默认: `true`）
- `CLIPPER_TLS_RELOAD_INTERVAL` - 证书重新加载检查间隔秒数（默认: `0` = 禁用）

#### ACME 环境变量（需要 `acme` 特性）

- `CLIPPER_ACME_ENABLED` - 启用自动证书管理（默认: `false`）
- `CLIPPER_ACME_DOMAIN` - 证书的域名
- `CLIPPER_ACME_EMAIL` - Let's Encrypt 通知联系邮箱
- `CLIPPER_ACME_STAGING` - 使用测试环境进行测试（默认: `false`）
- `CLIPPER_CERTS_DIR` - 证书缓存目录（默认: `~/.config/com.0d0a.clipper/certs/`）

#### 示例：使用 Let's Encrypt 的 HTTPS

```bash
CLIPPER_ACME_ENABLED=true \
CLIPPER_ACME_DOMAIN=clips.example.com \
CLIPPER_ACME_EMAIL=admin@example.com \
cargo run --bin clipper-server --features acme
```

#### 使用自签名证书

用于开发或内部部署，您可以使用自签名证书：

1. **生成自签名证书**：

```bash
# 生成私钥和自签名证书（有效期 365 天）
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes \
  -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"
```

2. **使用自签名证书启动服务器**：

```bash
CLIPPER_TLS_ENABLED=true \
CLIPPER_TLS_CERT=./cert.pem \
CLIPPER_TLS_KEY=./key.pem \
cargo run --bin clipper-server --features tls
```

3. **从 CLI 或桌面应用连接**：

连接使用自签名证书的服务器时，CLI 和桌面应用都使用类似 SSH 的指纹验证：

- **首次连接**：显示证书的 SHA-256 指纹供验证
- **信任决策**：您可以选择永久信任该证书
- **指纹存储**：受信任的指纹存储在 `~/.config/com.0d0a.clipper/settings.json`
- **安全警告**：如果指纹发生变化（可能的中间人攻击），您会看到类似 SSH 的"REMOTE HOST IDENTIFICATION HAS CHANGED"警告

CLI 交互示例：
```
$ clipper-cli --url https://clips.example.com:3000 list
The authenticity of host 'clips.example.com' can't be established.
The server's certificate is not signed by a trusted Certificate Authority (CA).
This could mean:
  - The server is using a self-signed certificate
  - The server's CA is not in your system's trust store
  - Someone may be intercepting your connection (man-in-the-middle attack)

Certificate SHA256 fingerprint:
  a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2

Full fingerprint (verify with server administrator):
  A1:B2:C3:D4:E5:F6:A7:B8
  C9:D0:E1:F2:A3:B4:C5:D6
  E7:F8:A9:B0:C1:D2:E3:F4
  A5:B6:C7:D8:E9:F0:A1:B2

Are you sure you want to trust this certificate and continue connecting (yes/no)?
```

CLI 和桌面应用共享相同的受信任证书存储，因此在一处信任的证书在另一处也会自动信任。

### 运行服务器

基本用法：
```bash
cargo run --bin clipper-server
```

自定义端口：
```bash
cargo run --bin clipper-server -- --port 8080
```

使用自定义配置：
```bash
cargo run --bin clipper-server -- --config config.toml
```

使用环境变量：
```bash
CLIPPER_DB_PATH=/var/lib/clipper/db PORT=8080 cargo run --bin clipper-server
```

服务器默认在 `http://0.0.0.0:3000` 启动（可配置）。

## Web UI

服务器包含内置 Web UI，可通过根 URL 访问（例如 `http://localhost:3000/`）。

### Web UI 功能

- 查看、搜索、编辑和删除剪贴
- 拖放上传文件
- 发送剪贴板内容按钮（用于手动剪贴板同步）
- 通过 WebSocket 实时更新（仅 HTTPS）
- WebSocket 连接状态指示器（已连接/已断开/需要 HTTPS）
- WebSocket 通知时自动刷新剪贴列表
- 主题支持（浅色/深色/自动）
- 国际化（英语/中文）
- 收藏夹和日期筛选
- 无限滚动分页
- 接近自动清理日期的剪贴视觉淡出效果（启用清理时）

### 构建嵌入式 Web UI

用于 Docker 部署，构建嵌入式 Web UI：

```bash
cd clipper-server/web && npm install && npm run build
cargo build -p clipper-server --release --features embed-web
```

## REST API 端点

### 健康检查

```
GET /health
```

如果服务器正在运行则返回 `OK`。

### 版本和状态

```
GET /version
```

返回服务器版本和状态信息。

**响应**：`200 OK`
```json
{
  "version": "0.10.0",
  "uptime_secs": 3600,
  "active_ws_connections": 5,
  "config": {
    "port": 3000,
    "tls_enabled": false,
    "acme_enabled": false,
    "cleanup_enabled": true,
    "cleanup_retention_days": 30
  }
}
```

### 创建剪贴

```
POST /clips
Content-Type: application/json

{
  "content": "要存储的文本内容",
  "tags": ["tag1", "tag2"],
  "additional_notes": "可选备注"
}
```

**响应**：`201 Created`
```json
{
  "id": "abc123",
  "content": "要存储的文本内容",
  "created_at": "2025-11-26T10:00:00Z",
  "tags": ["tag1", "tag2"],
  "additional_notes": "可选备注"
}
```

### 上传文件

```
POST /clips/upload
Content-Type: multipart/form-data
```

表单字段：
- `file` - 要上传的文件（必需）
- `tags` - 逗号分隔的标签列表（可选）
- `additional_notes` - 文件备注（可选）

**响应**：`201 Created`
```json
{
  "id": "abc123",
  "content": "文件内容（文本）或 'Binary file: filename'",
  "created_at": "2025-11-26T10:00:00Z",
  "tags": ["tag1", "tag2"],
  "additional_notes": "可选备注",
  "file_attachment": "stored_file_key"
}
```

### 列出剪贴

```
GET /clips?start_date=<RFC3339>&end_date=<RFC3339>&tags=<comma-separated>&page=<number>&page_size=<number>
```

查询参数（全部可选）：
- `start_date` - 筛选此日期之后创建的剪贴（RFC3339 格式）
- `end_date` - 筛选此日期之前创建的剪贴（RFC3339 格式）
- `tags` - 逗号分隔的标签列表筛选
- `page` - 页码（默认: 1）
- `page_size` - 每页条目数（默认: 20）

**响应**：`200 OK`
```json
{
  "items": [
    {
      "id": "abc123",
      "content": "文本内容",
      "created_at": "2025-11-26T10:00:00Z",
      "tags": ["tag1", "tag2"],
      "additional_notes": "可选备注"
    }
  ],
  "total": 100,
  "page": 1,
  "page_size": 20,
  "total_pages": 5
}
```

### 搜索剪贴

```
GET /clips/search?q=<query>&start_date=<RFC3339>&end_date=<RFC3339>&tags=<comma-separated>&page=<number>&page_size=<number>
```

查询参数：
- `q` - 搜索查询（必需）
- `start_date` - 筛选此日期之后创建的剪贴（RFC3339 格式，可选）
- `end_date` - 筛选此日期之前创建的剪贴（RFC3339 格式，可选）
- `tags` - 逗号分隔的标签列表筛选（可选）
- `page` - 页码（默认: 1，可选）
- `page_size` - 每页条目数（默认: 20，可选）

**响应**：`200 OK`（与列出剪贴相同的分页格式）

### 获取剪贴

```
GET /clips/:id
```

**响应**：`200 OK`
```json
{
  "id": "abc123",
  "content": "文本内容",
  "created_at": "2025-11-26T10:00:00Z",
  "tags": ["tag1", "tag2"],
  "additional_notes": "可选备注",
  "file_attachment": "optional_file_key"
}
```

### 更新剪贴

```
PUT /clips/:id
Content-Type: application/json

{
  "tags": ["new_tag1", "new_tag2"],
  "additional_notes": "更新的备注"
}
```

两个字段都是可选的。省略字段则保持不变。

**响应**：`200 OK`（与获取剪贴相同的格式）

### 删除剪贴

```
DELETE /clips/:id
```

**响应**：`204 No Content`

### 获取剪贴文件附件

```
GET /clips/:id/file
```

如果剪贴有文件附件则返回文件内容。

**响应**：`200 OK`，二进制文件内容

## WebSocket API

连接到 WebSocket 端点以接收实时更新：

```
ws://localhost:3000/ws
```

### 消息格式

服务器为剪贴更新发送 JSON 消息：

#### 新剪贴
```json
{
  "type": "new_clip",
  "id": "abc123",
  "content": "文本内容",
  "tags": ["tag1", "tag2"]
}
```

#### 更新的剪贴
```json
{
  "type": "updated_clip",
  "id": "abc123"
}
```

#### 删除的剪贴
```json
{
  "type": "deleted_clip",
  "id": "abc123"
}
```

#### 清理的剪贴
```json
{
  "type": "clips_cleaned_up",
  "ids": ["abc123", "def456"],
  "count": 2
}
```

### 客户端消息

客户端可以发送：
- **Ping 消息** - 服务器响应 pong 以保持连接活跃
- **认证消息** - 服务器启用认证时需要：
  ```json
  {"type": "auth", "token": "your-secret-token"}
  ```
  服务器响应：
  ```json
  {"type": "auth_response", "success": true}
  ```
  或
  ```json
  {"type": "auth_response", "success": false, "error": "Invalid token"}
  ```
- **文本消息** - 由服务器记录（保留用于将来功能）

## 使用示例

### 使用 curl

创建剪贴：
```bash
curl -X POST http://localhost:3000/clips \
  -H "Content-Type: application/json" \
  -d '{"content": "Hello, world!", "tags": ["greeting"]}'
```

分页列出剪贴：
```bash
curl "http://localhost:3000/clips?page=1&page_size=10"
```

分页搜索剪贴：
```bash
curl "http://localhost:3000/clips/search?q=hello&tags=greeting&page=1&page_size=20"
```

上传文件：
```bash
curl -X POST http://localhost:3000/clips/upload \
  -F "file=@/path/to/your/file.txt" \
  -F "tags=document,important" \
  -F "additional_notes=这是一个测试文件"
```

带身份验证：
```bash
# 所有请求带认证头
curl -H "Authorization: Bearer your-secret-token" \
  http://localhost:3000/clips

# 文件下载使用查询参数
curl "http://localhost:3000/clips/abc123/file?token=your-secret-token" -o file.txt
```

### 使用 WebSocket (JavaScript)

```javascript
const ws = new WebSocket('ws://localhost:3000/ws');

ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  console.log('收到更新:', update);

  if (update.type === 'new_clip') {
    console.log('新剪贴创建:', update.id);
  } else if (update.type === 'updated_clip') {
    console.log('剪贴已更新:', update.id);
  } else if (update.type === 'deleted_clip') {
    console.log('剪贴已删除:', update.id);
  }
};

ws.onopen = () => {
  console.log('已连接到 clipper 服务器');
};
```

### 使用 Rust 客户端

```rust
use clipper_client::{ClipperClient, SearchFilters};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端（可选认证）
    let client = ClipperClient::new("http://localhost:3000")
        .with_token("your-secret-token".to_string()); // 可选

    // 创建剪贴
    let clip = client.create_clip(
        "Hello, World!".to_string(),
        vec!["greeting".to_string()],
        None,
    ).await?;

    // 分页搜索
    let result = client.search_clips(
        "Hello",
        SearchFilters::new(),
        1,  // page
        20, // page_size
    ).await?;

    println!("在第 {} 页（共 {} 页）找到 {} 个剪贴",
             result.items.len(), result.page, result.total_pages);

    Ok(())
}
```

## 架构

- **axum** - 用于 REST API 和 WebSocket 的 Web 框架
- **tokio** - 用于非阻塞 I/O 的异步运行时
- **tower-http** - CORS 和追踪中间件
- **clipper_indexer** - 后端存储和搜索引擎
- **broadcast channel** - 用于实时更新的 WebSocket 发布/订阅
- **clap + config** - 多源配置管理

## 错误处理

所有错误以 JSON 返回：

```json
{
  "error": "错误消息描述"
}
```

HTTP 状态码：
- `400 Bad Request` - 输入无效（JSON 格式错误、缺少必需字段）
- `404 Not Found` - 资源未找到（剪贴 ID 不存在）
- `500 Internal Server Error` - 服务器错误（数据库问题、存储错误）

## 测试

运行完整测试套件：

```bash
# 运行所有服务器测试
cargo test -p clipper-server

# 运行集成测试（必须顺序执行）
cargo test --test api_tests -p clipper-server -- --test-threads=1
```

测试覆盖：
- 创建带可选字段和不带可选字段的剪贴
- 上传文件（文本和二进制）
- 带筛选和分页列出剪贴
- 带全文查询和分页搜索剪贴
- 按 ID 获取剪贴
- 更新剪贴元数据
- 删除剪贴
- 文件附件检索
- 所有操作的 WebSocket 通知

**共 18 个测试 - 全部通过 ✓**

## 部署

### 生产环境注意事项

1. **数据库路径**：生产环境使用持久化存储：
   ```bash
   CLIPPER_DB_PATH=/var/lib/clipper/db \
   CLIPPER_STORAGE_PATH=/var/lib/clipper/storage \
   cargo run --release --bin clipper-server
   ```

2. **日志**：配置适当的日志级别：
   ```bash
   RUST_LOG=clipper_server=info,tower_http=info cargo run --release --bin clipper-server
   ```

3. **端口绑定**：生产环境建议在服务器前使用反向代理（nginx、caddy）

4. **CORS**：服务器在开发模式下使用宽松的 CORS。生产环境请适当配置。

5. **优雅关闭**：服务器处理 SIGTERM 和 SIGINT 信号以实现干净关闭。

### Docker 部署

项目包含生产就绪的多阶段 Dockerfile，构建带嵌入式 Web UI 和完整 TLS 支持的 clipper-server。

#### Docker 快速开始

```bash
# 从项目根目录构建镜像
docker build -t clipper-server .

# 仅 HTTP 运行
docker run -d \
  --name clipper \
  -p 3000:3000 \
  -v clipper-data:/data \
  clipper-server

# 访问 http://localhost:3000
```

#### Docker HTTPS（手动证书）

```bash
docker run -d \
  --name clipper \
  -p 3000:3000 \
  -p 443:443 \
  -v clipper-data:/data \
  -v /path/to/certs:/certs:ro \
  -e CLIPPER_TLS_ENABLED=true \
  -e CLIPPER_TLS_CERT=/certs/cert.pem \
  -e CLIPPER_TLS_KEY=/certs/key.pem \
  clipper-server
```

#### Docker HTTPS（Let's Encrypt）

```bash
docker run -d \
  --name clipper \
  -p 80:3000 \
  -p 443:443 \
  -v clipper-data:/data \
  -e CLIPPER_ACME_ENABLED=true \
  -e CLIPPER_ACME_DOMAIN=clips.example.com \
  -e CLIPPER_ACME_EMAIL=admin@example.com \
  clipper-server
```

#### Docker Compose

```yaml
version: "3.8"
services:
  clipper:
    build: .
    ports:
      - "3000:3000"
      - "443:443"
    volumes:
      - clipper-data:/data
      - ./certs:/certs:ro  # 可选：用于手动 TLS
    environment:
      - RUST_LOG=clipper_server=info
      # 认证（公共部署推荐）：
      # - CLIPPER_BEARER_TOKEN=your-secret-token
      # 手动证书 TLS：
      # - CLIPPER_TLS_ENABLED=true
      # - CLIPPER_TLS_CERT=/certs/cert.pem
      # - CLIPPER_TLS_KEY=/certs/key.pem
      # 或使用 Let's Encrypt：
      # - CLIPPER_ACME_ENABLED=true
      # - CLIPPER_ACME_DOMAIN=clips.example.com
      # - CLIPPER_ACME_EMAIL=admin@example.com
    restart: unless-stopped

volumes:
  clipper-data:
```

#### Docker 环境变量

| 变量 | 默认值 | 描述 |
|------|--------|------|
| `CLIPPER_DB_PATH` | `/data/db` | 数据库目录 |
| `CLIPPER_STORAGE_PATH` | `/data/storage` | 文件存储目录 |
| `CLIPPER_LISTEN_ADDR` | `0.0.0.0` | 监听地址 |
| `PORT` | `3000` | HTTP 端口 |
| `RUST_LOG` | `clipper_server=info` | 日志级别 |
| `CLIPPER_TLS_ENABLED` | `false` | 启用 HTTPS |
| `CLIPPER_TLS_PORT` | `443` | HTTPS 端口 |
| `CLIPPER_TLS_CERT` | `/certs/cert.pem` | TLS 证书路径 |
| `CLIPPER_TLS_KEY` | `/certs/key.pem` | TLS 私钥路径 |
| `CLIPPER_TLS_REDIRECT` | `true` | 将 HTTP 重定向到 HTTPS |
| `CLIPPER_ACME_ENABLED` | `false` | 启用 Let's Encrypt |
| `CLIPPER_ACME_DOMAIN` | - | 证书域名 |
| `CLIPPER_ACME_EMAIL` | - | 联系邮箱 |
| `CLIPPER_ACME_STAGING` | `false` | 使用测试环境 |
| `CLIPPER_CERTS_DIR` | `/data/certs` | ACME 证书缓存 |
| `CLIPPER_BEARER_TOKEN` | - | 身份验证 Bearer 令牌（如设置，所有请求需要认证） |

#### Docker 卷

- `/data` - 数据库和文件的持久化存储
- `/certs` - 可选：在此挂载您的 TLS 证书

#### 多架构支持

Docker 镜像通过 Docker buildx 支持多架构：

```bash
# 构建多平台
docker buildx build --platform linux/amd64,linux/arm64 -t clipper-server .
```

## 许可证

请参阅主项目许可证。
