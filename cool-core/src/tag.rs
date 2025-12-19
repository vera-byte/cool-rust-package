//! URL 标签模块
//!
//! 对应 TypeScript 版本的 `tag/data.ts` 与 `decorator/tag.ts` 中的运行时数据部分。
//!
//! 在 Rust 版本中我们不直接依赖装饰器，而是提供一个全局的标签存储，
//! 由上层在注册路由或构建模块时主动写入。

use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 标签类型
///
/// 与 Node 版本中的 `TagTypes` 对齐。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TagType {
    /// 忽略 Token 校验
    #[serde(rename = "ignoreToken")]
    IgnoreToken,
    /// 忽略签名校验
    #[serde(rename = "ignoreSign")]
    IgnoreSign,
    /// 自定义标签
    Custom(String),
}

impl TagType {
    /// 获取作为 key 使用的字符串
    pub fn as_key(&self) -> String {
        match self {
            TagType::IgnoreToken => "ignoreToken".to_string(),
            TagType::IgnoreSign => "ignoreSign".to_string(),
            TagType::Custom(s) => s.clone(),
        }
    }
}

/// URL 标签存储
///
/// 结构与 Node 版本的 `CoolUrlTagData` 类似，负责管理各种标签下的 URL 列表。
#[derive(Default)]
pub struct UrlTagStore {
    data: RwLock<HashMap<String, Vec<String>>>,
}

impl UrlTagStore {
    /// 创建新的标签存储
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }

    /// 为一组 URL 添加类级标签
    ///
    /// 等价于 Node 版本中 `CoolUrlTag` 在类上打标后由 `CoolUrlTagData.classTag` 汇总的行为。
    pub fn add_class_tag<K, S>(&self, key: K, urls: &[S])
    where
        K: Into<String>,
        S: AsRef<str>,
    {
        let key = key.into();
        let mut store = self.data.write();
        let entry = store.entry(key).or_default();
        for url in urls {
            let u = url.as_ref().to_string();
            if !entry.contains(&u) {
                entry.push(u);
            }
        }
    }

    /// 为单个 URL 添加方法级标签
    ///
    /// 等价于 Node 版本中 `CoolTag` 在方法上打标后由 `CoolUrlTagData.methodTag` 汇总的行为。
    pub fn add_method_tag<K, S>(&self, key: K, url: S)
    where
        K: Into<String>,
        S: AsRef<str>,
    {
        let key = key.into();
        let mut store = self.data.write();
        let entry = store.entry(key).or_default();
        let u = url.as_ref().to_string();
        if !entry.contains(&u) {
            entry.push(u);
        }
    }

    /// 根据 key 获取 URL 列表
    ///
    /// - `scope = Some("admin")` 时，只返回以 `/admin/` 开头的 URL；
    /// - `scope = Some("app")` 时，只返回以 `/app/` 开头的 URL；
    /// - 其他情况返回全部。
    pub fn by_key(&self, key: &str, scope: Option<&str>) -> Vec<String> {
        let store = self.data.read();
        let list = match store.get(key) {
            Some(v) => v.clone(),
            None => return Vec::new(),
        };

        match scope {
            Some("admin") => list
                .into_iter()
                .filter(|u| u.starts_with("/admin/"))
                .collect(),
            Some("app") => list
                .into_iter()
                .filter(|u| u.starts_with("/app/"))
                .collect(),
            _ => list,
        }
    }
}

static GLOBAL_URL_TAG_STORE: OnceCell<UrlTagStore> = OnceCell::new();

/// 获取全局 URL 标签存储
pub fn global_url_tag_store() -> &'static UrlTagStore {
    GLOBAL_URL_TAG_STORE.get_or_init(UrlTagStore::default)
}
