use axum::{
    http::StatusCode,
    routing::{get, post},
    Json,
    Router,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use solver_core::{SwapIntent, solve_swap_intent, SwapSolution};

#[tokio::main]
async fn main() {
    println!("Starting Solana Intent Solver Service...");

    let app = app();

    let addr: SocketAddr = "0.0.0.0:3000".parse().expect("valid socket address");
    println!("Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind listener");
    axum::serve(listener, app).await.expect("server exited");
}

fn app() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/solve", post(solve_handler))
}

async fn health_check() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

async fn solve_handler(Json(intent): Json<SwapIntent>) -> (StatusCode, Json<SwapSolution>) {
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
    use tower::util::ServiceExt;
    use serde_json::json;

    #[tokio::test]
    async fn test_health_check() {
        let app = app();

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

        assert_eq!(body, json!({ "status": "ok" }));
    }

    #[tokio::test]
    async fn test_solve_endpoint() {
        let app = app();

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
        assert!(value["transaction_id"].as_str().unwrap().starts_with("mocked-tx-"));
    }
}
