//! RPC 事务管理器
//!
//! 管理分布式事务的生命周期，存储事务状态，支持事务传播。

use crate::transaction::TransactionOptions;
use parking_lot::RwLock;
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

/// 事务状态
pub struct TransactionState {
    /// 事务对象（使用 Option 以便在提交/回滚时取出）
    pub txn: Arc<Mutex<Option<DatabaseTransaction>>>,
    /// 创建时间
    pub created_at: Instant,
    /// 连接名称
    pub connection_name: String,
}

impl TransactionState {
    pub fn new(txn: DatabaseTransaction, connection_name: String) -> Self {
        Self {
            txn: Arc::new(Mutex::new(Some(txn))),
            created_at: Instant::now(),
            connection_name,
        }
    }

    /// 检查是否过期（30分钟）
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > Duration::from_secs(1800)
    }
}

/// RPC 事务管理器
pub struct RpcTransactionManager {
    /// 存储的事务（key: 事务 ID）
    transactions: Arc<RwLock<HashMap<String, TransactionState>>>,
}

impl RpcTransactionManager {
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建新事务
    pub async fn create_transaction(
        &self,
        db: &DatabaseConnection,
        options: &TransactionOptions,
    ) -> Result<String, sea_orm::DbErr> {
        let txn = db.begin().await?;
        let transaction_id = Uuid::new_v4().to_string();

        let mut transactions = self.transactions.write();
        transactions.insert(
            transaction_id.clone(),
            TransactionState::new(
                txn,
                options
                    .connection_name
                    .clone()
                    .unwrap_or_else(|| "default".to_string()),
            ),
        );

        Ok(transaction_id)
    }

    /// 获取事务（返回事务的 Arc 引用）
    pub fn get_transaction(
        &self,
        transaction_id: &str,
    ) -> Option<Arc<Mutex<Option<DatabaseTransaction>>>> {
        let transactions = self.transactions.read();
        transactions
            .get(transaction_id)
            .map(|state| Arc::clone(&state.txn))
    }

    /// 提交事务
    pub async fn commit(&self, transaction_id: &str) -> Result<(), sea_orm::DbErr> {
        let txn_arc = {
            let mut transactions = self.transactions.write();
            transactions
                .remove(transaction_id)
                .map(|state| Arc::clone(&state.txn))
        };

        if let Some(txn_arc) = txn_arc {
            let mut txn_guard = txn_arc.lock().await;
            if let Some(txn) = txn_guard.take() {
                txn.commit().await?;
            }
        }
        Ok(())
    }

    /// 回滚事务
    pub async fn rollback(&self, transaction_id: &str) -> Result<(), sea_orm::DbErr> {
        let txn_arc = {
            let mut transactions = self.transactions.write();
            transactions
                .remove(transaction_id)
                .map(|state| Arc::clone(&state.txn))
        };

        if let Some(txn_arc) = txn_arc {
            let mut txn_guard = txn_arc.lock().await;
            if let Some(txn) = txn_guard.take() {
                let _ = txn.rollback().await;
            }
        }
        Ok(())
    }

    /// 清理过期事务
    pub async fn cleanup_expired(&self) {
        let mut transactions = self.transactions.write();
        let expired_ids: Vec<String> = transactions
            .iter()
            .filter(|(_id, state)| state.is_expired())
            .map(|(id, _)| id.clone())
            .collect();

        for id in expired_ids {
            if let Some(state) = transactions.remove(&id) {
                // 异步清理，不阻塞
                let txn = Arc::clone(&state.txn);
                tokio::spawn(async move {
                    let mut txn_guard = txn.lock().await;
                    if let Some(txn) = txn_guard.take() {
                        let _ = txn.rollback().await;
                    }
                });
            }
        }
    }
}

impl Default for RpcTransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局事务管理器
use once_cell::sync::OnceCell;

static GLOBAL_TRANSACTION_MANAGER: OnceCell<RpcTransactionManager> = OnceCell::new();

/// 获取全局事务管理器
pub fn global_transaction_manager() -> &'static RpcTransactionManager {
    GLOBAL_TRANSACTION_MANAGER.get_or_init(RpcTransactionManager::new)
}
