use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Serialize;
use serde_json::{json, Value};
use solana_sdk::signature::Signer;
use solana_sdk::{message::Message, system_instruction, transaction::VersionedTransaction};
use solver_core::{
    executor::TransactionExecutor,
    fee_estimator::FeeEstimator,
    rpc_manager::{ConnectionManager, RpcHealth},
    solve_swap_intent_with_jupiter, JupiterOrderResponse, SwapIntent,
};
use std::env;
use std::sync::Arc;
use base64::Engine; // for base64 decode Engine API

mod payer_manager;
use payer_manager::PayerManager;

#[derive(Clone)]
struct AppState {
    connection_manager: Arc<ConnectionManager>,
    fee_estimator: Arc<FeeEstimator>,
    payer_manager: Arc<PayerManager>,
    executor: Arc<TransactionExecutor>, // New field
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("Starting Solana Intent Solver Service...");

    let rpc_url = env::var("RPC_URL").expect("FATAL: RPC_URL environment variable not set.");
    let rpc_urls = vec![rpc_url];
    println!("[RPC Manager] Using endpoint: {}", rpc_urls[0]);

    // 1. Create the Arc for the ConnectionManager ONCE.
    let connection_manager = Arc::new(ConnectionManager::new(rpc_urls));
    
    // 2. Start the health checker on the original Arc.
    connection_manager.start_health_checker();

    // 3. Create other components by CLONING the Arc.
    //    Cloning an Arc is cheap; it just bumps a reference counter.
    let fee_estimator = Arc::new(FeeEstimator::new(connection_manager.clone()));
    fee_estimator.start_fee_monitor();

    let payer_manager = Arc::new(PayerManager::from_env(connection_manager.clone()));
    payer_manager.start_balance_monitor();
    
    let executor = Arc::new(TransactionExecutor::new(connection_manager.clone()));

    // 4. NOW, create the AppState. This will MOVE the final ownership of the
    //    original `connection_manager` Arc and the other Arcs into the state.
    let app_state = AppState {
        connection_manager, // The original is moved here.
        fee_estimator,
        payer_manager,
        executor,
    };

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
        // Add a new route to test the executor
        .route("/test_execute", post(test_execute_handler))
        .route("/execute", post(execute_handler))
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
            println!("[API] Error solving intent: {e}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get order from Jupiter: {e}"),
            ))
        }
    }
}

/// A simple handler to test our TransactionExecutor.
/// It creates and executes a transaction that sends 0.001 SOL to itself.
async fn test_execute_handler(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    println!("[API] Received /test_execute request");
    let payer = state.payer_manager.get_keypair();
    let client = state.connection_manager.get_healthy_client().await;

    // 1. Create the instruction.
    let instruction =
        system_instruction::transfer(&payer.pubkey(), &payer.pubkey(), 1_000_000); // 0.001 SOL

    // 2. Get a recent blockhash. This is required for signing.
    let (blockhash, _) = client
        .get_latest_blockhash_with_commitment(client.commitment())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get blockhash: {}", e),
            )
        })?;

    // 3. Build the message with the blockhash.
    let message = solana_sdk::message::VersionedMessage::Legacy(Message::new_with_blockhash(
        &[instruction],
        Some(&payer.pubkey()),
        &blockhash,
    ));

    // 4. Create AND SIGN the transaction in a single step using try_new.
    let tx = VersionedTransaction::try_new(message, &[payer]).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to sign transaction: {}", e),
        )
    })?; // The '?' handles the Result

    // 5. Pass the now-signed transaction to the executor to send and confirm.
    match state.executor.execute_transaction(&tx).await {
        Ok(signature) => Ok(Json(json!({ "signature": signature.to_string() }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Execution failed: {}", e),
        )),
    }
}

async fn execute_handler(
    State(state): State<AppState>,
    Json(intent): Json<SwapIntent>,
) -> Result<Json<Value>, (StatusCode, String)> {
    println!("[API] Received /execute request for intent: {:?}", intent);

    // 1) Get quote/order from Jupiter
    let quote = solve_swap_intent_with_jupiter(&intent).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get quote: {}", e),
        )
    })?;

    if let Some(err) = &quote.error_message {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Jupiter returned error: {}", err),
        ));
    }

    println!("[API] Step 1/3: Got quote successfully.");

    // 2) Extract the transaction string returned by Jupiter (if present)
    let tx_b64 = quote.transaction.as_ref().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "Quote did not include a transaction".to_string(),
        )
    })?;

    // 3) Decode and deserialize the VersionedTransaction
    let tx_bytes = base64::engine::general_purpose::STANDARD.decode(tx_b64).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to decode transaction: {}", e),
        )
    })?;

    let tx: VersionedTransaction = bincode::deserialize(&tx_bytes).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to deserialize transaction: {}", e),
        )
    })?;

    // 4) Execute it
    match state.executor.execute_transaction(&tx).await {
        Ok(signature) => {
            println!("[API] Step 3/3: Execution successful!");
            Ok(Json(json!({ "signature": signature.to_string() })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Execution failed: {}", e),
        )),
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
    use std::env;

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
