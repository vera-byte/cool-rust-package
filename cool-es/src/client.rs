//! Elasticsearch 客户端

use crate::EsConfig;
use elasticsearch::{
    http::request::JsonBody,
    http::transport::{SingleNodeConnectionPool, TransportBuilder},
    BulkParts, Elasticsearch, IndexParts, SearchParts,
};
use serde_json::Value;
use thiserror::Error;
use url::Url;

/// ES 错误
#[derive(Error, Debug)]
pub enum EsError {
    #[error("连接错误: {0}")]
    Connection(String),
    #[error("索引错误: {0}")]
    Index(String),
    #[error("搜索错误: {0}")]
    Search(String),
    #[error("文档错误: {0}")]
    Document(String),
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("ES 客户端错误: {0}")]
    Client(#[from] elasticsearch::Error),
}

pub type EsResult<T> = Result<T, EsError>;

/// Elasticsearch 客户端
pub struct EsClient {
    client: Elasticsearch,
}

impl EsClient {
    /// 创建客户端
    pub async fn new(config: &EsConfig) -> EsResult<Self> {
        let node = config
            .nodes
            .first()
            .ok_or_else(|| EsError::Connection("节点地址不能为空".to_string()))?;

        let url =
            Url::parse(node).map_err(|e| EsError::Connection(format!("无效的 URL: {}", e)))?;

        let conn_pool = SingleNodeConnectionPool::new(url);
        let mut builder = TransportBuilder::new(conn_pool);

        // 设置认证
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            builder = builder.auth(elasticsearch::auth::Credentials::Basic(
                username.clone(),
                password.clone(),
            ));
        }

        let transport = builder
            .build()
            .map_err(|e| EsError::Connection(format!("构建传输层失败: {}", e)))?;

        let client = Elasticsearch::new(transport);

        Ok(Self { client })
    }

    /// 检查连接
    pub async fn ping(&self) -> EsResult<bool> {
        let response = self.client.ping().send().await?;
        Ok(response.status_code().is_success())
    }

    /// 创建索引
    pub async fn create_index(&self, index: &str, body: Value) -> EsResult<()> {
        let response = self
            .client
            .indices()
            .create(elasticsearch::indices::IndicesCreateParts::Index(index))
            .body(body)
            .send()
            .await?;

        if !response.status_code().is_success() {
            return Err(EsError::Index(format!("创建索引失败: {}", index)));
        }

        Ok(())
    }

    /// 删除索引
    pub async fn delete_index(&self, index: &str) -> EsResult<()> {
        let response = self
            .client
            .indices()
            .delete(elasticsearch::indices::IndicesDeleteParts::Index(&[index]))
            .send()
            .await?;

        if !response.status_code().is_success() {
            return Err(EsError::Index(format!("删除索引失败: {}", index)));
        }

        Ok(())
    }

    /// 检查索引是否存在
    pub async fn index_exists(&self, index: &str) -> EsResult<bool> {
        let response = self
            .client
            .indices()
            .exists(elasticsearch::indices::IndicesExistsParts::Index(&[index]))
            .send()
            .await?;

        Ok(response.status_code().is_success())
    }

    /// 索引文档
    pub async fn index(&self, index: &str, id: &str, body: Value) -> EsResult<()> {
        let response = self
            .client
            .index(IndexParts::IndexId(index, id))
            .body(body)
            .send()
            .await?;

        if !response.status_code().is_success() {
            return Err(EsError::Document("索引文档失败".to_string()));
        }

        Ok(())
    }

    /// 获取文档
    pub async fn get(&self, index: &str, id: &str) -> EsResult<Option<Value>> {
        let response = self
            .client
            .get(elasticsearch::GetParts::IndexId(index, id))
            .send()
            .await?;

        if response.status_code() == 404 {
            return Ok(None);
        }

        if !response.status_code().is_success() {
            return Err(EsError::Document("获取文档失败".to_string()));
        }

        let body = response.json::<Value>().await?;
        Ok(body.get("_source").cloned())
    }

    /// 更新文档
    pub async fn update(&self, index: &str, id: &str, body: Value) -> EsResult<()> {
        let response = self
            .client
            .update(elasticsearch::UpdateParts::IndexId(index, id))
            .body(serde_json::json!({ "doc": body }))
            .send()
            .await?;

        if !response.status_code().is_success() {
            return Err(EsError::Document("更新文档失败".to_string()));
        }

        Ok(())
    }

    /// 删除文档
    pub async fn delete(&self, index: &str, id: &str) -> EsResult<()> {
        let response = self
            .client
            .delete(elasticsearch::DeleteParts::IndexId(index, id))
            .send()
            .await?;

        if !response.status_code().is_success() {
            return Err(EsError::Document("删除文档失败".to_string()));
        }

        Ok(())
    }

    /// 搜索
    pub async fn search(&self, index: &str, body: Value) -> EsResult<Value> {
        let response = self
            .client
            .search(SearchParts::Index(&[index]))
            .body(body)
            .send()
            .await?;

        if !response.status_code().is_success() {
            return Err(EsError::Search("搜索失败".to_string()));
        }

        let body = response.json::<Value>().await?;
        Ok(body)
    }

    /// 批量操作
    pub async fn bulk(&self, body: Vec<JsonBody<Value>>) -> EsResult<()> {
        let response = self.client.bulk(BulkParts::None).body(body).send().await?;

        if !response.status_code().is_success() {
            return Err(EsError::Document("批量操作失败".to_string()));
        }

        Ok(())
    }

    /// 获取原始客户端
    pub fn raw(&self) -> &Elasticsearch {
        &self.client
    }
}
