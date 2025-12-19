//! RPC 测试工具
//!
//! 对应 TypeScript 版本的 `rpc/src/test.ts`
//!
//! 提供本地开发调试用的 RPC 测试接口

use crate::broker::CoolRpc;
use crate::RpcError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// RPC 测试请求
#[derive(Debug, Serialize, Deserialize)]
pub struct RpcTestRequest {
    /// 服务名称
    pub name: String,
    /// 服务类名
    pub service: String,
    /// 方法名
    pub method: String,
    /// 参数
    pub params: Value,
}

/// RPC 测试响应
#[derive(Debug, Serialize, Deserialize)]
pub struct RpcTestResponse {
    /// 是否成功
    pub success: bool,
    /// 响应数据
    pub data: Option<Value>,
    /// 错误信息
    pub error: Option<String>,
}

/// RPC 测试工具
///
/// 提供本地开发调试用的 RPC 测试功能
pub struct RpcTest {
    rpc: CoolRpc,
}

impl RpcTest {
    /// 创建 RPC 测试工具
    pub fn new(rpc: CoolRpc) -> Self {
        Self { rpc }
    }

    /// 测试 RPC 调用
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let test = RpcTest::new(rpc);
    /// let result = test.test(RpcTestRequest {
    ///     name: "user-service".to_string(),
    ///     service: "UserService".to_string(),
    ///     method: "getById".to_string(),
    ///     params: json!({"id": 1}),
    /// }).await?;
    /// ```
    pub async fn test(&self, request: RpcTestRequest) -> Result<RpcTestResponse, RpcError> {
        match self
            .rpc
            .call(
                &request.name,
                &request.service,
                &request.method,
                request.params,
            )
            .await
        {
            Ok(data) => Ok(RpcTestResponse {
                success: true,
                data: Some(data),
                error: None,
            }),
            Err(e) => Ok(RpcTestResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }),
        }
    }
}

#[cfg(feature = "web")]
pub mod handler {
    //! HTTP 处理器（需要 web feature）
    //!
    //! 提供 HTTP 接口用于测试 RPC 调用

    use super::{RpcTest, RpcTestRequest, RpcTestResponse};
    use crate::broker::CoolRpc;
    use salvo::prelude::*;

    /// 创建 RPC 测试路由
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use salvo::prelude::*;
    ///
    /// let rpc = CoolRpc::new(config).await?;
    /// let router = Router::new()
    ///     .push(create_rpc_test_router(rpc));
    /// ```
    pub fn create_rpc_test_router(rpc: CoolRpc) -> Router {
        Router::with_path("/rpc/test")
            .post(test_handler)
            .data(RpcTest::new(rpc))
    }

    /// 测试处理器
    #[handler]
    async fn test_handler(
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
    ) -> Result<(), salvo::Error> {
        let test = depot
            .obtain::<RpcTest>()
            .ok_or_else(|| salvo::Error::other("RpcTest not found"))?;

        let request: RpcTestRequest = req.parse_json().await?;

        match test.test(request).await {
            Ok(response) => {
                res.render(Json(response));
                Ok(())
            }
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(RpcTestResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }));
                Ok(())
            }
        }
    }
}
