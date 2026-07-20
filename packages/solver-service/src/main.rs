use axum::http::HeaderValue;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use base64::Engine;
use serde::Serialize;
use serde_json::{json, Value};
use solana_sdk::transaction::VersionedTransaction;
use solver_core::{
    executor::TransactionExecutor,
    fee_estimator::FeeEstimator,
    rpc_manager::{ConnectionManager, RpcHealth},
    solve_swap_intent_with_jupiter, JupiterOrderResponse, SwapIntent,
};
use std::env;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

mod payer_manager;
use payer_manager::PayerManager;

#[derive(Clone)]
struct AppState {
    connection_manager: Arc<ConnectionManager>,
    fee_estimator: Arc<FeeEstimator>,
    payer_manager: Arc<PayerManager>,
    executor: Arc<TransactionExecutor>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting Solana Intent Solver Service...");

    let rpc_url = env::var("RPC_URL").expect("FATAL: RPC_URL environment variable not set.");
    let rpc_urls = vec![rpc_url];
    tracing::info!(endpoint = %rpc_urls[0], "Using RPC endpoint");

    let connection_manager = Arc::new(ConnectionManager::new(rpc_urls));
    connection_manager.start_health_checker();

    let fee_estimator = Arc::new(FeeEstimator::new(connection_manager.clone()));
    fee_estimator.start_fee_monitor();

    let payer_manager = Arc::new(PayerManager::from_env(connection_manager.clone()));
    payer_manager.start_balance_monitor();

    let executor = Arc::new(TransactionExecutor::new(connection_manager.clone()));

    let app_state = AppState {
        connection_manager,
        fee_estimator,
        payer_manager,
        executor,
    };

    let app = app(app_state);

    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    tracing::info!(addr = %bind_addr, "Server listening");
    axum::serve(listener, app).await.unwrap();
}

fn app(app_state: AppState) -> Router {
    let cors = if let Ok(origin) = env::var("ALLOWED_ORIGIN") {
        match origin.parse::<HeaderValue>() {
            Ok(val) => {
                tracing::info!(origin = %origin, "Configuring CORS for specific origin");
                CorsLayer::new()
                    .allow_origin(AllowOrigin::exact(val))
                    .allow_methods(Any)
                    .allow_headers(Any)
            }
            Err(_) => {
                tracing::warn!("Invalid ALLOWED_ORIGIN value, falling back to allow all");
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any)
            }
        }
    } else {
        tracing::info!("CORS configured to allow all origins");
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    Router::new()
        .route("/health", get(health_check))
        .route("/fees", get(fees_handler))
        .route("/solve", post(solve_handler))
        .route("/execute", post(execute_handler))
        .layer(cors)
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

async fn fees_handler(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let low_fee = state.fee_estimator.get_priority_fee_for_level("low").await;
    let medium_fee = state
        .fee_estimator
        .get_priority_fee_for_level("medium")
        .await;
    let high_fee = state.fee_estimator.get_priority_fee_for_level("high").await;
    let very_high_fee = state
        .fee_estimator
        .get_priority_fee_for_level("very-high")
        .await;

    let response = json!({
        "priority_fees": {
            "low": low_fee,
            "medium": medium_fee,
            "high": high_fee,
            "very_high": very_high_fee,
        },
        "unit": "micro_lamports_per_cu",
        "description": {
            "low": "Cheapest, may take longer to land",
            "medium": "Balanced cost and speed",
            "high": "Fast inclusion, recommended for most swaps",
            "very_high": "Highest priority, fastest inclusion"
        }
    });

    (StatusCode::OK, Json(response))
}

async fn solve_handler(
    State(_state): State<AppState>,
    Json(intent): Json<SwapIntent>,
) -> Result<Json<JupiterOrderResponse>, (StatusCode, String)> {
    if let Err(e) = intent.validate() {
        tracing::warn!(error = %e, "Invalid solve request");
        return Err((StatusCode::BAD_REQUEST, e));
    }

    tracing::info!(
        input = %intent.input_mint,
        output = %intent.output_mint,
        amount = intent.amount,
        "Processing solve request"
    );

    match solve_swap_intent_with_jupiter(&intent).await {
        Ok(order) => {
            tracing::info!("Successfully got quote from Jupiter");
            Ok(Json(order))
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to solve intent");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get order from Jupiter: {e}"),
            ))
        }
    }
}

