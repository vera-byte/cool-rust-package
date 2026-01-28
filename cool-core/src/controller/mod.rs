//! 控制器模块
//!
//! 对应 TypeScript 版本的 `controller/`

use crate::entity::{DeleteParam, ListQuery, PageQuery, QueryOption};
use crate::error::{CoolError, CoolResponse, CoolResult};
use crate::service::BaseService;
use salvo::prelude::*;
use serde_json::Value;
use std::sync::Arc;

/// CRUD API 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrudApi {
    Add,
    Delete,
    Update,
    Page,
    Info,
    List,
}

impl CrudApi {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Add,
            Self::Delete,
            Self::Update,
            Self::Page,
            Self::Info,
            Self::List,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Delete => "delete",
            Self::Update => "update",
            Self::Page => "page",
            Self::Info => "info",
            Self::List => "list",
        }
    }
}

/// 控制器配置
#[derive(Debug, Clone, Default)]
pub struct ControllerOption {
    /// 路由前缀
    pub prefix: Option<String>,
    /// 启用的 API
    pub api: Vec<CrudApi>,
    /// 分页查询配置
    pub page_query_op: QueryOption,
    /// 列表查询配置
    pub list_query_op: QueryOption,
    /// info 忽略返回属性
    pub info_ignore_property: Vec<String>,
}

/// 控制器基类 trait
///
/// 提供通用的 CRUD 接口处理
#[async_trait]
pub trait BaseController: Send + Sync + 'static {
    /// 获取控制器配置
    fn option(&self) -> &ControllerOption;

    /// 获取表名
    fn table_name(&self) -> &str;

    /// 获取数据库连接
    fn db(&self) -> &sea_orm::DatabaseConnection;
}

/// CRUD 处理器
///
/// 用于处理通用的 CRUD 请求
#[allow(dead_code)]
pub struct CrudHandler {
    table: String,
    db: std::sync::Arc<sea_orm::DatabaseConnection>,
}

impl CrudHandler {
    pub fn new(table: impl Into<String>, db: std::sync::Arc<sea_orm::DatabaseConnection>) -> Self {
        Self {
            table: table.into(),
            db,
        }
    }
}

/// 从 Depot 中获取通用 CRUD Service
fn get_crud_service(depot: &Depot) -> CoolResult<std::sync::Arc<dyn BaseService + Send + Sync>> {
    tracing::debug!("尝试从 Depot 获取 CRUD 服务");
    match depot.obtain::<std::sync::Arc<dyn BaseService + Send + Sync>>() {
        Ok(svc) => {
            tracing::debug!("成功获取 CRUD 服务");
            Ok(svc.clone())
        }
        Err(e) => {
            tracing::debug!("获取 CRUD 服务失败: {:?}", e);
            Err(CoolError::comm(
                "未找到 CRUD 服务实例，请先在 Depot 中注册 `crud_service`",
            ))
        }
    }
}

/// 新增接口
pub async fn handle_add(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    let body: Value = match req.parse_json().await {
        Ok(v) => v,
        Err(e) => {
            res.render(Json(CoolResponse::<()>::fail(e.to_string())));
            return;
        }
    };

    let service = match get_crud_service(depot) {
        Ok(s) => s,
        Err(e) => {
            res.render(Json(CoolResponse::<()>::from_error(&e)));
            return;
        }
    };

    match service.add(body).await {
        Ok(result) => res.render(Json(CoolResponse::ok(result))),
        Err(e) => res.render(Json(CoolResponse::<()>::from_error(&e))),
    }
}

/// 删除接口
pub async fn handle_delete(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    let param: DeleteParam = match req.parse_json().await {
        Ok(v) => v,
        Err(e) => {
            res.render(Json(CoolResponse::<()>::fail(e.to_string())));
            return;
        }
    };

    let service = match get_crud_service(depot) {
        Ok(s) => s,
        Err(e) => {
            res.render(Json(CoolResponse::<()>::from_error(&e)));
            return;
        }
    };

    match service.delete(param).await {
        Ok(_) => res.render(Json(CoolResponse::<()>::ok_empty())),
        Err(e) => res.render(Json(CoolResponse::<()>::from_error(&e))),
    }
}

/// 修改接口
pub async fn handle_update(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    let body: Value = match req.parse_json().await {
        Ok(v) => v,
        Err(e) => {
            res.render(Json(CoolResponse::<()>::fail(e.to_string())));
            return;
        }
    };

    let service = match get_crud_service(depot) {
        Ok(s) => s,
        Err(e) => {
            res.render(Json(CoolResponse::<()>::from_error(&e)));
            return;
        }
    };

    match service.update(body).await {
        Ok(_) => res.render(Json(CoolResponse::<()>::ok_empty())),
        Err(e) => res.render(Json(CoolResponse::<()>::from_error(&e))),
    }
}

/// 分页查询接口
pub async fn handle_page(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    let query: PageQuery = req.parse_queries().unwrap_or_default();

    let service = match get_crud_service(depot) {
        Ok(s) => s,
        Err(e) => {
            res.render(Json(CoolResponse::<()>::from_error(&e)));
            return;
        }
    };

    // 这里暂时使用默认的 QueryOption，后续可以按需从 Depot 或配置中注入
    let option = QueryOption::default();

    match service.page(query, option).await {
        Ok(result) => res.render(Json(CoolResponse::ok(result))),
        Err(e) => res.render(Json(CoolResponse::<()>::from_error(&e))),
    }
}

