# Clipper Indexer

使用 SurrealDB 和 object_store 进行剪贴板条目索引和搜索的 Rust 库。

## 功能特性

- **持久化存储** - 使用 SurrealDB 和 RocksDB 后端实现可靠的数据持久化
- **全文搜索** - 由 SurrealDB 的全文搜索引擎驱动，支持 BM25 排名
- **分页支持** - 搜索和列表操作内置分页
- **文件附件** - 使用 object_store crate 存储和检索文件
- **灵活筛选** - 按日期范围、标签和全文查询搜索
- **类型安全** - 完全类型化的 API 和完善的错误处理

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
clipper_indexer = { path = "../clipper_indexer" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## 快速开始

```rust
use clipper_indexer::{ClipperIndexer, SearchFilters, PagingParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化索引器
    let indexer = ClipperIndexer::new("./db", "./storage").await?;

    // 添加文本条目
    let entry = indexer
        .add_entry_from_text(
            "Hello, World!".to_string(),
            vec!["greeting".to_string()],
            Some("一条友好的消息".to_string()),
        )
        .await?;

    println!("创建的条目 ID: {}", entry.id);

    // 分页搜索条目
    let paging = PagingParams { page: 1, page_size: 20 };
    let result = indexer
        .search_entries("Hello", SearchFilters::new(), paging)
        .await?;

    println!("找到 {} 个条目（第 {} 页，共 {} 页）",
             result.total, result.page, result.total_pages);

    Ok(())
}
```

## API 概览

### 初始化

```rust
let indexer = ClipperIndexer::new(db_path, storage_path).await?;
```

### 从文本添加条目

```rust
let entry = indexer
    .add_entry_from_text(
        content,
        tags,
        optional_notes,
    )
    .await?;
```

### 从文件添加条目

```rust
let entry = indexer
    .add_entry_from_file(
        file_path,
        tags,
        optional_notes,
    )
    .await?;
```

文件内容使用 `object_store` 存储，文件路径保存在条目中。

### 从文件内容添加条目

用于上传的文件或内存中的内容：

```rust
let entry = indexer
    .add_entry_from_file_content(
        file_bytes,
        filename,
        tags,
        optional_notes,
    )
    .await?;
```

### 检索条目

```rust
let entry = indexer.get_entry(&entry_id).await?;
```

### 更新条目

```rust
let updated = indexer
    .update_entry(
        &entry_id,
        Some(new_tags),
        Some(new_notes),
    )
    .await?;
```

### 分页搜索条目

带可选筛选和分页的全文搜索：

```rust
use chrono::{Duration, Utc};

let filters = SearchFilters::new()
    .with_tags(vec!["rust".to_string()])
    .with_date_range(
        Utc::now() - Duration::days(7),
        Utc::now(),
    );

let paging = PagingParams {
    page: 1,
    page_size: 20,
};

let result = indexer
    .search_entries("search query", filters, paging)
    .await?;

println!("第 {} 页，共 {} 页，条目总数: {}",
         result.page, result.total_pages, result.total);

for entry in result.items {
    println!("- {}: {}", entry.id, entry.content);
}
```

### 分页列出条目

带筛选列出条目（不使用全文搜索）：

```rust
let filters = SearchFilters::new()
    .with_tags(vec!["important".to_string()]);

let paging = PagingParams::default(); // page: 1, page_size: 20

let result = indexer.list_entries(filters, paging).await?;

println!("显示 {} / {} 个条目",
         result.items.len(), result.total);
```

### 获取文件内容

对于有文件附件的条目：

```rust
if let Some(file_key) = entry.file_attachment {
    let content = indexer.get_file_content(&file_key).await?;
    // 使用字节内容
}
```

### 删除条目

```rust
indexer.delete_entry(&entry_id).await?;
```

这也会删除任何关联的文件附件。

### 清理旧条目

删除超过指定天数的条目（不包括带有 "favorite" 等有意义标签的条目）：

```rust
// 清理超过 30 天的条目，返回已删除条目的 ID
let deleted_ids = indexer.cleanup_entries(30).await?;
println!("已清理 {} 个旧条目", deleted_ids.len());
```

## 分页

库为搜索和列表操作提供内置分页支持：

### PagingParams

```rust
pub struct PagingParams {
    pub page: usize,        // 页码（从 1 开始）
    pub page_size: usize,   // 每页条目数
}

// 默认：page 1, page_size 20
let paging = PagingParams::default();

// 自定义分页
let paging = PagingParams { page: 2, page_size: 50 };
```

### PagedResult

```rust
pub struct PagedResult<T> {
    pub items: Vec<T>,       // 当前页的条目
    pub total: usize,        // 条目总数
    pub page: usize,         // 当前页码
    pub page_size: usize,    // 每页条目数
    pub total_pages: usize,  // 总页数
}
```

## 数据库架构

库会自动创建以下架构：

### 表: clipboard

| 字段 | 类型 | 描述 |
|------|------|------|
| id | string | 唯一标识符 (UUID) |
| content | string | 条目的文本内容 |
| created_at | datetime | 创建时间戳 |
| tags | array\<string\> | 标签列表 |
| additional_notes | option\<string\> | 可选备注 |
| file_attachment | option\<string\> | 可选文件存储键 |
| search_content | string | 用于全文搜索的组合内容 |

### 索引

- `idx_created_at`: `created_at` 字段索引，用于高效日期范围查询
- `idx_tags`: `tags` 字段索引，用于标签筛选
- `idx_search_content`: 全文搜索索引，支持 BM25 排名和高亮

## 示例

查看 [examples](./examples) 目录获取更详细的使用示例：

```bash
cargo run --example basic_usage
```

## 测试

运行完整测试套件：

```bash
cargo test
```

测试覆盖：
- 从文本和文件添加条目
- 从文件内容（字节）添加条目
- 检索和更新条目
- 带分页的全文搜索功能
- 日期范围和标签筛选
- 文件存储和检索
- 条目删除
- 分页边界情况

## 错误处理

库提供完善的错误类型：

```rust
pub enum IndexerError {
    Database(surrealdb::Error),
    ObjectStore(object_store::Error),
    Io(std::io::Error),
    NotFound(String),
    Serialization(String),
    InvalidInput(String),
}
```

所有操作返回 `Result<T, IndexerError>`。

## 架构

- **数据库层** - SurrealDB 和 RocksDB 后端提供 ACID 事务和强大查询能力
- **存储层** - object_store 通过简洁的抽象处理文件持久化
- **搜索** - 由 SurrealDB 内置 FTS 和自定义分析器驱动的全文搜索
- **分页** - 带元数据计算的高效 LIMIT/OFFSET 查询
- **模型** - 使用 serde 序列化的类型安全数据模型

## 性能考虑

- **分页** - 使用 SQL LIMIT 和 OFFSET 实现高效分页检索
- **索引** - 自动为 created_at、tags 和 search_content 建立索引以加快查询
- **文件存储** - 大文件与数据库分开存储以保持查询性能
- **搜索** - BM25 排名即使在大数据集下也能提供相关结果

## 系统要求

- Rust 1.91 或更高版本
- Tokio 运行时（异步）
- 足够的磁盘空间用于数据库和文件存储

## 许可证

请参阅主项目许可证。
