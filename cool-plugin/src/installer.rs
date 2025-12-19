//! 插件安装器
//!
//! 对应 TypeScript 版本的 `service/info.ts` 中的 `install`, `data`, `check` 等方法
//!
//! 提供插件 ZIP 包解析、安装、卸载等功能

use crate::exception::CoolCommException;
use crate::plugin::PluginInfo;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// 插件安装错误
#[derive(Error, Debug)]
pub enum InstallerError {
    #[error("插件信息不完整: {0}")]
    IncompleteInfo(String),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP 错误: {0}")]
    Zip(String),
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("通用错误: {0}")]
    Common(#[from] CoolCommException),
}

pub type InstallerResult<T> = Result<T, InstallerError>;

/// 插件数据
///
/// 对应 TypeScript 版本的 `data()` 方法返回的数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginData {
    /// 插件 JSON 信息
    pub plugin_json: PluginInfo,
    /// README 内容
    pub readme: String,
    /// Logo（Base64 编码）
    pub logo: String,
    /// 插件内容（JavaScript）
    pub content: String,
    /// TypeScript 内容
    pub ts_content: String,
}

/// 插件检查结果
///
/// 对应 TypeScript 版本的 `check()` 方法返回的结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckResult {
    /// 类型 0：插件信息不完整
    Incomplete = 0,
    /// 类型 1：插件已存在，继续安装将覆盖
    Exists = 1,
    /// 类型 2：已存在同名 Hook 插件，多个相同的 Hook 插件只能同时开启一个
    HookExists = 2,
    /// 类型 3：检查通过
    Ok = 3,
}

impl CheckResult {
    /// 获取检查结果消息
    pub fn message(&self) -> &'static str {
        match self {
            CheckResult::Incomplete => "插件信息不完整",
            CheckResult::Exists => "插件已存在，继续安装将覆盖",
            CheckResult::HookExists => "已存在同名Hook插件，你可以继续安装，但是多个相同的Hook插件只能同时开启一个",
            CheckResult::Ok => "检查通过",
        }
    }
}

/// 插件安装器
///
/// 提供插件安装、卸载、检查等功能
pub struct PluginInstaller {
    /// 插件存储目录
    plugin_dir: PathBuf,
}

impl PluginInstaller {
    /// 创建新的插件安装器
    ///
    /// # 参数
    ///
    /// * `plugin_dir` - 插件存储目录路径
    pub fn new(plugin_dir: impl AsRef<Path>) -> Self {
        Self {
            plugin_dir: plugin_dir.as_ref().to_path_buf(),
        }
    }

    /// 从 ZIP 文件提取插件数据
    ///
    /// 对应 TypeScript 版本的 `data()` 方法
    ///
    /// # 参数
    ///
    /// * `zip_path` - ZIP 文件路径
    ///
    /// # 返回
    ///
    /// 返回插件数据，包括 plugin.json、readme、logo、content、tsContent
    ///
    /// # 注意
    ///
    /// 此方法需要 `zip` feature 启用，否则会返回错误
    #[cfg(feature = "zip")]
    pub fn extract_plugin_data(&self, zip_path: impl AsRef<Path>) -> InstallerResult<PluginData> {
        use zip::ZipArchive;
        use std::fs::File;
        use std::io::Read;

        let file = File::open(zip_path.as_ref())?;
        let mut archive = ZipArchive::new(file)
            .map_err(|e| InstallerError::Zip(format!("无法打开 ZIP 文件: {}", e)))?;

        // 辅助函数：从 ZIP 中读取文件内容
        let mut get_file_content = |entry_name: &str, encoding: &str| -> InstallerResult<String> {
            let mut file = archive
                .by_name(entry_name)
                .map_err(|_| {
                    InstallerError::IncompleteInfo(format!("文件 {} 不存在", entry_name))
                })?;

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|e| InstallerError::Io(e))?;

            match encoding {
                "base64" => {
                    use base64::Engine;
                    Ok(base64::engine::general_purpose::STANDARD.encode(&buffer))
                }
                "utf-8" | _ => String::from_utf8(buffer)
                    .map_err(|e| InstallerError::IncompleteInfo(format!("UTF-8 解码失败: {}", e))),
            }
        };

        // 读取 plugin.json
        let plugin_json_str = get_file_content("plugin.json", "utf-8")?;
        let plugin_json: PluginInfo = serde_json::from_str(&plugin_json_str)
            .map_err(|e| InstallerError::IncompleteInfo(format!("plugin.json 解析失败: {}", e)))?;

        // 读取 readme
        let readme = get_file_content(&plugin_json.readme, "utf-8")
            .map_err(|_| InstallerError::IncompleteInfo("readme 文件不存在".to_string()))?;

        // 读取 logo（Base64 编码）
        let logo = get_file_content(&plugin_json.logo, "base64")
            .map_err(|_| InstallerError::IncompleteInfo("logo 文件不存在".to_string()))?;

