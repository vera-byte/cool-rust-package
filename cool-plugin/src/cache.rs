//! 插件缓存存储
//!
//! 对应 TypeScript 版本的 `cache/store.ts`
//!
//! 提供基于文件系统的缓存存储功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// 缓存错误
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("缓存操作失败: {0}")]
    OperationFailed(String),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type CacheResult<T> = Result<T, CacheError>;

/// 缓存项
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheItem<T> {
    value: T,
    expires_at: Option<u64>, // Unix 时间戳（秒）
}

/// 文件系统缓存存储
///
/// 对应 TypeScript 版本的 `FsCacheStore`
pub struct FsCacheStore {
    /// 缓存目录路径
    cache_dir: PathBuf,
    /// 默认 TTL（秒），-1 表示永不过期
    default_ttl: i64,
    /// 内存缓存（用于快速访问）
    memory_cache: parking_lot::RwLock<HashMap<String, (serde_json::Value, Option<u64>)>>,
}

impl FsCacheStore {
    /// 创建新的缓存存储
    ///
    /// # 参数
    ///
    /// * `options` - 配置选项
    ///   - `path`: 缓存目录路径，默认为 "cache"
    ///   - `ttl`: 默认 TTL（秒），-1 表示永不过期，默认为 -1
    pub fn new(options: CacheOptions) -> CacheResult<Self> {
        let cache_dir = PathBuf::from(options.path.unwrap_or_else(|| "cache".to_string()));
        let default_ttl = options.ttl.unwrap_or(-1);

        // 确保缓存目录存在
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        Ok(Self {
            cache_dir,
            default_ttl,
            memory_cache: parking_lot::RwLock::new(HashMap::new()),
        })
    }

    /// 获取缓存值
    ///
    /// 对应 TypeScript 版本的 `get`
    pub fn get<T>(&self, key: &str) -> CacheResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        // 先检查内存缓存
        {
            let memory = self.memory_cache.read();
            if let Some((value, expires_at)) = memory.get(key) {
                // 检查是否过期
                if let Some(expires) = *expires_at {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    if now >= expires {
                        // 已过期，从内存缓存中移除
                        drop(memory);
                        let mut memory = self.memory_cache.write();
                        memory.remove(key);
                    } else {
                        // 未过期，返回缓存值
                        let value: T = serde_json::from_value(value.clone())?;
                        return Ok(Some(value));
                    }
                } else {
                    // 永不过期
                    let value: T = serde_json::from_value(value.clone())?;
                    return Ok(Some(value));
                }
            }
        }

        // 从文件系统读取
        let file_path = self.get_file_path(key);
        if !file_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&file_path)?;
        let item: CacheItem<T> = serde_json::from_str(&content)?;

        // 检查是否过期
        if let Some(expires_at) = item.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if now >= expires_at {
                // 已过期，删除文件
                let _ = fs::remove_file(&file_path);
                return Ok(None);
            }
        }

        // 更新内存缓存
        // 注意：由于 T 可能没有实现 Serialize，我们无法将值存储在内存缓存中
        // 为了性能，这里暂时不更新内存缓存，下次访问时从文件系统读取
        // 如果需要内存缓存，需要确保 T 同时实现 Serialize 和 Deserialize

        Ok(Some(item.value))
    }

    /// 设置缓存值
    ///
    /// 对应 TypeScript 版本的 `set`
    pub fn set<T>(&self, key: &str, value: T, ttl: Option<i64>) -> CacheResult<()>
    where
        T: Serialize,
    {
        let ttl_seconds = ttl.unwrap_or(self.default_ttl);
        let expires_at = if ttl_seconds > 0 {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Some(now + ttl_seconds as u64)
        } else {
            None // 永不过期
        };

        let item = CacheItem { value, expires_at };

        // 保存到文件系统
        let file_path = self.get_file_path(key);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string(&item)?;
        fs::write(&file_path, content)?;

        // 更新内存缓存
        // 注意：由于 T 可能没有实现 Serialize，我们无法将值存储在内存缓存中
        // 为了性能，这里暂时不更新内存缓存，下次访问时从文件系统读取
        // 如果需要内存缓存，需要确保 T 同时实现 Serialize 和 Deserialize

        Ok(())
    }

    /// 删除缓存
    ///
    /// 对应 TypeScript 版本的 `del`
    pub fn del(&self, key: &str) -> CacheResult<()> {
        // 从内存缓存删除
        {
            let mut memory = self.memory_cache.write();
            memory.remove(key);
        }

        // 从文件系统删除
        let file_path = self.get_file_path(key);
        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }

        Ok(())
    }

    /// 重置所有缓存
    ///
    /// 对应 TypeScript 版本的 `reset`
    pub fn reset(&self) -> CacheResult<()> {
        // 清空内存缓存
        {
            let mut memory = self.memory_cache.write();
            memory.clear();
        }

        // 删除缓存目录下的所有文件
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(path)?;
                }
            }
        }

        Ok(())
    }

    /// 获取缓存文件路径
    fn get_file_path(&self, key: &str) -> PathBuf {
        // 使用 key 的哈希值作为文件名，避免特殊字符问题
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        let filename = format!("{:x}.json", hash);

        self.cache_dir.join(filename)
    }
}

/// 缓存配置选项
#[derive(Debug, Clone, Default)]
pub struct CacheOptions {
    /// 缓存目录路径
    pub path: Option<String>,
    /// 默认 TTL（秒），-1 表示永不过期
    pub ttl: Option<i64>,
}

/// 创建缓存存储的便捷函数
///
/// 对应 TypeScript 版本的 `CoolCacheStore`
pub fn create_cache_store(options: CacheOptions) -> CacheResult<FsCacheStore> {
    FsCacheStore::new(options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_store() {
        let temp_dir = std::env::temp_dir().join("cool_plugin_cache_test");
        let _ = fs::remove_dir_all(&temp_dir);

        let options = CacheOptions {
            path: Some(temp_dir.to_string_lossy().to_string()),
            ttl: Some(60),
        };

        let store = FsCacheStore::new(options).unwrap();

        // 测试设置和获取
        store.set("test_key", "test_value", None).unwrap();
        let value: Option<String> = store.get("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // 测试删除
        store.del("test_key").unwrap();
        let value: Option<String> = store.get("test_key").unwrap();
        assert_eq!(value, None);

        // 清理
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
