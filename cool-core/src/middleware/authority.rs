//! 权限中间件
//!
//! 对应 TypeScript 版本的 `middleware/authority.ts`

use super::{AdminInfo, DepotExt, JwtClaims};
use crate::error::{CoolError, CoolResponse};
use salvo::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;

/// 权限中间件配置
#[derive(Debug, Clone)]
pub struct AuthorityConfig {
    /// JWT 密钥
    pub jwt_secret: String,
    /// 忽略 Token 验证的 URL
    pub ignore_urls: HashSet<String>,
    /// Token Header 名称
    pub token_header: String,
}

impl Default for AuthorityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "cool-admin-rust".to_string(),
            ignore_urls: HashSet::new(),
            token_header: "Authorization".to_string(),
        }
    }
}

impl AuthorityConfig {
    pub fn new(jwt_secret: impl Into<String>) -> Self {
        Self {
            jwt_secret: jwt_secret.into(),
            ..Default::default()
        }
    }

    /// 添加忽略的 URL
    pub fn ignore_url(mut self, url: impl Into<String>) -> Self {
        self.ignore_urls.insert(url.into());
        self
    }

    /// 批量添加忽略的 URL
    pub fn ignore_urls<I, S>(mut self, urls: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for url in urls {
            self.ignore_urls.insert(url.into());
        }
        self
    }
}

/// 权限中间件
pub struct AuthorityMiddleware {
    config: Arc<AuthorityConfig>,
}

impl AuthorityMiddleware {
    pub fn new(config: AuthorityConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

#[async_trait]
impl Handler for AuthorityMiddleware {
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        let path = req.uri().path();

        // 检查是否在忽略列表中
        if self.config.ignore_urls.contains(path) {
            ctrl.call_next(req, depot, res).await;
            return;
        }

        // 检查路径是否以忽略的前缀开头
        for ignore_url in &self.config.ignore_urls {
            if ignore_url.ends_with("*") {
                let prefix = &ignore_url[..ignore_url.len() - 1];
                if path.starts_with(prefix) {
                    ctrl.call_next(req, depot, res).await;
                    return;
                }
            }
        }

        // 获取 Token
        let token = req
            .header::<String>(&self.config.token_header)
            .or_else(|| req.query::<String>("token"));

        let token = match token {
            Some(t) => {
                // 移除 "Bearer " 前缀
                if t.starts_with("Bearer ") {
                    t[7..].to_string()
                } else {
                    t
                }
            }
            None => {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(CoolResponse::<()>::from_error(&CoolError::unauthorized())));
                ctrl.skip_rest();
                return;
            }
        };

        // 验证 Token
        match JwtClaims::verify_token(&token, &self.config.jwt_secret) {
            Ok(claims) => {
                // 检查是否过期
                let now = chrono::Utc::now().timestamp();
                if claims.exp < now {
                    res.status_code(StatusCode::UNAUTHORIZED);
                    res.render(Json(CoolResponse::<()>::fail("Token 已过期")));
                    ctrl.skip_rest();
                    return;
                }

                // 设置用户信息到 Depot
                depot.set_admin(AdminInfo::from(claims));
                ctrl.call_next(req, depot, res).await;
            }
            Err(e) => {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(CoolResponse::<()>::from_error(&e)));
                ctrl.skip_rest();
            }
        }
    }
}

/// 创建权限中间件
pub fn authority(config: AuthorityConfig) -> AuthorityMiddleware {
    AuthorityMiddleware::new(config)
}

