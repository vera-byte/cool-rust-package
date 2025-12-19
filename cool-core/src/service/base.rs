//! 服务基类
//!
//! 对应 TypeScript 版本的 `service/base.ts`

use crate::entity::{DeleteParam, Id, ListQuery, PageQuery, QueryOption};
use crate::error::{CoolError, CoolResult, PageResult};
use crate::event::{events, global_event_manager, SoftDeleteEvent};
use async_trait::async_trait;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use serde_json::Value;
use std::sync::Arc;

/// 修改类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifyType {
    /// 新增
    Add,
    /// 修改
    Update,
    /// 删除
    Delete,
}

/// 服务基类 trait
///
/// 提供通用的 CRUD 操作
#[async_trait]
pub trait BaseService: Send + Sync {
    /// 获取数据库连接
    fn db(&self) -> &DatabaseConnection;

    /// 获取表名
    fn table_name(&self) -> &str;

    /// 新增
    async fn add(&self, data: Value) -> CoolResult<Value> {
        let data = self.modify_before(data, ModifyType::Add).await?;

        // 构建 INSERT SQL
        let columns: Vec<String> = data
            .as_object()
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default();

        if columns.is_empty() {
            return Err(CoolError::validate("数据不能为空"));
        }

        let placeholders: Vec<String> = columns.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table_name(),
            columns.join(", "),
            placeholders.join(", ")
        );

        let values: Vec<sea_orm::Value> = columns
            .iter()
            .filter_map(|col| data.get(col))
            .map(|v| json_to_sea_value(v))
            .collect();

        let stmt = Statement::from_sql_and_values(self.db().get_database_backend(), &sql, values);

        let result = self.db().execute(stmt).await?;
        let id = result.last_insert_id();

        self.modify_after(data.clone(), ModifyType::Add).await?;

