//! # cool-rpc
//!
//! cool-admin Rust RPC 微服务库，提供分布式服务调用能力。
//!
//! ## 功能特性
//!
//! - 🚀 基于 Redis 的服务发现
//! - 📡 支持同步和异步 RPC 调用
//! - 📢 事件广播
//! - 🔄 负载均衡
//! - 💓 心跳检测
//!
//! ## 快速开始
//!
//! ```rust,ignore
//! use cool_rpc::prelude::*;
//!
//! // 创建 RPC 服务
//! let rpc = CoolRpc::new(RpcConfig {
//!     name: "user-service".to_string(),
//!     redis_url: "redis://127.0.0.1:6379".to_string(),
//!     ..Default::default()
//! }).await?;
//!
//! // 注册服务
//! rpc.register("UserService", user_service).await?;
//!
//! // 调用远程服务
//! let result = rpc.call("order-service", "OrderService", "getById", json!({"id": 1})).await?;
//! ```

mod broker;
mod service;
mod event;
mod registry;
mod debug;
mod transaction;
mod test;

pub use broker::*;
pub use service::*;
pub use event::*;
pub use registry::*;
pub use transaction::*;
pub use test::*;
// 调试工具目前仅在某些场景使用，避免未使用导出产生警告
// pub use debug::*;

use serde::{Deserialize, Serialize};

/// 预导入模块
pub mod prelude {
    pub use crate::broker::CoolRpc;
    pub use crate::event::{RpcEvent, RpcEventHandler};
    pub use crate::registry::{global_rpc_registry, RpcRegistry, RpcServiceMeta};
    pub use crate::service::{
        BaseRpcService, RpcMysqlService, RpcPostgresService, RpcService, RpcServiceHandler,
        RpcSqliteService,
    };
    pub use crate::test::{RpcTest, RpcTestRequest, RpcTestResponse};
    pub use crate::transaction::{
        global_transaction_manager, IsolationLevel, RpcTransactionManager, TransactionEvent,
        TransactionEventHandler, TransactionOptions, register_transaction_handler,
    };
    pub use crate::RpcConfig;
    pub use async_trait::async_trait;
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::json;
    
    #[cfg(feature = "web")]
    pub use crate::test::handler::create_rpc_test_router;
}

/// RPC 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    /// 服务名称
    pub name: String,
    /// Redis URL
    pub redis_url: String,
    /// 心跳间隔（秒）
    pub heartbeat_interval: u64,
    /// 服务超时时间（秒）
    pub service_timeout: u64,
    /// 调用超时时间（秒）
    pub call_timeout: u64,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            name: "cool-service".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            heartbeat_interval: 5,
            service_timeout: 30,
            call_timeout: 10,
        }
    }
}

