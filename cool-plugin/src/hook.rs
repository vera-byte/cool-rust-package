//! 钩子系统

use async_trait::async_trait;
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// 钩子上下文
#[derive(Debug, Clone)]
pub struct HookContext {
    /// 钩子名称
    pub name: String,
    /// 钩子数据
    pub data: Value,
    /// 是否继续执行
    pub proceed: bool,
    /// 返回结果
    pub result: Option<Value>,
}

impl HookContext {
    pub fn new(name: &str, data: Value) -> Self {
        Self {
            name: name.to_string(),
            data,
            proceed: true,
            result: None,
        }
    }

    /// 设置结果
    pub fn set_result(&mut self, result: Value) {
        self.result = Some(result);
    }

    /// 停止执行后续钩子
    pub fn stop(&mut self) {
        self.proceed = false;
    }
}

/// 钩子 trait
#[async_trait]
pub trait Hook: Send + Sync {
    /// 获取钩子名称
    fn name(&self) -> &str;

    /// 获取优先级（数字越小优先级越高）
    fn priority(&self) -> i32 {
        0
    }

    /// 执行钩子
    async fn execute(&self, ctx: &mut HookContext);
}

/// 钩子管理器
pub struct HookManager {
    hooks: RwLock<HashMap<String, Vec<Arc<dyn Hook>>>>,
}

impl HookManager {
    pub fn new() -> Self {
        Self {
            hooks: RwLock::new(HashMap::new()),
        }
    }

    /// 注册钩子
    pub fn register<H: Hook + 'static>(&self, hook_name: &str, hook: H) {
        let mut hooks = self.hooks.write();
        let list = hooks.entry(hook_name.to_string()).or_insert_with(Vec::new);
        list.push(Arc::new(hook));

        // 按优先级排序
        list.sort_by_key(|h| h.priority());
    }

    /// 执行钩子
    pub async fn execute(&self, hook_name: &str, data: Value) -> HookContext {
        let mut ctx = HookContext::new(hook_name, data);

        let hooks = {
            let hooks = self.hooks.read();
            hooks.get(hook_name).cloned().unwrap_or_default()
        };

        for hook in hooks {
            if !ctx.proceed {
                break;
            }
            hook.execute(&mut ctx).await;
        }

        ctx
    }

    /// 移除钩子
    pub fn remove(&self, hook_name: &str) {
        let mut hooks = self.hooks.write();
        hooks.remove(hook_name);
    }

    /// 清空所有钩子
    pub fn clear(&self) {
        let mut hooks = self.hooks.write();
        hooks.clear();
    }

    /// 获取钩子列表
    pub fn list(&self, hook_name: &str) -> Vec<String> {
        let hooks = self.hooks.read();
        hooks
            .get(hook_name)
            .map(|list| list.iter().map(|h| h.name().to_string()).collect())
            .unwrap_or_default()
    }
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局钩子管理器
static GLOBAL_HOOK_MANAGER: once_cell::sync::Lazy<HookManager> =
    once_cell::sync::Lazy::new(HookManager::new);

/// 获取全局钩子管理器
pub fn global_hook_manager() -> &'static HookManager {
    &GLOBAL_HOOK_MANAGER
}

/// 预定义的钩子名称
pub mod hook_names {
    /// 上传前
    pub const BEFORE_UPLOAD: &str = "beforeUpload";
    /// 上传后
    pub const AFTER_UPLOAD: &str = "afterUpload";
    /// 请求前
    pub const BEFORE_REQUEST: &str = "beforeRequest";
    /// 请求后
    pub const AFTER_REQUEST: &str = "afterRequest";
    /// 数据库操作前
    pub const BEFORE_DB: &str = "beforeDb";
    /// 数据库操作后
    pub const AFTER_DB: &str = "afterDb";
}
