use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Serialize;
use serde_json::{json, Value};
use solver_core::fee_estimator::FeeEstimator;
use solver_core::rpc_manager::{ConnectionManager, RpcHealth};
use solver_core::{solve_swap_intent_with_jupiter, JupiterOrderResponse, SwapIntent};
use std::env;
use std::sync::Arc;

mod payer_manager;
use payer_manager::PayerManager;

#[derive(Clone)]
struct AppState {
    connection_manager: Arc<ConnectionManager>,
    fee_estimator: Arc<FeeEstimator>,
    payer_manager: Arc<PayerManager>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    println!("Starting Solana Intent Solver Service...");

    // Read the RPC URL from the .env file. Panic if it's not set.
    let rpc_url = env::var("RPC_URL").expect("FATAL: RPC_URL environment variable not set.");

    // The rpc_urls vector now contains only the URL from your config.
    let rpc_urls = vec![rpc_url];
    println!("[RPC Manager] Using endpoint: {}", rpc_urls[0]);

    let connection_manager = Arc::new(ConnectionManager::new(rpc_urls));
    connection_manager.start_health_checker();

    let fee_estimator = Arc::new(FeeEstimator::new(connection_manager.clone()));
    fee_estimator.start_fee_monitor();

    let payer_manager = Arc::new(PayerManager::from_env(connection_manager.clone()));
    payer_manager.start_balance_monitor();

    let app_state = AppState {
        connection_manager,
        fee_estimator,
        payer_manager,
    };

    // Create the router and pass the state to it.
    let app = app(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

/// Creates the Axum application router with shared state.
fn app(app_state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/solve", post(solve_handler))
        .with_state(app_state)
}

async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let rpc_health = state.connection_manager.get_health_status().await;

    let low_fee = state.fee_estimator.get_priority_fee_for_level("low").await;
    let medium_fee = state
        .fee_estimator
        .get_priority_fee_for_level("medium")
        .await;
    let high_fee = state.fee_estimator.get_priority_fee_for_level("high").await;

    #[derive(Serialize)]
    struct HealthResponse {
        status: &'static str,
        payer_wallet: String,
        rpc_endpoints: Vec<RpcHealth>,
        priority_fees: Value,
    }

    let response = HealthResponse {
        status: "ok",
        payer_wallet: state.payer_manager.public_key().to_string(),
        rpc_endpoints: rpc_health,
        priority_fees: json!({
            "low": low_fee,
            "medium": medium_fee,
            "high": high_fee,
        }),
    };

    (StatusCode::OK, Json(json!(response)))
}

async fn solve_handler(
    State(_state): State<AppState>,
    Json(intent): Json<SwapIntent>,
) -> Result<Json<JupiterOrderResponse>, (StatusCode, String)> {
    println!("[API] Received solve request: {intent:?}");

    match solve_swap_intent_with_jupiter(&intent).await {
        Ok(order) => Ok(Json(order)),
        Err(e) => {
            eprintln!("[API] Error solving intent: {e}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get order from Jupiter: {e}"),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use solana_sdk::signature::Signer;
    use tower::ServiceExt;

    fn setup_test_environment() -> (AppState, solana_sdk::signature::Keypair) {
        let mock_keypair = solana_sdk::signature::Keypair::new();

        env::set_var("PRIVATE_KEY", mock_keypair.to_base58_string());
        env::remove_var("SEED_PHRASE");

        let rpc_urls = vec!["https://api.devnet.solana.com".to_string()];
        let connection_manager = Arc::new(ConnectionManager::new(rpc_urls));
        let fee_estimator = Arc::new(FeeEstimator::new(connection_manager.clone()));
        let payer_manager = Arc::new(PayerManager::from_env(connection_manager.clone()));

        let app_state = AppState {
            connection_manager,
            fee_estimator,
            payer_manager,
        };

        (app_state, mock_keypair)
    }

    #[tokio::test]
    async fn test_health_check() {
        let (app_state, mock_keypair) = setup_test_environment();
        let expected_pubkey = mock_keypair.pubkey().to_string();
        let app = app(app_state);

        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let body: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(body["status"], "ok");
        assert_eq!(body["payer_wallet"], expected_pubkey);
    }

    #[tokio::test]
    #[ignore] // This test requires internet access.
    async fn test_solve_endpoint_integration() {
        let (app_state, _) = setup_test_environment();
        let app = app(app_state);

        // A real swap intent: 0.1 SOL to USDC, with a taker address.
        let payload = json!({
            "inputMint": "So11111111111111111111111111111111111111112",
            "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "amount": 100000000, // 0.1 SOL
            // We can use a real address here. The API doesn't check if the taker has funds, only that it's a valid pubkey.
            "taker": "jdocuPgEAjMfihABsPgKEvYtsmMzjUHeq9LX4Hvs7f3"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/solve")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let order: JupiterOrderResponse = serde_json::from_slice(&body).unwrap();

        // The API call is successful even if it returns an error message in the JSON.
        // This is a great test because it proves we are correctly communicating with the API.
        assert!(order.error_message.is_some());
        assert_eq!(order.error_message.unwrap(), "Insufficient funds");

        // We still get quote data back, which is what we care about.
        assert!(order.out_amount.parse::<u64>().unwrap() > 0);
    }
}
