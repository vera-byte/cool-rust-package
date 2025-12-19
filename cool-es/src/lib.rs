//! # cool-es
//!
//! cool-admin Rust Elasticsearch 集成库。
//!
//! ## 功能特性
//!
//! - 🔍 Elasticsearch 客户端封装
//! - 📄 索引管理
//! - 🔎 文档 CRUD
//! - 🔍 搜索查询构建器
//!
//! ## 快速开始
//!
//! ```rust,ignore
//! use cool_es::prelude::*;
//!
//! let client = EsClient::new(&["http://localhost:9200"]).await?;
//!
//! // 创建索引
//! client.create_index("users", json!({
//!     "mappings": {
//!         "properties": {
//!             "name": { "type": "text" },
//!             "age": { "type": "integer" }
//!         }
//!     }
//! })).await?;
//!
//! // 索引文档
//! client.index("users", "1", json!({
//!     "name": "张三",
//!     "age": 25
//! })).await?;
//!
//! // 搜索
//! let results = client.search("users", json!({
//!     "query": {
//!         "match": { "name": "张三" }
//!     }
//! })).await?;
//! ```

mod client;
mod index;
mod search;
mod traits;

pub use client::*;
pub use index::*;
pub use search::*;
pub use traits::*;

use serde::{Deserialize, Serialize};

/// 预导入模块
pub mod prelude {
    pub use crate::client::EsClient;
    pub use crate::index::IndexManager;
    pub use crate::search::{SearchBuilder, SearchResult};
    pub use crate::traits::CoolEs;
    pub use crate::{EsConfig, EsIndexConfig};
    pub use async_trait::async_trait;
    pub use serde_json::json;
}

/// Elasticsearch 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsConfig {
    /// 节点地址列表
    pub nodes: Vec<String>,
    /// 用户名
    pub username: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 连接超时（秒）
    pub timeout: u64,
}

impl Default for EsConfig {
    fn default() -> Self {
        Self {
            nodes: vec!["http://localhost:9200".to_string()],
            username: None,
            password: None,
            timeout: 30,
        }
    }
}

/// ES 索引配置
///
/// 对应 TypeScript 版本的 `EsConfig`（装饰器参数）
#[derive(Debug, Clone)]
pub struct EsIndexConfig {
    /// 索引名称
    pub name: &'static str,
    /// 分片数
    pub shards: u32,
    /// 副本数
    pub replicas: u32,
    /// 分析器列表
    pub analyzers: Vec<String>,
}

impl Default for EsIndexConfig {
    fn default() -> Self {
        Self {
            name: "",
            shards: 8,
            replicas: 1,
            analyzers: Vec::new(),
        }
    }
}
