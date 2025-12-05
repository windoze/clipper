# Clipper CLI

用于与 Clipper 服务器管理剪贴板条目的命令行界面工具。

## 功能特性

- **创建剪贴** - 从命令行创建带标签和备注的剪贴
- **搜索剪贴** - 支持全文搜索和筛选
- **列出和筛选** - 按标签和日期范围筛选剪贴
- **更新元数据** - 更新剪贴的标签和备注
- **删除剪贴** - 按 ID 删除剪贴
- **监听模式** - 实时接收剪贴通知
- **分页支持** - 搜索和列表操作支持分页
- **身份验证** - 支持需要身份验证的服务器
- **多种输出格式** - JSON（默认）或纯文本

## 安装

从源码构建：

```bash
cargo build --release -p clipper-cli
```

二进制文件将生成在 `target/release/clipper-cli`。

## 配置

CLI 可以通过环境变量配置：

- `CLIPPER_URL` - 服务器 URL（默认：`http://localhost:3000`）
- `CLIPPER_TOKEN` - 用于身份验证的 Bearer 令牌（可选）

示例：
```bash
export CLIPPER_URL=http://clipper-server.local:8080
export CLIPPER_TOKEN=your-secret-token
```

## 使用方法

### 前提条件

使用 CLI 前必须先启动 Clipper 服务器：

```bash
cargo run --bin clipper-server
```

或使用自定义配置：
```bash
CLIPPER_DB_PATH=./data/db cargo run --bin clipper-server
```

### 基本命令

```bash
clipper-cli [选项] <命令>

选项：
  -u, --url <URL>      服务器 URL [环境变量: CLIPPER_URL] [默认: http://localhost:3000]
  -t, --token <TOKEN>  身份验证的 Bearer 令牌 [环境变量: CLIPPER_TOKEN]
  -h, --help           打印帮助信息
```

## 命令详解

### create - 创建新剪贴

```bash
clipper-cli create <CONTENT> [选项]

参数：
  <CONTENT>  剪贴内容（文本）

选项：
  -t, --tags <TAGS>              标签（逗号分隔）
  -n, --notes <NOTES>            备注
  -h, --help                     打印帮助信息

示例：
  # 简单剪贴
  clipper-cli create "Hello, World!"

  # 带标签
  clipper-cli create "重要会议笔记" --tags work,meeting

  # 带标签和备注
  clipper-cli create "TODO: 审查 PR" --tags todo,urgent --notes "周五前完成"

  # 从 stdin 管道输入内容
  echo "剪贴板内容" | xargs clipper-cli create
```

**输出**：包含创建的剪贴详情的 JSON

### get - 按 ID 获取剪贴

```bash
clipper-cli get <ID> [选项]

参数：
  <ID>  剪贴 ID

选项：
  -f, --format <FORMAT>  输出格式：json 或 text [默认: json]
  -h, --help             打印帮助信息

示例：
  # 获取 JSON 格式
  clipper-cli get abc123

  # 仅获取内容（文本格式）
  clipper-cli get abc123 --format text

  # 保存内容到文件
  clipper-cli get abc123 --format text > output.txt
```

### search - 搜索剪贴

```bash
clipper-cli search <QUERY> [选项]

参数：
  <QUERY>  搜索查询

选项：
  -t, --tags <TAGS>                  按标签筛选（逗号分隔）
      --start-date <START_DATE>      按开始日期筛选（ISO 8601 格式）
      --end-date <END_DATE>          按结束日期筛选（ISO 8601 格式）
  -p, --page <PAGE>                  页码 [默认: 1]
      --page-size <PAGE_SIZE>        每页条目数 [默认: 20]
  -f, --format <FORMAT>              输出格式：json 或 text [默认: json]
  -h, --help                         打印帮助信息

示例：
  # 基本搜索
  clipper-cli search hello

  # 带标签筛选的搜索
  clipper-cli search meeting --tags work

  # 多条件搜索
  clipper-cli search report --tags work,important --start-date 2025-11-01T00:00:00Z

  # 分页搜索
  clipper-cli search todo --page 2 --page-size 10

  # 文本输出（更易于解析）
  clipper-cli search notes --format text
```