        Ok(serde_json::json!({ "id": id }))
    }

    /// 删除
    async fn delete(&self, param: DeleteParam) -> CoolResult<()> {
        let data = serde_json::to_value(&param)?;
        self.modify_before(data.clone(), ModifyType::Delete).await?;

        let ids_str = param
            .ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let sql = format!(
            "DELETE FROM {} WHERE id IN ({})",
            self.table_name(),
            ids_str
        );
        let stmt = Statement::from_string(self.db().get_database_backend(), sql);
        self.db().execute(stmt).await?;

        self.modify_after(data, ModifyType::Delete).await?;

        Ok(())
    }

    /// 软删除
    async fn soft_delete(&self, ids: Vec<Id>) -> CoolResult<()> {
        let now = chrono::Utc::now();
        let ids_str = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let sql = format!(
            "UPDATE {} SET delete_time = '{}' WHERE id IN ({})",
            self.table_name(),
            now.format("%Y-%m-%d %H:%M:%S"),
            ids_str
        );
        let stmt = Statement::from_string(self.db().get_database_backend(), sql);
        self.db().execute(stmt).await?;

        // 触发软删除事件，方便其他模块（如回收站、日志等）进行处理
        //
        // 说明：
        // - entity 使用表名，保持与 TS 版本中实体名称语义接近
        // - tenant_id 当前无法在基类中获取，先置为 None，后续可在有租户上下文的 Service 中扩展
        global_event_manager()
            .emit(
                events::SOFT_DELETE,
                SoftDeleteEvent {
                    entity: self.table_name().to_string(),
                    ids: ids.clone(),
                    tenant_id: None,
                },
            )
            .await;

        Ok(())
    }

    /// 修改
    async fn update(&self, data: Value) -> CoolResult<()> {
        let id = data
            .get("id")
            .ok_or_else(CoolError::no_id)?
            .as_i64()
            .ok_or_else(CoolError::no_id)?;

        let data = self.modify_before(data, ModifyType::Update).await?;

        // 构建 UPDATE SQL
        let updates: Vec<String> = data
            .as_object()
            .map(|obj| {
                obj.iter()
                    .filter(|(k, _)| *k != "id")
                    .map(|(k, _)| format!("{} = ?", k))
                    .collect()
            })
            .unwrap_or_default();

        if updates.is_empty() {
            return Ok(());
        }

        let sql = format!(
            "UPDATE {} SET {} WHERE id = ?",
            self.table_name(),
            updates.join(", ")
        );

        let mut values: Vec<sea_orm::Value> = data
            .as_object()
            .map(|obj| {
                obj.iter()
                    .filter(|(k, _)| *k != "id")
                    .map(|(_, v)| json_to_sea_value(v))
                    .collect()
            })
            .unwrap_or_default();
        values.push(sea_orm::Value::BigInt(Some(id)));

        let stmt = Statement::from_sql_and_values(self.db().get_database_backend(), &sql, values);

        self.db().execute(stmt).await?;
        self.modify_after(data, ModifyType::Update).await?;

        Ok(())
    }

    /// 查询单条记录
    async fn info(&self, id: Id, _ignore_fields: Option<Vec<String>>) -> CoolResult<Option<Value>> {
        let sql = format!("SELECT * FROM {} WHERE id = ? LIMIT 1", self.table_name());
        let stmt = Statement::from_sql_and_values(
            self.db().get_database_backend(),
            &sql,
            vec![sea_orm::Value::BigInt(Some(id))],
        );

        let result = self.db().query_one(stmt).await?;

        Ok(result.map(|row| self.map_row(row)))
    }

    /// 分页查询
    ///
    /// 说明：
    /// - 支持基础的关键字模糊查询与排序（基于 `QueryOption`）
    /// - 在生成 SQL 之前会做字段名和关键字长度的安全检查，避免明显的 SQL 注入
    /// - 更复杂的关联和条件可以通过 `native_query` 或上层服务自定义
    ///
    /// 与 TS 版本的 QueryOp 对齐：
    /// - `key_word_like_fields`：关键字模糊查询
    /// - `order_by`：排序配置
    /// - `field_eq` / `field_like`：可通过 [`page_with_filters`] 传入自定义过滤参数使用
    async fn page(
        &self,
        query: PageQuery,
        mut option: QueryOption,
    ) -> CoolResult<PageResult<Value>> {
        let offset = query.offset();
        let size = query.size;

        // 基础参数安全性检查
        validate_query_safety(&query, &option)?;

        // 构建 SELECT 子句
        let select_sql = if option.select.is_empty() {
            "*".to_string()
        } else {
            // 这里只是简单地拼接字段名，默认调用方传入的是完整的列名（带别名）
            for col in &option.select {
                validate_identifier(col)?;
            }
            option.select.join(", ")
        };

        // 合并 left_join 到 joins，兼容 TS 命名
        if !option.left_join.is_empty() {
            option.joins.extend(option.left_join.clone());
        }

        // 构建 FROM + JOIN 子句
        let mut from_sql = format!("{}", self.table_name());
        for join in &option.joins {
            validate_identifier(&join.entity)?;
            validate_identifier(&join.alias)?;
            // 条件里也做一层基础校验，防止明显的注入特征
            if join.condition.contains(|c| {
                c == ';' || c == '#' || c == '\'' || c == '"' || c == '\n' || c == '\r'
            }) {
                return Err(CoolError::validate("关联条件包含非法字符"));
            }

            let join_kw = match join.join_type {
                crate::entity::JoinType::Inner => "INNER JOIN",
                crate::entity::JoinType::Left => "LEFT JOIN",
            };
            from_sql.push(' ');
            from_sql.push_str(join_kw);
            from_sql.push(' ');
            from_sql.push_str(&join.entity);
            from_sql.push_str(" AS ");
            from_sql.push_str(&join.alias);
            from_sql.push_str(" ON ");
            from_sql.push_str(&join.condition);
        }

        // 构建 WHERE 子句和参数（关键字模糊 + where_and）
        let mut where_sql = String::new();
        let mut params: Vec<sea_orm::Value> = Vec::new();

        if let Some(ref kw) = query.key_word {
            if !option.key_word_like_fields.is_empty() {
                where_sql.push_str(" WHERE ");
                for (idx, col) in option.key_word_like_fields.iter().enumerate() {
                    if idx > 0 {
                        where_sql.push_str(" OR ");
                    }
                    where_sql.push_str(&format!("{} LIKE ?", col));
                    params.push(sea_orm::Value::String(Some(Box::new(format!("%{}%", kw)))));
                }
            }
        }

        // 追加 where_and 片段（仅做非常基础的安全检查）
        for cond in &option.where_and {
            if cond.contains(';')
                || cond.contains("--")
                || cond.contains("/*")
                || cond.contains("*/")
            {
                return Err(CoolError::validate("where_and 包含非法字符"));
            }
            if where_sql.is_empty() {
                where_sql.push_str(" WHERE ");
            } else {
                where_sql.push_str(" AND ");
            }
            where_sql.push_str(cond);
        }

        // 追加 extra_where 片段（带参数）
        for frag in &option.extra_where {
            if frag.sql.contains(';')
                || frag.sql.contains("--")
                || frag.sql.contains("/*")
                || frag.sql.contains("*/")
            {
                return Err(CoolError::validate("extra_where 包含非法字符"));
            }
            if where_sql.is_empty() {
                where_sql.push_str(" WHERE ");
            } else {
                where_sql.push_str(" AND ");
            }
            where_sql.push_str(&format!("({})", frag.sql));
            for p in &frag.params {
                params.push(json_to_sea_value(p));
            }
        }

        // 构建 ORDER BY 子句：优先使用前端传入的 order/sort，其次使用 QueryOption.order_by
        let mut order_sql = String::new();
        if let Some(ref order_field) = query.order {
            validate_identifier(order_field)?;
            let asc = query.is_asc();
            order_sql.push_str(" ORDER BY ");
            order_sql.push_str(order_field);
            order_sql.push_str(if asc { " ASC" } else { " DESC" });
        } else if !option.order_by.is_empty() {
            order_sql.push_str(" ORDER BY ");
            let mut first = true;
            for (col, asc) in &option.order_by {
                validate_identifier(col)?;
                if !first {
                    order_sql.push_str(", ");
                }
                first = false;
                order_sql.push_str(col);
                order_sql.push_str(if *asc { " ASC" } else { " DESC" });
            }
        }

        // 查询总数
        let count_sql = format!("SELECT COUNT(*) as count FROM {}{}", from_sql, where_sql);
        let count_stmt = Statement::from_sql_and_values(
            self.db().get_database_backend(),
            count_sql,
            params.clone(),
        );
        let count_result = self.db().query_one(count_stmt).await?;
        let total: u64 = count_result
            .as_ref()
            .and_then(|r| r.try_get_by_index::<i64>(0).ok())
            .map(|v| v as u64)
            .unwrap_or(0);

        // 查询数据
        let sql = format!(
            "SELECT {} FROM {}{}{} LIMIT ? OFFSET ?",
            select_sql, from_sql, where_sql, order_sql
        );
        let mut data_params = params;
        data_params.push(sea_orm::Value::BigUnsigned(Some(size)));
        data_params.push(sea_orm::Value::BigUnsigned(Some(offset)));

        let stmt =
            Statement::from_sql_and_values(self.db().get_database_backend(), sql, data_params);
        let results = self.db().query_all(stmt).await?;

        let list: Vec<Value> = results.into_iter().map(|row| self.map_row(row)).collect();

        Ok(PageResult::new(list, query.page, size, total))
    }

    /// 分页查询（带过滤参数）
    ///
    /// 对齐 TS 版本中 `fieldEq` / `fieldLike` 的能力：
    /// - `filters` 为一个 JSON 对象，key 为 `FieldCondition.request_param`
    /// - `field_eq`：生成 `AND column = ?`
    /// - `field_like`：生成 `AND column LIKE ?`
    async fn page_with_filters(
        &self,
        query: PageQuery,
        filters: &Value,
        mut option: QueryOption,
    ) -> CoolResult<PageResult<Value>> {
        // 先复用基础分页逻辑的大部分实现，但手动展开 WHERE 构建，加入 eq/like 条件
        let offset = query.offset();
        let size = query.size;

        // 基础参数安全性检查
        validate_query_safety(&query, &option)?;

        // 合并 left_join 到 joins，兼容 TS 命名
        if !option.left_join.is_empty() {
            option.joins.extend(option.left_join.clone());
        }

        // 构建 SELECT 子句
        let select_sql = if option.select.is_empty() {
            "*".to_string()
        } else {
            for col in &option.select {
                validate_identifier(col)?;
            }
            option.select.join(", ")
        };

        // 构建 FROM + JOIN 子句
        let mut from_sql = format!("{}", self.table_name());
        for join in &option.joins {
            validate_identifier(&join.entity)?;
            validate_identifier(&join.alias)?;
            if join.condition.contains(|c| {
                c == ';' || c == '#' || c == '\'' || c == '"' || c == '\n' || c == '\r'
            }) {
                return Err(CoolError::validate("关联条件包含非法字符"));
            }

            let join_kw = match join.join_type {
                crate::entity::JoinType::Inner => "INNER JOIN",
                crate::entity::JoinType::Left => "LEFT JOIN",
            };
            from_sql.push(' ');
            from_sql.push_str(join_kw);
            from_sql.push(' ');
            from_sql.push_str(&join.entity);
            from_sql.push_str(" AS ");
            from_sql.push_str(&join.alias);
            from_sql.push_str(" ON ");
            from_sql.push_str(&join.condition);
        }

        // 构建 WHERE 子句和参数：关键字 + 等值 + 模糊字段 + where_and
        let mut where_sql = String::new();
        let mut params: Vec<sea_orm::Value> = Vec::new();
        let mut has_where = false;

        // 关键字模糊查询
        if let Some(ref kw) = query.key_word {
            if !option.key_word_like_fields.is_empty() {
                validate_keyword(kw)?;
                where_sql.push_str(" WHERE ");
                has_where = true;
                for (idx, col) in option.key_word_like_fields.iter().enumerate() {
                    validate_identifier(col)?;
                    if idx > 0 {
                        where_sql.push_str(" OR ");
                    }
                    where_sql.push_str(&format!("{} LIKE ?", col));
                    params.push(sea_orm::Value::String(Some(Box::new(format!("%{}%", kw)))));
                }
            }
        }

        // 等值字段
        for cond in &option.field_eq {
            validate_identifier(&cond.column)?;
            if let Some(value) = filters.get(&cond.request_param) {
                if !value.is_null() {
                    if !has_where {
                        where_sql.push_str(" WHERE ");
                        has_where = true;
                    } else {
                        where_sql.push_str(" AND ");
                    }
                    where_sql.push_str(&format!("{} = ?", cond.column));
                    params.push(json_to_sea_value(value));
                }
            }
        }

        // 模糊匹配字段
        for cond in &option.field_like {
            validate_identifier(&cond.column)?;
            if let Some(value) = filters.get(&cond.request_param) {
                if let Some(val_str) = value.as_str() {
                    validate_keyword(val_str)?;
                    if !has_where {
                        where_sql.push_str(" WHERE ");
                        has_where = true;
                    } else {
                        where_sql.push_str(" AND ");
                    }
                    where_sql.push_str(&format!("{} LIKE ?", cond.column));
                    params.push(sea_orm::Value::String(Some(Box::new(format!(
                        "%{}%",
                        val_str
                    )))));
                }
            }
        }

        // 追加 where_and 片段（仅做非常基础的安全检查）
        for cond in &option.where_and {
            if cond.contains(';')
                || cond.contains("--")
                || cond.contains("/*")
                || cond.contains("*/")
            {
                return Err(CoolError::validate("where_and 包含非法字符"));
            }
            if !has_where {
                where_sql.push_str(" WHERE ");
                has_where = true;
            } else {
                where_sql.push_str(" AND ");
            }
            where_sql.push_str(cond);
        }

        // 追加 extra_where 片段（带参数）
        for frag in &option.extra_where {
            if frag.sql.contains(';')
                || frag.sql.contains("--")
                || frag.sql.contains("/*")
                || frag.sql.contains("*/")
            {
                return Err(CoolError::validate("extra_where 包含非法字符"));
            }
            if !has_where {
                where_sql.push_str(" WHERE ");
                has_where = true;
            } else {
                where_sql.push_str(" AND ");
            }
            where_sql.push_str(&format!("({})", frag.sql));
            for p in &frag.params {
                params.push(json_to_sea_value(p));
            }
        }

        // 构建 ORDER BY 子句
        let mut order_sql = String::new();
        if let Some(ref order_field) = query.order {
            validate_identifier(order_field)?;
            let asc = query.is_asc();
            order_sql.push_str(" ORDER BY ");
            order_sql.push_str(order_field);
            order_sql.push_str(if asc { " ASC" } else { " DESC" });
        } else if !option.order_by.is_empty() {
            order_sql.push_str(" ORDER BY ");
            let mut first = true;
            for (col, asc) in &option.order_by {
                validate_identifier(col)?;
                if !first {
                    order_sql.push_str(", ");
                }
                first = false;
                order_sql.push_str(col);
                order_sql.push_str(if *asc { " ASC" } else { " DESC" });
            }
        }

        // 查询总数
        let count_sql = format!("SELECT COUNT(*) as count FROM {}{}", from_sql, where_sql);
        let count_stmt = Statement::from_sql_and_values(
            self.db().get_database_backend(),
            count_sql,
            params.clone(),
        );
        let count_result = self.db().query_one(count_stmt).await?;
        let total: u64 = count_result
            .as_ref()
            .and_then(|r| r.try_get_by_index::<i64>(0).ok())
            .map(|v| v as u64)
            .unwrap_or(0);

        // 查询数据
        let sql = format!(
            "SELECT {} FROM {}{}{} LIMIT ? OFFSET ?",
            select_sql, from_sql, where_sql, order_sql
        );
        let mut data_params = params;
        data_params.push(sea_orm::Value::BigUnsigned(Some(size)));
        data_params.push(sea_orm::Value::BigUnsigned(Some(offset)));

        let stmt =
            Statement::from_sql_and_values(self.db().get_database_backend(), sql, data_params);
        let results = self.db().query_all(stmt).await?;

        let list: Vec<Value> = results.into_iter().map(|row| self.map_row(row)).collect();

        Ok(PageResult::new(list, query.page, size, total))
    }

    /// 列表查询（不分页）
    ///
    /// 说明：
    /// - 支持基础的关键字模糊查询与排序
    /// - 在生成 SQL 之前会做字段名和关键字长度的安全检查
    async fn list(&self, query: ListQuery, option: QueryOption) -> CoolResult<Vec<Value>> {
        // 构建 SELECT 子句
        let select_sql = if option.select.is_empty() {
            "*".to_string()
        } else {
            for col in &option.select {
                validate_identifier(col)?;
            }
            option.select.join(", ")
        };

        // 构建 FROM + JOIN 子句
        let mut from_sql = format!("{}", self.table_name());
        for join in &option.joins {
            validate_identifier(&join.entity)?;
            validate_identifier(&join.alias)?;
            if join.condition.contains(|c| {
                c == ';' || c == '#' || c == '\'' || c == '"' || c == '\n' || c == '\r'
            }) {
                return Err(CoolError::validate("关联条件包含非法字符"));
            }

            let join_kw = match join.join_type {
                crate::entity::JoinType::Inner => "INNER JOIN",
                crate::entity::JoinType::Left => "LEFT JOIN",
            };
            from_sql.push(' ');
            from_sql.push_str(join_kw);
            from_sql.push(' ');
            from_sql.push_str(&join.entity);
            from_sql.push_str(" AS ");
            from_sql.push_str(&join.alias);
            from_sql.push_str(" ON ");
            from_sql.push_str(&join.condition);
        }

        let mut where_sql = String::new();
        let mut params: Vec<sea_orm::Value> = Vec::new();

        if let Some(ref kw) = query.key_word {
            if !option.key_word_like_fields.is_empty() {
                validate_keyword(kw)?;
                where_sql.push_str(" WHERE ");
                for (idx, col) in option.key_word_like_fields.iter().enumerate() {
                    validate_identifier(col)?;
                    if idx > 0 {
                        where_sql.push_str(" OR ");
                    }
                    where_sql.push_str(&format!("{} LIKE ?", col));
                    params.push(sea_orm::Value::String(Some(Box::new(format!("%{}%", kw)))));
                }
            }
        }

        let mut order_sql = String::new();
        if let Some(ref order_field) = query.order {
            validate_identifier(order_field)?;
            let asc = query
                .sort
                .as_ref()
                .map(|s| s.to_lowercase() == "asc")
                .unwrap_or(false);
            order_sql.push_str(" ORDER BY ");
            order_sql.push_str(order_field);
            order_sql.push_str(if asc { " ASC" } else { " DESC" });
        } else if !option.order_by.is_empty() {
            order_sql.push_str(" ORDER BY ");
            let mut first = true;
            for (col, asc) in &option.order_by {
                validate_identifier(col)?;
                if !first {
                    order_sql.push_str(", ");
                }
                first = false;
                order_sql.push_str(col);
                order_sql.push_str(if *asc { " ASC" } else { " DESC" });
            }
        }

        let sql = format!(
            "SELECT {} FROM {}{}{}",
            select_sql, from_sql, where_sql, order_sql
        );
        let stmt = Statement::from_sql_and_values(self.db().get_database_backend(), sql, params);
        let results = self.db().query_all(stmt).await?;

        Ok(results.into_iter().map(|row| self.map_row(row)).collect())
    }

    /// 原生 SQL 查询
    async fn native_query(&self, sql: &str, params: Vec<Value>) -> CoolResult<Vec<Value>> {
        let values: Vec<sea_orm::Value> =
            params.into_iter().map(|v| json_to_sea_value(&v)).collect();

        let stmt = Statement::from_sql_and_values(self.db().get_database_backend(), sql, values);

        let results = self.db().query_all(stmt).await?;

        Ok(results.into_iter().map(|row| self.map_row(row)).collect())
    }

    /// 执行 SQL
    async fn execute(&self, sql: &str) -> CoolResult<u64> {
        let stmt = Statement::from_string(self.db().get_database_backend(), sql.to_string());
        let result = self.db().execute(stmt).await?;
        Ok(result.rows_affected())
    }

    /// 修改前置钩子
    async fn modify_before(&self, data: Value, _modify_type: ModifyType) -> CoolResult<Value> {
        Ok(data)
    }

    /// 修改后置钩子
    async fn modify_after(&self, _data: Value, _modify_type: ModifyType) -> CoolResult<()> {
        Ok(())
    }

    /// 行数据映射
    ///
    /// 说明：
    /// - 默认实现返回调试字符串，保证兼容所有数据库类型
    /// - 具体业务 Service 可以重写此方法，根据表结构把 `QueryResult` 精准映射为字段级 JSON
    fn map_row(&self, row: sea_orm::QueryResult) -> Value {
        Value::String(format!("{:?}", row))
    }
}

