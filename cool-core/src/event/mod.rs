//! 事件模块
//!
//! 对应 TypeScript 版本的 `event/`

use parking_lot::RwLock;
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::broadcast;

/// 事件名称常量
pub mod events {
    /// 软删除事件
    pub const SOFT_DELETE: &str = "onSoftDelete";
    /// 服务成功启动事件
    pub const SERVER_READY: &str = "onServerReady";
    /// 服务就绪事件
    pub const READY: &str = "onReady";
    /// ES 数据改变事件
    pub const ES_DATA_CHANGE: &str = "esDataChange";
}

/// 事件数据类型
pub type EventData = Arc<dyn Any + Send + Sync>;

/// 事件处理器类型
pub type EventHandler = Arc<
    dyn Fn(EventData) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
>;

/// 事件管理器
pub struct EventManager {
    /// 事件处理器映射
    handlers: RwLock<HashMap<String, Vec<EventHandler>>>,
    /// 广播发送器
    sender: broadcast::Sender<(String, EventData)>,
}

impl EventManager {
    /// 创建新的事件管理器
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self {
            handlers: RwLock::new(HashMap::new()),
            sender,
        }
    }

    /// 注册事件处理器
    pub fn on<F, Fut>(&self, event: &str, handler: F)
    where
        F: Fn(EventData) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let handler: EventHandler = Arc::new(move |data: EventData| {
            let fut = handler(data);
            Box::pin(fut) as Pin<Box<dyn Future<Output = ()> + Send>>
        });

        let mut handlers = self.handlers.write();
        handlers
            .entry(event.to_string())
            .or_insert_with(Vec::new)
            .push(handler);
    }

    /// 触发事件
    pub async fn emit<T: Any + Send + Sync>(&self, event: &str, data: T) {
        let data: EventData = Arc::new(data);

        // 调用注册的处理器：先复制一份处理器列表，避免在持有锁的情况下 `.await`
        let handlers_vec: Vec<EventHandler> = {
        let handlers = self.handlers.read();
            handlers.get(event).cloned().unwrap_or_default()
        };
        for handler in handlers_vec {
                let data_clone = Arc::clone(&data);
                handler(data_clone).await;
        }

        // 广播事件
        let _ = self.sender.send((event.to_string(), data));
    }

    /// 订阅事件（用于跨模块监听）
    pub fn subscribe(&self) -> broadcast::Receiver<(String, EventData)> {
        self.sender.subscribe()
    }

    /// 移除事件处理器
    pub fn off(&self, event: &str) {
        let mut handlers = self.handlers.write();
        handlers.remove(event);
    }

    /// 移除所有事件处理器
    pub fn clear(&self) {
        let mut handlers = self.handlers.write();
        handlers.clear();
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局事件管理器
static GLOBAL_EVENT_MANAGER: once_cell::sync::Lazy<EventManager> =
    once_cell::sync::Lazy::new(EventManager::new);

/// 获取全局事件管理器
pub fn global_event_manager() -> &'static EventManager {
    &GLOBAL_EVENT_MANAGER
}

/// 软删除事件数据
#[derive(Debug, Clone)]
pub struct SoftDeleteEvent {
    /// 实体名称
    pub entity: String,
    /// 删除的 ID 列表
    pub ids: Vec<i64>,
    /// 租户 ID
    pub tenant_id: Option<i64>,
}

/// 服务就绪事件数据
#[derive(Debug, Clone)]
pub struct ServerReadyEvent {
    /// 服务地址
    pub address: String,
    /// 端口
    pub port: u16,
}

/// ES 数据改变事件数据
#[derive(Debug, Clone)]
pub struct EsDataChangeEvent {
    /// 索引名称
    pub index: String,
    /// 操作类型: "create" | "update" | "delete"
    pub operation: String,
    /// 文档 ID
    pub doc_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_manager() {
        let manager = EventManager::new();
        let received = Arc::new(RwLock::new(false));
        let received_clone = Arc::clone(&received);

        manager.on("test", move |_data| {
            let received = Arc::clone(&received_clone);
            async move {
                *received.write() = true;
            }
        });

        manager.emit("test", "hello").await;

        assert!(*received.read());
    }
}

