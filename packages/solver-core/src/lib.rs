pub mod error;
pub mod executor;
pub mod fee_estimator;
pub mod rpc_manager;
pub mod swapper;

// Re-export the key components so solver-service can use them easily.
pub use error::SolverError;
pub use executor::{SimulationResult, TransactionExecutor};
pub use swapper::{solve_swap_intent_with_jupiter, JupiterOrderResponse, SwapIntent};
