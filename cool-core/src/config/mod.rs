//! 配置模块
//!
//! 对应 TypeScript 版本的 `interface.ts` 中的配置定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cool 框架主配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CoolConfig {
    /// 是否自动导入数据库
    pub init_db: bool,
    /// 是否自动导入模块菜单
    pub init_menu: bool,
    /// 判断是否初始化的方式: "file" | "db"
    pub init_judge: String,
    /// Eps 接口文档
    pub eps: bool,
    /// 多租户配置
    pub tenant: Option<TenantConfig>,
    /// 多语言配置
    pub i18n: Option<I18nConfig>,
    /// CRUD 配置
    pub crud: CrudConfig,
    /// 文件上传配置
    pub file: Option<FileConfig>,
    /// Redis 配置
    pub redis: Option<RedisConfig>,
    /// JWT 配置
    pub jwt: JwtConfig,
}

/// 多租户配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TenantConfig {
    /// 是否开启
    pub enable: bool,
    /// 需要过滤多租户的 URL
    #[serde(default)]
    pub urls: Vec<String>,
}

/// 多语言配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct I18nConfig {
    /// 是否开启
    pub enable: bool,
    /// 支持的语言列表
    #[serde(default)]
    pub languages: Vec<String>,
    /// 翻译服务地址
    pub service_url: Option<String>,
}

/// CRUD 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrudConfig {
    /// 软删除
    #[serde(default = "default_soft_delete")]
    pub soft_delete: bool,
    /// 分页查询每页条数
    #[serde(default = "default_page_size")]
    pub page_size: u64,
    /// 插入方式: "normal" | "save"
    #[serde(default = "default_upsert")]
    pub upsert: String,
}

impl Default for CrudConfig {
    fn default() -> Self {
        Self {
            soft_delete: true,
            page_size: 20,
            upsert: "normal".to_string(),
        }
    }
}

fn default_soft_delete() -> bool {
    true
}

fn default_page_size() -> u64 {
    20
}

fn default_upsert() -> String {
    "normal".to_string()
}

/// 文件上传配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    /// 上传模式: "local" | "cloud" | "other"
    pub mode: String,
    /// 本地上传文件地址前缀
    pub domain: Option<String>,
    /// 阿里云 OSS 配置
    pub oss: Option<OssConfig>,
    /// 腾讯云 COS 配置
    pub cos: Option<CosConfig>,
    /// 七牛云配置
    pub qiniu: Option<QiniuConfig>,
    /// AWS S3 配置
    pub aws: Option<AwsConfig>,
}

/// 阿里云 OSS 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OssConfig {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub bucket: String,
    pub endpoint: String,
    pub timeout: Option<String>,
    pub exp_after: Option<u64>,
    pub max_size: Option<u64>,
    pub host: Option<String>,
    pub public_domain: Option<String>,
}

/// 腾讯云 COS 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosConfig {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub bucket: String,
    pub region: String,
    pub public_domain: String,
    pub duration_seconds: Option<u64>,
    pub allow_prefix: Option<String>,
    pub allow_actions: Option<Vec<String>>,
}

/// 七牛云配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QiniuConfig {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub bucket: String,
    pub region: String,
    pub public_domain: String,
    pub upload_url: Option<String>,
    pub file_key: Option<String>,
}

/// AWS S3 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub bucket: String,
    pub region: String,
    pub fields: Option<HashMap<String, String>>,
    pub conditions: Option<Vec<serde_json::Value>>,
    pub expires: Option<u64>,
    pub public_domain: Option<String>,
    pub force_path_style: Option<bool>,
}

/// Redis 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: u8,
}

impl RedisConfig {
    /// 获取连接 URL
    pub fn url(&self) -> String {
        match &self.password {
            Some(pwd) if !pwd.is_empty() => {
                format!("redis://:{}@{}:{}/{}", pwd, self.host, self.port, self.db)
            }
            _ => format!("redis://{}:{}/{}", self.host, self.port, self.db),
        }
    }
}

/// JWT 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// 密钥
    #[serde(default = "default_jwt_secret")]
    pub secret: String,
    /// 过期时间（秒）
    #[serde(default = "default_jwt_expire")]
    pub expire: u64,
    /// 刷新过期时间（秒）
    #[serde(default = "default_jwt_refresh_expire")]
    pub refresh_expire: u64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "cool-admin-rust".to_string(),
            expire: 7200,
            refresh_expire: 604800,
        }
    }
}

fn default_jwt_secret() -> String {
    "cool-admin-rust".to_string()
}

fn default_jwt_expire() -> u64 {
    7200
}

fn default_jwt_refresh_expire() -> u64 {
    604800
}

/// 模块配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleConfig {
    /// 模块名称
    pub name: String,
    /// 模块描述
    pub description: String,
    /// 模块加载顺序，默认为0，值越大越优先加载
    #[serde(default)]
    pub order: i32,
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库类型: "mysql" | "postgres" | "sqlite"
    pub r#type: String,
    /// 主机地址
    pub host: Option<String>,
    /// 端口
    pub port: Option<u16>,
    /// 用户名
    pub username: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 数据库名
    pub database: String,
    /// 最大连接数
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// 最小连接数
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    /// 连接超时（秒）
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,
    /// 是否打印 SQL 日志
    #[serde(default)]
    pub logging: bool,
    /// 是否同步数据库结构
    #[serde(default)]
    pub synchronize: bool,
}

fn default_max_connections() -> u32 {
    100
}

fn default_min_connections() -> u32 {
    5
}

fn default_connect_timeout() -> u64 {
    10
}

impl DatabaseConfig {
    /// 获取数据库连接 URL
    pub fn url(&self) -> String {
        match self.r#type.as_str() {
            "sqlite" => format!("sqlite://{}?mode=rwc", self.database),
            "mysql" => {
                format!(
                    "mysql://{}:{}@{}:{}/{}",
                    self.username.as_deref().unwrap_or("root"),
                    self.password.as_deref().unwrap_or(""),
                    self.host.as_deref().unwrap_or("localhost"),
                    self.port.unwrap_or(3306),
                    self.database
                )
            }
            "postgres" => {
                format!(
                    "postgres://{}:{}@{}:{}/{}",
                    self.username.as_deref().unwrap_or("postgres"),
                    self.password.as_deref().unwrap_or(""),
                    self.host.as_deref().unwrap_or("localhost"),
                    self.port.unwrap_or(5432),
                    self.database
                )
            }
            _ => panic!("不支持的数据库类型: {}", self.r#type),
        }
    }
}
