use serde::{Deserialize, Serialize};

/// User's intent to perform a swap
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwapIntent {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    pub slippage_bps: u16,
}

/// Response from Jupiter Quote API
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct JupiterQuoteResponse {
    pub input_mint: String,
    pub in_amount: String,
    pub output_mint: String,
    pub out_amount: String,
    pub route_plan: serde_json::Value,
}

/// Solve a swap intent using Jupiter
pub async fn solve_swap_intent_with_jupiter(
    intent: &SwapIntent,
) -> Result<JupiterQuoteResponse, reqwest::Error> {
    let url = format!(
        "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
        intent.input_mint, intent.output_mint, intent.amount, intent.slippage_bps
    );

    println!("[Solver] Calling Jupiter API: {url}");

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;

    let quote_response = response.json::<JupiterQuoteResponse>().await?;

    Ok(quote_response)
}
