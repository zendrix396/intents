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
use solver_core::{solve_swap_intent, SwapIntent, SwapSolution};
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

    let rpc_urls = vec![
        "https://api.devnet.solana.com".to_string(),
        "https://api.mainnet-beta.solana.com".to_string(),
    ];

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
) -> (StatusCode, Json<SwapSolution>) {
    let solution = solve_swap_intent(intent).await;
    (StatusCode::OK, Json(solution))
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

    #[tokio::test]
    async fn test_health_check() {
        let mock_keypair = solana_sdk::signature::Keypair::new();
        let expected_pubkey = mock_keypair.pubkey().to_string();
        std::env::remove_var("SEED_PHRASE");
      
        let rpc_urls = vec!["https://api.devnet.solana.com".to_string()];
        let connection_manager = Arc::new(ConnectionManager::new(rpc_urls));
        let fee_estimator = Arc::new(FeeEstimator::new(connection_manager.clone()));
        let payer_manager = Arc::new(PayerManager::from_env(connection_manager.clone()));

        let test_state = AppState {
            connection_manager,
            fee_estimator,
            payer_manager,
        };

        let app = app(test_state);

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
        assert!(body["rpc_endpoints"].is_array());
        assert!(body["priority_fees"].is_object());
        assert!(body["priority_fees"]["medium"].is_number());
    }

    #[tokio::test]
    async fn test_solve_endpoint() {
        let mock_keypair = solana_sdk::signature::Keypair::new();
        std::env::remove_var("SEED_PHRASE");

        let rpc_urls = vec!["https://api.devnet.solana.com".to_string()];
        let connection_manager = Arc::new(ConnectionManager::new(rpc_urls));
        let fee_estimator = Arc::new(FeeEstimator::new(connection_manager.clone()));
        let payer_manager = Arc::new(PayerManager::from_env(connection_manager.clone()));

        let test_state = AppState {
            connection_manager,
            fee_estimator,
            payer_manager,
        };

        let app = app(test_state);

        let payload = json!({
            "input_mint": "So11111111111111111111111111111111111111112",
            "output_mint": "USDC111111111111111111111111111111111111111",
            "amount": 1000
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

        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["expected_out"], json!(990));
        assert!(value["transaction_id"]
            .as_str()
            .unwrap()
            .starts_with("mocked-tx-"));
    }
}
