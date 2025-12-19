//! # cool-task
//!
//! cool-admin Rust 任务队列库，基于 Redis 实现分布式任务队列。
//!
//! ## 功能特性
//!
//! - 🚀 基于 Redis 的分布式任务队列
//! - ⏰ 支持定时任务（Cron 表达式）
//! - 🔄 支持延迟任务
//! - 📊 任务状态管理
//! - 🔁 失败重试机制
//!
//! ## 快速开始
//!
//! ```rust,ignore
//! use cool_task::prelude::*;
//!
//! #[derive(Serialize, Deserialize)]
//! struct EmailJob {
//!     to: String,
//!     subject: String,
//!     body: String,
//! }
//!
//! #[async_trait]
//! impl JobHandler for EmailJob {
//!     async fn handle(&self) -> JobResult<()> {
//!         // 发送邮件逻辑
//!         Ok(())
//!     }
//! }
//! ```

mod base;
mod job;
mod queue;
mod scheduler;
mod worker;

pub use base::*;
pub use job::*;
pub use queue::*;
pub use scheduler::*;
pub use worker::*;

/// 队列类型，对齐 TS 版本的 `type?: 'comm' | 'getter' | 'noworker' | 'single'`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueType {
    /// 普通队列，带 Worker
    Comm,
    /// 仅获取队列信息（对应 TS 的 getter）
    Getter,
    /// 不自动创建 Worker（仅生产，不消费）
    NoWorker,
    /// 单例队列（通常用于全局唯一任务）
    Single,
}

impl Default for QueueType {
    fn default() -> Self {
        QueueType::Comm
    }
}

/// 队列配置，对齐 TS 版本 `CoolQueue` 装饰器的配置结构
///
/// 在 Rust 中通过 `#[cool_queue(...)]` 宏自动生成。
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// 队列类型
    pub queue_type: QueueType,
    /// 默认任务配置
    pub default_job: job::JobOptions,
    /// Worker 配置
    pub worker: worker::WorkerConfig,
}

/// 预导入模块
pub mod prelude {
    pub use crate::base::BaseQueue;
    pub use crate::job::{Job, JobHandler, JobOptions, JobResult, JobStatus};
    pub use crate::queue::Queue;
    pub use crate::scheduler::Scheduler;
    pub use crate::worker::Worker;
    pub use crate::{QueueConfig, QueueType};
    pub use async_trait::async_trait;
    pub use serde::{Deserialize, Serialize};
}

/// 任务配置
#[derive(Debug, Clone)]
pub struct TaskConfig {
    /// Redis URL
    pub redis_url: String,
    /// 队列名称前缀
    pub prefix: String,
    /// 默认重试次数
    pub default_retries: u32,
    /// 默认超时时间（秒）
    pub default_timeout: u64,
    /// Worker 并发数
    pub concurrency: usize,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            prefix: "cool:task".to_string(),
            default_retries: 3,
            default_timeout: 60,
            concurrency: 4,
        }
    }
}
