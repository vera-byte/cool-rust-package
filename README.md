# Cool Admin Rust Packages

> 🦀 cool-admin-midway 的 Rust 实现版本

## 📦 包列表

| 包名 | 描述 | 对应 TS 包 |
|------|------|-----------|
| `cool-core` | 核心库：CRUD、服务基类、控制器、中间件 | `@cool-midway/core` |
| `cool-macros` | 过程宏：装饰器风格的代码生成 | - |
| `cool-task` | 任务队列：基于 Redis 的分布式任务 | `@cool-midway/task` |
| `cool-rpc` | RPC 微服务：服务发现、远程调用 | `@cool-midway/rpc` |
| `cool-es` | Elasticsearch 集成 | `@cool-midway/es` |
| `cool-plugin` | 插件系统：生命周期管理、钩子 | `@cool-midway/plugin-cli` |

## 🚀 快速开始

### 添加依赖

```toml
[dependencies]
cool-core = { path = "../packages-rust/cool-core" }
cool-macros = { path = "../packages-rust/cool-macros" }
```

### 示例代码

```rust
use cool_core::prelude::*;
use sea_orm::entity::prelude::*;

// 定义实体
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "goods")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub title: String,
    pub price: Decimal,
    #[sea_orm(column_name = "createTime")]
    pub create_time: DateTimeUtc,
    #[sea_orm(column_name = "updateTime")]
    pub update_time: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// 定义服务
pub struct GoodsService {
    db: Arc<DatabaseConnection>,
}

// 定义控制器
#[cool_controller(
    prefix = "/admin/goods",
    api = [add, delete, update, page, info, list]
)]
pub struct GoodsController {
    service: GoodsService,
}
```

## 🏗️ 项目结构

```
packages-rust/
├── Cargo.toml              # 工作区配置
├── README.md
├── cool-core/              # 核心库
│   ├── src/
│   │   ├── lib.rs          # 库入口
│   │   ├── cache/          # 缓存模块
│   │   ├── config/         # 配置模块
│   │   ├── constant/       # 常量定义
│   │   ├── controller/     # 控制器基类
│   │   ├── entity/         # 实体定义
│   │   ├── error/          # 错误处理
│   │   ├── event/          # 事件系统
│   │   ├── middleware/     # 中间件
│   │   ├── module/         # 模块管理
│   │   ├── service/        # 服务基类
│   │   └── util/           # 工具函数
│   └── Cargo.toml
├── cool-macros/            # 过程宏
│   ├── src/
│   │   └── lib.rs          # 宏定义
│   └── Cargo.toml
├── cool-task/              # 任务队列
│   ├── src/
│   │   ├── lib.rs
│   │   ├── job.rs          # 任务定义
│   │   ├── queue.rs        # 队列实现
│   │   ├── worker.rs       # Worker
│   │   └── scheduler.rs    # 调度器
│   └── Cargo.toml
├── cool-rpc/               # RPC 微服务
│   ├── src/
│   │   ├── lib.rs
│   │   ├── broker.rs       # RPC Broker
│   │   ├── service.rs      # 服务定义
│   │   └── event.rs        # 事件
│   └── Cargo.toml
├── cool-es/                # Elasticsearch
│   ├── src/
│   │   ├── lib.rs
│   │   ├── client.rs       # ES 客户端
│   │   ├── index.rs        # 索引管理
│   │   └── search.rs       # 搜索构建器
│   └── Cargo.toml
└── cool-plugin/            # 插件系统
    ├── src/
    │   ├── lib.rs
    │   ├── plugin.rs       # 插件定义
    │   ├── registry.rs     # 注册表
    │   └── hook.rs         # 钩子系统
    └── Cargo.toml
```

## 🔧 技术栈

| 领域 | TypeScript | Rust |
|------|-----------|------|
| Web 框架 | Midway.js (Koa) | Salvo |
| ORM | TypeORM | SeaORM |
| 任务队列 | BullMQ | 自实现 (Redis) |
| RPC | Moleculer | 自实现 (Redis) |
| 缓存 | cache-manager | Redis + 内存 |
| 验证 | class-validator | validator |
| JWT | jsonwebtoken | jsonwebtoken |
| 日志 | @midwayjs/logger | tracing |

## 📚 模块对照

### cool-core

| TS 模块 | Rust 模块 | 说明 |
|---------|-----------|------|
| `decorator/controller.ts` | `controller/mod.rs` | CRUD 控制器 |
| `service/base.ts` | `service/base.rs` | 服务基类 |
| `entity/base.ts` | `entity/mod.rs` | 实体基类 |
| `exception/*` | `error/mod.rs` | 异常处理 |
| `middleware/authority.ts` | `middleware/authority.rs` | 权限中间件 |
| `middleware/log.ts` | `middleware/log.rs` | 日志中间件 |
| `cache/store.ts` | `cache/mod.rs` | 缓存存储 |
| `event/index.ts` | `event/mod.rs` | 事件系统 |
| `module/*` | `module/mod.rs` | 模块管理 |
| `interface.ts` | `config/mod.rs` | 配置定义 |
| `constant/global.ts` | `constant/mod.rs` | 全局常量 |
| `util/*` | `util/mod.rs` | 工具函数 |

### cool-task

| TS 模块 | Rust 模块 | 说明 |
|---------|-----------|------|
| `base.ts` | `job.rs` | 任务基类 |
| `queue.ts` | `queue.rs` + `worker.rs` | 队列和 Worker |
| `decorator/queue.ts` | - | 宏实现 |

### cool-rpc

| TS 模块 | Rust 模块 | 说明 |
|---------|-----------|------|
| `rpc.ts` | `broker.rs` | RPC Broker |
| `service/*` | `service.rs` | 服务定义 |
| `decorator/event/*` | `event.rs` | 事件 |

## 🔨 构建

```bash
# 构建所有包
cd packages-rust
cargo build

# 运行测试
cargo test

# 检查
cargo clippy
```

## 📖 文档

生成文档：

```bash
cargo doc --open
```

## 📄 许可证

MIT License

