//! 模块菜单导入（简化版）
//!
//! 对齐 TS 版 `module/menu.ts` 的核心流程：
//! - 扫描模块列表
//! - 读取 `menu.json`
//! - 采用文件锁方式避免重复导入
//! - 预留事件/持久化钩子（此处仅返回内存数据）

use crate::error::CoolError;
use crate::module::mod::ModuleManager;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 菜单导入结果：module -> menu.json 内容
pub type MenuImportResult = HashMap<String, Value>;

/// 菜单导入器
pub struct MenuImporter {
    /// 运行根路径（通常为项目根）
    pub run_path: PathBuf,
    /// 模块列表
    pub module_manager: ModuleManager,
    /// 锁目录，默认 `../lock/menu`
    pub lock_dir: PathBuf,
}

impl MenuImporter {
    pub fn new(run_path: impl Into<PathBuf>, module_manager: ModuleManager) -> Self {
        let run_path = run_path.into();
        let lock_dir = run_path.join("..").join("lock").join("menu");
        Self {
            run_path,
            module_manager,
            lock_dir,
        }
    }

    /// 导入所有模块的菜单（file 模式）
    pub fn import_all(&self) -> Result<MenuImportResult, CoolError> {
        let mut data = HashMap::new();

        // 准备锁目录
        if !self.lock_dir.exists() {
            fs::create_dir_all(&self.lock_dir)?;
        }

        // 遍历模块
        for module in self.module_manager.modules() {
            let module_path = self.run_path.join("modules").join(&module.name);
            let menu_path = module_path.join("menu.json");
            let lock_path = self.lock_dir.join(format!("{}.menu.lock", module.name));

            if !menu_path.exists() {
                continue;
            }
            // 已存在锁文件则跳过
            if lock_path.exists() {
                continue;
            }

            // 读取菜单
            let menu_str = fs::read_to_string(&menu_path)?;
            let menu_json: Value = serde_json::from_str(&menu_str)
                .map_err(|e| CoolError::comm(format!("菜单解析失败: {} -> {}", menu_path.display(), e)))?;

            data.insert(module.name.clone(), menu_json);

            // 写锁
            fs::write(lock_path, b"success")?;
        }

        Ok(data)
    }
}

