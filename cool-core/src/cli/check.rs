//! 检查配置文件工具
//!
//! 对应 TypeScript 版本的 `bin/check.ts`
//!
//! 检查并替换配置文件中的默认 key

use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// 检查配置
#[derive(Debug, Clone)]
pub struct CheckConfig {
    path: String,
    pattern: String,
}

/// 检查并替换单个配置文件
pub async fn check_and_replace_file(
    config: &CheckConfig,
    base_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = base_dir.join(&config.path);

    if !file_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&file_path)?;
    if content.contains(&config.pattern) {
        println!("{}，key is default, auto replace it", config.path);
        let new_content = content.replace(&config.pattern, &Uuid::new_v4().to_string());
        fs::write(&file_path, new_content)?;
    }

    Ok(())
}

/// 检查配置文件
///
/// 对应 TypeScript 版本的 `check()` 函数
pub async fn check() -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = std::env::current_dir()?;

    let configs = vec![
        CheckConfig {
            path: "src/config/config.default.ts".to_string(),
            pattern: "cool-admin-keys-xxxxxx".to_string(),
        },
        CheckConfig {
            path: "src/modules/base/config.ts".to_string(),
            pattern: "cool-admin-xxxxxx".to_string(),
        },
        CheckConfig {
            path: "src/modules/user/config.ts".to_string(),
            pattern: "cool-app-xxxxx".to_string(),
        },
    ];

    for config in &configs {
        check_and_replace_file(config, &base_dir).await?;
    }

    Ok(())
}
