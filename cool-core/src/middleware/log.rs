//! 日志中间件
//!
//! 对应 TypeScript 版本的 `middleware/log.ts`

use super::DepotExt;
use salvo::prelude::*;
use std::time::Instant;
use tracing::{info, warn};

/// 请求日志中间件
pub struct RequestLogMiddleware {
    /// 是否记录请求体
    log_body: bool,
    /// 忽略的路径
    ignore_paths: Vec<String>,
}

impl RequestLogMiddleware {
    pub fn new() -> Self {
        Self {
            log_body: false,
            ignore_paths: vec![],
        }
    }

    /// 设置是否记录请求体
    pub fn log_body(mut self, enable: bool) -> Self {
        self.log_body = enable;
        self
    }

    /// 添加忽略的路径
    pub fn ignore_path(mut self, path: impl Into<String>) -> Self {
        self.ignore_paths.push(path.into());
        self
    }
}

impl Default for RequestLogMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Handler for RequestLogMiddleware {
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        let path = req.uri().path().to_string();

        // 检查是否在忽略列表中
        for ignore_path in &self.ignore_paths {
            if path.starts_with(ignore_path) {
                ctrl.call_next(req, depot, res).await;
                return;
            }
        }

        let method = req.method().to_string();
        let start = Instant::now();

        // 获取客户端 IP
        let client_ip = req
            .remote_addr()
            .as_ipv4()
            .map(|addr| addr.to_string())
            .or_else(|| req.remote_addr().as_ipv6().map(|addr| addr.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        // 获取用户信息
        let user_id = depot
            .admin()
            .map(|admin| admin.user_id.to_string())
            .unwrap_or_else(|| "anonymous".to_string());

        // 执行请求
        ctrl.call_next(req, depot, res).await;

        // 计算耗时
        let duration = start.elapsed();
        let status = res.status_code.unwrap_or(StatusCode::OK);

        // 记录日志
        if status.is_success() {
            info!(
                method = %method,
                path = %path,
                status = %status.as_u16(),
                duration_ms = %duration.as_millis(),
                ip = %client_ip,
                user_id = %user_id,
                "request completed"
            );
        } else {
            warn!(
                method = %method,
                path = %path,
                status = %status.as_u16(),
                duration_ms = %duration.as_millis(),
                ip = %client_ip,
                user_id = %user_id,
                "request failed"
            );
        }
    }
}

/// 创建请求日志中间件
pub fn request_log() -> RequestLogMiddleware {
    RequestLogMiddleware::new()
}
