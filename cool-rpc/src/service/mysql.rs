//! MySQL RPC Service
//!
//! 对应 TypeScript 版本的 `rpc/src/service/mysql.ts`

use super::base::BaseRpcService;
use async_trait::async_trait;
use cool_core::entity::{DeleteParam, Id, ListQuery, PageQuery, QueryOption};
use cool_core::error::{CoolError, CoolResult, PageResult};
use cool_core::service::{BaseService, MySqlService};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection};
use serde_json::Value;
use std::sync::Arc;

/// MySQL RPC Service
pub struct RpcMysqlService {
    /// 底层 MySQL 服务
    inner: MySqlService,
}

impl RpcMysqlService {
    /// 创建 MySQL RPC Service，校验后端类型
    pub fn new(db: Arc<DatabaseConnection>, table: impl Into<String>) -> Result<Self, CoolError> {
        if db.get_database_backend() != DatabaseBackend::MySql {
            return Err(CoolError::comm("当前连接不是 MySQL 后端"));
        }
        Ok(Self {
            inner: MySqlService::new(db, table)?,
        })
    }
}

#[async_trait]
impl BaseService for RpcMysqlService {
    fn db(&self) -> &DatabaseConnection {
        self.inner.db()
    }

    fn table_name(&self) -> &str {
        self.inner.table_name()
    }
}

#[async_trait]
impl BaseRpcService for RpcMysqlService {
    fn db(&self) -> &DatabaseConnection {
        self.inner.db()
    }

    fn table_name(&self) -> &str {
        self.inner.table_name()
    }

    async fn add(&self, data: Value) -> CoolResult<Value> {
        self.inner.add(data).await
    }

    async fn delete(&self, param: DeleteParam) -> CoolResult<()> {
        self.inner.delete(param).await
    }

    async fn soft_delete(&self, ids: Vec<Id>) -> CoolResult<()> {
        self.inner.soft_delete(ids).await
    }

    async fn update(&self, data: Value) -> CoolResult<()> {
        self.inner.update(data).await
    }

    async fn info(&self, id: Id, ignore_fields: Option<Vec<String>>) -> CoolResult<Option<Value>> {
        self.inner.info(id, ignore_fields).await
    }

    async fn page(&self, query: PageQuery, option: QueryOption) -> CoolResult<PageResult<Value>> {
        self.inner.page(query, option).await
    }

    async fn page_with_filters(
        &self,
        query: PageQuery,
        filters: &Value,
        option: QueryOption,
    ) -> CoolResult<PageResult<Value>> {
        self.inner.page_with_filters(query, filters, option).await
    }

    async fn list(&self, query: ListQuery, option: QueryOption) -> CoolResult<Vec<Value>> {
        self.inner.list(query, option).await
    }

    async fn native_query(&self, sql: &str, params: Vec<Value>) -> CoolResult<Vec<Value>> {
        self.inner.native_query(sql, params).await
    }

    async fn execute(&self, sql: &str) -> CoolResult<u64> {
        self.inner.execute(sql).await
    }
}
