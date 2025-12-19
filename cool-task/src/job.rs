//! 任务定义

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 任务错误
#[derive(Error, Debug)]
pub enum JobError {
    #[error("任务执行失败: {0}")]
    ExecutionFailed(String),
    #[error("任务超时")]
    Timeout,
    #[error("任务被取消")]
    Cancelled,
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Redis 错误: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("其他错误: {0}")]
    Other(#[from] anyhow::Error),
}

/// 任务结果
pub type JobResult<T> = Result<T, JobError>;

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// 等待中
    Waiting,
    /// 运行中
    Active,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 延迟中
    Delayed,
    /// 暂停
    Paused,
}

/// 任务选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobOptions {
    /// 延迟执行（毫秒）
    #[serde(default)]
    pub delay: Option<u64>,
    /// 最大重试次数
    #[serde(default = "default_retries")]
    pub retries: u32,
    /// 超时时间（秒）
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// 优先级（数字越大优先级越高）
    #[serde(default)]
    pub priority: i32,
    /// Cron 表达式（用于定时任务）
    #[serde(default)]
    pub cron: Option<String>,
    /// 任务 ID（不指定则自动生成）
    #[serde(default)]
    pub job_id: Option<String>,
}

fn default_retries() -> u32 {
    3
}

fn default_timeout() -> u64 {
    60
}

impl Default for JobOptions {
    fn default() -> Self {
        Self {
            delay: None,
            retries: 3,
            timeout: 60,
            priority: 0,
            cron: None,
            job_id: None,
        }
    }
}

/// 任务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// 任务 ID
    pub id: String,
    /// 队列名称
    pub queue: String,
    /// 任务名称
    pub name: String,
    /// 任务数据
    pub data: serde_json::Value,
    /// 任务状态
    pub status: JobStatus,
    /// 任务选项
    pub options: JobOptions,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 开始处理时间
    pub started_at: Option<DateTime<Utc>>,
    /// 完成时间
    pub finished_at: Option<DateTime<Utc>>,
    /// 已尝试次数
    pub attempts: u32,
    /// 错误信息
    pub error: Option<String>,
    /// 返回值
    pub result: Option<serde_json::Value>,
}

impl Job {
    /// 创建新任务
    pub fn new(queue: &str, name: &str, data: serde_json::Value, options: JobOptions) -> Self {
        Self {
            id: options
                .job_id
                .clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            queue: queue.to_string(),
            name: name.to_string(),
            data,
            status: JobStatus::Waiting,
            options,
            created_at: Utc::now(),
            started_at: None,
            finished_at: None,
            attempts: 0,
            error: None,
            result: None,
        }
    }

    /// 标记为运行中
    pub fn mark_active(&mut self) {
        self.status = JobStatus::Active;
        self.started_at = Some(Utc::now());
        self.attempts += 1;
    }

    /// 标记为完成
    pub fn mark_completed(&mut self, result: Option<serde_json::Value>) {
        self.status = JobStatus::Completed;
        self.finished_at = Some(Utc::now());
        self.result = result;
    }

    /// 标记为失败
    pub fn mark_failed(&mut self, error: &str) {
        self.status = JobStatus::Failed;
        self.finished_at = Some(Utc::now());
        self.error = Some(error.to_string());
    }

    /// 是否可以重试
    pub fn can_retry(&self) -> bool {
        self.attempts < self.options.retries
    }

    /// 获取执行耗时（毫秒）
    pub fn duration_ms(&self) -> Option<i64> {
        match (self.started_at, self.finished_at) {
            (Some(start), Some(end)) => Some((end - start).num_milliseconds()),
            _ => None,
        }
    }
}

/// 任务处理器 trait
#[async_trait]
pub trait JobHandler: Send + Sync {
    /// 处理任务
    async fn handle(&self) -> JobResult<serde_json::Value>;

    /// 获取任务名称
    fn name(&self) -> &'static str;

    /// 任务失败时的钩子
    async fn on_failed(&self, _error: &JobError) {}

    /// 任务完成时的钩子
    async fn on_completed(&self, _result: &serde_json::Value) {}
}

/// 任务处理器工厂 trait
pub trait JobHandlerFactory: Send + Sync {
    /// 创建任务处理器
    fn create(&self, data: serde_json::Value) -> Box<dyn JobHandler>;

    /// 获取任务名称
    fn name(&self) -> &'static str;
}
