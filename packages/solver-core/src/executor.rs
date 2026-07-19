use crate::rpc_manager::ConnectionManager;
use crate::SolverError;
use solana_sdk::{signature::Signature, transaction::VersionedTransaction};
use std::sync::Arc;
use std::time::Duration;

/// Result of a transaction simulation.
#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub success: bool,
    pub units_consumed: u64,
    pub error: Option<String>,
}

const MAX_RETRIES: u32 = 3;
const BASE_RETRY_DELAY_MS: u64 = 1000;

/// Responsible for sending and confirming pre-signed transactions
/// with retry logic and exponential backoff.
pub struct TransactionExecutor {
    connection_manager: Arc<ConnectionManager>,
}

impl TransactionExecutor {
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        Self { connection_manager }
    }

    /// Simulates a transaction without sending it, returning compute units consumed and errors.
    pub async fn simulate_transaction(
        &self,
        transaction: &VersionedTransaction,
    ) -> Result<SimulationResult, SolverError> {
        let client = self.connection_manager.get_healthy_client().await;

        tracing::info!("Simulating transaction as pre-flight check...");

        match client.simulate_transaction(transaction).await {
            Ok(response) => {
                let sim = response.value;
                let units = sim.units_consumed.unwrap_or(0);
                if let Some(err) = sim.err {
                    let err_str = format!("{:?}", err);
                    tracing::warn!(
                        error = %err_str,
                        units_consumed = units,
                        "Transaction simulation failed"
                    );
                    return Ok(SimulationResult {
                        success: false,
                        units_consumed: units,
                        error: Some(err_str),
                    });
                }

                tracing::info!(units_consumed = units, "Transaction simulation passed");
                Ok(SimulationResult {
                    success: true,
                    units_consumed: units,
                    error: None,
                })
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to simulate transaction");
                Err(SolverError::TransactionExecutionFailed(format!(
                    "Simulation RPC call failed: {}",
                    e
                )))
            }
        }
    }

    /// Sends and confirms a transaction with retry logic.
    /// Retries up to MAX_RETRIES times with exponential backoff on transient errors.
    pub async fn execute_transaction(
        &self,
        transaction: &VersionedTransaction,
    ) -> Result<Signature, SolverError> {
        let mut last_error = None;

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                let delay = BASE_RETRY_DELAY_MS * 2u64.pow(attempt - 1);
                tracing::warn!(
                    attempt,
                    max_retries = MAX_RETRIES,
                    delay_ms = delay,
                    "Retrying transaction after delay"
                );
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }

            let client = self.connection_manager.get_healthy_client().await;

            tracing::info!(attempt, "Sending transaction...");

            match client.send_and_confirm_transaction(transaction).await {
                Ok(signature) => {
                    tracing::info!(
                        signature = %signature,
                        attempts = attempt + 1,
                        "Transaction confirmed successfully"
                    );
                    return Ok(signature);
                }
                Err(e) => {
                    let err_str = e.to_string();
                    tracing::warn!(
                        attempt,
                        error = %err_str,
                        "Transaction attempt failed"
                    );

                    // Don't retry on non-transient errors
                    if is_non_retryable_error(&err_str) {
                        tracing::error!(error = %err_str, "Non-retryable transaction error");
                        return Err(SolverError::TransactionExecutionFailed(format!(
                            "Transaction failed (non-retryable): {}",
                            err_str
                        )));
                    }

                    last_error = Some(err_str);
                }
            }
        }

        let err_msg = last_error.unwrap_or_else(|| "Unknown error".to_string());
        tracing::error!(
            error = %err_msg,
            "Transaction failed after all retry attempts"
        );
        Err(SolverError::TransactionExecutionFailed(format!(
            "Transaction failed after {} retries: {}",
            MAX_RETRIES, err_msg
        )))
    }
}

/// Determines if an error is non-retryable (e.g., simulation failures, invalid signatures).
fn is_non_retryable_error(err: &str) -> bool {
    let lower = err.to_lowercase();
    lower.contains("invalid signature")
        || lower.contains("simulation failed")
        || lower.contains("transaction signature verification failure")
        || lower.contains("account not found")
        || lower.contains("insufficient funds")
}
