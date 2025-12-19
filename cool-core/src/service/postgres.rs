//! PostgreSQL 专用 Service 包装
//!
//! 对齐 TS 版本的 `service/postgres.ts`。

use super::base::{BaseService, SimpleService};
use crate::error::CoolError;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection};
use std::sync::Arc;

/// Postgres Service
pub struct PostgresService {
    inner: SimpleService,
}

impl PostgresService {
    /// 创建 Postgres Service，校验后端类型
    pub fn new(db: Arc<DatabaseConnection>, table: impl Into<String>) -> Result<Self, CoolError> {
        if db.get_database_backend() != DatabaseBackend::Postgres {
            return Err(CoolError::comm("当前连接不是 Postgres 后端"));
        }
        Ok(Self {
            inner: SimpleService::new(db, table),
        })
    }
}

#[async_trait::async_trait]
impl BaseService for PostgresService {
    fn db(&self) -> &DatabaseConnection {
        self.inner.db()
    }

    fn table_name(&self) -> &str {
        self.inner.table_name()
    }
}
