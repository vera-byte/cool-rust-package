//! 异常处理模块
//!
//! 对应 TypeScript 版本的 `exception/`

use crate::constant::{error_info, ResCode};
use salvo::prelude::*;
use serde::Serialize;
use thiserror::Error;

/// Cool 框架统一错误类型
#[derive(Error, Debug)]
pub enum CoolError {
    /// 核心异常
    #[error("{0}")]
    Core(String),

    /// 验证异常
    #[error("{0}")]
    Validate(String),

    /// 通用业务异常
    #[error("{0}")]
    Comm(String),

    /// 未授权
    #[error("{0}")]
    Unauthorized(String),

    /// 禁止访问
    #[error("{0}")]
    Forbidden(String),

    /// 未找到
    #[error("{0}")]
    NotFound(String),

    /// 数据库错误
    #[error("数据库错误: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// JSON 序列化错误
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),

    /// Redis 错误
    #[error("Redis 错误: {0}")]
    Redis(#[from] redis::RedisError),

    /// JWT 错误
    #[error("JWT 错误: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    /// 其他错误
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl CoolError {
    /// 创建核心异常
    pub fn core<S: Into<String>>(msg: S) -> Self {
        Self::Core(msg.into())
    }

    /// 创建验证异常
    pub fn validate<S: Into<String>>(msg: S) -> Self {
        Self::Validate(msg.into())
    }

    /// 创建通用异常
    pub fn comm<S: Into<String>>(msg: S) -> Self {
        Self::Comm(msg.into())
    }

    /// 创建未授权异常
    pub fn unauthorized() -> Self {
        Self::Unauthorized(error_info::UNAUTHORIZED.to_string())
    }

    /// 创建禁止访问异常
    pub fn forbidden() -> Self {
        Self::Forbidden(error_info::FORBIDDEN.to_string())
    }

    /// 创建未找到异常
    pub fn not_found() -> Self {
        Self::NotFound(error_info::NOT_FOUND.to_string())
    }

    /// 没有实体异常
    pub fn no_entity() -> Self {
        Self::Core(error_info::NO_ENTITY.to_string())
    }

    /// 没有ID异常
    pub fn no_id() -> Self {
        Self::Validate(error_info::NO_ID.to_string())
    }

    /// 获取错误码
    pub fn code(&self) -> ResCode {
        match self {
            Self::Core(_) => ResCode::CoreError,
            Self::Validate(_) => ResCode::ValidateFail,
            Self::Comm(_) => ResCode::CommError,
            Self::Unauthorized(_) => ResCode::Unauthorized,
            Self::Forbidden(_) => ResCode::Forbidden,
            Self::NotFound(_) => ResCode::NotFound,
            Self::Database(_) => ResCode::CoreError,
            Self::Json(_) => ResCode::ValidateFail,
            Self::Redis(_) => ResCode::CoreError,
            Self::Jwt(_) => ResCode::Unauthorized,
            Self::Other(_) => ResCode::Fail,
        }
    }

    /// 获取 HTTP 状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized(_) | Self::Jwt(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Validate(_) | Self::Json(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Cool 框架结果类型
pub type CoolResult<T> = Result<T, CoolError>;

/// API 响应结构
#[derive(Debug, Clone, Serialize)]
pub struct CoolResponse<T: Serialize> {
    /// 响应码
    pub code: i32,
    /// 响应消息
    pub message: String,
    /// 响应数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> CoolResponse<T> {
    /// 成功响应
    pub fn ok(data: T) -> Self {
        Self {
            code: ResCode::Success.code(),
            message: "success".to_string(),
            data: Some(data),
        }
    }

    /// 成功响应（无数据）
    pub fn ok_empty() -> CoolResponse<()> {
        CoolResponse {
            code: ResCode::Success.code(),
            message: "success".to_string(),
            data: None,
        }
    }

    /// 失败响应
    pub fn fail<S: Into<String>>(msg: S) -> CoolResponse<()> {
        CoolResponse {
            code: ResCode::Fail.code(),
            message: msg.into(),
            data: None,
        }
    }

    /// 从错误创建响应
    pub fn from_error(err: &CoolError) -> CoolResponse<()> {
        CoolResponse {
            code: err.code().code(),
            message: err.to_string(),
            data: None,
        }
    }
}

/// 实现 Salvo 的错误处理
#[async_trait]
impl Writer for CoolError {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let status = self.status_code();
        let response = CoolResponse::<()>::from_error(&self);

        res.status_code(status);
        res.render(Json(response));
    }
}

/// 分页响应结构
#[derive(Debug, Clone, Serialize)]
pub struct PageResult<T: Serialize> {
    /// 数据列表
    pub list: Vec<T>,
    /// 分页信息
    pub pagination: Pagination,
}

/// 分页信息
#[derive(Debug, Clone, Serialize)]
pub struct Pagination {
    /// 当前页
    pub page: u64,
    /// 每页条数
    pub size: u64,
    /// 总条数
    pub total: u64,
}

impl<T: Serialize> PageResult<T> {
    /// 创建分页结果
    pub fn new(list: Vec<T>, page: u64, size: u64, total: u64) -> Self {
        Self {
            list,
            pagination: Pagination { page, size, total },
        }
    }
}

