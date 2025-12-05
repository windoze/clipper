# Clipper Client

用于与 Clipper 服务器 REST API 和 WebSocket 通知交互的 Rust 客户端库。

## 功能特性

- **完整的 REST API 支持** - 创建、读取、更新、删除剪贴
- **搜索和筛选** - 支持日期范围和标签筛选的全文搜索
- **分页支持** - 搜索和列表操作内置分页
- **实时通知** - WebSocket 支持实时剪贴更新
- **文件操作** - 上传文件和下载附件
- **身份验证支持** - 可选的 Bearer 令牌身份验证
- **异步/等待** - 基于 Tokio 构建，实现高效异步 I/O
- **类型安全** - 强类型 API 和完善的错误处理

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
clipper-client = { path = "../clipper-client" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## 快速开始

```rust
use clipper_client::{ClipperClient, SearchFilters};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端（可选身份验证）
    let client = ClipperClient::new("http://localhost:3000")
        .with_token("your-secret-token".to_string()); // 可选

    // 创建剪贴
    let clip = client
        .create_clip(
            "Hello, World!".to_string(),
            vec!["greeting".to_string()],
            Some("我的第一个剪贴".to_string()),
        )
        .await?;

    println!("创建的剪贴 ID: {}", clip.id);

    // 分页搜索剪贴
    let result = client
        .search_clips("Hello", SearchFilters::new(), 1, 20)
        .await?;

    println!("找到 {} 个剪贴（第 {} 页，共 {} 页）",
             result.total, result.page, result.total_pages);

    Ok(())
}
```

## API 参考

### 创建剪贴

```rust
let clip = client
    .create_clip(
        content: String,
        tags: Vec<String>,
        additional_notes: Option<String>,
    )
    .await?;
```

### 上传文件

```rust
let file_content = std::fs::read("document.txt")?;

let clip = client
    .upload_file(
        file_content,
        "document.txt".to_string(),
        vec!["documents".to_string()],
        Some("重要文档".to_string()),
    )
    .await?;
```

### 按 ID 获取剪贴

```rust
let clip = client.get_clip("clip_id").await?;
```

### 更新剪贴

```rust
let updated = client
    .update_clip(
        "clip_id",
        Some(vec!["new_tag".to_string()]),
        Some("更新的备注".to_string()),
    )
    .await?;
```

### 搜索剪贴

```rust
use clipper_client::SearchFilters;
use chrono::{Duration, Utc};

// 带筛选和分页的搜索
let filters = SearchFilters::new()
    .with_tags(vec!["important".to_string()])
    .with_start_date(Utc::now() - Duration::days(7))
    .with_end_date(Utc::now());

let result = client.search_clips("query", filters, 1, 20).await?;
println!("第 {} 页，共 {} 页，总计: {}", result.page, result.total_pages, result.total);

for clip in result.items {
    println!("- {}: {}", clip.id, clip.content);
}
```

### 列出剪贴

```rust
// 分页列出所有剪贴
let result = client.list_clips(SearchFilters::new(), 1, 20).await?;

// 带筛选列出
let filters = SearchFilters::new()
    .with_tags(vec!["work".to_string()]);
let result = client.list_clips(filters, 1, 50).await?;
```

### 删除剪贴

```rust
client.delete_clip("clip_id").await?;
```

## 身份验证

如果服务器需要身份验证，使用 `with_token()` 方法：

```rust
use clipper_client::ClipperClient;

let client = ClipperClient::new("http://localhost:3000")
    .with_token("your-secret-token".to_string());

// 后续所有请求都会包含 Authorization 头
let clips = client.list_clips(SearchFilters::new(), 1, 20).await?;
```

令牌会自动：
- 作为 `Authorization: Bearer <token>` 头发送给 REST API 请求
- WebSocket 连接后作为基于消息的身份验证发送
- 作为 `?token=<token>` 查询参数附加到文件下载

## WebSocket 通知

在剪贴创建、更新或删除时接收实时更新：

```rust
use clipper_client::{ClipperClient, ClipNotification};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClipperClient::new("http://localhost:3000");

    // 创建用于接收通知的通道
    let (tx, mut rx) = mpsc::unbounded_channel();

    // 订阅通知（返回任务句柄）
    let handle = client.subscribe_notifications(tx).await?;

    // 处理通知
    tokio::spawn(async move {
        while let Some(notification) = rx.recv().await {
            match notification {
                ClipNotification::NewClip { id, content, tags } => {
                    println!("新剪贴创建: {} - {}", id, content);
                }
                ClipNotification::UpdatedClip { id } => {
                    println!("剪贴已更新: {}", id);
                }
                ClipNotification::DeletedClip { id } => {
                    println!("剪贴已删除: {}", id);
                }
                ClipNotification::ClipsCleanedUp { ids, count } => {
                    println!("{} 个旧剪贴已清理", count);
                }
            }
        }
    });

    // 保持连接活跃
    handle.await??;

    Ok(())
}
```

## 错误处理

客户端提供完善的错误类型：

```rust
use clipper_client::ClientError;

match client.get_clip("id").await {
    Ok(clip) => println!("获取到剪贴: {}", clip.content),
    Err(ClientError::NotFound(msg)) => println!("未找到: {}", msg),
    Err(ClientError::BadRequest(msg)) => println!("错误请求: {}", msg),
    Err(ClientError::ServerError { status, message }) => {
        println!("服务器错误 {}: {}", status, message)
    }
    Err(e) => println!("错误: {}", e),
}
```

## 测试

库包含需要运行 clipper-server 的完整集成测试：

```bash
# 启动服务器（在另一个终端）
cargo run --bin clipper-server

# 运行测试
cargo test -p clipper-client --test integration_tests -- --test-threads=1
```

测试覆盖：
- 创建带可选字段和不带可选字段的剪贴
- 上传文件（文本和二进制）
- 按 ID 获取剪贴
- 更新剪贴元数据
- 带筛选的搜索和列出
- 删除剪贴
- 所有操作的 WebSocket 通知

**共 18 个测试 - 全部通过 ✓**

## 示例

### 完整 CRUD 示例

```rust
use clipper_client::ClipperClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端（服务器需要身份验证时可选令牌）
    let client = ClipperClient::new("http://localhost:3000");
    // .with_token("your-secret-token".to_string()); // 如果服务器需要身份验证则取消注释

    // 创建
    let clip = client
        .create_clip(
            "我的重要笔记".to_string(),
            vec!["work".to_string(), "important".to_string()],
            Some("别忘了！".to_string()),
        )
        .await?;

    println!("已创建: {}", clip.id);

    // 读取
    let retrieved = client.get_clip(&clip.id).await?;
    println!("内容: {}", retrieved.content);

    // 更新
    let updated = client
        .update_clip(
            &clip.id,
            Some(vec!["work".to_string(), "done".to_string()]),
            Some("已完成".to_string()),
        )
        .await?;

    println!("更新后的标签: {:?}", updated.tags);

    // 删除
    client.delete_clip(&clip.id).await?;
    println!("已删除");

    Ok(())
}
```

### 文件上传示例

```rust
use clipper_client::ClipperClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClipperClient::new("http://localhost:3000");

    // 从磁盘读取文件
    let file_content = std::fs::read("report.pdf")?;

    // 上传文件
    let clip = client
        .upload_file(
            file_content,
            "report.pdf".to_string(),
            vec!["reports".to_string(), "monthly".to_string()],
            Some("2025年11月报告".to_string()),
        )
        .await?;

    println!("文件已上传为剪贴: {}", clip.id);
    println!("文件存储位置: {:?}", clip.file_attachment);

    Ok(())
}
```

### 多条件筛选和分页搜索

```rust
use clipper_client::{ClipperClient, SearchFilters};
use chrono::{Duration, Utc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClipperClient::new("http://localhost:3000");

    // 搜索过去一周内带特定标签的剪贴
    let filters = SearchFilters::new()
        .with_start_date(Utc::now() - Duration::days(7))
        .with_end_date(Utc::now())
        .with_tags(vec!["important".to_string(), "work".to_string()]);

    // 分页搜索（第1页，每页20条）
    let result = client.search_clips("meeting", filters, 1, 20).await?;

    println!("共找到 {} 个剪贴", result.total);
    for clip in result.items {
        println!("找到: {} - {:?}", clip.content, clip.tags);
    }

    Ok(())
}
```

## 架构

- **HTTP 客户端** - 使用 `reqwest` 进行 REST API 调用
- **WebSocket** - 使用 `tokio-tungstenite` 进行实时通知
- **异步运行时** - 基于 Tokio 实现高效异步操作
- **类型安全** - 使用 serde 序列化的强类型模型

## 系统要求

- Rust 1.91 或更高版本
- Tokio 运行时
- 运行中的 clipper-server 实例

## 许可证

请参阅主项目许可证。