async fn execute_handler(
    State(state): State<AppState>,
    Json(intent): Json<SwapIntent>,
) -> Result<Json<Value>, (StatusCode, String)> {
    if let Err(e) = intent.validate() {
        tracing::warn!(error = %e, "Invalid execute request");
        return Err((StatusCode::BAD_REQUEST, e));
    }

    tracing::info!(
        input = %intent.input_mint,
        output = %intent.output_mint,
        amount = intent.amount,
        "Processing /execute request"
    );

    let start_time = std::time::Instant::now();

    let quote = solve_swap_intent_with_jupiter(&intent).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get quote");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get quote: {}", e),
        )
    })?;

    if let Some(err) = &quote.error_message {
        tracing::warn!(error = %err, "Jupiter returned error");
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Jupiter returned error: {}", err),
        ));
    }

    tracing::info!("Step 1/3: Got quote successfully");

    let tx_b64 = quote.transaction.as_ref().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "Quote did not include a transaction".to_string(),
        )
    })?;

    let tx_bytes = base64::engine::general_purpose::STANDARD
        .decode(tx_b64)
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to decode transaction");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to decode transaction: {}", e),
            )
        })?;

    let tx: VersionedTransaction = bincode::deserialize(&tx_bytes).map_err(|e| {
        tracing::error!(error = %e, "Failed to deserialize transaction");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to deserialize transaction: {}", e),
        )
    })?;

    // Pre-flight simulation
    tracing::info!("Step 2/3: Running pre-flight simulation...");
    let sim_result = state
        .executor
        .simulate_transaction(&tx)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Simulation RPC call failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Simulation failed: {}", e),
            )
        })?;

    if !sim_result.success {
        tracing::warn!(
            error = ?sim_result.error,
            units_consumed = sim_result.units_consumed,
            "Transaction simulation failed, aborting execution"
        );
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Transaction simulation failed: {}",
                sim_result
                    .error
                    .unwrap_or_else(|| "unknown error".to_string())
            ),
        ));
    }

    let priority_fee = state.fee_estimator.get_priority_fee_for_level("high").await;

    tracing::info!(
        units_consumed = sim_result.units_consumed,
        "Step 2/3: Simulation passed"
    );

    match state.executor.execute_transaction(&tx).await {
        Ok(signature) => {
            let elapsed_ms = start_time.elapsed().as_millis() as u64;
            tracing::info!(
                signature = %signature,
                elapsed_ms,
                priority_fee,
                units_consumed = sim_result.units_consumed,
                "Step 3/3: Execution successful"
            );
            Ok(Json(json!({
                "signature": signature.to_string(),
                "priority_fee": priority_fee,
                "priority_fee_unit": "micro_lamports_per_cu",
                "execution_time_ms": elapsed_ms,
                "units_consumed": sim_result.units_consumed,
                "in_amount": quote.in_amount,
                "out_amount": quote.out_amount,
            })))
        }
        Err(e) => {
            tracing::error!(error = %e, "Execution failed");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Execution failed: {}", e),
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
    use std::env;
    use tower::ServiceExt;

    fn setup_test_environment() -> (AppState, solana_sdk::signature::Keypair) {
        let mock_keypair = solana_sdk::signature::Keypair::new();

        env::set_var("PRIVATE_KEY", mock_keypair.to_base58_string());
        env::remove_var("SEED_PHRASE");

        let rpc_urls = vec!["https://api.devnet.solana.com".to_string()];
        let connection_manager = Arc::new(ConnectionManager::new(rpc_urls));
        let fee_estimator = Arc::new(FeeEstimator::new(connection_manager.clone()));
        let payer_manager = Arc::new(PayerManager::from_env(connection_manager.clone()));
        let executor = Arc::new(TransactionExecutor::new(connection_manager.clone()));

        let app_state = AppState {
            connection_manager,
            fee_estimator,
            payer_manager,
            executor,
        };

        (app_state, mock_keypair)
    }

    #[tokio::test]
    #[serial_test::serial]
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
    #[serial_test::serial]
    async fn test_solve_validation_empty_input_mint() {
        let (app_state, _) = setup_test_environment();
        let app = app(app_state);

        let payload = json!({
            "inputMint": "",
            "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "amount": 100000000,
            "taker": "jdocuPgEAjMfihABsPgKEvYtsmMzjUHeq9LX4Hvs7f3"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/solve")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_solve_validation_same_tokens() {
        let (app_state, _) = setup_test_environment();
        let app = app(app_state);

        let payload = json!({
            "inputMint": "So11111111111111111111111111111111111111112",
            "outputMint": "So11111111111111111111111111111111111111112",
            "amount": 100000000,
            "taker": "jdocuPgEAjMfihABsPgKEvYtsmMzjUHeq9LX4Hvs7f3"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/solve")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_solve_validation_zero_amount() {
        let (app_state, _) = setup_test_environment();
        let app = app(app_state);

        let payload = json!({
            "inputMint": "So11111111111111111111111111111111111111112",
            "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "amount": 0,
            "taker": "jdocuPgEAjMfihABsPgKEvYtsmMzjUHeq9LX4Hvs7f3"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/solve")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_solve_validation_invalid_taker() {
        let (app_state, _) = setup_test_environment();
        let app = app(app_state);

        let payload = json!({
            "inputMint": "So11111111111111111111111111111111111111112",
            "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "amount": 100000000,
            "taker": "short"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/solve")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[serial_test::serial]
    #[ignore]
    async fn test_solve_endpoint_integration() {
        let (app_state, _) = setup_test_environment();
        let app = app(app_state);

        let payload = json!({
            "inputMint": "So11111111111111111111111111111111111111112",
            "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "amount": 100000000,
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

        assert!(order.error_message.is_some());
        assert_eq!(order.error_message.unwrap(), "Insufficient funds");
        assert!(order.out_amount.parse::<u64>().unwrap() > 0);
    }
}
