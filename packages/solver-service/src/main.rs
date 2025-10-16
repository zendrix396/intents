use axum::{
    http::StatusCode,
    routing::get,
    Json,
    Router,
};
use serde_json::{json, Value};
use std::net::SocketAddr;

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
}

async fn health_check() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::util::ServiceExt;

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
}
