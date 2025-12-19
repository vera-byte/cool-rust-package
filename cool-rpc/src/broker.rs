//! RPC Broker

use crate::event::{RpcEvent, RpcEventHandler};
use crate::service::RpcServiceHandler;
use crate::RpcConfig;
use parking_lot::RwLock;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

/// RPC 错误
#[derive(Error, Debug)]
pub enum RpcError {
    #[error("服务未找到: {0}")]
    ServiceNotFound(String),
    #[error("方法未找到: {0}")]
    MethodNotFound(String),
    #[error("调用超时")]
    Timeout,
    #[error("Redis 错误: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("HTTP 错误: {0}")]
    Http(#[from] reqwest::Error),
    #[error("其他错误: {0}")]
    Other(String),
}

pub type RpcResult<T> = Result<T, RpcError>;

/// 服务节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceNode {
    /// 节点 ID
    pub id: String,
    /// 服务名称
    pub name: String,
    /// 地址
    pub address: String,
    /// 端口
    pub port: u16,
    /// 最后心跳时间
    pub last_heartbeat: i64,
    /// 服务列表
    pub services: Vec<String>,
}

/// RPC Broker
pub struct CoolRpc {
    /// 配置
    config: RpcConfig,
    /// 节点 ID
    node_id: String,
    /// Redis 连接
    conn: MultiplexedConnection,
    /// 注册的服务
    services: Arc<RwLock<HashMap<String, Box<dyn RpcServiceHandler>>>>,
    /// 事件处理器
    event_handlers: Arc<RwLock<HashMap<String, Vec<Box<dyn RpcEventHandler>>>>>,
    /// 是否运行中
    running: Arc<RwLock<bool>>,
}

impl CoolRpc {
    /// 创建 RPC Broker
    pub async fn new(config: RpcConfig) -> RpcResult<Self> {
        let client = redis::Client::open(config.redis_url.as_str())?;
        let conn = client.get_multiplexed_async_connection().await?;
        let node_id = format!("{}-{}", config.name, uuid::Uuid::new_v4());

        Ok(Self {
            config,
            node_id,
            conn,
            services: Arc::new(RwLock::new(HashMap::new())),
            event_handlers: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// 获取节点 ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// 注册服务
    pub fn register<S: RpcServiceHandler + 'static>(&self, name: &str, service: S) {
        let mut services = self.services.write();
        services.insert(name.to_string(), Box::new(service));
        tracing::info!("RPC 服务已注册: {}", name);
    }

    /// 注册事件处理器
    pub fn on<H: RpcEventHandler + 'static>(&self, event: &str, handler: H) {
        let mut handlers = self.event_handlers.write();
        handlers
            .entry(event.to_string())
            .or_default()
            .push(Box::new(handler));
    }

    /// 调用远程服务
    pub async fn call(
        &self,
        service_name: &str,
        service: &str,
        method: &str,
        params: serde_json::Value,
    ) -> RpcResult<serde_json::Value> {
        // 查找服务节点
        let node = self.discover_service(service_name).await?;

        // 构建请求
        let url = format!("http://{}:{}/rpc/call", node.address, node.port);
        let request = RpcRequest {
            service: service.to_string(),
            method: method.to_string(),
            params,
        };

        // 发送请求
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&request)
            .timeout(Duration::from_secs(self.config.call_timeout))
            .send()
            .await?;

        let result: RpcResponse = response.json().await?;

        if result.success {
            Ok(result.data.unwrap_or(serde_json::Value::Null))
        } else {
            Err(RpcError::Other(result.error.unwrap_or_default()))
        }
    }

    /// 发送事件
    pub async fn emit(&self, event: &str, data: serde_json::Value) -> RpcResult<()> {
        let mut conn = self.conn.clone();
        let channel = format!("cool:rpc:event:{}", event);
        let message = serde_json::to_string(&RpcEvent {
            name: event.to_string(),
            data,
            source: self.node_id.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        })?;

        conn.publish::<_, _, ()>(&channel, &message).await?;
        Ok(())
    }

    /// 广播事件
    pub async fn broadcast(&self, event: &str, data: serde_json::Value) -> RpcResult<()> {
        let mut conn = self.conn.clone();
        let channel = "cool:rpc:broadcast";
        let message = serde_json::to_string(&RpcEvent {
            name: event.to_string(),
            data,
            source: self.node_id.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        })?;

        conn.publish::<_, _, ()>(channel, &message).await?;
        Ok(())
    }

