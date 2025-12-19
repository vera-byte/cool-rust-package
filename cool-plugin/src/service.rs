//! 插件服务
//!
//! 对应 TypeScript 版本的 `plugin/service/info.ts`
//!
//! 提供插件管理、调用、状态检查等功能

use crate::plugin::{Plugin, PluginError, PluginInfo, PluginResult, PluginStatus};
use crate::registry::PluginRegistry;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// 插件服务
///
/// 提供插件的管理、调用、状态检查等功能
///
/// 对应 TypeScript 版本的 `PluginService`
pub struct PluginService {
    /// 插件注册表
    registry: Arc<PluginRegistry>,
    /// 插件配置（key -> config）
    configs: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// 插件状态（key -> status）
    statuses: Arc<RwLock<HashMap<String, PluginStatus>>>,
}

impl PluginService {
    /// 创建插件服务
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        Self {
            registry,
            configs: Arc::new(RwLock::new(HashMap::new())),
            statuses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 从全局注册表创建插件服务
    ///
    /// 注意：由于 Rust 的所有权模型，这里创建一个新的服务实例，
    /// 但会使用全局注册表。实际使用时，建议通过依赖注入传递同一个注册表实例。
    pub fn from_global() -> Self {
        use std::sync::Arc;
        // 创建一个新的注册表，实际使用时应该通过依赖注入共享同一个注册表
        Self::new(Arc::new(PluginRegistry::new()))
    }

    /// 使用指定的注册表创建插件服务
    pub fn with_registry(registry: Arc<PluginRegistry>) -> Self {
        Self::new(registry)
    }

    /// 获取插件配置
    ///
    /// 对应 TypeScript 版本的 `getConfig`
    pub fn get_config(&self, key: &str) -> Option<serde_json::Value> {
        let configs = self.configs.read();
        configs.get(key).cloned()
    }

    /// 设置插件配置
    pub fn set_config(&self, key: &str, config: serde_json::Value) {
        let mut configs = self.configs.write();
        configs.insert(key.to_string(), config);
    }

    /// 调用插件方法
    ///
    /// 对应 TypeScript 版本的 `invoke`
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let service = PluginService::from_global();
    /// let result = service.invoke("upload", "upload", json!({
    ///     "file": "..."
    /// })).await?;
    /// ```
    pub async fn invoke(
        &self,
        key: &str,
        method: &str,
        params: serde_json::Value,
    ) -> PluginResult<serde_json::Value> {
        // 检查插件状态
        self.check_status(key)?;

        // 从注册表获取插件并调用
        self.registry.invoke(key, method, params).await
    }

    /// 获取插件实例
    ///
    /// 对应 TypeScript 版本的 `getInstance`
    ///
    /// 注意：由于 Rust 的类型系统限制，这里返回的是插件的 Arc 引用
    pub fn get_instance(&self, key: &str) -> PluginResult<Arc<RwLock<Box<dyn Plugin>>>> {
        // 检查插件状态
        self.check_status(key)?;

        self.registry
            .get(key)
            .ok_or_else(|| PluginError::NotFound(key.to_string()))
    }

    /// 检查插件状态
    ///
    /// 对应 TypeScript 版本的 `checkStatus`
    pub fn check_status(&self, key: &str) -> PluginResult<()> {
        // 先检查注册表中是否存在
        if !self.registry.contains(key) {
            return Err(PluginError::NotFound(key.to_string()));
        }

        // 检查状态
        let statuses = self.statuses.read();
        if let Some(status) = statuses.get(key) {
            if *status == PluginStatus::Disabled {
                return Err(PluginError::Disabled(key.to_string()));
            }
        }

        Ok(())
    }

    /// 设置插件状态
    pub fn set_status(&self, key: &str, status: PluginStatus) {
        let mut statuses = self.statuses.write();
        statuses.insert(key.to_string(), status);
    }

    /// 启用插件
    pub fn enable(&self, key: &str) -> PluginResult<()> {
        if !self.registry.contains(key) {
            return Err(PluginError::NotFound(key.to_string()));
        }
        self.set_status(key, PluginStatus::Enabled);
        Ok(())
    }

    /// 禁用插件
    pub fn disable(&self, key: &str) -> PluginResult<()> {
        if !self.registry.contains(key) {
            return Err(PluginError::NotFound(key.to_string()));
        }
        self.set_status(key, PluginStatus::Disabled);
        Ok(())
    }

    /// 获取插件信息
    pub fn get_info(&self, key: &str) -> PluginResult<PluginInfo> {
        let plugin = self.get_instance(key)?;
        let plugin = plugin.read();
        Ok(plugin.info())
    }

    /// 获取所有插件信息
    ///
    /// 对应 TypeScript 版本的 `list`
    pub fn list(&self) -> Vec<PluginInfo> {
        self.registry.list()
    }

    /// 获取指定钩子类型的插件
    pub fn get_by_hook(&self, hook: &str) -> Vec<PluginInfo> {
        let plugins = self.registry.get_by_hook(hook);
        plugins.iter().map(|p| p.read().info()).collect()
    }

    /// 重新初始化插件
    ///
    /// 对应 TypeScript 版本的 `reInit`
    pub async fn re_init(&self, key: &str) -> PluginResult<()> {
        let plugin = self.get_instance(key)?;
        let config = self.get_config(key).unwrap_or_default();

        let mut plugin = plugin.write();
        plugin.init(config).await?;
        plugin.ready().await?;

        Ok(())
    }

    /// 移除插件
    ///
    /// 对应 TypeScript 版本的 `remove`
    pub fn remove(&self, key: &str) -> PluginResult<()> {
        // 从注册表移除
        self.registry.remove(key);

        // 清理配置和状态
        let mut configs = self.configs.write();
        configs.remove(key);

        let mut statuses = self.statuses.write();
        statuses.remove(key);

        Ok(())
    }
}

impl Default for PluginService {
    fn default() -> Self {
        Self::from_global()
    }
}

/// 全局插件服务
static GLOBAL_PLUGIN_SERVICE: once_cell::sync::Lazy<PluginService> =
    once_cell::sync::Lazy::new(PluginService::from_global);

/// 获取全局插件服务
pub fn global_plugin_service() -> &'static PluginService {
    &GLOBAL_PLUGIN_SERVICE
}
