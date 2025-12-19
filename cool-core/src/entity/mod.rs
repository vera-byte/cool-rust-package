//! 实体模块
//!
//! 对应 TypeScript 版本的 `entity/`

#[cfg(feature = "mongo")]
pub mod mongo;
#[cfg(feature = "mongo")]
pub use mongo::*;

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 基础实体 trait
///
/// 所有实体都应该实现此 trait，提供统一的基础字段
pub trait BaseEntity: EntityTrait {
    /// 获取 ID 列
    fn id_column() -> Self::Column;
    /// 获取创建时间列
    fn create_time_column() -> Self::Column;
    /// 获取更新时间列
    fn update_time_column() -> Self::Column;
}

/// 软删除实体 trait
///
/// 支持软删除的实体应该实现此 trait
pub trait SoftDeleteEntity: BaseEntity {
    /// 获取删除时间列
    fn delete_time_column() -> Self::Column;
}

/// 多租户实体 trait
///
/// 支持多租户的实体应该实现此 trait
pub trait TenantEntity: BaseEntity {
    /// 获取租户 ID 列
    fn tenant_id_column() -> Self::Column;
}

/// 基础模型字段
///
/// 用于在 Model 中嵌入基础字段
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseFields {
    /// 创建时间
    #[serde(rename = "createTime")]
    pub create_time: Option<DateTime<Utc>>,
    /// 更新时间
    #[serde(rename = "updateTime")]
    pub update_time: Option<DateTime<Utc>>,
}

impl Default for BaseFields {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            create_time: Some(now),
            update_time: Some(now),
        }
    }
}

/// 软删除字段
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SoftDeleteFields {
    /// 删除时间
    #[serde(rename = "deleteTime", skip_serializing_if = "Option::is_none")]
    pub delete_time: Option<DateTime<Utc>>,
}

/// 多租户字段
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TenantFields {
    /// 租户 ID
    #[serde(rename = "tenantId", skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<i64>,
}

/// 通用 ID 类型
pub type Id = i64;

/// 分页查询参数
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PageQuery {
    /// 当前页码（从 1 开始）
    #[serde(default = "default_page")]
    pub page: u64,
    /// 每页条数
    #[serde(default = "default_size")]
    pub size: u64,
    /// 排序字段
    pub order: Option<String>,
    /// 排序方式: "asc" | "desc"
    pub sort: Option<String>,
    /// 关键字搜索
    #[serde(rename = "keyWord")]
    pub key_word: Option<String>,
}

fn default_page() -> u64 {
    1
}

fn default_size() -> u64 {
    20
}

impl PageQuery {
    /// 获取偏移量
    pub fn offset(&self) -> u64 {
        (self.page.saturating_sub(1)) * self.size
    }

    /// 是否升序
    pub fn is_asc(&self) -> bool {
        self.sort
            .as_ref()
            .map(|s| s.to_lowercase() == "asc")
            .unwrap_or(false)
    }
}

/// 列表查询参数
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListQuery {
    /// 排序字段
    pub order: Option<String>,
    /// 排序方式: "asc" | "desc"
    pub sort: Option<String>,
    /// 关键字搜索
    #[serde(rename = "keyWord")]
    pub key_word: Option<String>,
}

/// 删除参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteParam {
    /// 要删除的 ID 列表
    pub ids: Vec<Id>,
}

/// 新增返回结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddResult {
    /// 新增的 ID
    pub id: Id,
}

/// 批量新增返回结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAddResult {
    /// 新增的 ID 列表
    pub ids: Vec<Id>,
}

/// 查询选项（对齐 TS 版 QueryOp，保留核心字段）
#[derive(Debug, Clone, Default)]
pub struct QueryOption {
    /// 需要模糊查询的字段
    pub key_word_like_fields: Vec<String>,
    /// 查询字段
    pub select: Vec<String>,
    /// 字段模糊查询
    pub field_like: Vec<FieldCondition>,
    /// 字段相等查询
    pub field_eq: Vec<FieldCondition>,
    /// 排序配置
    pub order_by: Vec<(String, bool)>, // (字段, 是否升序)
    /// 关联配置
    pub joins: Vec<JoinOption>,
    /// 额外的 AND 条件（字符串片段），用于快速拼接简单 where
    pub where_and: Vec<String>,
    /// 左连接配置（与 joins 等价，此处为兼容 TS 命名）
    pub left_join: Vec<JoinOption>,
    /// 自定义 where 片段（带参数），用于复杂过滤
    pub extra_where: Vec<WhereFragment>,
}

