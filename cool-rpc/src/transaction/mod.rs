//! RPC 事务模块
//!
//! 对应 TypeScript 版本的 `rpc/src/decorator/transaction.ts` 和 `rpc/src/transaction/event.ts`
//!
//! 提供分布式事务支持，允许跨服务的事务传播。

mod event;
mod manager;

pub use event::*;
pub use manager::*;

/// 事务隔离级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    /// 读未提交
    ReadUncommitted,
    /// 读已提交
    ReadCommitted,
    /// 可重复读
    RepeatableRead,
    /// 串行化
    Serializable,
}

/// 事务选项
#[derive(Debug, Clone, Default)]
pub struct TransactionOptions {
    /// 连接名称
    pub connection_name: Option<String>,
    /// 隔离级别
    pub isolation: Option<IsolationLevel>,
}
