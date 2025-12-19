//! 异常过滤器中间件
//!
//! 对应 TypeScript 版本的 `exception/filter.ts`
//!
//! 提供全局异常处理，统一错误响应格式

use crate::constant::ResCode;
use crate::error::CoolError;
use salvo::prelude::*;
use serde::Serialize;
use tracing::error;

/// 错误响应结构
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// 错误码
    pub code: i32,
    /// 错误消息
    pub message: String,
}

/// 异常过滤器中间件
///
/// 捕获所有异常并统一格式化为错误响应
///
/// 对应 TypeScript 版本的 `exception/filter.ts`
///
/// 注意：在 Salvo 中，错误通常通过 `?` 操作符自动转换为 HTTP 响应。
/// 这个中间件主要用于统一错误响应格式，确保所有错误都返回统一的 JSON 格式。
pub struct ExceptionFilter;

#[async_trait::async_trait]
impl Handler for ExceptionFilter {
    async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
        // 先执行后续处理器
        ctrl.call_next(req, depot, res).await;

        // 获取状态码
        let status = res.status_code.unwrap_or(StatusCode::OK);
        
        // 如果响应状态码是错误状态码，且还没有设置响应体，则统一格式化
        if status.is_client_error() || status.is_server_error() {
            // 检查是否已经有响应体
            if res.body.is_none() {
                // 尝试从 depot 获取错误信息
                if let Ok(err_msg) = depot.get::<String>("error") {
                    error!("Request error: {}", err_msg);
                    let error_response = ErrorResponse {
                        code: status.as_u16() as i32,
                        message: err_msg.clone(),
                    };
                    res.render(Json(error_response));
                } else {
                    // 默认错误响应
                    let error_response = ErrorResponse {
                        code: ResCode::CommError.code(),
                        message: "请求处理失败".to_string(),
                    };
                    res.render(Json(error_response));
                }
            }
        }
    }
}

/// 创建异常过滤器中间件
pub fn exception_filter() -> ExceptionFilter {
    ExceptionFilter
}

/// 错误处理辅助函数
///
/// 将 CoolError 转换为统一的错误响应
pub fn handle_error(err: &CoolError) -> ErrorResponse {
    error!("CoolError: {:?}", err);
    ErrorResponse {
        code: err.code().code(),
        message: err.to_string(),
    }
}