**输出**：
- JSON 格式：包含元数据的完整分页结果
- 文本格式：每个条目一行（ID + 内容），分页信息输出到 stderr

### update - 更新剪贴元数据

```bash
clipper-cli update <ID> [选项]

参数：
  <ID>  剪贴 ID

选项：
  -t, --tags <TAGS>      新标签（逗号分隔）
  -n, --notes <NOTES>    新备注
  -h, --help             打印帮助信息

示例：
  # 更新标签
  clipper-cli update abc123 --tags done,archived

  # 更新备注
  clipper-cli update abc123 --notes "于 2025-11-26 完成"

  # 同时更新
  clipper-cli update abc123 --tags work,completed --notes "已完成"
```

**注意**：必须至少提供 `--tags` 或 `--notes` 之一。

### delete - 删除剪贴

```bash
clipper-cli delete <ID>

参数：
  <ID>  剪贴 ID

示例：
  clipper-cli delete abc123
```

### watch - 监听实时通知

```bash
clipper-cli watch

示例：
  # 监听并显示所有剪贴事件
  clipper-cli watch

  # 使用 jq 筛选事件（需要安装 jq）
  clipper-cli watch | jq 'select(.type == "new_clip")'

  # 保存事件到文件
  clipper-cli watch > clips.ndjson
```

**输出**：NDJSON（换行分隔的 JSON）- 每行一个 JSON 对象

通知类型：
```json
{"type":"new_clip","id":"abc123","content":"Hello","tags":["greeting"]}
{"type":"updated_clip","id":"abc123"}
{"type":"deleted_clip","id":"abc123"}
{"type":"clips_cleaned_up","ids":["abc123","def456"],"count":2}
```

## 输出格式

### JSON 格式（默认）

美化打印的 JSON，包含完整剪贴详情：

```json
{
  "id": "abc123",
  "content": "Hello, World!",
  "created_at": "2025-11-26T10:00:00Z",
  "tags": ["greeting"],
  "additional_notes": "一条友好的消息"
}
```

对于搜索/列表命令，包含分页元数据：

```json
{
  "items": [...],
  "total": 100,
  "page": 1,
  "page_size": 20,
  "total_pages": 5
}
```

### 文本格式

便于处理的纯文本输出：

```bash
# get 命令 - 仅输出内容
Hello, World!

# search 命令 - 输出 ID 和内容
abc123
Hello, World!

def456
另一个剪贴
```

分页信息打印到 stderr，不会干扰内容的管道传输。

## 分页

搜索和列表操作支持分页：

```bash
# 获取第一页（默认：20 条）
clipper-cli search "query"

# 获取第二页，自定义页面大小
clipper-cli search "query" --page 2 --page-size 50

# 大页面用于批量操作
clipper-cli search "query" --page-size 100
```

## 高级用法

### 脚本示例

从文件创建多个剪贴：
```bash
while IFS= read -r line; do
  clipper-cli create "$line" --tags imported
done < input.txt
```

搜索并删除旧剪贴：
```bash
clipper-cli search "" --tags temporary --end-date 2025-11-01T00:00:00Z --format json | \
  jq -r '.items[].id' | \
  while read id; do
    clipper-cli delete "$id"
  done
```

实时监控新剪贴：
```bash
clipper-cli watch | jq 'select(.type == "new_clip") | .content'
```

导出所有剪贴到 JSON：
```bash
# 先获取总页数
total_pages=$(clipper-cli search "" --page 1 --page-size 100 --format json | jq '.total_pages')

# 获取所有页
for page in $(seq 1 $total_pages); do
  clipper-cli search "" --page $page --page-size 100 --format json | jq '.items[]'
done > all_clips.json
```

### 与其他工具集成

**fzf 集成**（模糊搜索）：
```bash
clipper-cli search "" --format text | fzf
```

**rofi 集成**（GUI 菜单）：
```bash
clip_id=$(clipper-cli search "" --format text | rofi -dmenu -i -p "剪贴:" | head -1)
clipper-cli get "$clip_id" --format text | xclip -selection clipboard
```

