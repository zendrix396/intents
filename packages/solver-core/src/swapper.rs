use serde::{Deserialize, Serialize};

/// Represents the user's intent, now including the taker address.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwapIntent {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    pub taker: String,
    /// Slippage tolerance in basis points (e.g. 50 = 0.5%). Defaults to 50 if not provided.
    #[serde(default = "default_slippage_bps")]
    pub slippage_bps: u16,
}

fn default_slippage_bps() -> u16 {
    50
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
        // Slippage validation: 0-10000 bps (0-100%)
        if self.slippage_bps > 10000 {
            return Err("slippageBps must be between 0 and 10000".to_string());
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
        "https://lite-api.jup.ag/ultra/v1/order?inputMint={}&outputMint={}&amount={}&taker={}&slippageBps={}",
        intent.input_mint, intent.output_mint, intent.amount, intent.taker, intent.slippage_bps
    );

    tracing::info!(
        "Calling Jupiter Ultra API for swap: {} -> {} (slippage: {}bps)",
        intent.input_mint,
        intent.output_mint,
        intent.slippage_bps
    );

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;

    let order_response = response.json::<JupiterOrderResponse>().await?;

    Ok(order_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_intent() -> SwapIntent {
        SwapIntent {
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            amount: 100_000_000,
            taker: "jdocuPgEAjMfihABsPgKEvYtsmMzjUHeq9LX4Hvs7f3".to_string(),
            slippage_bps: 50,
        }
    }

    #[test]
    fn test_valid_intent() {
        assert!(valid_intent().validate().is_ok());
    }

    #[test]
    fn test_slippage_defaults_to_50() {
        let json = r#"{"inputMint":"So11111111111111111111111111111111111111112","outputMint":"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v","amount":100000000,"taker":"jdocuPgEAjMfihABsPgKEvYtsmMzjUHeq9LX4Hvs7f3"}"#;
        let intent: SwapIntent = serde_json::from_str(json).unwrap();
        assert_eq!(intent.slippage_bps, 50);
    }

    #[test]
    fn test_slippage_valid_values() {
        let mut intent = valid_intent();
        intent.slippage_bps = 0;
        assert!(intent.validate().is_ok());

        intent.slippage_bps = 10000;
        assert!(intent.validate().is_ok());
    }

    #[test]
    fn test_slippage_too_high() {
        let mut intent = valid_intent();
        intent.slippage_bps = 10001;
        assert!(intent.validate().is_err());
        assert!(intent.validate().unwrap_err().contains("slippageBps"));
    }

    #[test]
    fn test_empty_input_mint() {
        let mut intent = valid_intent();
        intent.input_mint = String::new();
        assert!(intent.validate().is_err());
    }

    #[test]
    fn test_same_tokens() {
        let mut intent = valid_intent();
        intent.output_mint = intent.input_mint.clone();
        assert!(intent.validate().is_err());
    }

    #[test]
    fn test_zero_amount() {
        let mut intent = valid_intent();
        intent.amount = 0;
        assert!(intent.validate().is_err());
    }

    #[test]
    fn test_invalid_taker() {
        let mut intent = valid_intent();
        intent.taker = "short".to_string();
        assert!(intent.validate().is_err());
    }
}
