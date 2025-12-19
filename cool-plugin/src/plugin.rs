//! 插件定义

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use thiserror::Error;

/// 插件错误
#[derive(Error, Debug)]
pub enum PluginError {
    #[error("插件初始化失败: {0}")]
    InitFailed(String),
    #[error("插件未找到: {0}")]
    NotFound(String),
    #[error("插件已禁用: {0}")]
    Disabled(String),
    #[error("方法未找到: {0}")]
    MethodNotFound(String),
    #[error("参数错误: {0}")]
    InvalidParams(String),
    #[error("执行错误: {0}")]
    ExecutionError(String),
    #[error("其他错误: {0}")]
    Other(#[from] anyhow::Error),
}

pub type PluginResult<T> = Result<T, PluginError>;

/// 插件信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginInfo {
    /// 插件名称
    pub name: String,
    /// 插件唯一标识
    pub key: String,
    /// 钩子类型
    pub hook: String,
    /// 版本
    pub version: String,
    /// 描述
    pub description: String,
    /// 作者
    pub author: String,
    /// Logo
    pub logo: String,
    /// README
    pub readme: String,
    /// 配置
    pub config: serde_json::Value,
}

/// 插件状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    /// 已安装
    Installed,
    /// 已启用
    Enabled,
    /// 已禁用
    Disabled,
    /// 错误
    Error,
}

/// 插件 trait
#[async_trait]
pub trait Plugin: Send + Sync {
    /// 获取插件信息
    fn info(&self) -> PluginInfo;

    /// 获取插件状态
    fn status(&self) -> PluginStatus {
        PluginStatus::Enabled
    }

    /// 插件初始化
    async fn init(&mut self, _config: serde_json::Value) -> PluginResult<()> {
        Ok(())
    }

    /// 插件就绪
    async fn ready(&mut self) -> PluginResult<()> {
        Ok(())
    }

    /// 插件销毁
    async fn destroy(&mut self) -> PluginResult<()> {
        Ok(())
    }

    /// 调用插件方法
    async fn invoke(&self, method: &str, _params: serde_json::Value) -> PluginResult<serde_json::Value> {
        Err(PluginError::MethodNotFound(method.to_string()))
    }

    /// 转换为 Any（用于类型转换）
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

