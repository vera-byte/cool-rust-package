//! EPS (Endpoint Service) 模块
//!
//! 对应 TypeScript 版本的 `rest/eps.ts`，用于对外提供
//! - 模块信息
//! - 控制器及其路由信息
//! - 实体列信息
//! - 分页查询配置
//!
//! Rust 版本暂不做自动扫描，而是提供一个可复用的数据结构和全局注册表，
//! 由各业务模块在初始化时主动写入。

use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 模块基础信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    /// 模块名
    pub name: String,
    /// 描述
    pub description: String,
}

/// 列信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    /// 属性名（驼峰）
    pub property_name: String,
    /// 字段类型
    pub r#type: String,
    /// 长度
    pub length: Option<u32>,
    /// 注释
    pub comment: Option<String>,
    /// 是否可空
    pub nullable: bool,
    /// 默认值
    pub default_value: Option<String>,
    /// 字典标记
    pub dict: Option<serde_json::Value>,
    /// 原始来源（如 `a.userName`）
    pub source: String,
}

/// 查询配置（精简版）
///
/// 只保留在前端和 EPS 中需要关注的部分。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueryOpInfo {
    /// 关键字模糊查询字段
    pub key_word_like_fields: Vec<String>,
    /// 字段相等查询
    pub field_eq: Vec<String>,
    /// 字段模糊查询
    pub field_like: Vec<String>,
    /// 排序配置：(字段名, 是否升序)
    pub order_by: Vec<(String, bool)>,
}

/// 路由信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfo {
    /// 请求方法：GET/POST/PUT/DELETE...
    pub method: String,
    /// URL 路径
    pub path: String,
    /// 简要说明
    pub summary: Option<String>,
    /// 标签
    pub tag: Option<String>,
    /// 是否忽略 Token 校验
    pub ignore_token: bool,
}

/// 控制器的 EPS 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerEps {
    /// 所属模块
    pub module: String,
    /// 路由前缀
    pub prefix: String,
    /// 实体名称
    pub entity_name: Option<String>,
    /// 类型信息（例如列表类型说明）
    pub r#type: Option<String>,
    /// 类型描述
    pub description: Option<String>,
    /// 路由列表
    pub api: Vec<RouteInfo>,
    /// 实体列信息
    pub columns: Vec<ColumnInfo>,
    /// 分页查询配置
    pub page_query_op: Option<QueryOpInfo>,
    /// 分页列信息（通常基于 select/join 计算出的平铺列）
    pub page_columns: Vec<ColumnInfo>,
}

/// EPS 数据作用域
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EpsScope {
    /// 后台管理端
    Admin,
    /// App 端
    App,
}

/// EPS 注册表
///
/// 与 Node 版本的 `CoolEps` 中 `admin/app/module` 三个字段的结构保持一致。
#[derive(Default)]
pub struct EpsRegistry {
    admin: RwLock<HashMap<String, Vec<ControllerEps>>>,
    app: RwLock<HashMap<String, Vec<ControllerEps>>>,
    module: RwLock<HashMap<String, ModuleInfo>>,
}

impl EpsRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册模块信息
    pub fn register_module(&self, key: impl Into<String>, info: ModuleInfo) {
        let mut modules = self.module.write();
        modules.insert(key.into(), info);
    }

    /// 注册控制器 EPS 信息
    pub fn register_controller(
        &self,
        scope: EpsScope,
        module: impl Into<String>,
        eps: ControllerEps,
    ) {
        let module_key = module.into();
        let target = match scope {
            EpsScope::Admin => &self.admin,
            EpsScope::App => &self.app,
        };
        let mut map = target.write();
        map.entry(module_key).or_default().push(eps);
    }

    /// 获取后台端的 EPS 数据（按模块分组）
    pub fn admin_eps(&self) -> HashMap<String, Vec<ControllerEps>> {
        self.admin.read().clone()
    }

    /// 获取 App 端的 EPS 数据（按模块分组）
    pub fn app_eps(&self) -> HashMap<String, Vec<ControllerEps>> {
        self.app.read().clone()
    }

    /// 获取模块基础信息
    pub fn modules(&self) -> HashMap<String, ModuleInfo> {
        self.module.read().clone()
    }
}

static GLOBAL_EPS_REGISTRY: OnceCell<EpsRegistry> = OnceCell::new();

/// 获取全局 EPS 注册表
pub fn global_eps_registry() -> &'static EpsRegistry {
    GLOBAL_EPS_REGISTRY.get_or_init(EpsRegistry::default)
}

/// EPS HTTP 处理器（可选 Web 集成）
///
/// 对应 TS 版本中的 `/eps` 接口，用于向前端暴露：
/// - 模块基础信息
/// - Admin/App 端的控制器 EPS 列表
///
/// 路由推荐：
/// - GET `/admin/eps`
/// - GET `/app/eps`
#[cfg(feature = "web")]
pub mod handler {
    use super::*;
    use salvo::prelude::*;
    use serde_json::json;

    /// 获取后台管理端 EPS 数据
    #[handler]
    pub async fn admin_eps(_req: &mut Request, res: &mut Response) {
        let registry = global_eps_registry();
        let data = json!({
            "modules": registry.modules(),
            "eps": registry.admin_eps(),
        });
        res.render(Json(data));
    }

    /// 获取 App 端 EPS 数据
    #[handler]
    pub async fn app_eps(_req: &mut Request, res: &mut Response) {
        let registry = global_eps_registry();
        let data = json!({
            "modules": registry.modules(),
            "eps": registry.app_eps(),
        });
        res.render(Json(data));
    }

    /// 构建 EPS 路由
    ///
    /// 示例：
    /// ```rust,ignore
    /// use cool_core::eps::handler::eps_router;
    /// let router = eps_router("/admin/eps", "/app/eps");
    /// ```
    pub fn eps_router(admin_path: &str, app_path: &str) -> salvo::Router {
        Router::new()
            .push(Router::with_path(admin_path).get(admin_eps))
            .push(Router::with_path(app_path).get(app_eps))
    }
}
