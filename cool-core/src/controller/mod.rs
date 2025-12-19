//! 控制器模块
//!
//! 对应 TypeScript 版本的 `controller/`

use crate::entity::{DeleteParam, ListQuery, PageQuery, QueryOption};
use crate::error::{CoolError, CoolResponse, CoolResult};
use crate::service::BaseService;
use salvo::prelude::*;
use serde_json::Value;

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
fn get_crud_service(depot: &Depot) -> CoolResult<&dyn BaseService> {
    if let Ok(svc) = depot.get::<&'static dyn BaseService>("crud_service") {
        Ok(*svc)
    } else {
        Err(CoolError::comm(
            "未找到 CRUD 服务实例，请先在 Depot 中注册 `crud_service`",
        ))
    }
}

/// 新增接口
#[handler]
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
#[handler]
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
#[handler]
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
#[handler]
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
#[handler]
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
#[handler]
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

/// 构建 CRUD 路由
///
/// 根据控制器配置自动生成路由
pub fn build_crud_router(prefix: &str, apis: &[CrudApi]) -> Router {
    let mut router = Router::with_path(prefix);

    for api in apis {
        match api {
            CrudApi::Add => {
                router = router.push(Router::with_path("add").post(handle_add));
            }
            CrudApi::Delete => {
                router = router.push(Router::with_path("delete").post(handle_delete));
            }
            CrudApi::Update => {
                router = router.push(Router::with_path("update").post(handle_update));
            }
            CrudApi::Page => {
                router = router.push(Router::with_path("page").post(handle_page));
            }
            CrudApi::Info => {
                router = router.push(Router::with_path("info").get(handle_info));
            }
            CrudApi::List => {
                router = router.push(Router::with_path("list").post(handle_list));
            }
        }
    }

    router
}
