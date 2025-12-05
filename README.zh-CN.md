# Clipper

一款现代化、跨平台的剪贴板管理器，支持全文搜索、实时同步，拥有精美的桌面界面。

[![Homepage](https://img.shields.io/badge/homepage-clipper.unwritten.codes-blue)](https://clipper.unwritten.codes)
![Version](https://img.shields.io/badge/version-0.16.0-blue)
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

从 [Releases](https://github.com/user/clipper/releases) 页面下载适合你平台的最新版本。

> **注意：** macOS 二进制文件已签名并经过公证。Windows 和 Linux 二进制文件未进行代码签名。请参阅[平台说明](#平台说明)了解各平台的运行方法。

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/user/clipper.git
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

- Rust 1.70+
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
