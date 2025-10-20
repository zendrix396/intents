use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolverError {
    #[error("Reqwest failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("API returned a non-success status: {0}")]
    ApiError(String),

    // New error variant for transaction failures
    #[error("Transaction execution failed: {0}")]
    TransactionExecutionFailed(String),
}
