//! 索引管理

use crate::client::{EsClient, EsResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 索引信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    /// 索引名称
    pub name: String,
    /// 文档数量
    pub docs_count: u64,
    /// 存储大小
    pub store_size: String,
    /// 健康状态
    pub health: String,
    /// 状态
    pub status: String,
}

/// 索引管理器
pub struct IndexManager<'a> {
    client: &'a EsClient,
}

impl<'a> IndexManager<'a> {
    pub fn new(client: &'a EsClient) -> Self {
        Self { client }
    }

    /// 创建索引
    pub async fn create(
        &self,
        name: &str,
        mappings: Value,
        settings: Option<Value>,
    ) -> EsResult<()> {
        let mut body = serde_json::json!({
            "mappings": mappings
        });

        if let Some(s) = settings {
            body["settings"] = s;
        }

        self.client.create_index(name, body).await
    }

    /// 删除索引
    pub async fn delete(&self, name: &str) -> EsResult<()> {
        self.client.delete_index(name).await
    }

    /// 检查索引是否存在
    pub async fn exists(&self, name: &str) -> EsResult<bool> {
        self.client.index_exists(name).await
    }

    /// 获取索引信息
    pub async fn info(&self, name: &str) -> EsResult<Option<IndexInfo>> {
        // 简化实现
        if !self.exists(name).await? {
            return Ok(None);
        }

        Ok(Some(IndexInfo {
            name: name.to_string(),
            docs_count: 0,
            store_size: "0b".to_string(),
            health: "green".to_string(),
            status: "open".to_string(),
        }))
    }

    /// 重建索引
    pub async fn reindex(&self, source: &str, dest: &str) -> EsResult<()> {
        let body = serde_json::json!({
            "source": { "index": source },
            "dest": { "index": dest }
        });

        // 使用原始客户端执行 reindex
        let response = self
            .client
            .raw()
            .reindex()
            .body(body)
            .send()
            .await
            .map_err(crate::client::EsError::from)?;

        if !response.status_code().is_success() {
            return Err(crate::client::EsError::Index("重建索引失败".to_string()));
        }

        Ok(())
    }
}
