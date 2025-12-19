//! MySQL 专用 Service 包装
//!
//! 对齐 TS 版本的 `service/mysql.ts`，主要提供：
//! - 便捷的构造函数
//! - 后端校验（确保当前连接是 MySQL）
//! - 复用 `SimpleService` 的所有 CRUD 能力

use super::base::{BaseService, SimpleService};
use crate::error::CoolError;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection};
use std::sync::Arc;

/// MySQL Service
pub struct MySqlService {
    inner: SimpleService,
}

impl MySqlService {
    /// 创建 MySQL Service，校验后端类型
    pub fn new(db: Arc<DatabaseConnection>, table: impl Into<String>) -> Result<Self, CoolError> {
        if db.get_database_backend() != DatabaseBackend::MySql {
            return Err(CoolError::comm("当前连接不是 MySQL 后端"));
        }
        Ok(Self {
            inner: SimpleService::new(db, table),
        })
    }
}

#[async_trait::async_trait]
impl BaseService for MySqlService {
    fn db(&self) -> &DatabaseConnection {
        self.inner.db()
    }

    fn table_name(&self) -> &str {
        self.inner.table_name()
    }
}
