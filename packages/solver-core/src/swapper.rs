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

    println!("[Solver] Calling Jupiter Ultra API: {url}");

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;

    let order_response = response.json::<JupiterOrderResponse>().await?;

    Ok(order_response)
}