        // 读取 content（JavaScript）
        let content = get_file_content("src/index.js", "utf-8")
            .map_err(|_| InstallerError::IncompleteInfo("src/index.js 文件不存在".to_string()))?;

        // 读取 tsContent（TypeScript）
        let ts_content = get_file_content("source/index.ts", "utf-8")
            .map_err(|_| InstallerError::IncompleteInfo("source/index.ts 文件不存在".to_string()))?;

        Ok(PluginData {
            plugin_json,
            readme,
            logo,
            content,
            ts_content,
        })
    }

    /// 从 ZIP 文件提取插件数据（无 zip feature 版本）
    ///
    /// 当未启用 `zip` feature 时，此方法会返回错误
    #[cfg(not(feature = "zip"))]
    pub fn extract_plugin_data(&self, _zip_path: impl AsRef<Path>) -> InstallerResult<PluginData> {
        Err(InstallerError::Zip(
            "ZIP 功能未启用，请在 Cargo.toml 中添加 zip 依赖并启用 zip feature".to_string(),
        ))
    }

    /// 检查插件
    ///
    /// 对应 TypeScript 版本的 `check()` 方法
    ///
    /// # 参数
    ///
    /// * `zip_path` - ZIP 文件路径
    /// * `existing_plugins` - 已存在的插件列表（key -> (is_hook, is_enabled)）
    ///
    /// # 返回
    ///
    /// 返回检查结果
    pub fn check_plugin(
        &self,
        zip_path: impl AsRef<Path>,
        existing_plugins: &std::collections::HashMap<String, (bool, bool)>,
    ) -> InstallerResult<(CheckResult, Option<String>)> {
        // 尝试提取插件数据
        let data = match self.extract_plugin_data(&zip_path) {
            Ok(d) => d,
            Err(e) => {
                return Ok((
                    CheckResult::Incomplete,
                    Some(format!("插件信息不完整: {}", e)),
                ));
            }
        };

        let key = &data.plugin_json.key;

        // 检查插件 key 不能为 "plugin"
        if key == "plugin" {
            return Err(InstallerError::Common(CoolCommException::new(
                "插件key不能为plugin，请更换其他key",
            )));
        }

        // 检查是否已存在
        if let Some((is_hook, is_enabled)) = existing_plugins.get(key) {
            if !is_hook {
                // 非 Hook 插件已存在
                return Ok((CheckResult::Exists, None));
            } else if *is_enabled {
                // Hook 插件已存在且已启用
                return Ok((CheckResult::HookExists, None));
            }
        }

        Ok((CheckResult::Ok, None))
    }

    /// 保存插件数据到文件
    ///
    /// 对应 TypeScript 版本的 `saveData()` 方法
    ///
    /// # 参数
    ///
    /// * `key_name` - 插件 key
    /// * `data` - 插件数据
    pub fn save_plugin_data(
        &self,
        key_name: &str,
        data: &PluginData,
    ) -> InstallerResult<()> {
        let file_path = self.get_plugin_path(key_name);

        // 确保目录存在
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 构建保存的数据结构
        let save_data = serde_json::json!({
            "content": {
                "type": "comm",
                "data": data.content
            },
            "tsContent": {
                "type": "ts",
                "data": data.ts_content
            }
        });

        // 写入文件
        fs::write(&file_path, serde_json::to_string_pretty(&save_data)?)?;

        Ok(())
    }

    /// 获取插件数据
    ///
    /// 对应 TypeScript 版本的 `getData()` 方法
    ///
    /// # 参数
    ///
    /// * `key_name` - 插件 key
    ///
    /// # 返回
    ///
    /// 返回插件数据（content 和 tsContent）
    pub fn get_plugin_data(
        &self,
        key_name: &str,
    ) -> InstallerResult<Option<serde_json::Value>> {
        let file_path = self.get_plugin_path(key_name);

        if !file_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&file_path)?;
        let data: serde_json::Value = serde_json::from_str(&content)?;

        Ok(Some(data))
    }

    /// 删除插件数据
    ///
    /// 对应 TypeScript 版本的 `deleteData()` 方法
    ///
    /// * `key_name` - 插件 key
    pub fn delete_plugin_data(&self, key_name: &str) -> InstallerResult<()> {
        let file_path = self.get_plugin_path(key_name);

        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }

        Ok(())
    }

    /// 获取插件文件路径
    ///
    /// 对应 TypeScript 版本的 `pluginPath()` 方法
    fn get_plugin_path(&self, key_name: &str) -> PathBuf {
        self.plugin_dir.join(key_name)
    }
}

impl Default for PluginInstaller {
    fn default() -> Self {
        Self::new("plugins")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result() {
        assert_eq!(CheckResult::Ok.message(), "检查通过");
        assert_eq!(CheckResult::Exists.message(), "插件已存在，继续安装将覆盖");
    }
}

