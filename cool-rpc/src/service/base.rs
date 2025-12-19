//! RPC 服务基类
//!
//! 对应 TypeScript 版本的 `rpc/src/service/base.ts`
//!
//! 提供 RPC Service 的数据库操作能力，让 RPC Service 可以复用 cool-core 的 CRUD 功能。

use cool_core::entity::{DeleteParam, Id, ListQuery, PageQuery, QueryOption};
use cool_core::error::{CoolResult, PageResult};
use cool_core::service::ModifyType;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use serde_json::Value;

/// RPC 服务基类 trait
///
/// 提供数据库操作能力，让 RPC Service 可以执行 CRUD 操作
///
/// 注意：具体的 RPC Service（如 `RpcMysqlService`）应该同时实现 `BaseService` 和 `BaseRpcService`，
/// 这样可以直接复用 cool-core 的所有 CRUD 能力。
#[async_trait]
pub trait BaseRpcService: Send + Sync {
    /// 获取数据库连接
    fn db(&self) -> &DatabaseConnection;

    /// 获取表名
    fn table_name(&self) -> &str;

    /// 新增
    async fn add(&self, data: Value) -> CoolResult<Value>;

    /// 删除
    async fn delete(&self, param: DeleteParam) -> CoolResult<()>;

    /// 软删除
    async fn soft_delete(&self, ids: Vec<Id>) -> CoolResult<()>;

    /// 修改
    async fn update(&self, data: Value) -> CoolResult<()>;

    /// 查询单条记录
    async fn info(&self, id: Id, ignore_fields: Option<Vec<String>>) -> CoolResult<Option<Value>>;

    /// 分页查询
    async fn page(
        &self,
        query: PageQuery,
        option: QueryOption,
    ) -> CoolResult<PageResult<Value>>;

    /// 分页查询（带过滤参数）
    async fn page_with_filters(
        &self,
        query: PageQuery,
        filters: &Value,
        option: QueryOption,
    ) -> CoolResult<PageResult<Value>>;

    /// 列表查询（不分页）
    async fn list(&self, query: ListQuery, option: QueryOption) -> CoolResult<Vec<Value>>;

    /// 原生 SQL 查询
    async fn native_query(&self, sql: &str, params: Vec<Value>) -> CoolResult<Vec<Value>>;

    /// 执行 SQL
    async fn execute(&self, sql: &str) -> CoolResult<u64>;

    /// 修改前置钩子（可重写）
    async fn modify_before(&self, data: Value, _modify_type: ModifyType) -> CoolResult<Value> {
        Ok(data)
    }

    /// 修改后置钩子（可重写）
    async fn modify_after(&self, _data: Value, _modify_type: ModifyType) -> CoolResult<()> {
        Ok(())
    }
}

