//! 插件异常处理
//!
//! 对应 TypeScript 版本的 `exception/*.ts`

use crate::constant::GlobalConfig;

/// 异常基类
///
/// 对应 TypeScript 版本的 `BaseException`
#[derive(Debug)]
pub struct BaseException {
    /// 异常名称
    pub name: String,
    /// 状态码
    pub status: u32,
    /// 错误消息
    pub message: String,
}

impl BaseException {
    /// 创建新的异常
    pub fn new(name: String, code: u32, message: String) -> Self {
        Self {
            name,
            status: code,
            message,
        }
    }
}

impl std::fmt::Display for BaseException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.message)
    }
}

/// 通用异常
///
/// 对应 TypeScript 版本的 `CoolCommException`
#[derive(Debug)]
pub struct CoolCommException {
    inner: BaseException,
}

impl CoolCommException {
    /// 创建新的通用异常
    pub fn new(message: impl Into<String>) -> Self {
        let config = GlobalConfig::get_instance();
        let message = message.into();
        let msg = if message.is_empty() {
            config.res_message().comm_fail().to_string()
        } else {
            message
        };

        Self {
            inner: BaseException::new(
                "CoolCommException".to_string(),
                config.res_code().comm_fail(),
                msg,
            ),
        }
    }

    /// 获取状态码
    pub fn status(&self) -> u32 {
        self.inner.status
    }

    /// 获取错误消息
    pub fn message(&self) -> &str {
        &self.inner.message
    }
}

impl std::fmt::Display for CoolCommException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for CoolCommException {}

/// 核心异常
///
/// 对应 TypeScript 版本的 `CoolCoreException`
#[derive(Debug)]
pub struct CoolCoreException {
    inner: BaseException,
}

impl CoolCoreException {
    /// 创建新的核心异常
    pub fn new(message: impl Into<String>) -> Self {
        let config = GlobalConfig::get_instance();
        let message = message.into();
        let msg = if message.is_empty() {
            config.res_message().core_fail().to_string()
        } else {
            message
        };

        Self {
            inner: BaseException::new(
                "CoolCoreException".to_string(),
                config.res_code().core_fail(),
                msg,
            ),
        }
    }

    /// 获取状态码
    pub fn status(&self) -> u32 {
        self.inner.status
    }

    /// 获取错误消息
    pub fn message(&self) -> &str {
        &self.inner.message
    }
}

impl std::fmt::Display for CoolCoreException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for CoolCoreException {}

/// 验证异常
///
/// 对应 TypeScript 版本的 `CoolValidateException`
#[derive(Debug)]
pub struct CoolValidateException {
    inner: BaseException,
}

impl CoolValidateException {
    /// 创建新的验证异常
    pub fn new(message: impl Into<String>) -> Self {
        let config = GlobalConfig::get_instance();
        let message = message.into();
        let msg = if message.is_empty() {
            config.res_message().validate_fail().to_string()
        } else {
            message
        };

        Self {
            inner: BaseException::new(
                "CoolValidateException".to_string(),
                config.res_code().validate_fail(),
                msg,
            ),
        }
    }

    /// 获取状态码
    pub fn status(&self) -> u32 {
        self.inner.status
    }

    /// 获取错误消息
    pub fn message(&self) -> &str {
        &self.inner.message
    }
}

impl std::fmt::Display for CoolValidateException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for CoolValidateException {}

