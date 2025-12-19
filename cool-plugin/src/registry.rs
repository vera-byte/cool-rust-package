//! 插件注册表

use crate::plugin::{Plugin, PluginError, PluginInfo, PluginResult, PluginStatus};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// 插件注册表
pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, Arc<RwLock<Box<dyn Plugin>>>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }

    /// 注册插件
    pub fn register<P: Plugin + 'static>(&self, plugin: P) {
        let info = plugin.info();
        let key = info.key.clone();

        let mut plugins = self.plugins.write();
        plugins.insert(key.clone(), Arc::new(RwLock::new(Box::new(plugin))));

        tracing::info!("插件已注册: {} ({})", info.name, key);
    }

    /// 获取插件
    pub fn get(&self, key: &str) -> Option<Arc<RwLock<Box<dyn Plugin>>>> {
        let plugins = self.plugins.read();
        plugins.get(key).cloned()
    }

    /// 移除插件
    pub fn remove(&self, key: &str) -> Option<Arc<RwLock<Box<dyn Plugin>>>> {
        let mut plugins = self.plugins.write();
        plugins.remove(key)
    }

    /// 检查插件是否存在
    pub fn contains(&self, key: &str) -> bool {
        let plugins = self.plugins.read();
        plugins.contains_key(key)
    }

    /// 获取所有插件信息
    pub fn list(&self) -> Vec<PluginInfo> {
        let plugins = self.plugins.read();
        plugins.values().map(|p| p.read().info()).collect()
    }

    /// 获取指定类型的插件
    pub fn get_by_hook(&self, hook: &str) -> Vec<Arc<RwLock<Box<dyn Plugin>>>> {
        let plugins = self.plugins.read();
        plugins
            .values()
            .filter(|p| p.read().info().hook == hook)
            .cloned()
            .collect()
    }

    /// 初始化所有插件
    pub async fn init_all(&self, configs: HashMap<String, serde_json::Value>) -> PluginResult<()> {
        let plugins: Vec<_> = {
            let plugins = self.plugins.read();
            plugins.values().cloned().collect()
        };

        for plugin in plugins {
            let key = plugin.read().info().key.clone();
            let config = configs.get(&key).cloned().unwrap_or_default();

            let mut plugin = plugin.write();
            plugin.init(config).await?;
        }

        Ok(())
    }

    /// 启动所有插件
    pub async fn ready_all(&self) -> PluginResult<()> {
        let plugins: Vec<_> = {
            let plugins = self.plugins.read();
            plugins.values().cloned().collect()
        };

        for plugin in plugins {
            let mut plugin = plugin.write();
            plugin.ready().await?;
        }

        Ok(())
    }

    /// 销毁所有插件
    pub async fn destroy_all(&self) -> PluginResult<()> {
        let plugins: Vec<_> = {
            let plugins = self.plugins.read();
            plugins.values().cloned().collect()
        };

        for plugin in plugins.into_iter().rev() {
            let mut plugin = plugin.write();
            plugin.destroy().await?;
        }

        Ok(())
    }

    /// 调用插件方法
    pub async fn invoke(
        &self,
        key: &str,
        method: &str,
        params: serde_json::Value,
    ) -> PluginResult<serde_json::Value> {
        let plugin = self
            .get(key)
            .ok_or_else(|| PluginError::NotFound(key.to_string()))?;

        let plugin = plugin.read();
        if plugin.status() == PluginStatus::Disabled {
            return Err(PluginError::Disabled(key.to_string()));
        }

        plugin.invoke(method, params).await
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局插件注册表
static GLOBAL_PLUGIN_REGISTRY: once_cell::sync::Lazy<PluginRegistry> =
    once_cell::sync::Lazy::new(PluginRegistry::new);

/// 获取全局插件注册表
pub fn global_plugin_registry() -> &'static PluginRegistry {
    &GLOBAL_PLUGIN_REGISTRY
}