    /// 发现服务
    async fn discover_service(&self, name: &str) -> RpcResult<ServiceNode> {
        let mut conn = self.conn.clone();
        let key = format!("cool:rpc:services:{}", name);

        // 获取所有节点
        let nodes: Vec<String> = conn.smembers(&key).await?;

        if nodes.is_empty() {
            return Err(RpcError::ServiceNotFound(name.to_string()));
        }

        // 简单的随机负载均衡
        let idx = rand::random::<i32>().rem_euclid(nodes.len() as i32);
        let node_data = nodes
            .get(idx as usize)
            .ok_or(RpcError::ServiceNotFound(name.to_string()))?;
        let node: ServiceNode = serde_json::from_str(node_data)?;

        // 检查节点是否超时
        let now = chrono::Utc::now().timestamp();
        if now - node.last_heartbeat > self.config.service_timeout as i64 {
            return Err(RpcError::ServiceNotFound(name.to_string()));
        }

        Ok(node)
    }

    /// 启动服务
    pub async fn start(&self, address: &str, port: u16) -> RpcResult<()> {
        {
            let mut running = self.running.write();
            if *running {
                return Ok(());
            }
            *running = true;
        }

        // 注册服务节点
        self.register_node(address, port).await?;

        // 启动心跳
        self.start_heartbeat(address, port);

        // 启动事件监听
        self.start_event_listener();

        tracing::info!("RPC 服务已启动: {}:{}", address, port);
        Ok(())
    }

    /// 注册节点
    async fn register_node(&self, address: &str, port: u16) -> RpcResult<()> {
        let mut conn = self.conn.clone();
        let services: Vec<String> = {
            let services = self.services.read();
            services.keys().cloned().collect()
        };

        let node = ServiceNode {
            id: self.node_id.clone(),
            name: self.config.name.clone(),
            address: address.to_string(),
            port,
            last_heartbeat: chrono::Utc::now().timestamp(),
            services,
        };

        let node_json = serde_json::to_string(&node)?;
        let key = format!("cool:rpc:services:{}", self.config.name);
        conn.sadd::<_, _, ()>(&key, &node_json).await?;

        Ok(())
    }

    /// 启动心跳
    fn start_heartbeat(&self, address: &str, port: u16) {
        let config = self.config.clone();
        let node_id = self.node_id.clone();
        let conn = self.conn.clone();
        let services = Arc::clone(&self.services);
        let running = Arc::clone(&self.running);
        let address = address.to_string();

        tokio::spawn(async move {
            loop {
                if !*running.read() {
                    break;
                }

                let service_names: Vec<String> = {
                    let services = services.read();
                    services.keys().cloned().collect()
                };

                let node = ServiceNode {
                    id: node_id.clone(),
                    name: config.name.clone(),
                    address: address.clone(),
                    port,
                    last_heartbeat: chrono::Utc::now().timestamp(),
                    services: service_names,
                };

                let mut conn = conn.clone();
                if let Ok(node_json) = serde_json::to_string(&node) {
                    let key = format!("cool:rpc:services:{}", config.name);
                    let _ = conn.sadd::<_, _, ()>(&key, &node_json).await;
                }

                tokio::time::sleep(Duration::from_secs(config.heartbeat_interval)).await;
            }
        });
    }

    /// 启动事件监听
    fn start_event_listener(&self) {
        let _event_handlers = Arc::clone(&self.event_handlers);
        let running = Arc::clone(&self.running);
        let config = self.config.clone();

        tokio::spawn(async move {
            let client = match redis::Client::open(config.redis_url.as_str()) {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Redis 连接失败: {}", e);
                    return;
                }
            };

            let mut pubsub = match client.get_async_pubsub().await {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("Redis PubSub 连接失败: {}", e);
                    return;
                }
            };

            // 订阅广播频道
            let _ = pubsub.subscribe("cool:rpc:broadcast").await;

            loop {
                if !*running.read() {
                    break;
                }

                // 接收消息
                // 注意：这里简化处理，实际需要使用 pubsub.on_message()
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
    }

    /// 停止服务
    pub fn stop(&self) {
        let mut running = self.running.write();
        *running = false;
    }

    /// 处理 RPC 请求
    pub async fn handle_request(&self, request: RpcRequest) -> RpcResponse {
        let services = self.services.read();

        match services.get(&request.service) {
            Some(service) => match service.call(&request.method, request.params).await {
                Ok(data) => RpcResponse {
                    success: true,
                    data: Some(data),
                    error: None,
                },
                Err(e) => RpcResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                },
            },
            None => RpcResponse {
                success: false,
                data: None,
                error: Some(format!("服务未找到: {}", request.service)),
            },
        }
    }
}

/// RPC 请求
#[derive(Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    pub service: String,
    pub method: String,
    pub params: serde_json::Value,
}

/// RPC 响应
#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}
