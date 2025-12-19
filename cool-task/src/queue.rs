//! 队列实现

use crate::job::{Job, JobOptions, JobResult, JobStatus};
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;

/// 队列
#[derive(Clone)]
pub struct Queue {
    /// 队列名称
    name: String,
    /// 前缀
    prefix: String,
    /// Redis 连接
    conn: MultiplexedConnection,
}

impl Queue {
    /// 创建队列
    pub async fn new(name: &str, prefix: &str, redis_url: &str) -> JobResult<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(Self {
            name: name.to_string(),
            prefix: prefix.to_string(),
            conn,
        })
    }

    /// 获取 Redis key
    fn key(&self, suffix: &str) -> String {
        format!("{}:{}:{}", self.prefix, self.name, suffix)
    }

    /// 添加任务
    pub async fn add(
        &self,
        name: &str,
        data: serde_json::Value,
        options: JobOptions,
    ) -> JobResult<Job> {
        let job = Job::new(&self.name, name, data, options);
        let job_json = serde_json::to_string(&job)?;

        let mut conn = self.conn.clone();

        // 存储任务数据
        let job_key = self.key(&format!("job:{}", job.id));
        conn.set::<_, _, ()>(&job_key, &job_json).await?;

        // 根据是否延迟决定放入哪个队列
        if let Some(delay) = job.options.delay {
            let score = chrono::Utc::now().timestamp_millis() + delay as i64;
            conn.zadd::<_, _, _, ()>(self.key("delayed"), &job.id, score)
                .await?;
        } else {
            // 按优先级放入等待队列
            let score = -job.options.priority as f64;
            conn.zadd::<_, _, _, ()>(self.key("waiting"), &job.id, score)
                .await?;
        }

        Ok(job)
    }

    /// 批量添加任务
    pub async fn add_bulk(
        &self,
        jobs: Vec<(String, serde_json::Value, JobOptions)>,
    ) -> JobResult<Vec<Job>> {
        let mut results = Vec::new();
        for (name, data, options) in jobs {
            let job = self.add(&name, data, options).await?;
            results.push(job);
        }
        Ok(results)
    }

    /// 获取任务
    pub async fn get_job(&self, job_id: &str) -> JobResult<Option<Job>> {
        let mut conn = self.conn.clone();
        let job_key = self.key(&format!("job:{}", job_id));
        let job_json: Option<String> = conn.get(&job_key).await?;

        match job_json {
            Some(json) => {
                let job: Job = serde_json::from_str(&json)?;
                Ok(Some(job))
            }
            None => Ok(None),
        }
    }

    /// 更新任务
    pub async fn update_job(&self, job: &Job) -> JobResult<()> {
        let mut conn = self.conn.clone();
        let job_key = self.key(&format!("job:{}", job.id));
        let job_json = serde_json::to_string(job)?;
        conn.set::<_, _, ()>(&job_key, &job_json).await?;
        Ok(())
    }

    /// 获取下一个待处理任务
    pub async fn get_next_job(&self) -> JobResult<Option<Job>> {
        let mut conn = self.conn.clone();

        // 先检查延迟队列
        let now = chrono::Utc::now().timestamp_millis();
        let delayed_key = self.key("delayed");
        let waiting_key = self.key("waiting");

        // 移动到期的延迟任务到等待队列
        let delayed_jobs: Vec<String> = conn.zrangebyscore(&delayed_key, 0, now).await?;

        for job_id in delayed_jobs {
            conn.zrem::<_, _, ()>(&delayed_key, &job_id).await?;
            conn.zadd::<_, _, _, ()>(&waiting_key, &job_id, 0).await?;
        }

        // 从等待队列获取任务
        let job_ids: Vec<String> = conn.zrange(&waiting_key, 0, 0).await?;

        if let Some(job_id) = job_ids.first() {
            // 移除并移到活跃队列
            conn.zrem::<_, _, ()>(&waiting_key, job_id).await?;
            conn.sadd::<_, _, ()>(self.key("active"), job_id).await?;

            // 获取任务详情
            if let Some(mut job) = self.get_job(job_id).await? {
                job.mark_active();
                self.update_job(&job).await?;
                return Ok(Some(job));
            }
        }

        Ok(None)
    }

    /// 完成任务
    pub async fn complete_job(
        &self,
        job: &mut Job,
        result: Option<serde_json::Value>,
    ) -> JobResult<()> {
        let mut conn = self.conn.clone();

        job.mark_completed(result);
        self.update_job(job).await?;

        // 从活跃队列移除
        conn.srem::<_, _, ()>(self.key("active"), &job.id).await?;
        // 添加到完成队列
        conn.zadd::<_, _, _, ()>(
            self.key("completed"),
            &job.id,
            chrono::Utc::now().timestamp_millis(),
        )
        .await?;

        Ok(())
    }

    /// 失败任务
    pub async fn fail_job(&self, job: &mut Job, error: &str) -> JobResult<()> {
        let mut conn = self.conn.clone();

        job.mark_failed(error);

        if job.can_retry() {
            // 放回等待队列重试
            job.status = JobStatus::Waiting;
            self.update_job(job).await?;
            conn.srem::<_, _, ()>(self.key("active"), &job.id).await?;
            conn.zadd::<_, _, _, ()>(self.key("waiting"), &job.id, 0)
                .await?;
        } else {
            // 移到失败队列
            self.update_job(job).await?;
            conn.srem::<_, _, ()>(self.key("active"), &job.id).await?;
            conn.zadd::<_, _, _, ()>(
                self.key("failed"),
                &job.id,
                chrono::Utc::now().timestamp_millis(),
            )
            .await?;
        }

        Ok(())
    }

    /// 暂停队列
    pub async fn pause(&self) -> JobResult<()> {
        let mut conn = self.conn.clone();
        conn.set::<_, _, ()>(self.key("paused"), "1").await?;
        Ok(())
    }

    /// 恢复队列
    pub async fn resume(&self) -> JobResult<()> {
        let mut conn = self.conn.clone();
        conn.del::<_, ()>(self.key("paused")).await?;
        Ok(())
    }

    /// 检查队列是否暂停
    pub async fn is_paused(&self) -> JobResult<bool> {
        let mut conn = self.conn.clone();
        let paused: Option<String> = conn.get(self.key("paused")).await?;
        Ok(paused.is_some())
    }

    /// 获取队列长度
    pub async fn count(&self, status: JobStatus) -> JobResult<u64> {
        let mut conn = self.conn.clone();
        let key = match status {
            JobStatus::Waiting => self.key("waiting"),
            JobStatus::Active => self.key("active"),
            JobStatus::Completed => self.key("completed"),
            JobStatus::Failed => self.key("failed"),
            JobStatus::Delayed => self.key("delayed"),
            _ => return Ok(0),
        };

        let count: u64 = if status == JobStatus::Active {
            conn.scard(&key).await?
        } else {
            conn.zcard(&key).await?
        };

        Ok(count)
    }

    /// 清空队列
    pub async fn obliterate(&self) -> JobResult<()> {
        let mut conn = self.conn.clone();
        let pattern = format!("{}:{}:*", self.prefix, self.name);
        let keys: Vec<String> = conn.keys(&pattern).await?;
        if !keys.is_empty() {
            conn.del::<_, ()>(keys).await?;
        }
        Ok(())
    }
}
