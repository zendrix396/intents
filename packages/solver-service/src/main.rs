use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Serialize;
use serde_json::{json, Value};
use solver_core::rpc_manager::{ConnectionManager, RpcHealth};
use solver_core::{solve_swap_intent, SwapIntent, SwapSolution};
use std::sync::Arc;
use tokio;

// Define a struct to hold the shared application state.
// We use Arc to safely share the ConnectionManager across async tasks.
#[derive(Clone)]
struct AppState {
    connection_manager: Arc<ConnectionManager>,
}

#[tokio::main]
async fn main() {
    println!("Starting Solana Intent Solver Service...");

    // For now, we'll hardcode some RPC URLs. Later, this will come from a config file.
    let rpc_urls = vec![
        "https://api.mainnet-beta.solana.com".to_string(),
        "https://solana-api.projectserum.com".to_string(), // A common fallback
        "https://bad-rpc-url.example.com".to_string(),     // A fake one to test failover
    ];

    // Create a new ConnectionManager and wrap it in an Arc for thread-safe sharing.
    let connection_manager = Arc::new(ConnectionManager::new(rpc_urls));

    // Start the background health checker task.
    connection_manager.start_health_checker();

    // Create our application state.
    let app_state = AppState { connection_manager };

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

// Make our handler accept the shared state.
async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let rpc_health = state.connection_manager.get_health_status().await;

    // Convert the health status into a serializable format.
    #[derive(Serialize)]
    struct HealthResponse {
        status: &'static str,
        rpc_endpoints: Vec<RpcHealth>,
    }

    let response = HealthResponse {
        status: "ok",
        rpc_endpoints: rpc_health,
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
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        // Create a ConnectionManager for testing purposes.
        let rpc_urls = vec!["https://api.devnet.solana.com".to_string()];
        let connection_manager = Arc::new(ConnectionManager::new(rpc_urls));

        let test_state = AppState { connection_manager };

        // Create our app with the test state.
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

        // Check for the new response structure.
        assert_eq!(body["status"], "ok");
        assert!(body["rpc_endpoints"].is_array());
        assert_eq!(
            body["rpc_endpoints"][0]["url"],
            "https://api.devnet.solana.com"
        );
    }

    #[tokio::test]
    async fn test_solve_endpoint() {
        let rpc_urls = vec!["https://api.devnet.solana.com".to_string()];
        let connection_manager = Arc::new(ConnectionManager::new(rpc_urls));

        let test_state = AppState { connection_manager };

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
