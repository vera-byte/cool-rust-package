//! Worker 实现

use crate::job::JobHandlerFactory;
use crate::queue::Queue;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

/// Worker 配置
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// 并发数
    pub concurrency: usize,
    /// 轮询间隔（毫秒）
    pub poll_interval: u64,
    /// 是否自动启动
    pub auto_start: bool,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            concurrency: 4,
            poll_interval: 1000,
            auto_start: true,
        }
    }
}

/// Worker
pub struct Worker {
    /// 队列
    queue: Arc<Queue>,
    /// 配置
    config: WorkerConfig,
    /// 任务处理器工厂映射
    handlers: Arc<RwLock<HashMap<String, Box<dyn JobHandlerFactory>>>>,
    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
    /// 是否运行中
    running: Arc<RwLock<bool>>,
}

impl Worker {
    /// 创建 Worker
    pub fn new(queue: Queue, config: WorkerConfig) -> Self {
        let concurrency = config.concurrency;
        Self {
            queue: Arc::new(queue),
            config,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(concurrency)),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// 注册任务处理器
    pub fn register<F: JobHandlerFactory + 'static>(&self, factory: F) {
        let mut handlers = self.handlers.write();
        handlers.insert(factory.name().to_string(), Box::new(factory));
    }

    /// 启动 Worker
    pub async fn start(&self) {
        {
            let mut running = self.running.write();
            if *running {
                return;
            }
            *running = true;
        }

        tracing::info!("Worker 已启动，并发数: {}", self.config.concurrency);

        let queue = Arc::clone(&self.queue);
        let handlers = Arc::clone(&self.handlers);
        let semaphore = Arc::clone(&self.semaphore);
        let running = Arc::clone(&self.running);
        let poll_interval = self.config.poll_interval;

        tokio::spawn(async move {
            loop {
                // 检查是否停止
                if !*running.read() {
                    break;
                }

                // 检查队列是否暂停
                if let Ok(paused) = queue.is_paused().await {
                    if paused {
                        tokio::time::sleep(Duration::from_millis(poll_interval)).await;
                        continue;
                    }
                }

                // 获取信号量
                let permit = match semaphore.clone().try_acquire_owned() {
                    Ok(permit) => permit,
                    Err(_) => {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        continue;
                    }
                };

                // 获取下一个任务
                let job = match queue.get_next_job().await {
                    Ok(Some(job)) => job,
                    Ok(None) => {
                        drop(permit);
                        tokio::time::sleep(Duration::from_millis(poll_interval)).await;
                        continue;
                    }
                    Err(e) => {
                        tracing::error!("获取任务失败: {}", e);
                        drop(permit);
                        tokio::time::sleep(Duration::from_millis(poll_interval)).await;
                        continue;
                    }
                };

                // 查找处理器
                let handler = {
                    let handlers = handlers.read();
                    handlers.get(&job.name).map(|f| f.create(job.data.clone()))
                };

                let queue_clone = Arc::clone(&queue);
                let mut job_clone = job.clone();

                // 异步处理任务
                tokio::spawn(async move {
                    let _permit = permit; // 保持 permit 直到任务完成

                    match handler {
                        Some(handler) => {
                            tracing::info!("开始处理任务: {} ({})", job_clone.name, job_clone.id);

                            // 设置超时
                            let timeout_duration = Duration::from_secs(job_clone.options.timeout);
                            let result =
                                tokio::time::timeout(timeout_duration, handler.handle()).await;

                            match result {
                                Ok(Ok(value)) => {
                                    tracing::info!(
                                        "任务完成: {} ({})",
                                        job_clone.name,
                                        job_clone.id
                                    );
                                    handler.on_completed(&value).await;
                                    let _ =
                                        queue_clone.complete_job(&mut job_clone, Some(value)).await;
                                }
                                Ok(Err(e)) => {
                                    tracing::error!(
                                        "任务失败: {} ({}) - {}",
                                        job_clone.name,
                                        job_clone.id,
                                        e
                                    );
                                    handler.on_failed(&e).await;
                                    let _ =
                                        queue_clone.fail_job(&mut job_clone, &e.to_string()).await;
                                }
                                Err(_) => {
                                    tracing::error!(
                                        "任务超时: {} ({})",
                                        job_clone.name,
                                        job_clone.id
                                    );
                                    let _ =
                                        queue_clone.fail_job(&mut job_clone, "任务执行超时").await;
                                }
                            }
                        }
                        None => {
                            tracing::error!("未找到任务处理器: {}", job_clone.name);
                            let _ = queue_clone
                                .fail_job(&mut job_clone, "未找到任务处理器")
                                .await;
                        }
                    }
                });
            }

            tracing::info!("Worker 已停止");
        });
    }

    /// 停止 Worker
    pub fn stop(&self) {
        let mut running = self.running.write();
        *running = false;
    }

    /// 是否运行中
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }
}
