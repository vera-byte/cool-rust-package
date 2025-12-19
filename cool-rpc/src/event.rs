//! RPC 事件

use async_trait::async_trait;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RPC 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcEvent {
    /// 事件名称
    pub name: String,
    /// 事件数据
    pub data: serde_json::Value,
    /// 事件来源节点
    pub source: String,
    /// 时间戳
    pub timestamp: i64,
}

/// 事件处理器 trait
#[async_trait]
pub trait RpcEventHandler: Send + Sync {
    /// 处理事件
    async fn handle(&self, event: &RpcEvent);

    /// 获取事件名称（None 表示监听所有事件）
    fn event_name(&self) -> Option<&str> {
        None
    }
}

/// RPC 事件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcEventMeta {
    /// 事件名称
    pub name: String,
    /// 处理函数名称（仅用于调试/文档）
    pub handler: String,
}

/// RPC 事件注册表
#[derive(Default)]
pub struct RpcEventRegistry {
    events: RwLock<HashMap<String, Vec<RpcEventMeta>>>,
}

impl RpcEventRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册事件处理器元数据
    pub fn register(&self, service: &str, meta: RpcEventMeta) {
        let mut map = self.events.write();
        map.entry(service.to_string()).or_default().push(meta);
    }

    /// 获取所有事件元数据
    pub fn all(&self) -> HashMap<String, Vec<RpcEventMeta>> {
        self.events.read().clone()
    }
}

static GLOBAL_RPC_EVENT_REGISTRY: OnceCell<RpcEventRegistry> = OnceCell::new();

/// 获取全局事件注册表
pub fn global_event_registry() -> &'static RpcEventRegistry {
    GLOBAL_RPC_EVENT_REGISTRY.get_or_init(RpcEventRegistry::default)
}

/// 事件元数据 HTTP 接口（可选）
#[cfg(feature = "web")]
pub mod handler {
    use super::*;
    use salvo::prelude::*;
    use serde_json::json;

    /// 获取所有 RPC 事件元数据
    #[handler]
    pub async fn rpc_events(_req: &mut Request, res: &mut Response) {
        let registry = global_event_registry();
        let data = json!({
            "events": registry.all(),
        });
        res.render(Json(data));
    }

    /// 构建 RPC 事件路由
    pub fn rpc_event_router(path: &str) -> salvo::Router {
        Router::with_path(path).get(rpc_events)
    }
}
