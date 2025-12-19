//! 队列基类
//!
//! 对齐 TS 版本的 `BaseCoolQueue`，提供一组易用的高阶 API：
//! - `add` / `add_bulk`
//! - `pause` / `resume` / `is_paused`
//! - `obliterate`
//!
//! 使用方式示例：
//!
//! ```rust,ignore
//! use cool_task::prelude::*;
//!
//! pub struct EmailQueue {
//!     inner: BaseQueue,
//! }
//!
//! impl EmailQueue {
//!     pub async fn new(config: TaskConfig) -> JobResult<Self> {
//!         let base = BaseQueue::new("email", config).await?;
//!         Ok(Self { inner: base })
//!     }
//!
//!     pub async fn send(&self, to: &str, subject: &str, body: &str) -> JobResult<Job> {
//!         let data = serde_json::json!({ "to": to, "subject": subject, "body": body });
//!         self.inner.add("send", data, JobOptions::default()).await
//!     }
//! }
//! ```

use crate::job::{Job, JobOptions, JobResult, JobStatus};
use crate::queue::Queue;
use crate::worker::{Worker, WorkerConfig};
use crate::TaskConfig;
use std::sync::Arc;

/// 队列基类
///
/// 封装 `Queue` 和 `Worker`，提供更贴近 TS `BaseCoolQueue` 的使用体验。
pub struct BaseQueue {
    /// 队列名称
    pub name: String,
    /// 队列前缀
    pub prefix: String,
    /// 任务配置
    pub task_config: TaskConfig,
    /// 底层队列
    pub queue: Arc<Queue>,
    /// 可选 Worker
    pub worker: Option<Worker>,
}

impl BaseQueue {
    /// 创建新的队列基类实例，并根据配置自动创建 Queue/Worker
    pub async fn new(name: &str, config: TaskConfig) -> JobResult<Self> {
        let queue = Queue::new(name, &config.prefix, &config.redis_url).await?;
        let worker_cfg = WorkerConfig {
            concurrency: config.concurrency,
            ..Default::default()
        };
        let worker = Worker::new(queue.clone(), worker_cfg);

        Ok(Self {
            name: name.to_string(),
            prefix: config.prefix.clone(),
            task_config: config,
            queue: Arc::new(queue),
            worker: Some(worker),
        })
    }

    /// 只创建生产者（无 Worker）
    pub async fn producer_only(name: &str, config: TaskConfig) -> JobResult<Self> {
        let queue = Queue::new(name, &config.prefix, &config.redis_url).await?;
        Ok(Self {
            name: name.to_string(),
            prefix: config.prefix.clone(),
            task_config: config,
            queue: Arc::new(queue),
            worker: None,
        })
    }

    /// 启动 Worker（如果存在）
    pub async fn start_worker(&self) {
        if let Some(worker) = &self.worker {
            worker.start().await;
        }
    }

    /// 停止 Worker（如果存在）
    pub fn stop_worker(&self) {
        if let Some(worker) = &self.worker {
            worker.stop();
        }
    }

    /// 发送单个任务
    pub async fn add(
        &self,
        name: &str,
        data: serde_json::Value,
        options: JobOptions,
    ) -> JobResult<Job> {
        self.queue.add(name, data, options).await
    }

    /// 批量发送任务
    pub async fn add_bulk(
        &self,
        jobs: Vec<(String, serde_json::Value, JobOptions)>,
    ) -> JobResult<Vec<Job>> {
        self.queue.add_bulk(jobs).await
    }

    /// 暂停队列
    pub async fn pause(&self) -> JobResult<()> {
        self.queue.pause().await
    }

    /// 恢复队列
    pub async fn resume(&self) -> JobResult<()> {
        self.queue.resume().await
    }

    /// 队列是否暂停
    pub async fn is_paused(&self) -> JobResult<bool> {
        self.queue.is_paused().await
    }

    /// 获取各状态任务数量
    pub async fn count(&self, status: JobStatus) -> JobResult<u64> {
        self.queue.count(status).await
    }

    /// 清空队列（包括任务数据）
    pub async fn obliterate(&self) -> JobResult<()> {
        self.queue.obliterate().await
    }
}


