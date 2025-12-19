//! RPC 服务模块
//!
//! 对应 TypeScript 版本的 `rpc/src/service/`
//!
//! 包含：
//! - RPC 服务定义（`RpcService`, `RpcServiceHandler`）
//! - RPC 服务数据库操作能力（`BaseRpcService`, `RpcMysqlService` 等）

mod base;
mod mysql;
mod postgres;
mod sqlite;

use async_trait::async_trait;
use thiserror::Error;

/// 服务错误
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("方法未找到: {0}")]
    MethodNotFound(String),
    #[error("参数错误: {0}")]
    InvalidParams(String),
    #[error("执行错误: {0}")]
    ExecutionError(String),
}

/// 服务处理器 trait
#[async_trait]
pub trait RpcServiceHandler: Send + Sync {
    /// 调用服务方法
    async fn call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, ServiceError>;
}

/// RPC 服务定义
pub struct RpcService {
    /// 服务名称
    pub name: String,
}

impl RpcService {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

/// 服务构建器
pub struct RpcServiceBuilder {
    name: String,
}

impl RpcServiceBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn build(self) -> RpcService {
        RpcService { name: self.name }
    }
}

// 导出数据库操作相关类型
pub use base::*;
pub use mysql::*;
pub use postgres::*;
pub use sqlite::*;

