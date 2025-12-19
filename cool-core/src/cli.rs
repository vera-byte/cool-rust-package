//! CLI 工具模块
//!
//! 对应 TypeScript 版本的 `bin/`
//!
//! 提供命令行工具功能，包括配置检查和实体文件生成

pub mod check;
pub mod entity;

pub use check::*;
pub use entity::*;

