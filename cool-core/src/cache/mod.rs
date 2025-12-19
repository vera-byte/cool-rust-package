//! 缓存模块
//!
//! 对应 TypeScript 版本的 `cache/`

use crate::error::CoolResult;
use async_trait::async_trait;
use parking_lot::RwLock;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 缓存 trait
#[async_trait]
pub trait CacheStore: Send + Sync {
    /// 获取缓存
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> CoolResult<Option<T>>;

    /// 设置缓存
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CoolResult<()>;

    /// 删除缓存
    async fn del(&self, key: &str) -> CoolResult<()>;

    /// 检查缓存是否存在
    async fn exists(&self, key: &str) -> CoolResult<bool>;

    /// 清空所有缓存
    async fn clear(&self) -> CoolResult<()>;

    /// 获取所有 key
    async fn keys(&self, pattern: &str) -> CoolResult<Vec<String>>;
}

/// 内存缓存项
struct CacheItem {
    value: String,
    expire_at: Option<Instant>,
}

impl CacheItem {
    fn is_expired(&self) -> bool {
        self.expire_at
            .map(|exp| Instant::now() > exp)
            .unwrap_or(false)
    }
}

/// 内存缓存
pub struct MemoryCache {
    store: Arc<RwLock<HashMap<String, CacheItem>>>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 清理过期的缓存
    pub fn cleanup(&self) {
        let mut store = self.store.write();
        store.retain(|_, item| !item.is_expired());
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CacheStore for MemoryCache {
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> CoolResult<Option<T>> {
        let store = self.store.read();
        match store.get(key) {
            Some(item) if !item.is_expired() => {
                let value: T = serde_json::from_str(&item.value)?;
                Ok(Some(value))
            }
            _ => Ok(None),
        }
    }

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CoolResult<()> {
        let mut store = self.store.write();
        let value_str = serde_json::to_string(value)?;
        let item = CacheItem {
            value: value_str,
            expire_at: ttl.map(|d| Instant::now() + d),
        };
        store.insert(key.to_string(), item);
        Ok(())
    }

    async fn del(&self, key: &str) -> CoolResult<()> {
        let mut store = self.store.write();
        store.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> CoolResult<bool> {
        let store = self.store.read();
        match store.get(key) {
            Some(item) => Ok(!item.is_expired()),
            None => Ok(false),
        }
    }

    async fn clear(&self) -> CoolResult<()> {
        let mut store = self.store.write();
        store.clear();
        Ok(())
    }

    async fn keys(&self, pattern: &str) -> CoolResult<Vec<String>> {
        let store = self.store.read();
        let pattern = pattern.replace('*', "");
        let keys: Vec<String> = store
            .keys()
            .filter(|k| {
                if pattern.is_empty() {
                    true
                } else {
                    k.contains(&pattern)
                }
            })
            .cloned()
            .collect();
        Ok(keys)
    }
}

/// Redis 缓存
pub struct RedisCache {
    conn: MultiplexedConnection,
    prefix: String,
}

impl RedisCache {
    pub async fn new(url: &str, prefix: impl Into<String>) -> CoolResult<Self> {
        let client = redis::Client::open(url)?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(Self {
            conn,
            prefix: prefix.into(),
        })
    }

    fn full_key(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}:{}", self.prefix, key)
        }
    }
}

#[async_trait]
impl CacheStore for RedisCache {
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> CoolResult<Option<T>> {
        let mut conn = self.conn.clone();
        let full_key = self.full_key(key);
        let value: Option<String> = conn.get(&full_key).await?;
        match value {
            Some(v) => {
                let result: T = serde_json::from_str(&v)?;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CoolResult<()> {
        let mut conn = self.conn.clone();
        let full_key = self.full_key(key);
        let value_str = serde_json::to_string(value)?;

        match ttl {
            Some(duration) => {
                conn.set_ex::<_, _, ()>(&full_key, &value_str, duration.as_secs())
                    .await?;
            }
            None => {
                conn.set::<_, _, ()>(&full_key, &value_str).await?;
            }
        }
        Ok(())
    }

    async fn del(&self, key: &str) -> CoolResult<()> {
        let mut conn = self.conn.clone();
        let full_key = self.full_key(key);
        conn.del::<_, ()>(&full_key).await?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> CoolResult<bool> {
        let mut conn = self.conn.clone();
        let full_key = self.full_key(key);
        let exists: bool = conn.exists(&full_key).await?;
        Ok(exists)
    }

    async fn clear(&self) -> CoolResult<()> {
        let mut conn = self.conn.clone();
        let pattern = self.full_key("*");
        let keys: Vec<String> = conn.keys(&pattern).await?;
        if !keys.is_empty() {
            conn.del::<_, ()>(keys).await?;
        }
        Ok(())
    }

    async fn keys(&self, pattern: &str) -> CoolResult<Vec<String>> {
        let mut conn = self.conn.clone();
        let full_pattern = self.full_key(pattern);
        let keys: Vec<String> = conn.keys(&full_pattern).await?;
        // 移除前缀
        let prefix_len = if self.prefix.is_empty() {
            0
        } else {
            self.prefix.len() + 1
        };
        let keys: Vec<String> = keys
            .into_iter()
            .map(|k| k[prefix_len..].to_string())
            .collect();
        Ok(keys)
    }
}

/// 缓存工厂
pub struct CacheFactory;

impl CacheFactory {
    /// 创建内存缓存
    pub fn memory() -> Arc<MemoryCache> {
        Arc::new(MemoryCache::new())
    }

    /// 创建 Redis 缓存
    pub async fn redis(url: &str, prefix: impl Into<String>) -> CoolResult<Arc<RedisCache>> {
        let cache = RedisCache::new(url, prefix).await?;
        Ok(Arc::new(cache))
    }
}
