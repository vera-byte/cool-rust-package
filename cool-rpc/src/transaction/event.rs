//! RPC 事务事件处理
//!
//! 对应 TypeScript 版本的 `rpc/src/transaction/event.ts`
//!
//! 处理事务提交/回滚事件，协调分布式事务。

use crate::broker::CoolRpc;
use crate::transaction::manager::global_transaction_manager;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::info;

/// 事务事件数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionEvent {
    /// 事务 ID
    pub rpc_transaction_id: String,
    /// 是否提交
    pub commit: bool,
}

/// 事务事件处理器
pub struct TransactionEventHandler;

#[async_trait]
impl crate::event::RpcEventHandler for TransactionEventHandler {
    async fn handle(&self, event: &crate::event::RpcEvent) {
        if event.name != "moleculer.transaction" {
            return;
        }

        let transaction_data: TransactionEvent = match serde_json::from_value(event.data.clone()) {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("解析事务事件失败: {}", e);
                return;
            }
        };

        info!(
            "处理事务事件: transaction_id={}, commit={}",
            transaction_data.rpc_transaction_id, transaction_data.commit
        );

        let manager = global_transaction_manager();

        if transaction_data.commit {
            if let Err(e) = manager.commit(&transaction_data.rpc_transaction_id).await {
                tracing::error!("提交事务失败: {}", e);
            } else {
                info!(
                    "事务已提交: transaction_id={}",
                    transaction_data.rpc_transaction_id
                );
            }
        } else {
            if let Err(e) = manager.rollback(&transaction_data.rpc_transaction_id).await {
                tracing::error!("回滚事务失败: {}", e);
            } else {
                info!(
                    "事务已回滚: transaction_id={}",
                    transaction_data.rpc_transaction_id
                );
            }
        }
    }

    fn event_name(&self) -> Option<&str> {
        Some("moleculer.transaction")
    }
}

/// 注册事务事件处理器到 RPC Broker
pub fn register_transaction_handler(rpc: &CoolRpc) {
    rpc.on("moleculer.transaction", TransactionEventHandler);
}

