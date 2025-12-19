//! 插件常量定义
//!
//! 对应 TypeScript 版本的 `constant/global.ts`

/// 返回码
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResCode {
    /// 成功
    Success = 1000,
    /// 失败
    CommFail = 1001,
    /// 参数验证失败
    ValidateFail = 1002,
    /// 核心异常
    CoreFail = 1003,
}

impl ResCode {
    /// 获取返回码数值
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// 返回信息
#[derive(Debug, Clone)]
pub struct ResMessage;

impl ResMessage {
    /// 成功
    pub const SUCCESS: &'static str = "success";
    /// 失败
    pub const COMM_FAIL: &'static str = "comm fail";
    /// 参数验证失败
    pub const VALIDATE_FAIL: &'static str = "validate fail";
    /// 核心异常
    pub const CORE_FAIL: &'static str = "core fail";
}

/// 错误提示
#[derive(Debug, Clone)]
pub struct ErrInfo;

impl ErrInfo {
    /// 未设置操作实体
    pub const NO_ENTITY: &'static str = "未设置操作实体";
    /// 查询参数[id]不存在
    pub const NO_ID: &'static str = "查询参数[id]不存在";
    /// 排序参数不正确
    pub const SORT_FIELD: &'static str = "排序参数不正确";
}

/// 事件名称
#[derive(Debug, Clone)]
pub struct Event;

impl Event {
    /// 软删除
    pub const SOFT_DELETE: &'static str = "onSoftDelete";
    /// 服务成功启动
    pub const SERVER_READY: &'static str = "onServerReady";
    /// 服务就绪
    pub const READY: &'static str = "onReady";
    /// ES 数据改变
    pub const ES_DATA_CHANGE: &'static str = "esDataChange";
}

/// 全局配置
///
/// 对应 TypeScript 版本的 `GlobalConfig`
pub struct GlobalConfig {
    res_code: ResCodeConfig,
    res_message: ResMessageConfig,
}

pub struct ResCodeConfig {
    success: u32,
    comm_fail: u32,
    validate_fail: u32,
    core_fail: u32,
}

pub struct ResMessageConfig {
    success: String,
    comm_fail: String,
    validate_fail: String,
    core_fail: String,
}

impl GlobalConfig {
    /// 获取单例实例
    pub fn get_instance() -> &'static GlobalConfig {
        static INSTANCE: once_cell::sync::Lazy<GlobalConfig> =
            once_cell::sync::Lazy::new(|| GlobalConfig {
                res_code: ResCodeConfig {
                    success: 1000,
                    comm_fail: 1001,
                    validate_fail: 1002,
                    core_fail: 1003,
                },
                res_message: ResMessageConfig {
                    success: "success".to_string(),
                    comm_fail: "comm fail".to_string(),
                    validate_fail: "validate fail".to_string(),
                    core_fail: "core fail".to_string(),
                },
            });
        &INSTANCE
    }

    /// 获取返回码配置
    pub fn res_code(&self) -> &ResCodeConfig {
        &self.res_code
    }

    /// 获取返回信息配置
    pub fn res_message(&self) -> &ResMessageConfig {
        &self.res_message
    }
}

impl ResCodeConfig {
    pub fn success(&self) -> u32 {
        self.success
    }

    pub fn comm_fail(&self) -> u32 {
        self.comm_fail
    }

    pub fn validate_fail(&self) -> u32 {
        self.validate_fail
    }

    pub fn core_fail(&self) -> u32 {
        self.core_fail
    }
}

impl ResMessageConfig {
    pub fn success(&self) -> &str {
        &self.success
    }

    pub fn comm_fail(&self) -> &str {
        &self.comm_fail
    }

    pub fn validate_fail(&self) -> &str {
        &self.validate_fail
    }

    pub fn core_fail(&self) -> &str {
        &self.core_fail
    }
}
