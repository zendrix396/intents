use crate::rpc_manager::ConnectionManager;
use crate::SolverError;
use solana_sdk::{
    signature::Signature,
    transaction::VersionedTransaction,
};
use std::sync::Arc;

/// Responsible for sending and confirming pre-signed transactions.
pub struct TransactionExecutor {
    connection_manager: Arc<ConnectionManager>,
}

impl TransactionExecutor {
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        Self { connection_manager }
    }

    /// Sends and confirms a transaction that has already been signed.
    pub async fn execute_transaction(
        &self,
        transaction: &VersionedTransaction,
    ) -> Result<Signature, SolverError> {
        let client = self.connection_manager.get_healthy_client().await;

        tracing::info!("Sending pre-signed transaction...");

        let signature = client
            .send_and_confirm_transaction(transaction)
            .await
            .map_err(|e| {
                tracing::error!("Transaction failed: {}", e);
                SolverError::TransactionExecutionFailed(format!("Transaction failed: {}", e))
            })?;

        tracing::info!("Transaction confirmed with signature: {}", signature);

        Ok(signature)
    }
}
