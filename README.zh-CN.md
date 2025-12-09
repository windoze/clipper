# Clipper

一款现代化、跨平台的剪贴板管理器，支持全文搜索、实时同步，拥有精美的桌面界面。

[![Homepage](https://img.shields.io/badge/homepage-clipper.unwritten.codes-blue)](https://clipper.unwritten.codes)
![Version](https://img.shields.io/badge/version-0.19.3-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)

[English](README.md) | 简体中文

## 功能特性

- **剪贴板监控** - 自动捕获剪贴板中的文本和图片
- **全文搜索** - 使用强大的 BM25 排序算法即时查找任何剪贴内容
- **标签与收藏** - 使用标签整理剪贴内容，标记你的收藏
- **文件附件** - 在文本剪贴旁存储文件
- **实时同步** - 基于 WebSocket 的跨设备同步
- **内置服务器** - 零配置启动，内嵌服务器
- **局域网共享** - 在本地网络中共享剪贴内容
- **HTTPS/TLS 支持** - 支持手动证书或 Let's Encrypt 自动证书的安全连接
- **自签名证书支持** - 类似 SSH 的指纹验证方式信任自签名证书
- **身份验证** - 可选的 Bearer Token 认证，保护 API 安全
- **自动清理** - 根据保留策略自动删除旧剪贴内容
- **剪贴分享** - 通过短链接公开分享剪贴内容，支持设置过期时间
- **Web 界面** - 浏览器访问，支持拖放上传文件
- **多语言支持** - 中英文界面
- **主题支持** - 浅色、深色和自动主题
- **跨平台** - 支持 macOS、Windows 和 Linux

## 快速开始

### 下载

从 [Releases](https://github.com/windoze/clipper/releases) 页面下载适合你平台的最新版本。

> **注意：** macOS 二进制文件已签名并经过公证。Windows 和 Linux 二进制文件未进行代码签名。请参阅[平台说明](#平台说明)了解各平台的运行方法。

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/windoze/clipper.git
cd clipper

# 构建桌面应用
cd clipper
npm install
npm run tauri:build
```

## 架构

Clipper 是一个模块化的 Rust 工作空间，包含六个主要组件：

```
clipper/
├── clipper-indexer/     # 核心库 - SurrealDB 存储和全文搜索
├── clipper-server/      # REST API + WebSocket 服务器 (Axum)，含内置 Web 界面
├── clipper-client/      # Rust 客户端库
├── clipper-cli/         # 命令行界面
├── clipper/             # 桌面应用 (Tauri 2 + React + TypeScript)
├── clipper-slint/       # 备选 GUI (Slint UI，未完成)
└── packages/clipper-ui/ # 共享的 React UI 组件
```

### 技术栈

| 组件 | 技术 |
|------|------|
| 核心存储 | SurrealDB + RocksDB 后端 |
| 全文搜索 | SurrealDB FTS + BM25 排序 |
| 文件存储 | object_store (本地文件系统) |
| 服务器 | Axum + Tower 中间件 |
| 桌面前端 | React 19 + TypeScript + Vite |
| 桌面后端 | Tauri 2 |
| 命令行 | clap |

## 桌面应用

桌面应用提供功能完整的剪贴板管理器，拥有现代化的界面。

### 功能

- **系统托盘** - 后台运行，快速访问
- **剪贴板监控** - 自动捕获文本和图片
- **无限滚动** - 流畅浏览大量内容
- **图片预览** - 点击预览图片剪贴
- **拖放上传** - 直接将文件拖入应用
- **开机启动** - 系统登录时自动启动

### 设置

| 设置项 | 说明 |
|--------|------|
| 服务器模式 | 内置（自动）或外部服务器 |
| 网络访问 | 允许局域网访问以进行多设备同步 |
| 内置服务器令牌 | 内置服务器的认证令牌（启用网络访问时显示） |
| 外部服务器令牌 | 连接外部服务器的认证令牌 |
| 主题 | 浅色、深色或自动（跟随系统） |
| 语言 | 中文或英文 |
| 通知 | 开启/关闭消息通知 |
| 开机启动 | 系统登录时自动启动 |

## 服务器

服务器可以独立运行，也可以与桌面应用捆绑运行。

### 独立服务器

```bash
# 使用默认配置运行
cargo run --bin clipper-server

# 使用自定义配置
cargo run --bin clipper-server -- \
  --db-path ./data/db \
  --storage-path ./data/storage \
  --port 3000
```

### 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `CLIPPER_DB_PATH` | `./data/db` | 数据库目录 |
| `CLIPPER_STORAGE_PATH` | `./data/storage` | 文件存储目录 |
| `CLIPPER_LISTEN_ADDR` | `0.0.0.0` | 服务器绑定地址 |
| `PORT` | `3000` | 服务器端口 |
| `CLIPPER_CLEANUP_ENABLED` | `false` | 启用自动清理 |
| `CLIPPER_CLEANUP_RETENTION_DAYS` | `30` | 剪贴保留天数 |
| `CLIPPER_CLEANUP_INTERVAL_HOURS` | `24` | 清理间隔小时数 |
| `CLIPPER_BEARER_TOKEN` | - | 身份验证令牌 |
| `CLIPPER_SHORT_URL_BASE` | - | 分享短链接基础 URL（启用分享功能） |
| `CLIPPER_SHORT_URL_EXPIRATION_HOURS` | `24` | 短链接默认过期时间（小时） |

### 身份验证

通过设置 Bearer Token 启用身份验证：

```bash
# 设置令牌以启用身份验证
cargo run --bin clipper-server -- --bearer-token your-secret-token

# 或通过环境变量
CLIPPER_BEARER_TOKEN=your-secret-token cargo run --bin clipper-server
```

启用身份验证后，所有 API 请求必须包含令牌：

```bash
curl -H "Authorization: Bearer your-secret-token" http://localhost:3000/clips
```

### TLS/HTTPS 配置

如需安全连接，请使用 TLS 功能构建：

```bash
# 手动证书
cargo build -p clipper-server --features tls

# 自动 Let's Encrypt 证书
cargo build -p clipper-server --features acme

# 完整 TLS 支持（含安全存储）
cargo build -p clipper-server --features full-tls
```

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `CLIPPER_TLS_ENABLED` | `false` | 启用 HTTPS |
| `CLIPPER_TLS_PORT` | `443` | HTTPS 端口 |
| `CLIPPER_TLS_CERT` | - | 证书路径 (PEM) |
| `CLIPPER_TLS_KEY` | - | 私钥路径 (PEM) |
| `CLIPPER_ACME_ENABLED` | `false` | 启用 Let's Encrypt |
| `CLIPPER_ACME_DOMAIN` | - | 证书域名 |
| `CLIPPER_ACME_EMAIL` | - | 联系邮箱 |

### Docker 部署

```bash
# 构建镜像
docker build -t clipper-server .

# 运行容器
docker run -d -p 3000:3000 -v clipper-data:/data clipper-server

# 访问 http://localhost:3000
```

### REST API

| 端点 | 方法 | 说明 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/clips` | GET | 列出剪贴（分页） |
| `/clips` | POST | 创建文本剪贴 |
| `/clips/upload` | POST | 上传文件剪贴 |
| `/clips/search` | GET | 搜索剪贴（分页） |
| `/clips/:id` | GET | 根据 ID 获取剪贴 |
| `/clips/:id` | PUT | 更新剪贴元数据 |
| `/clips/:id` | DELETE | 删除剪贴 |
| `/clips/:id/file` | GET | 下载文件附件 |
| `/clips/:id/short-url` | POST | 创建分享短链接 |
| `/s/:code` | GET | 解析短链接（公开） |
| `/export` | GET | 导出所有剪贴为 tar.gz 归档 |
| `/import` | POST | 从 tar.gz 归档导入剪贴 |
| `/ws` | WS | 实时通知 |

## 命令行工具

命令行界面提供对所有功能的完整访问。

```bash
# 创建剪贴
clipper-cli create "Hello, World!" --tags greeting,example

# 搜索剪贴
clipper-cli search "hello" --page 1 --page-size 20

# 监听实时更新
clipper-cli watch

# 获取特定剪贴
clipper-cli get <clip-id>

# 更新剪贴元数据
clipper-cli update <clip-id> --tags updated,important

# 删除剪贴
clipper-cli delete <clip-id>

# 分享剪贴（需要服务器设置 CLIPPER_SHORT_URL_BASE）
clipper-cli share <clip-id> --expires 48

# 导出所有剪贴到归档
clipper-cli export -o backup.tar.gz

# 从归档导入剪贴
clipper-cli import backup.tar.gz
```

### 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `CLIPPER_URL` | `http://localhost:3000` | 服务器地址 |
| `CLIPPER_TOKEN` | - | 身份验证令牌 |

### 身份验证

连接启用身份验证的服务器：

```bash
# 使用命令行选项
clipper-cli --token your-secret-token search "hello"

# 使用环境变量
CLIPPER_TOKEN=your-secret-token clipper-cli search "hello"
```

## 客户端库

使用 Rust 客户端库将 Clipper 集成到你的应用中。

```rust
use clipper_client::{ClipperClient, SearchFilters};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端（可选身份验证令牌）
    let client = ClipperClient::new("http://localhost:3000")
        .with_token("your-secret-token".to_string()); // 可选

    // 创建剪贴
    let clip = client
        .create_clip(
            "Hello, World!".to_string(),
            vec!["greeting".to_string()],
            None,
        )
        .await?;

    // 分页搜索
    let result = client
        .search_clips("Hello", SearchFilters::new(), 1, 20)
        .await?;

    println!("找到 {} 条剪贴", result.total);

    // 订阅实时更新
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    client.subscribe_notifications(tx).await?;

    while let Some(notification) = rx.recv().await {
        println!("更新: {:?}", notification);
    }

    Ok(())
}
```

## 开发

### 前置条件

- Rust 1.91+
- Node.js 18+
- Tauri 平台特定依赖（[查看文档](https://tauri.app/start/prerequisites/)）

### 构建

```bash
# 构建整个工作空间
cargo build --workspace

# 构建特定包
cargo build -p clipper-indexer
cargo build -p clipper-server
cargo build -p clipper-client
cargo build -p clipper-cli

# 构建桌面应用
cd clipper && npm install && npm run tauri:build

# 发布版本构建
cargo build --workspace --release
```

### 测试

```bash
# 运行所有测试
cargo test --workspace

# 运行服务器测试（顺序执行）
cargo test -p clipper-server -- --test-threads=1

# 运行客户端测试（顺序执行）
cargo test -p clipper-client -- --test-threads=1
```

## 项目结构

```
clipper/
├── CLAUDE.md              # AI 助手开发指南
├── Cargo.toml             # 工作空间配置
├── LICENSE                # MIT 许可证
├── README.md              # 英文说明文档
├── README.zh-CN.md        # 本文件
├── clipper/               # Tauri 桌面应用
│   ├── src/               # React 前端
│   ├── src-tauri/         # Tauri 后端 (Rust)
│   └── package.json
├── clipper-indexer/       # 核心索引库
│   ├── src/
│   └── README.md
├── clipper-server/        # REST API 服务器
│   ├── src/
│   └── README.md
├── clipper-client/        # Rust 客户端库
│   ├── src/
│   └── README.md
├── clipper-cli/           # 命令行界面
│   ├── src/
│   └── README.md
└── clipper-slint/         # 备选 Slint GUI
    └── src/
```

## 平台说明

### macOS

macOS 二进制文件已**签名并经过 Apple 公证**。首次启动时，macOS 可能会显示一个对话框，提示该应用来自已认证的开发者 - 点击**"打开"**即可继续。

可用格式：
- **DMG** - 磁盘映像，拖拽安装
- **app.zip** - 压缩的应用包，用于手动安装

### Windows

Windows SmartScreen 可能会显示警告，提示该应用来自"未知发布者"。

**解决方法：**

1. 当 SmartScreen 弹窗出现时，点击**"更多信息"**
2. 点击**"仍要运行"**

或者，你可以右键点击可执行文件，选择**属性**，然后在"常规"选项卡底部勾选**"解除锁定"**。

### Linux

Linux 通常没有相同的签名限制，但你可能需要使 AppImage 可执行：

```bash
chmod +x Clipper.AppImage
./Clipper.AppImage
```

如果遇到权限问题，也可以运行：

```bash
# 对于 AppImage
chmod +x Clipper*.AppImage

# 对于 .deb 包
sudo dpkg -i clipper_*.deb

# 对于 .rpm 包
sudo rpm -i clipper-*.rpm
```

## 安全注意事项

> **警告**：剪贴板是你电脑上最敏感的数据流之一。它经常包含密码、API 密钥、私人消息、财务信息和其他机密数据。Clipper 的设计会捕获并持久化存储所有剪贴板内容。请像对待密码管理器一样重视 Clipper 数据的安全性。

Clipper 会存储剪贴板历史记录，其中可能包含敏感信息。了解潜在的安全风险非常重要：

| 条件 | 潜在安全事件 |
|------|-------------|
| 服务器暴露到网络但未启用身份验证 | 未授权访问所有剪贴板历史，包括密码和敏感数据 |
| 在不可信网络上未使用 TLS | 中间人攻击可拦截剪贴板数据和身份验证令牌 |
| Bearer Token 过弱或泄露 | 完全访问权限：可读取、修改和删除所有剪贴内容 |
| 分享包含敏感内容的短链接 | 机密信息永久公开暴露 |
| 数据库/存储目录对所有用户可读 | 本地用户可访问所有剪贴板历史 |
| 剪贴板监控处于敏感工作流程中 | 密码、API 密钥、机密信息被自动捕获并持久化存储 |
| 备份归档存储不安全 | 备份泄露时完整剪贴板历史暴露 |

### 服务器安全

- **网络绑定**：默认情况下，服务器绑定到 `0.0.0.0`，可在所有网络接口上访问。如仅需本地使用，请设置 `CLIPPER_LISTEN_ADDR=127.0.0.1`。
- **身份验证**：将服务器暴露到网络时，务必启用 Bearer Token 身份验证（`CLIPPER_BEARER_TOKEN`）。未启用身份验证时，任何能访问网络的人都可以读取和修改你的剪贴板历史。
- **TLS/HTTPS**：在不可信网络上运行时使用 TLS 加密。可配置手动证书或自动 Let's Encrypt 证书。注意：ACME/Let's Encrypt 需要服务器通过 80 和 443 端口从公网可访问以进行域名验证。对于私有网络或 NAT 环境，请使用自己的证书或自签名证书。
- **短链接**：通过短链接分享的剪贴内容无需身份验证即可公开访问。请使用适当的过期时间，仅分享非敏感内容。
- **数据存储**：所有剪贴板数据存储在本地。确保数据库和存储目录具有适当的文件系统权限，不应对所有用户可读。
- **公网部署**：将 Clipper 暴露到公共互联网时，**务必同时启用 TLS 和身份验证**。如果你有域名，可使用内置的 ACME 支持自动获取和续期 Let's Encrypt 证书，或使用[反向代理](https://www.cloudflare.com/zh-cn/learning/cdn/glossary/reverse-proxy/)（如 Nginx 或 Caddy）进行 TLS 终止。
    **中国大陆用户注意**：在中国境内的服务器需要完成 [ICP 备案](https://beian.miit.gov.cn/)后才能开放 HTTP/HTTPS 公网访问，因此使用 ACME 功能前必须先完成备案。

### 客户端安全

- **自签名证书**：CLI 和桌面应用都支持类似 SSH 的指纹验证来信任自签名证书。首次连接时务必验证指纹与服务器匹配。
- **令牌存储**：Bearer Token 存储在设置文件中。确保此文件具有适当的权限（仅当前用户可读）。
- **剪贴板监控**：桌面应用持续监控系统剪贴板。请注意，复制的密码、API 密钥和其他敏感数据都会被捕获并存储。
- **内置服务器令牌**：在桌面应用中启用网络访问时，会生成一个随机的 Bearer Token 并显示在设置中。

### 通用建议

1. **私有网络**：仅在可信网络中暴露 Clipper，或使用 VPN
2. **强密码令牌**：使用长且随机的 Bearer Token 进行身份验证
3. **定期清理**：启用自动清理以限制历史剪贴板数据的暴露
4. **备份安全**：导出的归档包含所有剪贴板历史（包括附件）- 请安全存储备份
5. **敏感数据**：注意你复制到剪贴板的数据敏感性；Clipper 会无差别地存储所有内容

## 错误报告

你的错误报告有助于我们改进 Clipper。如果你遇到任何问题，请向我们报告，以便我们调查和修复。

### 如何报告

在我们的 [GitHub Issues](https://github.com/windoze/clipper/issues) 页面提交错误报告。

### 报告内容

请在错误报告中包含以下信息：

- **系统信息**：操作系统及版本、Clipper 版本、服务器部署方式（内置/独立/Docker）
- **重现步骤**：清晰的、有编号的重现问题的步骤
- **预期行为**：你期望发生什么
- **实际行为**：实际发生了什么
- **截图或录屏**：如适用，请提供问题的视觉证据

### 启用调试日志

调试日志为故障排查提供了有价值的信息。以下是启用方法：

**对于独立服务器 (`clipper-server`)：**

在启动服务器前设置 `RUST_LOG` 环境变量：

```bash
RUST_LOG=clipper_server=debug,tower_http=debug cargo run --bin clipper-server
```

**对于桌面应用（GUI）：**

编辑配置目录中的 `settings.json` 文件，添加：

```json
{
  "debug_logging": true
}
```

配置目录位置：
- **macOS**: `~/Library/Application Support/codes.unwritten.clipper/settings.json`
- **Windows**: `%APPDATA%\codes.unwritten.clipper\settings.json`
- **Linux**: `~/.config/codes.unwritten.clipper/settings.json`

然后重启应用。日志文件位置：
- **macOS**: `~/Library/Logs/codes.unwritten.clipper/clipper.log`
- **Windows**: `%LOCALAPPDATA%\codes.unwritten.clipper\logs\clipper.log`
- **Linux**: `~/.local/share/codes.unwritten.clipper/logs/clipper.log`

> **⚠️ 隐私警告**：调试日志可能包含你剪贴板历史中的敏感信息，包括密码、令牌和个人数据。**在提交错误报告前，请检查并删除日志中的任何敏感内容。**

## 贡献

欢迎贡献！请随时提交 Pull Request。

1. Fork 本仓库
2. 创建你的功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交你的更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启一个 Pull Request

## 许可证

本项目基于 MIT 许可证授权 - 详见 [LICENSE](LICENSE) 文件。

## 致谢

- [SurrealDB](https://surrealdb.com/) - 多模型数据库
- [Tauri](https://tauri.app/) - 桌面应用框架
- [Axum](https://github.com/tokio-rs/axum) - Web 框架
- [React](https://react.dev/) - UI 库
- [Nerd Fonts](https://www.nerdfonts.com/) - Symbols Nerd Font Mono 图标/Powerline 符号字体（MIT 许可证）
