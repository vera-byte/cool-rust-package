//! # cool-plugin
//!
//! cool-admin Rust 插件系统。
//!
//! ## 功能特性
//!
//! - 🔌 插件生命周期管理
//! - 📦 插件注册和发现
//! - 🔧 插件配置
//! - 🪝 钩子系统
//!
//! ## 快速开始
//!
//! ```rust,ignore
//! use cool_plugin::prelude::*;
//!
//! #[derive(Default)]
//! struct MyPlugin;
//!
//! #[async_trait]
//! impl Plugin for MyPlugin {
//!     fn info(&self) -> PluginInfo {
//!         PluginInfo {
//!             name: "my-plugin".to_string(),
//!             key: "my-plugin".to_string(),
//!             version: "1.0.0".to_string(),
//!             ..Default::default()
//!         }
//!     }
//!
//!     async fn ready(&mut self) -> PluginResult<()> {
//!         // 插件就绪
//!         Ok(())
//!     }
//! }
//! ```

mod plugin;
mod registry;
mod hook;
mod upload;
mod service;
mod cache;
mod exception;
mod constant;
mod installer;

pub use plugin::*;
pub use registry::*;
pub use hook::*;
pub use upload::*;
pub use service::*;
pub use cache::*;
pub use exception::*;
pub use constant::*;
pub use installer::*;

/// 预导入模块
pub mod prelude {
    pub use crate::cache::{create_cache_store, CacheError, CacheOptions, CacheResult, FsCacheStore};
    pub use crate::constant::{ErrInfo, Event, GlobalConfig, ResCode, ResMessage};
    pub use crate::exception::{
        BaseException, CoolCommException, CoolCoreException, CoolValidateException,
    };
    pub use crate::hook::{Hook, HookContext};
    pub use crate::plugin::{Plugin, PluginInfo, PluginResult, PluginStatus};
    pub use crate::registry::{global_plugin_registry, PluginRegistry};
    pub use crate::service::{global_plugin_service, PluginService};
    pub use crate::upload::{
        LocalUploadHook, Mode, ModeType, PathValidator, UploadContext, UploadError, UploadHook,
        UploadResult,
    };
    pub use crate::installer::{
        CheckResult, InstallerError, InstallerResult, PluginData, PluginInstaller,
    };
    pub use async_trait::async_trait;
}