/// 详情查询接口
pub async fn handle_info(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    let id: Option<i64> = req.query("id").and_then(|s: &str| s.parse::<i64>().ok());

    let id = match id {
        Some(id) => id,
        None => {
            res.render(Json(CoolResponse::<()>::fail("缺少 id 参数")));
            return;
        }
    };

    let service = match get_crud_service(depot) {
        Ok(s) => s,
        Err(e) => {
            res.render(Json(CoolResponse::<()>::from_error(&e)));
            return;
        }
    };

    // 暂不处理 ignore_property，可在上层自行过滤字段
    match service.info(id, None).await {
        Ok(data) => res.render(Json(CoolResponse::ok(data))),
        Err(e) => res.render(Json(CoolResponse::<()>::from_error(&e))),
    }
}

/// 列表查询接口
pub async fn handle_list(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    let query: ListQuery = req.parse_queries().unwrap_or_default();

    let service = match get_crud_service(depot) {
        Ok(s) => s,
        Err(e) => {
            res.render(Json(CoolResponse::<()>::from_error(&e)));
            return;
        }
    };

    // 这里暂时使用默认的 QueryOption，后续可以按需从 Depot 或配置中注入
    let option = QueryOption::default();

    match service.list(query, option).await {
        Ok(result) => res.render(Json(CoolResponse::ok(result))),
        Err(e) => res.render(Json(CoolResponse::<()>::from_error(&e))),
    }
}

/// CRUD 服务中间件
///
/// 将服务注入到 Depot 中供 CRUD handlers 使用
pub struct CrudServiceMiddleware {
    pub service: std::sync::Arc<dyn BaseService + Send + Sync>,
}

impl CrudServiceMiddleware {
    pub fn new(service: std::sync::Arc<dyn BaseService + Send + Sync>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Handler for CrudServiceMiddleware {
    async fn handle(
        &self,
        _req: &mut Request,
        _depot: &mut Depot,
        _res: &mut Response,
        _ctrl: &mut FlowCtrl,
    ) {
        // 将服务注入到 Depot 中
        tracing::debug!("注入 CRUD 服务到 Depot");
        _depot.inject(self.service.clone());
        _ctrl.call_next(_req, _depot, _res).await;
    }
}

/// 带 OpenAPI 注解的新增处理函数
#[salvo::handler]
pub async fn handle_add_with_oapi(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    handle_add(req, res, depot).await;
}

/// 带 OpenAPI 注解的删除处理函数
#[salvo::handler]
pub async fn handle_delete_with_oapi(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    handle_delete(req, res, depot).await;
}

/// 带 OpenAPI 注解的更新处理函数
#[salvo::handler]
pub async fn handle_update_with_oapi(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    handle_update(req, res, depot).await;
}

/// 带 OpenAPI 注解的分页处理函数
#[salvo::handler]
pub async fn handle_page_with_oapi(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    handle_page(req, res, depot).await;
}

/// 带 OpenAPI 注解的详情处理函数
#[salvo::handler]
pub async fn handle_info_with_oapi(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    handle_info(req, res, depot).await;
}

/// 带 OpenAPI 注解的列表处理函数
#[salvo::handler]
pub async fn handle_list_with_oapi(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    handle_list(req, res, depot).await;
}

/// 构建带有 OpenAPI 注解的 CRUD 路由（使用 Arc<dyn BaseService>）
///
/// 此函数会自动为所有 CRUD 操作添加 OpenAPI 注解
pub fn build_crud_router_with_arc(
    service: Arc<dyn BaseService + Send + Sync>,
    option: &ControllerOption,
) -> Router {
    let prefix = option.prefix.as_deref().unwrap_or("/");
    let mut router = Router::with_path(prefix);

    // 添加服务中间件
    let middleware = CrudServiceMiddleware { service };
    router = router.hoop(middleware);

    // 为每个 API 添加路由和 OpenAPI 注解
    for api in &option.api {
        match api {
            CrudApi::Add => {
                router = router.push(
                    Router::with_path("add")
                        .post(handle_add_with_oapi)
                        .oapi_tag("商品管理"),
                );
            }
            CrudApi::Delete => {
                router = router.push(
                    Router::with_path("delete")
                        .post(handle_delete_with_oapi)
                        .oapi_tag("商品管理"),
                );
            }
            CrudApi::Update => {
                router = router.push(
                    Router::with_path("update")
                        .post(handle_update_with_oapi)
                        .oapi_tag("商品管理"),
                );
            }
            CrudApi::Page => {
                router = router.push(
                    Router::with_path("page")
                        .post(handle_page_with_oapi)
                        .oapi_tag("商品管理"),
                );
            }
            CrudApi::Info => {
                router = router.push(
                    Router::with_path("info")
                        .get(handle_info_with_oapi)
                        .oapi_tag("商品管理"),
                );
            }
            CrudApi::List => {
                router = router.push(
                    Router::with_path("list")
                        .post(handle_list_with_oapi)
                        .oapi_tag("商品管理"),
                );
            }
        }
    }

    router
}
