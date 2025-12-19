//! 工具模块
//!
//! 对应 TypeScript 版本的 `util/`

use md5::{Digest, Md5};
use uuid::Uuid;

/// 生成 UUID v4
pub fn uuid() -> String {
    Uuid::new_v4().to_string()
}

/// 生成不带横杠的 UUID
pub fn uuid_simple() -> String {
    Uuid::new_v4().simple().to_string()
}

/// MD5 加密
pub fn md5<S: AsRef<[u8]>>(data: S) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// 生成随机字符串
pub fn random_string(len: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..len)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// 生成随机数字字符串
pub fn random_number_string(len: usize) -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..len).map(|_| rng.random_range(0..10).to_string()).collect()
}

/// 驼峰转下划线
pub fn camel_to_snake(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

/// 下划线转驼峰
pub fn snake_to_camel(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// 首字母大写
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// 首字母小写
pub fn uncapitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().chain(chars).collect(),
    }
}

/// 获取当前时间戳（秒）
pub fn timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

/// 获取当前时间戳（毫秒）
pub fn timestamp_millis() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// 格式化日期时间
pub fn format_datetime(dt: chrono::DateTime<chrono::Utc>, format: &str) -> String {
    dt.format(format).to_string()
}

/// 解析日期时间
pub fn parse_datetime(s: &str, format: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::NaiveDateTime::parse_from_str(s, format)
        .ok()
        .and_then(|dt| dt.and_local_timezone(chrono::Utc).single())
}

/// IP 地址工具
pub mod ip {
    /// 检查是否是内网 IP
    pub fn is_private(ip: &str) -> bool {
        if let Ok(addr) = ip.parse::<std::net::IpAddr>() {
            match addr {
                std::net::IpAddr::V4(v4) => {
                    v4.is_private() || v4.is_loopback() || v4.is_link_local()
                }
                std::net::IpAddr::V6(v6) => v6.is_loopback(),
            }
        } else {
            false
        }
    }

    /// 获取本机 IP
    pub fn local_ip() -> Option<String> {
        local_ip_address::local_ip().ok().map(|ip| ip.to_string())
    }
}

/// 字符串工具
pub mod string {
    /// 是否为空或空白
    pub fn is_blank(s: &str) -> bool {
        s.trim().is_empty()
    }

    /// 是否不为空且不为空白
    pub fn is_not_blank(s: &str) -> bool {
        !is_blank(s)
    }

    /// 截取字符串
    pub fn truncate(s: &str, max_len: usize) -> &str {
        if s.len() <= max_len {
            s
        } else {
            &s[..max_len]
        }
    }

    /// 移除 HTML 标签
    pub fn strip_html(s: &str) -> String {
        let re = regex::Regex::new(r"<[^>]*>").unwrap();
        re.replace_all(s, "").to_string()
    }

    /// 替换所有匹配的字符串
    ///
    /// 对应 TypeScript 版本的 `String.prototype.replaceAll`
    pub fn replace_all(s: &str, from: &str, to: &str) -> String {
        s.replace(from, to)
    }

    /// 首字母大写
    pub fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().chain(chars).collect(),
        }
    }

    /// 首字母小写
    pub fn uncapitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_lowercase().chain(chars).collect(),
        }
    }
}

/// 位置工具
///
/// 对应 TypeScript 版本的 `util/location.ts`
pub mod location {
    use std::env;
    use std::path::{Path, PathBuf};

    /// 获取当前可执行文件的目录
    pub fn get_executable_dir() -> Option<PathBuf> {
        env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
    }

    /// 获取当前工作目录
    pub fn get_current_dir() -> Option<PathBuf> {
        env::current_dir().ok()
    }

    /// 获取项目根目录（查找 Cargo.toml 或 package.json）
    pub fn get_project_root() -> Option<PathBuf> {
        let mut current = env::current_dir().ok()?;

        loop {
            // 检查是否存在 Cargo.toml（Rust 项目）
            if current.join("Cargo.toml").exists() {
                return Some(current);
            }

            // 检查是否存在 package.json（Node.js 项目）
            if current.join("package.json").exists() {
                return Some(current);
            }

            // 检查是否存在 dist 目录（编译后的目录）
            if current.join("dist").exists() {
                return Some(current);
            }

            // 向上查找
            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => break,
            }
        }

        None
    }

    /// 获取编译后的文件路径（dist 目录）
    ///
    /// 对应 TypeScript 版本的 `getDistPath()`
    pub fn get_dist_path() -> Option<PathBuf> {
        get_project_root().map(|root| root.join("dist"))
    }

    /// 规范化路径
    pub fn normalize_path(path: &Path) -> PathBuf {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    }

    /// 连接路径
    pub fn join_path<P: AsRef<Path>>(base: &Path, parts: &[P]) -> PathBuf {
        let mut result = base.to_path_buf();
        for part in parts {
            result = result.join(part);
        }
        result
    }

    /// 获取相对路径
    ///
    /// 注意：这是一个简化实现，如果需要完整功能，需要添加 pathdiff 依赖
    pub fn relative_path(_from: &Path, _to: &Path) -> Option<PathBuf> {
        // 简化实现：如果需要完整功能，可以添加 pathdiff 依赖
        // pathdiff::diff_paths(to, from)
        None
    }

    /// 检查路径是否存在
    pub fn path_exists(path: &Path) -> bool {
        path.exists()
    }

    /// 检查是否是文件
    pub fn is_file(path: &Path) -> bool {
        path.is_file()
    }

    /// 检查是否是目录
    pub fn is_dir(path: &Path) -> bool {
        path.is_dir()
    }

    /// 获取文件扩展名
    pub fn get_extension(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string())
    }

    /// 获取文件名（不含扩展名）
    pub fn get_stem(path: &Path) -> Option<String> {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .map(|s| s.to_string())
    }

    /// 获取文件名（含扩展名）
    pub fn get_filename(path: &Path) -> Option<String> {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_to_snake() {
        assert_eq!(camel_to_snake("userName"), "user_name");
        assert_eq!(camel_to_snake("UserName"), "user_name");
        assert_eq!(camel_to_snake("userId"), "user_id");
    }

    #[test]
    fn test_snake_to_camel() {
        assert_eq!(snake_to_camel("user_name"), "userName");
        assert_eq!(snake_to_camel("user_id"), "userId");
    }

    #[test]
    fn test_md5() {
        assert_eq!(md5("hello"), "5d41402abc4b2a76b9719d911017c592");
    }

    #[test]
    fn test_uuid() {
        let id = uuid();
        assert_eq!(id.len(), 36);
        assert!(id.contains('-'));
    }
}