/// JSON Value 转 SeaORM Value
fn json_to_sea_value(v: &Value) -> sea_orm::Value {
    match v {
        Value::Null => sea_orm::Value::String(None),
        Value::Bool(b) => sea_orm::Value::Bool(Some(*b)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                sea_orm::Value::BigInt(Some(i))
            } else if let Some(f) = n.as_f64() {
                sea_orm::Value::Double(Some(f))
            } else {
                sea_orm::Value::String(Some(Box::new(n.to_string())))
            }
        }
        Value::String(s) => sea_orm::Value::String(Some(Box::new(s.clone()))),
        _ => sea_orm::Value::String(Some(Box::new(v.to_string()))),
    }
}

/// 将 SeaORM 行数据转换为 `serde_json::Value`
///
/// 说明：
/// - 遍历行中的列和值，根据列名构建 JSON 对象
/// - 只处理常见基础类型，其余类型统一转为字符串，保证不 panic
#[allow(dead_code)]
fn row_to_json(row: sea_orm::QueryResult) -> Value {
    // 目前 SeaORM 未公开 QueryResult 的列/值字段结构，
    // 这里先使用 Debug 格式整体输出，保证接口可用。
    Value::String(format!("{:?}", row))
}

/// 校验 SQL 标识符（列名、排序字段等）的安全性
///
/// 约束：
/// - 仅允许字母、数字、下划线、点、逗号和空格
/// - 不允许出现 SQL 关键字符组合：`--`、`/*`、`*/`、`;`
fn validate_identifier(ident: &str) -> CoolResult<()> {
    if ident.is_empty() {
        return Err(CoolError::validate("字段名不能为空"));
    }

    if ident.contains("--") || ident.contains("/*") || ident.contains("*/") || ident.contains(';') {
        return Err(CoolError::validate("字段名包含非法字符"));
    }

    if !ident
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == ',' || c == ' ')
    {
        return Err(CoolError::validate(
            "字段名仅允许字母、数字、下划线、点、逗号和空格",
        ));
    }

    Ok(())
}

