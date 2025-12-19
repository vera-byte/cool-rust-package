# Cool Admin Rust Packages 使用教程

> 🦀 完整的 cool-admin-midway Rust 版本使用指南

## 📋 目录

1. [环境准备](#环境准备)
2. [项目创建](#项目创建)
3. [核心库 (cool-core)](#核心库-cool-core)
4. [RPC 微服务 (cool-rpc)](#rpc-微服务-cool-rpc)
5. [任务队列 (cool-task)](#任务队列-cool-task)
6. [Elasticsearch (cool-es)](#elasticsearch-cool-es)
7. [插件系统 (cool-plugin)](#插件系统-cool-plugin)
8. [完整示例项目](#完整示例项目)
9. [常见问题](#常见问题)

---

## 🚀 环境准备

### 1. 安装 Rust

```bash
# 安装 Rust（如果未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 验证安装
rustc --version
cargo --version
```

### 2. 安装依赖服务

#### Redis（用于缓存、RPC、任务队列）

```bash
# macOS
brew install redis
brew services start redis

# Linux (Ubuntu/Debian)
sudo apt-get install redis-server
sudo systemctl start redis

# Docker
docker run -d -p 6379:6379 redis:latest
```

#### MySQL/PostgreSQL/SQLite（数据库）

```bash
# MySQL
brew install mysql  # macOS
# 或使用 Docker
docker run -d -p 3306:3306 -e MYSQL_ROOT_PASSWORD=root mysql:8.0

# PostgreSQL
brew install postgresql  # macOS
# 或使用 Docker
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:15

# SQLite（通常已内置，无需安装）
```

#### Elasticsearch（可选，用于搜索功能）

```bash
# Docker
docker run -d -p 9200:9200 -p 9300:9300 -e "discovery.type=single-node" elasticsearch:8.5.0
```

---

## 📦 项目创建

### 1. 创建新项目

```bash
# 创建新的 Rust 项目
cargo new my-cool-app
cd my-cool-app
```

### 2. 配置 Cargo.toml

编辑 `Cargo.toml`，添加依赖：

```toml
[package]
name = "my-cool-app"
version = "0.1.0"
edition = "2021"

[dependencies]
# Cool Admin 核心包
cool-core = { path = "../packages-rust/cool-core" }
cool-macros = { path = "../packages-rust/cool-macros" }
cool-rpc = { path = "../packages-rust/cool-rpc" }
cool-task = { path = "../packages-rust/cool-task" }
cool-es = { path = "../packages-rust/cool-es" }
cool-plugin = { path = "../packages-rust/cool-plugin" }

# Web 框架
salvo = { version = "0.72", features = ["cors", "logging", "jwt-auth", "cache", "oapi"] }

# ORM
sea-orm = { version = "1.1", features = [
    "sqlx-mysql",
    "sqlx-postgres",
    "sqlx-sqlite",
    "runtime-tokio-rustls",
    "macros",
    "with-chrono",
    "with-json",
    "with-uuid",
] }

# 异步运行时
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 其他工具
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
```

### 3. 项目结构

```
my-cool-app/
├── Cargo.toml
├── src/
│   ├── main.rs          # 应用入口
│   ├── entity/          # 实体定义
│   │   └── goods.rs
│   ├── service/         # 服务层
│   │   └── goods.rs
│   ├── controller/      # 控制器层
│   │   └── goods.rs
│   └── module/          # 模块配置
│       └── goods.rs
└── migrations/          # 数据库迁移（可选）
```

---

## 🔧 核心库 (cool-core)

### 1. 定义实体

创建 `src/entity/goods.rs`：

```rust
use cool_core::prelude::*;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "goods")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub title: String,
    pub price: Decimal,
    pub stock: i32,
    #[sea_orm(column_name = "createTime")]
    pub create_time: DateTimeUtc,
    #[sea_orm(column_name = "updateTime")]
    pub update_time: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// 实现 BaseEntity trait（通过宏自动生成）
#[derive(CoolEntity)]
#[cool_entity(table_name = "goods")]
pub struct GoodsEntity;
```

### 2. 创建服务

创建 `src/service/goods.rs`：

```rust
use cool_core::prelude::*;
use std::sync::Arc;
use sea_orm::DatabaseConnection;

pub struct GoodsService {
    db: Arc<DatabaseConnection>,
}

impl GoodsService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BaseService for GoodsService {
    fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    fn table_name(&self) -> &str {
        "goods"
    }

    // 可选：重写 modify_before 钩子
    async fn modify_before(&self, data: Value, modify_type: ModifyType) -> CoolResult<Value> {
        match modify_type {
            ModifyType::Add => {
                // 新增前处理，例如设置创建时间
                let mut data = data;
                if let Some(obj) = data.as_object_mut() {
                    obj.insert(
                        "createTime".to_string(),
                        json!(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
                    );
                }
                Ok(data)
            }
            ModifyType::Update => {
                // 更新前处理，例如设置更新时间
                let mut data = data;
                if let Some(obj) = data.as_object_mut() {
                    obj.insert(
                        "updateTime".to_string(),
                        json!(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
                    );
                }
                Ok(data)
            }
            _ => Ok(data),
        }
    }
}
```

### 3. 创建控制器

创建 `src/controller/goods.rs`：

```rust
use cool_core::prelude::*;
use cool_macros::cool_controller;
use crate::service::goods::GoodsService;

#[cool_controller(
    prefix = "/admin/goods",
    api = ["add", "delete", "update", "page", "info", "list"]
)]
pub struct GoodsController {
    service: GoodsService,
}

impl GoodsController {
    pub fn new(service: GoodsService) -> Self {
        Self { service }
    }
}
```

### 4. 配置应用

创建 `src/main.rs`：

```rust
use cool_core::prelude::*;
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志
    cool_core::init_logger();

    // 连接数据库
    let db_url = "mysql://root:root@localhost:3306/cool_db";
    let db = Database::connect(db_url).await.expect("数据库连接失败");
    let db = Arc::new(db);

    // 创建服务
    let goods_service = crate::service::goods::GoodsService::new(db.clone());

    // 创建控制器
    let goods_controller = crate::controller::goods::GoodsController::new(goods_service);

    // 创建应用
    let config = CoolConfig::default();
    let app = CoolApp::new(config)
        .database(db)
        .register(goods_controller);

    // 启动服务
    app.run("127.0.0.1:7001").await
}
```

### 5. 使用中间件

#### 权限中间件

```rust
use cool_core::middleware::authority;

// 在路由中使用
let router = Router::new()
    .hoop(authority::AuthorityMiddleware::new(authority_config))
    .push(goods_controller.router());
```

#### 日志中间件

```rust
use cool_core::middleware::request_log;

let router = Router::new()
    .hoop(request_log::RequestLogMiddleware)
    .push(goods_controller.router());
```

#### 异常过滤器

```rust
use cool_core::middleware::exception_filter;

let router = Router::new()
    .hoop(exception_filter::ExceptionFilter)
    .push(goods_controller.router());
```

### 6. 使用缓存

```rust
use cool_core::prelude::*;

// 使用内存缓存
let cache = MemoryCache::new();
cache.set("key", "value", Some(Duration::from_secs(60))).await?;
let value: Option<String> = cache.get("key").await?;

// 使用 Redis 缓存
let cache = RedisCache::new("redis://127.0.0.1:6379").await?;
cache.set("key", "value", Some(Duration::from_secs(60))).await?;
let value: Option<String> = cache.get("key").await?;
```

### 7. 使用事件系统

```rust
use cool_core::prelude::*;

// 订阅事件
global_event_manager().on("user.created", |event| {
    println!("用户创建事件: {:?}", event);
}).await;

// 发布事件
global_event_manager().emit("user.created", json!({
    "user_id": 1,
    "username": "admin"
})).await;
```

### 8. 使用 CLI 工具

```rust
use cool_core::prelude::*;

// 检查配置
check().await?;

// 生成实体文件
generate_entities_file("src/entity").await?;

// 清空实体文件
clear_entities_file("src/entity").await?;
```

---

## 🔌 RPC 微服务 (cool-rpc)

### 1. 创建 RPC 服务

```rust
use cool_rpc::prelude::*;
use serde_json::json;

#[cool_rpc_service(name = "user-service")]
pub struct UserRpcService {
    db: Arc<DatabaseConnection>,
}

impl UserRpcService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl RpcServiceHandler for UserRpcService {
    async fn handle(&self, method: &str, params: Value) -> RpcResult<Value> {
        match method {
            "getById" => {
                let id = params["id"].as_i64().unwrap();
                // 查询用户逻辑
                Ok(json!({ "id": id, "name": "张三" }))
            }
            "create" => {
                // 创建用户逻辑
                Ok(json!({ "id": 1 }))
            }
            _ => Err(RpcError::method_not_found(method)),
        }
    }
}
```

### 2. 创建 RPC Broker

```rust
use cool_rpc::prelude::*;

let rpc_config = RpcConfig {
    name: "user-service".to_string(),
    redis_url: "redis://127.0.0.1:6379".to_string(),
    ..Default::default()
};

let rpc = CoolRpc::new(rpc_config).await?;

// 注册服务
let user_service = UserRpcService::new(db.clone());
rpc.register("UserService", user_service).await?;

// 启动 RPC Broker
rpc.start().await?;
```

### 3. 调用远程服务

```rust
use cool_rpc::prelude::*;

let rpc = CoolRpc::new(rpc_config).await?;

// 调用远程服务
let result = rpc.call(
    "order-service",      // 目标服务名
    "OrderService",       // 服务类名
    "getById",           // 方法名
    json!({ "id": 1 })   // 参数
).await?;

println!("结果: {:?}", result);
```

### 4. RPC 事务

```rust
use cool_rpc::prelude::*;

#[cool_rpc_transaction]
async fn create_order(db: &DatabaseConnection, params: Value) -> CoolResult<Value> {
    // 函数会自动在事务中执行
    // 如果成功，会广播提交事件
    // 如果失败，会广播回滚事件
    
    // 创建订单逻辑
    Ok(json!({ "order_id": 1 }))
}
```

### 5. RPC 事件

```rust
use cool_rpc::prelude::*;

#[cool_rpc_event_handler(service = "user-service", event = "user.created")]
async fn on_user_created(event: &RpcEvent) -> RpcResult<()> {
    println!("用户创建事件: {:?}", event);
    Ok(())
}
```

---

## 📋 任务队列 (cool-task)

### 1. 定义任务

```rust
use cool_task::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EmailJob {
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[async_trait]
impl JobHandler for EmailJob {
    async fn handle(&self) -> JobResult<()> {
        // 发送邮件逻辑
        println!("发送邮件到: {}, 主题: {}", self.to, self.subject);
        Ok(())
    }
}
```

### 2. 创建队列

```rust
use cool_task::prelude::*;

#[cool_queue(
    type = "comm",
    retries = 3,
    timeout = 60,
    concurrency = 4
)]
pub struct EmailQueue;

// 创建队列实例
let task_config = TaskConfig {
    redis_url: "redis://127.0.0.1:6379".to_string(),
    ..Default::default()
};

let queue = EmailQueue::build_queue(task_config).await?;
```

### 3. 添加任务

```rust
use cool_task::prelude::*;

let job = EmailJob {
    to: "user@example.com".to_string(),
    subject: "测试邮件".to_string(),
    body: "这是一封测试邮件".to_string(),
};

queue.add(job, JobOptions::default()).await?;
```

### 4. 定时任务（Cron）

```rust
use cool_task::prelude::*;

let scheduler = Scheduler::new(task_config.clone()).await?;

// 添加定时任务（每天凌晨执行）
scheduler.schedule(
    "0 0 * * *",  // Cron 表达式
    EmailJob { /* ... */ },
    JobOptions::default()
).await?;

// 启动调度器
scheduler.start().await?;
```

---

## 🔍 Elasticsearch (cool-es)

### 1. 创建 ES 客户端

```rust
use cool_es::prelude::*;

let es_config = EsConfig {
    nodes: vec!["http://localhost:9200".to_string()],
    ..Default::default()
};

let client = EsClient::new(&es_config.nodes).await?;
```

### 2. 定义索引

```rust
use cool_es::prelude::*;

#[cool_es_index(
    name = "users",
    shards = 8,
    replicas = 1
)]
pub struct UserIndex;
```

### 3. 索引管理

```rust
use cool_es::prelude::*;

// 创建索引
let index_manager = IndexManager::new(client.clone());
index_manager.create_index("users", json!({
    "mappings": {
        "properties": {
            "name": { "type": "text" },
            "age": { "type": "integer" }
        }
    }
})).await?;

// 删除索引
index_manager.delete_index("users").await?;
```

### 4. 索引文档

```rust
use cool_es::prelude::*;

// 索引文档
client.index("users", "1", json!({
    "name": "张三",
    "age": 25
})).await?;

// 更新文档
client.update("users", "1", json!({
    "age": 26
})).await?;

// 删除文档
client.delete("users", "1").await?;
```

### 5. 搜索

```rust
use cool_es::prelude::*;

// 使用搜索构建器
let builder = SearchBuilder::new("users");
let results = builder
    .query(json!({
        "match": { "name": "张三" }
    }))
    .size(10)
    .from(0)
    .execute(&client)
    .await?;

println!("搜索结果: {:?}", results);
```

---

## 🔌 插件系统 (cool-plugin)

### 1. 定义插件

```rust
use cool_plugin::prelude::*;

#[derive(Default)]
pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "my-plugin".to_string(),
            key: "my-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: Some("我的插件".to_string()),
            author: Some("Cool Admin".to_string()),
            ..Default::default()
        }
    }

    async fn ready(&mut self) -> PluginResult<()> {
        println!("插件已就绪");
        Ok(())
    }

    async fn stop(&mut self) -> PluginResult<()> {
        println!("插件已停止");
        Ok(())
    }
}
```

### 2. 注册插件

```rust
use cool_plugin::prelude::*;

let registry = global_plugin_registry();
let plugin = MyPlugin::default();
registry.register(plugin).await?;
```

### 3. 使用钩子

```rust
use cool_plugin::prelude::*;

// 定义上传钩子
pub struct MyUploadHook;

#[async_trait]
impl UploadHook for MyUploadHook {
    async fn upload(&self, ctx: &UploadContext) -> UploadResult<String> {
        // 上传逻辑
        Ok("上传成功".to_string())
    }
}

// 注册钩子
let hook = MyUploadHook;
global_plugin_registry().register_hook("upload", hook).await?;
```

### 4. 插件安装器

```rust
use cool_plugin::prelude::*;

let installer = PluginInstaller::new("plugins".to_string());

// 检查插件
let check_result = installer.check("my-plugin.zip").await?;
println!("检查结果: {:?}", check_result);

// 安装插件
let plugin_data = installer.install("my-plugin.zip").await?;
println!("插件数据: {:?}", plugin_data);
```

---

## 📝 完整示例项目

### 项目结构

```
my-cool-app/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── entity/
│   │   ├── mod.rs
│   │   └── goods.rs
│   ├── service/
│   │   ├── mod.rs
│   │   └── goods.rs
│   ├── controller/
│   │   ├── mod.rs
│   │   └── goods.rs
│   └── module/
│       ├── mod.rs
│       └── goods.rs
└── migrations/
    └── 001_create_goods.sql
```

### main.rs

```rust
use cool_core::prelude::*;
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;

mod entity;
mod service;
mod controller;
mod module;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志
    cool_core::init_logger();

    // 连接数据库
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:root@localhost:3306/cool_db".to_string());
    
    let db = Database::connect(db_url).await.expect("数据库连接失败");
    let db = Arc::new(db);

    // 创建服务
    let goods_service = service::goods::GoodsService::new(db.clone());

    // 创建控制器
    let goods_controller = controller::goods::GoodsController::new(goods_service);

    // 创建应用
    let config = CoolConfig::default();
    let app = CoolApp::new(config)
        .database(db)
        .register(goods_controller);

    // 启动服务
    let addr = std::env::var("ADDR").unwrap_or_else(|_| "127.0.0.1:7001".to_string());
    tracing::info!("🚀 服务启动在: http://{}", addr);
    
    app.run(&addr).await
}
```

### 运行项目

```bash
# 开发模式
cargo run

# 发布模式
cargo build --release
./target/release/my-cool-app
```

---

## ❓ 常见问题

### 1. 数据库连接失败

**问题**：`数据库连接失败`

**解决方案**：
- 检查数据库服务是否启动
- 检查连接字符串是否正确
- 检查数据库用户权限

### 2. Redis 连接失败

**问题**：`Redis 连接失败`

**解决方案**：
- 检查 Redis 服务是否启动：`redis-cli ping`
- 检查 Redis URL 是否正确
- 检查防火墙设置

### 3. 宏编译错误

**问题**：`proc_macro` 相关错误

**解决方案**：
- 确保 `cool-macros` 依赖已正确添加
- 检查 Rust 版本：`rustc --version`（建议 1.70+）
- 清理并重新编译：`cargo clean && cargo build`

### 4. 类型不匹配

**问题**：类型不匹配错误

**解决方案**：
- 检查实体字段类型是否与数据库表结构一致
- 使用 `#[sea_orm(column_name = "...")]` 指定列名
- 检查序列化/反序列化 trait 是否正确实现

### 5. 路由未生效

**问题**：路由访问 404

**解决方案**：
- 检查控制器是否正确注册
- 检查路由前缀是否正确
- 检查中间件是否拦截了请求

---

## 📚 更多资源

- [API 文档](https://docs.rs/cool-core)（待发布）
- [示例代码](../examples)
- [问题反馈](https://github.com/cool-team-official/cool-admin-midway/issues)

---

## 🎯 下一步

1. 阅读 [DETAILED_COMPARISON.md](./DETAILED_COMPARISON.md) 了解与 TypeScript 版本的对比
2. 查看 [FEATURE_COMPARISON.md](./FEATURE_COMPARISON.md) 了解功能完整度
3. 探索更多高级功能：事件系统、模块管理、EPS 等

---

**祝您使用愉快！** 🎉

