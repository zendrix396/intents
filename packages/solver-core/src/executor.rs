use crate::rpc_manager::ConnectionManager;
use crate::SolverError;
use solana_sdk::{
    signature::{Keypair, Signature},
    transaction::VersionedTransaction,
};
use std::sync::Arc;

/// Responsible for sending and confirming transactions.
pub struct TransactionExecutor {
    connection_manager: Arc<ConnectionManager>,
}

impl TransactionExecutor {
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        Self { connection_manager }
    }

    /// Sends and confirms a transaction.
    /// The transaction should already be signed when passed to this method.
    pub async fn execute_transaction(
        &self,
        transaction: VersionedTransaction,
        _payer: &Keypair,
    ) -> Result<Signature, SolverError> {
        let client = self.connection_manager.get_healthy_client().await;

        println!("[Executor] Sending transaction...");

        // Send the transaction. Using `send_and_confirm_transaction` is a simple way
        // to handle sending and polling for confirmation with retries.
        let signature = client
            .send_and_confirm_transaction(&transaction)
            .await
            .map_err(|e| {
                SolverError::TransactionExecutionFailed(format!("Transaction failed: {}", e))
            })?;

        println!(
            "[Executor] Transaction confirmed with signature: {}",
            signature
        );

        Ok(signature)
    }
}
