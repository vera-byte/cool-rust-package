//! # cool-core
//!
//! cool-admin Rust 核心库，提供 CRUD 自动化、服务基类、控制器基类等功能。
//!
//! ## 功能特性
//!
//! - 🚀 基于 Salvo 的高性能 Web 框架集成
//! - 🗃️ 基于 SeaORM 的多数据库支持（MySQL/PostgreSQL/SQLite）
//! - 🔧 自动 CRUD 生成（add/delete/update/page/info/list）
//! - 🛡️ 统一的异常处理
//! - 📝 参数验证
//! - 🔐 JWT 认证
//! - 💾 Redis 缓存支持
//! - 📡 事件系统
//! - 🔌 模块化架构
//!
//! ## 快速开始
//!
//! ```rust,ignore
//! use cool_core::prelude::*;
//!
//! // 定义实体
//! #[derive(Clone, Debug, DeriveEntityModel, Serialize, Deserialize)]
//! #[sea_orm(table_name = "goods")]
//! pub struct Model {
//!     #[sea_orm(primary_key)]
//!     pub id: i64,
//!     pub title: String,
//!     pub price: Decimal,
//!     pub create_time: DateTimeUtc,
//!     pub update_time: DateTimeUtc,
//! }
//!
//! // 定义服务
//! pub struct GoodsService {
//!     db: Arc<DatabaseConnection>,
//! }
//!
//! // 定义控制器
//! pub struct GoodsController {
//!     service: GoodsService,
//! }
//! ```
//!
//! ## 模块说明
//!
//! - `cache`: 缓存模块，支持内存缓存和 Redis 缓存
//! - `config`: 配置模块，定义框架配置结构
//! - `constant`: 常量模块，定义全局常量和枚举
//! - `controller`: 控制器模块，提供 CRUD 控制器基类
//! - `entity`: 实体模块，定义实体 trait 和通用结构
//! - `error`: 错误处理模块，统一的异常类型和响应
//! - `event`: 事件模块，提供事件发布订阅机制
//! - `middleware`: 中间件模块，包含权限、日志等中间件
//! - `module`: 模块管理，支持模块注册和路由构建
//! - `service`: 服务模块，提供 CRUD 服务基类
//! - `util`: 工具模块，提供常用工具函数

pub mod cache;
#[cfg(feature = "cli")]
pub mod cli;
pub mod config;
pub mod constant;
pub mod controller;
pub mod entity;
pub mod eps;
pub mod error;
pub mod event;
pub mod middleware;
pub mod module;
pub mod service;
pub mod tag;
pub mod util;

/// 预导入模块，包含常用类型和 trait
pub mod prelude {
    #![allow(ambiguous_glob_reexports)]
    // 缓存
    pub use crate::cache::{CacheFactory, CacheStore, MemoryCache, RedisCache};

    // 配置
    pub use crate::config::*;

    // 常量
    pub use crate::constant::*;

    // 控制器
    pub use crate::controller::{BaseController, ControllerOption, CrudApi};

    // 实体
    pub use crate::entity::*;

    // 错误处理
    pub use crate::error::{CoolError, CoolResponse, CoolResult, PageResult, Pagination};

    // 事件
    pub use crate::event::{global_event_manager, EventManager};

    // 中间件
    pub use crate::middleware::{
        authority, exception_filter, request_log, AdminInfo, AppUserInfo, AuthorityConfig,
        DepotAppUserExt, DepotExt, ExceptionFilter, JwtClaims,
    };

    // 模块
    pub use crate::module::{global_module_registry, Module, ModuleInfo, ModuleRegistry};

    // 服务
    pub use crate::service::{BaseService, ModifyType, SimpleService};

    // 标签与 EPS
    pub use crate::eps::{
        global_eps_registry, ColumnInfo as EpsColumnInfo, ControllerEps, EpsRegistry, EpsScope,
        ModuleInfo as EpsModuleInfo, QueryOpInfo as EpsQueryOpInfo, RouteInfo as EpsRouteInfo,
    };
    pub use crate::tag::{global_url_tag_store, TagType, UrlTagStore};

    // 工具
    pub use crate::util;

    // CLI 工具
    #[cfg(feature = "cli")]
    pub use crate::cli::{check, clear_entities_file, generate_entities_file};

    // 重导出常用依赖
    pub use async_trait::async_trait;
    pub use salvo::prelude::*;
    pub use sea_orm::{
        entity::prelude::*, ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection,
        EntityTrait, IntoActiveModel, ModelTrait, PaginatorTrait, QueryFilter, QueryOrder,
        QuerySelect,
    };
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{json, Value};
    pub use validator::Validate;
}

/// 框架版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 初始化日志
pub fn init_logger() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("cool_core=info".parse().unwrap()))
        .init();
}

/// 应用构建器
pub struct CoolApp {
    /// 配置
    pub config: config::CoolConfig,
    /// 数据库连接
    pub db: Option<sea_orm::DatabaseConnection>,
    /// 模块注册表
    pub modules: module::ModuleRegistry,
    /// 事件管理器
    pub events: event::EventManager,
}

impl CoolApp {
    /// 创建新的应用
    pub fn new(config: config::CoolConfig) -> Self {
        Self {
            config,
            db: None,
            modules: module::ModuleRegistry::new(),
            events: event::EventManager::new(),
        }
    }

    /// 设置数据库连接
    pub fn database(mut self, db: sea_orm::DatabaseConnection) -> Self {
        self.db = Some(db);
        self
    }

    /// 注册模块
    pub fn register<M: module::Module + 'static>(self, module: M) -> Self {
        self.modules.register(module);
        self
    }

    /// 构建路由
    pub fn build_router(&self, prefix: &str) -> salvo::Router {
        self.modules.build_router(prefix)
    }

    /// 启动服务
    pub async fn run(self, addr: impl Into<String>) -> std::io::Result<()> {
        use salvo::prelude::*;
        let addr = addr.into();
        // 初始化所有模块
        if let Err(e) = self.modules.init_all() {
            tracing::error!("模块初始化失败: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ));
        }

        // 构建路由
        let router = self.build_router("/");
        let doc = OpenApi::new("openapi", "0.0.1").merge_router(&router);
        let router = router
            .unshift(doc.into_router("/api-doc/openapi.json"))
            .unshift(SwaggerUi::new("/api-doc/openapi.json").into_router("/swagger-ui"));
        // 解析地址
        let listener = TcpListener::new(addr.clone()).bind().await;

        tracing::info!("🚀 服务已启动: http://{}", addr);

        // 触发服务就绪事件
        self.events
            .emit(
                event::events::SERVER_READY,
                event::ServerReadyEvent {
                    address: addr.to_string(),
                    port: 0,
                },
            )
            .await;

        // 启动服务
        Server::new(listener).serve(router).await;

        Ok(())
    }
}

impl Default for CoolApp {
    fn default() -> Self {
        Self::new(config::CoolConfig::default())
    }
}
