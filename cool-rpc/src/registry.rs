//! RPC 服务元数据注册表
//!
//! 用于配合宏 `#[cool_rpc_service]` 暴露 RPC 服务的元信息，
//! 方便在运行时进行服务发现、文档生成或调试。

use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RPC 服务元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcServiceMeta {
    /// 服务名称
    pub name: String,
    /// 方法列表
    pub methods: Vec<String>,
}

/// RPC 服务注册表
#[derive(Default)]
pub struct RpcRegistry {
    services: RwLock<HashMap<String, RpcServiceMeta>>,
}

impl RpcRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册服务元数据
    pub fn register_service(&self, meta: RpcServiceMeta) {
        let mut map = self.services.write();
        map.insert(meta.name.clone(), meta);
    }

    /// 获取所有服务元数据
    pub fn services(&self) -> HashMap<String, RpcServiceMeta> {
        self.services.read().clone()
    }
}

static GLOBAL_RPC_REGISTRY: OnceCell<RpcRegistry> = OnceCell::new();

/// 获取全局 RPC 注册表
pub fn global_rpc_registry() -> &'static RpcRegistry {
    GLOBAL_RPC_REGISTRY.get_or_init(RpcRegistry::default)
}

/// RPC 元数据 HTTP 接口（可选）
///
/// 对标 TS 版本里基于装饰器生成的 RPC 服务文档，这里提供一个简单的
/// JSON 接口，方便在前端或调试工具中查看已注册的 RPC 服务与方法。
#[cfg(feature = "web")]
pub mod handler {
    use super::*;
    use salvo::prelude::*;
    use serde_json::json;

    /// 获取所有 RPC 服务元数据
    #[handler]
    pub async fn rpc_services(_req: &mut Request, res: &mut Response) {
        let registry = global_rpc_registry();
        let data = json!({
            "services": registry.services(),
        });
        res.render(Json(data));
    }

    /// 构建 RPC 元数据路由
    ///
    /// 示例：
    /// ```rust,ignore
    /// use cool_rpc::registry::handler::rpc_registry_router;
    /// let router = rpc_registry_router("/rpc/services");
    /// ```
    pub fn rpc_registry_router(path: &str) -> salvo::Router {
        Router::with_path(path).get(rpc_services)
    }
}
