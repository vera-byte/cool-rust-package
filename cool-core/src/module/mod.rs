//! 模块管理（简化版）
//!
//! - 收集 `modules/` 目录下的模块名
//! - 读取模块内的 `config.{ts,js}`（本实现仅占位，不执行 JS）
//! - 提供模块列表给菜单导入使用

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 模块信息（扫描模块用，避免与运行时模块信息冲突）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyModuleInfo {
    /// 模块名
    pub name: String,
    /// 模块顺序，默认为 0，越大越靠前
    pub order: i32,
}

/// 模块配置管理
#[derive(Default)]
pub struct ModuleManager {
    modules: Vec<LegacyModuleInfo>,
}

impl ModuleManager {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    /// 从指定路径扫描模块（默认 `modules/`）
    pub fn scan_modules(&mut self, base_path: &str) -> Result<(), std::io::Error> {
        let dir = Path::new(base_path);
        if !dir.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            // 简化：顺序为 0；如果需要可读模块内某个配置文件决定顺序
            self.modules.push(LegacyModuleInfo { name, order: 0 });
        }
        // 降序排序（与 TS 保持 “order 越大越优先”）
        self.modules.sort_by(|a, b| b.order.cmp(&a.order));
        Ok(())
    }

    pub fn modules(&self) -> Vec<LegacyModuleInfo> {
        self.modules.clone()
    }
}

/// 便捷函数：使用默认路径 `modules/` 扫描
pub fn scan_default_modules(run_path: &str) -> Result<ModuleManager, std::io::Error> {
    let mut mm = ModuleManager::new();
    let base = Path::new(run_path).join("modules");
    mm.scan_modules(base.to_string_lossy().as_ref())?;
    Ok(mm)
}
/// 模块管理
///
/// 对应 TypeScript 版本的 `module/`
use crate::config::ModuleConfig;
use crate::error::CoolResult;
use parking_lot::RwLock;
use salvo::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// 模块信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    /// 模块名称
    pub name: String,
    /// 模块描述
    pub description: String,
    /// 模块路由前缀
    pub prefix: String,
    /// 模块加载顺序
    pub order: i32,
    /// 是否启用
    pub enabled: bool,
}

/// 模块 trait
pub trait Module: Send + Sync {
    /// 获取模块配置
    fn config(&self) -> ModuleConfig;

    /// 获取模块路由
    fn router(&self) -> Router;

    /// 模块初始化
    fn init(&self) -> CoolResult<()> {
        Ok(())
    }

    /// 模块销毁
    fn destroy(&self) -> CoolResult<()> {
        Ok(())
    }
}

/// 模块注册表
pub struct ModuleRegistry {
    modules: RwLock<HashMap<String, Arc<dyn Module>>>,
    order: RwLock<Vec<String>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: RwLock::new(HashMap::new()),
            order: RwLock::new(Vec::new()),
        }
    }

    /// 注册模块
    pub fn register<M: Module + 'static>(&self, module: M) -> &Self {
        let config = module.config();
        let name = config.name.clone();
        let order = config.order;

        let mut modules = self.modules.write();
        let mut order_list = self.order.write();

        modules.insert(name.clone(), Arc::new(module));

        // 按 order 排序插入
        let pos = order_list
            .iter()
            .position(|n| {
                modules
                    .get(n)
                    .map(|m| m.config().order < order)
                    .unwrap_or(true)
            })
            .unwrap_or(order_list.len());

        order_list.insert(pos, name);

        self
    }

    /// 获取所有模块（按顺序）
    pub fn get_all(&self) -> Vec<Arc<dyn Module>> {
        let modules = self.modules.read();
        let order = self.order.read();

        order
            .iter()
            .filter_map(|name| modules.get(name).cloned())
            .collect()
    }

    /// 获取指定模块
    pub fn get(&self, name: &str) -> Option<Arc<dyn Module>> {
        let modules = self.modules.read();
        modules.get(name).cloned()
    }

    /// 构建所有模块的路由
    pub fn build_router(&self, prefix: &str) -> Router {
        let mut router = Router::with_path(prefix);

        for module in self.get_all() {
            let config = module.config();
            router = router.push(module.router());
            tracing::info!(
                "模块 [{}] 已加载，路由前缀: {}",
                config.name,
                config.description
            );
        }

        router
    }

    /// 初始化所有模块
    pub fn init_all(&self) -> CoolResult<()> {
        for module in self.get_all() {
            module.init()?;
        }
        Ok(())
    }

    /// 销毁所有模块
    pub fn destroy_all(&self) -> CoolResult<()> {
        for module in self.get_all().into_iter().rev() {
            module.destroy()?;
        }
        Ok(())
    }

    /// 获取模块信息列表
    pub fn info_list(&self) -> Vec<ModuleInfo> {
        self.get_all()
            .iter()
            .map(|m| {
                let config = m.config();
                ModuleInfo {
                    name: config.name,
                    description: config.description,
                    prefix: String::new(),
                    order: config.order,
                    enabled: true,
                }
            })
            .collect()
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局模块注册表
static GLOBAL_MODULE_REGISTRY: once_cell::sync::Lazy<ModuleRegistry> =
    once_cell::sync::Lazy::new(ModuleRegistry::new);

/// 获取全局模块注册表
pub fn global_module_registry() -> &'static ModuleRegistry {
    &GLOBAL_MODULE_REGISTRY
}

/// 菜单项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    /// 菜单名称
    pub name: String,
    /// 菜单路径
    pub path: Option<String>,
    /// 菜单图标
    pub icon: Option<String>,
    /// 排序号
    pub order_num: i32,
    /// 是否显示
    pub is_show: bool,
    /// 子菜单
    #[serde(default)]
    pub children: Vec<MenuItem>,
}

/// 模块菜单配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuConfig {
    /// 菜单列表
    pub menus: Vec<MenuItem>,
}

/// 数据库初始化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    /// 初始化数据
    pub data: Vec<serde_json::Value>,
}
