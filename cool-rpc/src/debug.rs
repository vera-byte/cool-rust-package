//! RPC 调试接口（可选）
//!
//! 对标 TS 里常见的测试/调试控制器，这里提供两个简单的 HTTP 接口：
//! - `/rpc/debug/call` 远程调用
//! - `/rpc/debug/emit` 广播事件
//!
//! 约定：在路由构建时，把 `CoolRpc` 实例存入 `Depot`，key 为 `"cool_rpc"`。
//! 例如：
//! ```rust,ignore
//! let rpc = Arc::new(CoolRpc::new(cfg).await?);
//! let router = rpc_debug_router("/rpc/debug/call", "/rpc/debug/emit")
//!     .hoop(move |depot: &mut Depot, _| {
//!         depot.insert("cool_rpc", Arc::clone(&rpc));
//!         Ok(())
//!     });
//! ```

#[cfg(feature = "web")]
pub mod handler {
    use crate::broker::CoolRpc;
    use salvo::prelude::*;
    use serde::Deserialize;
    use serde_json::json;
    use std::sync::Arc;

    #[derive(Debug, Deserialize)]
    pub struct CallPayload {
        /// 目标节点服务名（注册的 serviceName，例如 "user-service"）
        pub target_service: String,
        /// 具体 Service 名称（注册时的名称）
        pub service: String,
        /// 方法名
        pub method: String,
        /// 参数（可选）
        #[serde(default)]
        pub params: serde_json::Value,
    }

    #[derive(Debug, Deserialize)]
    pub struct EmitPayload {
        /// 事件名
        pub event: String,
        /// 事件数据
        #[serde(default)]
        pub data: serde_json::Value,
    }

    /// 从 Depot 获取 CoolRpc
    fn get_rpc(depot: &Depot) -> Option<Arc<CoolRpc>> {
        depot.get::<Arc<CoolRpc>>("cool_rpc").cloned()
    }

    /// 远程调用调试接口
    #[handler]
    pub async fn rpc_debug_call(req: &mut Request, res: &mut Response, depot: &mut Depot) {
        let payload: CallPayload = match req.parse_json().await {
            Ok(v) => v,
            Err(e) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(json!({ "success": false, "error": e.to_string() })));
                return;
            }
        };

        let rpc = match get_rpc(depot) {
            Some(r) => r,
            None => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(json!({ "success": false, "error": "cool_rpc 未注入，请在路由构建时设置 depot.insert(\"cool_rpc\", Arc<CoolRpc>)" })));
                return;
            }
        };

        match rpc
            .call(
                &payload.target_service,
                &payload.service,
                &payload.method,
                payload.params,
            )
            .await
        {
            Ok(data) => res.render(Json(json!({ "success": true, "data": data }))),
            Err(e) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(json!({ "success": false, "error": e.to_string() })));
            }
        }
    }

    /// 广播事件调试接口
    #[handler]
    pub async fn rpc_debug_emit(req: &mut Request, res: &mut Response, depot: &mut Depot) {
        let payload: EmitPayload = match req.parse_json().await {
            Ok(v) => v,
            Err(e) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(json!({ "success": false, "error": e.to_string() })));
                return;
            }
        };

        let rpc = match get_rpc(depot) {
            Some(r) => r,
            None => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(json!({ "success": false, "error": "cool_rpc 未注入，请在路由构建时设置 depot.insert(\"cool_rpc\", Arc<CoolRpc>)" })));
                return;
            }
        };

        match rpc.emit(&payload.event, payload.data).await {
            Ok(_) => res.render(Json(json!({ "success": true }))),
            Err(e) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(json!({ "success": false, "error": e.to_string() })));
            }
        }
    }

    /// 构建调试路由
    pub fn rpc_debug_router(call_path: &str, emit_path: &str) -> Router {
        Router::new()
            .push(Router::with_path(call_path).post(rpc_debug_call))
            .push(Router::with_path(emit_path).post(rpc_debug_emit))
    }
}

