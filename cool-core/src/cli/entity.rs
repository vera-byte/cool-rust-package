//! 实体文件生成工具
//!
//! 对应 TypeScript 版本的 `bin/entity.ts`
//!
//! 生成或清空 entities.ts 文件

use std::fs;
use std::path::{Path, PathBuf};
use glob::Pattern;

const MODULES_PATH: &str = "src/modules";
const OUTPUT_FILE: &str = "src/entities.ts";

/// 生成 entities.ts 文件
///
/// 对应 TypeScript 版本的 `generateEntitiesFile()` 函数
pub fn generate_entities_file() -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = std::env::current_dir()?;
    let modules_path = base_dir.join(MODULES_PATH);
    let output_file = base_dir.join(OUTPUT_FILE);

    if !modules_path.exists() {
        return Err("src/modules directory not found".into());
    }

    // 扫描所有的 rs 文件（Rust 版本）
    let entity_files = find_entity_files(&modules_path)?;

    if entity_files.is_empty() {
        println!("No entity files found");
        return Ok(());
    }

    // 生成导入语句和导出数组
    let mut imports = Vec::new();
    let mut exports = Vec::new();

    for (index, file) in entity_files.iter().enumerate() {
        let relative_path = file
            .strip_prefix(&base_dir)
            .unwrap_or(file)
            .to_string_lossy()
            .replace('\\', "/");

        // Rust 版本：使用 mod 声明而不是 import
        let module_name = format!("entity_{}", index);
        imports.push(format!("mod {};", module_name));
        
        // 注意：Rust 的实体导出方式与 TypeScript 不同
        // 这里生成一个占位符，实际使用时需要手动调整
        exports.push(format!("    // {}", relative_path));
    }

    // 生成最终的文件内容
    let file_content = format!(
        "// 自动生成的文件，请勿手动修改\n\
         // 注意：Rust 版本的实体导出方式与 TypeScript 不同\n\
         // 请根据实际情况调整导出语句\n\
         {}\n\
         \n\
         // 导出所有实体\n\
         // pub use entity_0::*;\n\
         // pub use entity_1::*;\n\
         // ...\n",
        imports.join("\n")
    );

    // 确保输出目录存在
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)?;
    }

    // 写入文件
    fs::write(&output_file, file_content)?;
    println!("Entities file generated successfully: {}", output_file.display());

    Ok(())
}

/// 清空 entities.ts 文件
///
/// 对应 TypeScript 版本的 `clearEntitiesFile()` 函数
pub fn clear_entities_file() -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = std::env::current_dir()?;
    let output_file = base_dir.join(OUTPUT_FILE);

    let empty_content = "// 自动生成的文件，请勿手动修改\n\
                         // Rust 版本的实体导出方式与 TypeScript 不同\n\
                         // 请根据实际情况调整导出语句\n";

    // 确保输出目录存在
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_file, empty_content)?;
    println!("Entities file cleared successfully: {}", output_file.display());

    Ok(())
}

/// 查找所有实体文件
fn find_entity_files(base_dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut entity_files = Vec::new();
    let pattern = Pattern::new("**/entity/**/*.rs")?;

    if base_dir.is_dir() {
        find_entity_files_recursive(base_dir, base_dir, &pattern, &mut entity_files)?;
    }

    Ok(entity_files)
}

/// 递归查找实体文件
fn find_entity_files_recursive(
    base: &Path,
    dir: &Path,
    pattern: &Pattern,
    files: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(base).unwrap_or(&path);

            if path.is_dir() {
                find_entity_files_recursive(base, &path, pattern, files)?;
            } else if path.is_file() {
                let path_str = relative_path.to_string_lossy().replace('\\', "/");
                if pattern.matches(&path_str) {
                    files.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(())
}