**复制到系统剪贴板**：
```bash
# Linux (X11)
clipper-cli get abc123 --format text | xclip -selection clipboard

# macOS
clipper-cli get abc123 --format text | pbcopy

# WSL
clipper-cli get abc123 --format text | clip.exe
```

## 错误处理

CLI 返回适当的退出码：
- `0` - 成功
- `1` - 错误（连接失败、剪贴未找到、输入无效等）

错误信息打印到 stderr 并带有上下文：
```
Error: Failed to get clip

Caused by:
    404 Not Found: Clip not found: abc123
```

## 环境变量

- `CLIPPER_URL` - 服务器 URL（可用 `-u` 标志覆盖）
- `CLIPPER_TOKEN` - 身份验证的 Bearer 令牌（可用 `-t` 标志覆盖）
- `RUST_LOG` - 调试日志级别（例如 `RUST_LOG=debug clipper-cli search test`）

## 身份验证

如果服务器需要身份验证，请提供 Bearer 令牌：

```bash
# 使用命令行选项
clipper-cli --token your-secret-token search "hello"

# 使用环境变量
export CLIPPER_TOKEN=your-secret-token
clipper-cli search "hello"

# 一次性使用环境变量
CLIPPER_TOKEN=your-secret-token clipper-cli search "hello"
```

令牌会作为 `Authorization: Bearer <token>` 头随所有请求发送。

## 自签名证书支持

连接使用自签名证书的 HTTPS 服务器时，clipper-cli 提供类似 SSH 的证书验证：

1. **首次连接**：CLI 检测到不受信任的证书并显示：
   - 证书未由受信任 CA 签名的警告
   - 可能的原因（自签名、未知 CA 或潜在中间人攻击）
   - 服务器的 SHA-256 证书指纹供验证

2. **用户确认**：系统会提示您确认：`Are you sure you want to continue connecting (yes/no)?`

3. **信任持久化**：如果确认，指纹会保存到设置文件以供后续连接使用

4. **安全警告**：如果之前信任的证书指纹发生变化，CLI 会显示醒目警告（类似 SSH 的"REMOTE HOST IDENTIFICATION HAS CHANGED"）并中止连接

示例输出：
```
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
@    WARNING: UNTRUSTED SERVER CERTIFICATE!              @
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@

The server's certificate is not signed by a trusted Certificate Authority (CA).
This could mean:
  - The server is using a self-signed certificate
  - The server's CA is not in your system's trust store
  - Someone may be intercepting your connection (man-in-the-middle attack)

Host: example.com:443
SHA-256 Fingerprint:
  AB:CD:EF:12:34:56:...

Are you sure you want to continue connecting (yes/no)?
```

受信任的证书存储在 settings.json 文件的 `trustedCertificates` 中（与 Clipper 桌面应用共享）。

## 系统要求

- Rust 1.91 或更高版本（用于构建）
- 运行中的 clipper-server 实例
- 与服务器的网络连接

## 常见问题

**连接被拒绝**：
```
Error: Failed to create clip
Caused by: error sending request for url (http://localhost:3000/clips): connection error: Connection refused
```
→ 确保服务器正在运行：`cargo run --bin clipper-server`

**无效日期格式**：
```
Error: Invalid start_date format, use ISO 8601
```
→ 使用 RFC3339/ISO 8601 格式：`2025-11-26T10:00:00Z`

**服务器 URL 未找到**：
→ 设置正确的服务器 URL：`clipper-cli -u http://your-server:3000 search test`

**401 未授权**：
```
Error: Failed to search clips
Caused by: 401 Unauthorized: Invalid or missing authentication token
```
→ 服务器需要身份验证。提供令牌：`clipper-cli --token your-secret-token search test`

## 开发

从源码运行：
```bash
cargo run --bin clipper-cli -- create "测试剪贴"
```

运行测试：
```bash
cargo test -p clipper-cli
```

构建发布版本：
```bash
cargo build --release -p clipper-cli
./target/release/clipper-cli --help
```

## 相关项目

- **clipper-server** - REST API 服务器后端
- **clipper-client** - Rust 客户端库
- **clipper-indexer** - 核心索引和搜索库

## 许可证

请参阅主项目许可证。
