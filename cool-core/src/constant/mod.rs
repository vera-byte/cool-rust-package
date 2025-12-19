//! 全局常量定义
//!
//! 对应 TypeScript 版本的 `constant/global.ts`

use std::str::FromStr;

use crate::error::CoolError;

/// 错误信息常量
pub mod error_info {
    /// 没有实体
    pub const NO_ENTITY: &str = "该服务没有设置实体";
    /// 没有ID
    pub const NO_ID: &str = "缺少ID参数";
    /// 没有找到数据
    pub const NOT_FOUND: &str = "数据不存在";
    /// 参数验证失败
    pub const VALIDATE_FAIL: &str = "参数验证失败";
    /// 未授权
    pub const UNAUTHORIZED: &str = "未授权访问";
    /// 禁止访问
    pub const FORBIDDEN: &str = "禁止访问";
    /// 服务器错误
    pub const SERVER_ERROR: &str = "服务器内部错误";
}

/// 响应码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ResCode {
    /// 成功
    Success = 1000,
    /// 失败
    Fail = 1001,
    /// 验证失败
    ValidateFail = 1002,
    /// 核心异常
    CoreError = 1003,
    /// 通用异常
    CommError = 1004,
    /// 未授权
    Unauthorized = 1005,
    /// 禁止访问
    Forbidden = 1006,
    /// 未找到
    NotFound = 1007,
}

impl ResCode {
    /// 转换为 i32
    pub fn code(&self) -> i32 {
        *self as i32
    }
}

/// CRUD 操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrudType {
    /// 新增
    Add,
    /// 删除
    Delete,
    /// 修改
    Update,
    /// 分页查询
    Page,
    /// 详情查询
    Info,
    /// 列表查询
    List,
}

/// 实现 FromStr trait
impl FromStr for CrudType {
    type Err = CoolError;

    /// 从字符串解析为 CrudType
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "add" => Ok(Self::Add),
            "delete" => Ok(Self::Delete),
            "update" => Ok(Self::Update),
            "page" => Ok(Self::Page),
            "info" => Ok(Self::Info),
            "list" => Ok(Self::List),
            _ => Err(CoolError::validate(format!("无效的 CrudType: {}", s))),
        }
    }
}

// 独立 impl 块定义其他方法
impl CrudType {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Delete => "delete",
            Self::Update => "update",
            Self::Page => "page",
            Self::Info => "info",
            Self::List => "list",
        }
    }

    /// 获取描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Add => "新增",
            Self::Delete => "删除",
            Self::Update => "修改",
            Self::Page => "分页查询",
            Self::Info => "详情查询",
            Self::List => "列表查询",
        }
    }

    /// 获取请求方法
    pub fn method(&self) -> &'static str {
        match self {
            Self::Info => "GET",
            _ => "POST",
        }
    }
}

/// 文件上传模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileMode {
    /// 本地存储
    Local,
    /// 云存储
    Cloud,
    /// 其他
    Other,
}

/// 云存储类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudType {
    /// 阿里云 OSS
    Oss,
    /// 腾讯云 COS
    Cos,
    /// 七牛云
    Qiniu,
    /// AWS S3
    Aws,
}

/// 数据库类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbType {
    /// MySQL
    Mysql,
    /// PostgreSQL
    Postgres,
    /// SQLite
    Sqlite,
}

impl DbType {
    /// 从 SeaORM DatabaseBackend 转换
    pub fn from_backend(backend: sea_orm::DatabaseBackend) -> Self {
        match backend {
            sea_orm::DatabaseBackend::MySql => Self::Mysql,
            sea_orm::DatabaseBackend::Postgres => Self::Postgres,
            sea_orm::DatabaseBackend::Sqlite => Self::Sqlite,
        }
    }
}
