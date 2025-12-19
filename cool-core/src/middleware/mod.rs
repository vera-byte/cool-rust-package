//! 中间件模块
//!
//! 对应 TypeScript 版本的 `middleware/`

mod authority;
mod exception;
mod log;

pub use authority::*;
pub use exception::*;
pub use log::*;

use crate::error::CoolError;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// 用户 ID
    pub user_id: i64,
    /// 用户名
    pub username: Option<String>,
    /// 角色 ID 列表
    pub role_ids: Vec<i64>,
    /// 是否是管理员
    pub is_admin: bool,
    /// 租户 ID
    pub tenant_id: Option<i64>,
    /// 过期时间
    pub exp: i64,
    /// 签发时间
    pub iat: i64,
}

impl JwtClaims {
    /// 创建新的 Claims
    pub fn new(
        user_id: i64,
        username: Option<String>,
        role_ids: Vec<i64>,
        is_admin: bool,
        tenant_id: Option<i64>,
        expire_secs: u64,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            user_id,
            username,
            role_ids,
            is_admin,
            tenant_id,
            exp: now + expire_secs as i64,
            iat: now,
        }
    }

    /// 生成 token
    pub fn generate_token(&self, secret: &str) -> Result<String, CoolError> {
        encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(CoolError::from)
    }

    /// 验证并解析 token
    pub fn verify_token(token: &str, secret: &str) -> Result<Self, CoolError> {
        let token_data = decode::<Self>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(CoolError::from)?;

        Ok(token_data.claims)
    }
}

/// Admin 用户信息（存储在 Depot 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminInfo {
    /// 用户 ID
    pub user_id: i64,
    /// 用户名
    pub username: Option<String>,
    /// 角色 ID 列表
    pub role_ids: Vec<i64>,
    /// 是否是超级管理员
    pub is_admin: bool,
    /// 租户 ID
    pub tenant_id: Option<i64>,
}

impl From<JwtClaims> for AdminInfo {
    fn from(claims: JwtClaims) -> Self {
        Self {
            user_id: claims.user_id,
            username: claims.username,
            role_ids: claims.role_ids,
            is_admin: claims.is_admin,
            tenant_id: claims.tenant_id,
        }
    }
}

/// 从 Depot 获取 Admin 信息的扩展方法
pub trait DepotExt {
    fn admin(&self) -> Option<&AdminInfo>;
    fn set_admin(&mut self, admin: AdminInfo);
}

impl DepotExt for Depot {
    fn admin(&self) -> Option<&AdminInfo> {
        self.get::<AdminInfo>("admin").ok()
    }

    fn set_admin(&mut self, admin: AdminInfo) {
        self.insert("admin", admin);
    }
}

/// App 用户信息（存储在 Depot 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUserInfo {
    /// 用户 ID
    pub id: i64,
    /// 用户名
    pub username: Option<String>,
    /// 手机号
    pub phone: Option<String>,
    /// 租户 ID
    pub tenant_id: Option<i64>,
}

/// 从 Depot 获取 App 用户信息的扩展方法
pub trait DepotAppUserExt {
    fn app_user(&self) -> Option<&AppUserInfo>;
    fn set_app_user(&mut self, user: AppUserInfo);
}

impl DepotAppUserExt for Depot {
    fn app_user(&self) -> Option<&AppUserInfo> {
        self.get::<AppUserInfo>("app_user").ok()
    }

    fn set_app_user(&mut self, user: AppUserInfo) {
        self.insert("app_user", user);
    }
}
