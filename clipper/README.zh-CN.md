# Clipper 桌面应用

使用 Tauri 2 + React + TypeScript 构建的跨平台剪贴板管理器。

## 功能特性

### 核心功能
- **剪贴板监控** - 自动捕获剪贴板中的文本和图片
- **全文搜索** - 强大的搜索功能快速找到任何剪贴
- **标签和收藏** - 使用标签整理剪贴并标记收藏
- **文件附件** - 在文本剪贴旁存储文件
- **实时同步** - 基于 WebSocket 的跨设备同步

### 服务器选项
- **内置服务器** - 包含自动启动的 clipper-server，无需配置
- **外部服务器** - 连接到远程 clipper-server 用于团队/多设备使用
- **网络访问** - 启用局域网访问以在本地网络共享剪贴

### 用户界面
- **系统托盘** - 后台运行，从托盘快速访问
- **主题支持** - 浅色、深色和自动（跟随系统）主题
- **国际化** - 支持英语和中文
- **通知提示** - 可配置的通知系统
- **无限滚动** - 大量剪贴集合的流畅滚动
- **图片预览** - 点击预览图片剪贴
- **拖放支持** - 直接将文件拖放到应用中
- **视觉淡出** - 接近自动清理日期的剪贴逐渐淡出以指示即将过期

### 平台支持
- **macOS** - 完整支持，含开机启动
- **Windows** - 完整支持，含开机启动
- **Linux** - 完整支持，含开机启动

## 快速开始

### 前提条件

- Node.js 18+
- Rust 1.91+
- 平台特定依赖（参见 [Tauri 前提条件](https://tauri.app/start/prerequisites/)）

### 安装

```bash
# 克隆仓库
git clone https://github.com/user/clipper.git
cd clipper/clipper

# 安装依赖
npm install

# 开发模式运行
npm run tauri:dev

# 生产构建
npm run tauri:build
```

## 配置

设置存储在平台特定位置：
- **macOS**: `~/Library/Application Support/com.0d0a.clipper/settings.json`
- **Linux**: `~/.config/com.0d0a.clipper/settings.json`
- **Windows**: `%APPDATA%\com.0d0a.clipper\settings.json`

### 设置项

| 设置 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `serverAddress` | string | `http://localhost:3000` | 外部服务器 URL |
| `useBundledServer` | boolean | `true` | 使用内置或外部服务器 |
| `listenOnAllInterfaces` | boolean | `false` | 允许局域网访问（内置服务器） |
| `theme` | string | `auto` | 主题："light"、"dark" 或 "auto" |
| `language` | string | `null` | 语言："en"、"zh" 或 null（自动） |
| `openOnStartup` | boolean | `true` | 应用启动时显示窗口 |
| `startOnLogin` | boolean | `false` | 系统登录时启动应用 |
| `notificationsEnabled` | boolean | `true` | 显示通知提示 |
| `defaultSaveLocation` | string | `null` | 文件下载默认路径 |

### 自签名证书支持

连接使用自签名证书的外部 HTTPS 服务器时：

1. 应用显示安全对话框，包含：
   - 证书未由受信任 CA 签名的警告
   - 可能的原因（自签名、未知 CA 或潜在中间人攻击）
   - 服务器的 SHA-256 指纹供验证

2. 您可以选择信任该证书，将其保存以供后续连接使用

3. 如果之前信任的证书发生变化，系统会提示您验证新指纹

受信任的证书存储在设置文件的 `trustedCertificates` 中。

## 架构

```
clipper/
├── src/                    # React 前端
│   ├── components/         # React 组件
│   │   ├── ClipList.tsx
│   │   ├── ClipEntry.tsx
│   │   ├── SearchBox.tsx
│   │   ├── SettingsDialog.tsx
│   │   └── ...
│   ├── i18n/              # 国际化
│   └── App.tsx
├── src-tauri/             # Tauri 后端（Rust）
│   └── src/
│       ├── lib.rs         # 应用设置和插件
│       ├── commands.rs    # Tauri 命令
│       ├── state.rs       # 应用状态
│       ├── server.rs      # 内置服务器管理
│       ├── settings.rs    # 设置持久化
│       ├── clipboard.rs   # 剪贴板监控
│       ├── websocket.rs   # WebSocket 客户端
│       ├── tray.rs        # 系统托盘
│       └── autolaunch.rs  # 开机启动设置
└── package.json
```

## 开发

### 前端开发

前端使用以下技术构建：
- **React 19** - UI 组件
- **TypeScript** - 类型安全
- **Vite** - 快速开发构建
- **CSS** - 样式（无框架）

### 后端开发

Tauri 后端使用：
- **clipper-client** - 服务器通信
- **arboard** - 剪贴板访问
- **tokio** - 异步运行时
- **tauri-plugin-shell** - sidecar 管理

### 构建服务器二进制文件

内置服务器二进制文件在 Tauri 构建过程中自动构建：

```bash
npm run build:server  # 为当前平台构建 clipper-server
npm run tauri:build   # 完整生产构建
```

## Tauri 命令

以下命令可通过 `invoke()` 调用：

### 剪贴管理
- `list_clips(filters, page, page_size)` - 分页列出剪贴
- `search_clips(query, filters, page, page_size)` - 搜索剪贴
- `create_clip(content, tags, additional_notes)` - 创建新剪贴
- `update_clip(id, tags, additional_notes)` - 更新剪贴元数据
- `delete_clip(id)` - 删除剪贴
- `get_clip(id)` - 按 ID 获取剪贴
- `copy_to_clipboard(content)` - 复制内容到系统剪贴板
- `upload_file(path, tags, additional_notes)` - 上传文件作为剪贴
- `download_file(clip_id, filename)` - 下载文件附件

### 设置
- `get_settings()` - 获取当前设置
- `save_settings(settings)` - 保存设置
- `browse_directory()` - 打开文件夹选择对话框
- `check_auto_launch_status()` - 检查是否启用开机启动

### 服务器管理
- `get_server_url()` - 获取当前服务器 URL
- `is_bundled_server()` - 检查是否使用内置服务器
- `switch_to_bundled_server()` - 切换到内置服务器
- `switch_to_external_server(server_url)` - 切换到外部服务器
- `clear_all_data()` - 清除所有剪贴并重启服务器
- `toggle_listen_on_all_interfaces(listen_on_all)` - 切换局域网访问
- `get_local_ip_addresses()` - 获取本机局域网 IP 地址
- `update_tray_language(language)` - 更新托盘菜单语言

## 事件

应用向前端发出以下事件：

| 事件 | 载荷 | 描述 |
|------|------|------|
| `new-clip` | `{ id, content, tags }` | WebSocket 新剪贴（触发列表刷新） |
| `clip-updated` | `{ id }` | WebSocket 剪贴更新（触发列表刷新） |
| `clip-deleted` | `{ id }` | WebSocket 剪贴删除（触发列表刷新） |
| `clips-cleaned-up` | `{ ids, count }` | WebSocket 旧剪贴清理（触发列表刷新） |
| `clip-created` | `{ id, ... }` | 剪贴板监控创建的剪贴 |
| `open-settings` | - | 从托盘请求打开设置 |
| `server-switched` | - | 服务器模式已更改 |
| `data-cleared` | - | 所有数据已清除 |

## 许可证

请参阅主项目许可证。
