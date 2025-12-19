//! 本地文件上传实现
//!
//! 对应 TypeScript 版本的 `plugin/hooks/upload/index.ts`

use crate::plugin::PluginInfo;
use crate::upload::{
    Mode, ModeType, PathValidator, UploadContext, UploadError, UploadHook, UploadResult,
};
use async_trait::async_trait;
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// 本地文件上传钩子
///
/// 对应 TypeScript 版本的 `CoolPlugin`
pub struct LocalUploadHook {
    /// 插件信息
    plugin_info: PluginInfo,
    /// 上传基础路径
    base_path: PathBuf,
    /// 域名前缀
    domain: String,
}

impl LocalUploadHook {
    /// 创建本地上传钩子
    pub fn new(plugin_info: PluginInfo, base_path: impl Into<PathBuf>, domain: String) -> Self {
        Self {
            plugin_info,
            base_path: base_path.into(),
            domain,
        }
    }

    /// 获取日期目录（格式：YYYYMMDD）
    fn get_date_dir(&self) -> String {
        Local::now().format("%Y%m%d").to_string()
    }

    /// 确保目录存在
    fn ensure_dir(&self, dir_path: &Path) -> UploadResult<()> {
        if !dir_path.exists() {
            fs::create_dir_all(dir_path)?;
        }
        Ok(())
    }
}

#[async_trait]
impl UploadHook for LocalUploadHook {
    fn plugin_info(&self) -> &PluginInfo {
        &self.plugin_info
    }

    async fn get_mode(&self) -> UploadResult<Mode> {
        Ok(Mode {
            mode: ModeType::Local,
            r#type: "local".to_string(),
        })
    }

    async fn down_and_upload(&self, url: &str, file_name: Option<&str>) -> UploadResult<String> {
        // 下载文件
        let file_data = if url.starts_with("http://") || url.starts_with("https://") {
            // HTTP/HTTPS URL，需要下载
            // 注意：这里需要 reqwest，但为了避免依赖，暂时返回错误
            // 实际使用时可以通过依赖注入获取 HTTP 客户端
            return Err(UploadError::Http(
                "HTTP 下载功能需要 reqwest 依赖".to_string(),
            ));
        } else {
            // 本地文件路径
            fs::read(url)?
        };

        let date_dir = self.get_date_dir();
        let base_path = &self.base_path;

        // 从 URL 或文件名获取扩展名
        let extension = if let Some(name) = file_name {
            Path::new(name)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
        } else {
            Path::new(url)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
        };

        // 验证文件名安全性
        let safe_file_name = if let Some(name) = file_name {
            let sanitized = PathValidator::sanitize_path(name)?;
            // 只取文件名部分，去除可能的子目录
            Path::new(&sanitized)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&sanitized)
                .to_string()
        } else {
            format!("{}.{}", Uuid::new_v4(), extension)
        };

        // 创建目录
        let dir_path = base_path.join(&date_dir);
        self.ensure_dir(&dir_path)?;

        // 写入文件
        let target_path = dir_path.join(&safe_file_name);
        PathValidator::validate_target_path(&target_path, base_path)?;
        fs::write(&target_path, file_data)?;

        Ok(format!(
            "{}/upload/{}/{}",
            self.domain, date_dir, safe_file_name
        ))
    }

    async fn upload_with_key(&self, file_path: &Path, key: &str) -> UploadResult<String> {
        let date_dir = self.get_date_dir();
        let base_path = &self.base_path;

        // 验证 key 安全性
        let safe_key = PathValidator::sanitize_path(key)?;

        // 读取源文件
        let data = fs::read(file_path)?;

        // 构建目标路径
        let target_path = base_path.join(&date_dir).join(&safe_key);
        let dir_path = target_path.parent().unwrap_or(base_path);

        // 验证最终路径
        PathValidator::validate_target_path(&target_path, base_path)?;

        // 确保目录存在
        self.ensure_dir(dir_path)?;

        // 写入文件
        fs::write(&target_path, data)?;

        Ok(format!("{}/upload/{}/{}", self.domain, date_dir, safe_key))
    }

    async fn upload(&self, ctx: &UploadContext) -> UploadResult<String> {
        if ctx.file_data.is_empty() {
            return Err(UploadError::EmptyFile);
        }

        let date_dir = self.get_date_dir();
        let base_path = &self.base_path;

        // 验证 key 安全性（如果提供）
        let safe_key = if let Some(key) = &ctx.key {
            Some(PathValidator::sanitize_path(key)?)
        } else {
            None
        };

        // 获取文件扩展名
        let extension = Path::new(&ctx.filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        // 生成最终文件名
        let final_name = if let Some(key) = safe_key {
            key
        } else {
            format!("{}.{}", Uuid::new_v4(), extension)
        };

        let name = format!("{}/{}", date_dir, final_name);
        let target = base_path.join(&name);

        // 验证最终路径
        PathValidator::validate_target_path(&target, base_path)?;

        // 创建目录
        let dir_path = base_path.join(&date_dir);
        self.ensure_dir(&dir_path)?;

        // 写入文件
        fs::write(&target, &ctx.file_data)?;

        Ok(format!("{}/upload/{}", self.domain, name))
    }
}
