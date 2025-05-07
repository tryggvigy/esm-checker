use axum::{
    extract::Json,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use fetch_and_report::fetch_and_analyze_package;
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Deserialize)]
struct CheckRequest {
    package_names: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CheckResponse {
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    // Build our application with a route
    let app = Router::new()
        .route("/", get(serve_frontend))
        .route("/check", post(check_packages))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Run it
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn serve_frontend() -> impl IntoResponse {
    Html(include_str!("../static/index.html"))
}

async fn check_packages(
    Json(payload): Json<CheckRequest>,
) -> Result<Json<CheckResponse>, StatusCode> {
    match fetch_and_analyze_package(&payload.package_names).await {
        Ok(report) => Ok(Json(CheckResponse {
            success: true,
            data: Some(report),
            error: None,
        })),
        Err(e) => Ok(Json(CheckResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        })),
    }
}