impl QueryOption {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn key_word_like_fields<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.key_word_like_fields = fields.into_iter().map(Into::into).collect();
        self
    }

    pub fn select<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.select = fields.into_iter().map(Into::into).collect();
        self
    }

    pub fn add_field_eq(
        mut self,
        column: impl Into<String>,
        request_param: impl Into<String>,
    ) -> Self {
        self.field_eq
            .push(FieldCondition::with_param(column, request_param));
        self
    }

    pub fn add_field_like(
        mut self,
        column: impl Into<String>,
        request_param: impl Into<String>,
    ) -> Self {
        self.field_like
            .push(FieldCondition::with_param(column, request_param));
        self
    }

    pub fn add_order_by(mut self, column: impl Into<String>, asc: bool) -> Self {
        self.order_by.push((column.into(), asc));
        self
    }

    pub fn add_left_join(
        mut self,
        entity: impl Into<String>,
        alias: impl Into<String>,
        condition: impl Into<String>,
        join_type: JoinType,
    ) -> Self {
        self.left_join.push(JoinOption {
            entity: entity.into(),
            alias: alias.into(),
            condition: condition.into(),
            join_type,
        });
        self
    }

    pub fn add_where_and(mut self, cond: impl Into<String>) -> Self {
        self.where_and.push(cond.into());
        self
    }

    pub fn add_where_raw<I, V>(mut self, sql: impl Into<String>, params: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<serde_json::Value>,
    {
        self.extra_where.push(WhereFragment {
            sql: sql.into(),
            params: params.into_iter().map(Into::into).collect(),
        });
        self
    }

    /// 通过函数式方式追加 where 片段（便捷包装）
    pub fn add_where_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(WhereBuilder) -> WhereBuilder,
    {
        let builder = f(WhereBuilder::new());
        self.extra_where.extend(builder.build());
        self
    }
}

/// 字段条件
#[derive(Debug, Clone)]
pub struct FieldCondition {
    /// 数据库列名
    pub column: String,
    /// 请求参数名
    pub request_param: String,
}

impl FieldCondition {
    pub fn new(column: impl Into<String>) -> Self {
        let col = column.into();
        Self {
            request_param: col.clone(),
            column: col,
        }
    }

    pub fn with_param(column: impl Into<String>, request_param: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            request_param: request_param.into(),
        }
    }
}

/// 关联选项
#[derive(Debug, Clone)]
pub struct JoinOption {
    /// 关联实体名
    pub entity: String,
    /// 别名
    pub alias: String,
    /// 关联条件
    pub condition: String,
    /// 关联类型: "inner" | "left"
    pub join_type: JoinType,
}

/// 关联类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JoinType {
    /// 内连接
    Inner,
    /// 左连接
    #[default]
    Left,
}

/// 自定义 where 片段（带参数）
#[derive(Debug, Clone)]
pub struct WhereFragment {
    /// SQL 片段（不含 WHERE/AND）
    pub sql: String,
    /// 参数列表，对应 `?` 占位符
    pub params: Vec<serde_json::Value>,
}

/// where 片段构造器，便于以函数式方式累加过滤条件
#[derive(Default)]
pub struct WhereBuilder {
    frags: Vec<WhereFragment>,
}

impl WhereBuilder {
    pub fn new() -> Self {
        Self { frags: Vec::new() }
    }

    /// 添加等值条件：`column = ?`
    pub fn eq(mut self, column: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.frags.push(WhereFragment {
            sql: format!("{} = ?", column.into()),
            params: vec![value.into()],
        });
        self
    }

    /// 添加模糊条件：`column LIKE ?`，自动包裹 `%`
    pub fn like(mut self, column: impl Into<String>, value: impl Into<String>) -> Self {
        self.frags.push(WhereFragment {
            sql: format!("{} LIKE ?", column.into()),
            params: vec![serde_json::Value::String(format!("%{}%", value.into()))],
        });
        self
    }

    /// 添加自定义 SQL 片段
    pub fn raw<I, V>(mut self, sql: impl Into<String>, params: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<serde_json::Value>,
    {
        self.frags.push(WhereFragment {
            sql: sql.into(),
            params: params.into_iter().map(Into::into).collect(),
        });
        self
    }

    pub fn build(self) -> Vec<WhereFragment> {
        self.frags
    }
}
