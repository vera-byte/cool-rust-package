//! 定时任务调度器

use crate::job::{JobOptions, JobResult};
use crate::queue::Queue;
use cron::Schedule;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

/// 定时任务
#[derive(Clone)]
pub struct ScheduledJob {
    /// 任务名称
    pub name: String,
    /// Cron 表达式
    pub cron: String,
    /// 任务数据
    pub data: serde_json::Value,
    /// 任务选项
    pub options: JobOptions,
    /// 是否启用
    pub enabled: bool,
}

/// 定时任务调度器
pub struct Scheduler {
    /// 队列
    queue: Arc<Queue>,
    /// 定时任务映射
    jobs: Arc<RwLock<HashMap<String, ScheduledJob>>>,
    /// 是否运行中
    running: Arc<RwLock<bool>>,
}

impl Scheduler {
    /// 创建调度器
    pub fn new(queue: Queue) -> Self {
        Self {
            queue: Arc::new(queue),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// 添加定时任务
    pub fn add(&self, id: &str, job: ScheduledJob) -> JobResult<()> {
        // 验证 Cron 表达式
        Schedule::from_str(&job.cron).map_err(|e| {
            crate::job::JobError::Other(anyhow::anyhow!("无效的 Cron 表达式: {}", e))
        })?;

        let mut jobs = self.jobs.write();
        jobs.insert(id.to_string(), job);
        Ok(())
    }

    /// 移除定时任务
    pub fn remove(&self, id: &str) {
        let mut jobs = self.jobs.write();
        jobs.remove(id);
    }

    /// 启用定时任务
    pub fn enable(&self, id: &str) {
        let mut jobs = self.jobs.write();
        if let Some(job) = jobs.get_mut(id) {
            job.enabled = true;
        }
    }

    /// 禁用定时任务
    pub fn disable(&self, id: &str) {
        let mut jobs = self.jobs.write();
        if let Some(job) = jobs.get_mut(id) {
            job.enabled = false;
        }
    }

    /// 获取所有定时任务
    pub fn list(&self) -> Vec<(String, ScheduledJob)> {
        let jobs = self.jobs.read();
        jobs.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// 启动调度器
    pub async fn start(&self) {
        {
            let mut running = self.running.write();
            if *running {
                return;
            }
            *running = true;
        }

        tracing::info!("定时任务调度器已启动");

        let queue = Arc::clone(&self.queue);
        let jobs = Arc::clone(&self.jobs);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            loop {
                if !*running.read() {
                    break;
                }

                let now = chrono::Utc::now();

                // 检查所有定时任务
                let jobs_snapshot: Vec<(String, ScheduledJob)> = {
                    let jobs = jobs.read();
                    jobs.iter()
                        .filter(|(_, j)| j.enabled)
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                };

                for (id, job) in jobs_snapshot {
                    if let Ok(schedule) = Schedule::from_str(&job.cron) {
                        // 检查是否应该在当前时间执行
                        if let Some(next) = schedule.upcoming(chrono::Utc).next() {
                            let diff = (next - now).num_seconds();
                            if (0..=1).contains(&diff) {
                                tracing::info!("触发定时任务: {} ({})", job.name, id);

                                // 添加任务到队列
                                if let Err(e) = queue
                                    .add(&job.name, job.data.clone(), job.options.clone())
                                    .await
                                {
                                    tracing::error!("添加定时任务失败: {} - {}", id, e);
                                }
                            }
                        }
                    }
                }

                // 每秒检查一次
                tokio::time::sleep(Duration::from_secs(1)).await;
            }

            tracing::info!("定时任务调度器已停止");
        });
    }

    /// 停止调度器
    pub fn stop(&self) {
        let mut running = self.running.write();
        *running = false;
    }

    /// 是否运行中
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }
}
