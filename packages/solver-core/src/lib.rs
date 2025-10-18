pub mod fee_estimator;
pub mod rpc_manager;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct SwapIntent {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SwapSolution {
    pub transaction_id: String,
    pub expected_out: u64,
}

pub async fn solve_swap_intent(intent: SwapIntent) -> SwapSolution {
    // Mocked solution: echo a fake tx id and a naive expected_out
    let expected_out = intent.amount.saturating_mul(99) / 100; // assume 1% fee/impact
    SwapSolution {
        transaction_id: "mocked-tx-123".to_string(),
        expected_out,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_solve_swap_intent_mock() {
        let intent = SwapIntent {
            input_mint: "in".to_string(),
            output_mint: "out".to_string(),
            amount: 1000,
        };
        let solution = solve_swap_intent(intent.clone()).await;
        assert_eq!(solution.transaction_id, "mocked-tx-123");
        assert_eq!(solution.expected_out, 990);
    }
}
