use serde::{Deserialize, Serialize};

/// Represents the user's intent, now including the taker address.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwapIntent {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    pub taker: String,
}

impl SwapIntent {
    /// Validates the swap intent fields.
    /// Returns Ok(()) if valid, or an error message describing the issue.
    pub fn validate(&self) -> Result<(), String> {
        if self.input_mint.is_empty() {
            return Err("inputMint is required".to_string());
        }
        if self.output_mint.is_empty() {
            return Err("outputMint is required".to_string());
        }
        if self.input_mint == self.output_mint {
            return Err("inputMint and outputMint must be different".to_string());
        }
        if self.amount == 0 {
            return Err("amount must be greater than 0".to_string());
        }
        if self.taker.is_empty() {
            return Err("taker is required".to_string());
        }
        // Basic Solana pubkey validation: base58, 32-44 chars
        if self.taker.len() < 32 || self.taker.len() > 44 {
            return Err("taker must be a valid Solana public key".to_string());
        }
        Ok(())
    }
}

/// Represents the response from the Jupiter /ultra/v1/order API.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct JupiterOrderResponse {
    pub in_amount: String,
    pub out_amount: String,
    #[serde(default)]
    pub transaction: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
}

/// Solves a swap intent using Jupiter's Ultra Order API.
pub async fn solve_swap_intent_with_jupiter(
    intent: &SwapIntent,
) -> Result<JupiterOrderResponse, reqwest::Error> {
    let url = format!(
        "https://lite-api.jup.ag/ultra/v1/order?inputMint={}&outputMint={}&amount={}&taker={}",
        intent.input_mint, intent.output_mint, intent.amount, intent.taker
    );

    tracing::info!("Calling Jupiter Ultra API for swap: {} -> {}", intent.input_mint, intent.output_mint);

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;

    let order_response = response.json::<JupiterOrderResponse>().await?;

    Ok(order_response)
}
