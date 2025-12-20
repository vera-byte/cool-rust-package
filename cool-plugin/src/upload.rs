//! 插件上传钩子
//!
//! 对应 TypeScript 版本的 `plugin/hooks/upload/`
//!
//! 提供文件上传功能，包括：
//! - 文件上传
//! - 下载并上传
//! - 指定 key 上传

mod local;

pub use local::LocalUploadHook;

use crate::plugin::PluginInfo;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// 上传模式类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ModeType {
    /// 本地存储
    #[default]
    Local,
    /// OSS 存储
    Oss,
    /// COS 存储
    Cos,
    /// 其他
    Other,
}

/// 上传模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mode {
    /// 模式
    pub mode: ModeType,
    /// 类型
    pub r#type: String,
}

/// 上传上下文
#[derive(Debug, Clone)]
pub struct UploadContext {
    /// 文件数据
    pub file_data: Vec<u8>,
    /// 文件名
    pub filename: String,
    /// 可选的 key（路径）
    pub key: Option<String>,
    /// 其他字段
    pub fields: std::collections::HashMap<String, String>,
}

/// 上传错误
#[derive(Error, Debug)]
pub enum UploadError {
    #[error("非法的文件路径")]
    InvalidPath,
    #[error("文件路径超出允许范围")]
    PathOutOfRange,
    #[error("上传文件为空")]
    EmptyFile,
    #[error("上传失败: {0}")]
    UploadFailed(String),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP 错误: {0}")]
    Http(String),
}

pub type UploadResult<T> = Result<T, UploadError>;

/// 上传钩子 trait
///
/// 对应 TypeScript 版本的 `BaseUpload`
#[async_trait]
pub trait UploadHook: Send + Sync {
    /// 获取插件信息
    fn plugin_info(&self) -> &PluginInfo;

    /// 获得上传模式
    async fn get_mode(&self) -> UploadResult<Mode>;

    /// 获得原始操作对象（可选实现）
    async fn get_meta_file_obj(&self) -> UploadResult<Option<serde_json::Value>> {
        Ok(None)
    }

    /// 下载并上传
    ///
    /// 从 URL 下载文件并上传到存储服务
    async fn down_and_upload(&self, url: &str, file_name: Option<&str>) -> UploadResult<String>;

    /// 指定 Key(路径)上传，本地文件上传到存储服务
    ///
    /// 路径一致会覆盖源文件
    async fn upload_with_key(&self, file_path: &Path, key: &str) -> UploadResult<String>;

    /// 上传文件
    async fn upload(&self, ctx: &UploadContext) -> UploadResult<String>;
}

/// 路径安全验证工具
pub struct PathValidator;

impl PathValidator {
    /// 验证路径安全性，防止路径遍历攻击
    ///
    /// 对应 TypeScript 版本的 `sanitizePath`
    pub fn sanitize_path(user_input: &str) -> UploadResult<String> {
        if user_input.is_empty() {
            return Ok(String::new());
        }

        // 检查是否包含路径遍历字符
        if user_input.contains("..")
            || user_input.contains("./")
            || user_input.contains(".\\")
            || user_input.contains('\\')
            || user_input.contains("//")
            || user_input.contains('\0')
            || user_input.starts_with('/')
            || Self::is_windows_absolute_path(user_input)
        {
            return Err(UploadError::InvalidPath);
        }

        // 规范化路径后再次检查
        let normalized = Path::new(user_input)
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("/");

        if normalized.contains("..") || normalized.starts_with('/') {
            return Err(UploadError::InvalidPath);
        }

        Ok(normalized)
    }

    /// 检查是否是 Windows 绝对路径
    fn is_windows_absolute_path(path: &str) -> bool {
        // 检查 C:, D: 等格式
        if path.len() >= 2 {
            let first_char = path.chars().next().unwrap();
            let second_char = path.chars().nth(1).unwrap();
            if first_char.is_ascii_alphabetic() && second_char == ':' {
                return true;
            }
        }
        false
    }

    /// 验证最终路径是否在允许的目录内
    ///
    /// 对应 TypeScript 版本的 `validateTargetPath`
    pub fn validate_target_path(target_path: &Path, base_path: &Path) -> UploadResult<()> {
        let resolved_target = target_path.canonicalize().unwrap_or_else(|_| {
            // 如果路径不存在，使用父目录来解析
            target_path
                .parent()
                .unwrap_or(target_path)
                .canonicalize()
                .unwrap_or_else(|_| target_path.to_path_buf())
        });

        let resolved_base = base_path
            .canonicalize()
            .unwrap_or_else(|_| base_path.to_path_buf());

        if !resolved_target.starts_with(&resolved_base) {
            return Err(UploadError::PathOutOfRange);
        }

        Ok(())
    }
}