/// 校验关键字搜索内容，避免过长或明显注入字符
fn validate_keyword(kw: &str) -> CoolResult<()> {
    // 限制关键字长度，避免异常长输入
    if kw.len() > 256 {
        return Err(CoolError::validate("关键字过长"));
    }

    // 简单过滤明显的注入特征
    if kw.contains("--") || kw.contains("/*") || kw.contains("*/") || kw.contains(';') {
        return Err(CoolError::validate("关键字包含非法字符"));
    }

    Ok(())
}

/// 对查询配置做基础安全检查
fn validate_query_safety(query: &PageQuery, option: &QueryOption) -> CoolResult<()> {
    // 校验关键字
    if let Some(ref kw) = query.key_word {
        validate_keyword(kw)?;
    }

    // 校验模糊查询字段
    for col in &option.key_word_like_fields {
        validate_identifier(col)?;
    }

    Ok(())
}

/// 简化版服务
///
/// 用于快速创建服务实例
pub struct SimpleService {
    db: Arc<DatabaseConnection>,
    table: String,
}

impl SimpleService {
    pub fn new(db: Arc<DatabaseConnection>, table: impl Into<String>) -> Self {
        Self {
            db,
            table: table.into(),
        }
    }
}

#[async_trait]
impl BaseService for SimpleService {
    fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    fn table_name(&self) -> &str {
        &self.table
    }
}
