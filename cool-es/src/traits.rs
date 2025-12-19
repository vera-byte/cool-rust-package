//! ES 相关 trait 定义
//!
//! 对应 TypeScript 版本的 `ICoolEs` 接口

use async_trait::async_trait;
use serde_json::Value;

/// ES 索引 trait
///
/// 对应 TypeScript 版本的 `ICoolEs` 接口
///
/// 实现此 trait 的类型可以提供索引的映射信息（mappings），
/// 用于自动创建和管理 Elasticsearch 索引。
#[async_trait]
pub trait CoolEs: Send + Sync {
    /// 获取索引信息（映射定义）
    ///
    /// 返回索引的字段映射定义，用于创建索引时的 mappings 配置
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// impl CoolEs for UserIndex {
    ///     async fn index_info(&self) -> Value {
    ///         json!({
    ///             "name": { "type": "text" },
    ///             "age": { "type": "integer" },
    ///             "email": { "type": "keyword" }
    ///         })
    ///     }
    /// }
    /// ```
    async fn index_info(&self) -> Value;
}
