use axum::{
    extract::Json,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use fetch_and_report::fetch_and_analyze_package;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

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
    // Initialize tracing
    FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_level(true)
        .with_ansi(true)
        .pretty()
        .init();

    info!("Starting web server...");

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
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn serve_frontend() -> impl IntoResponse {
    info!("Serving frontend");
    Html(include_str!("../static/index.html"))
}

async fn check_packages(
    Json(payload): Json<CheckRequest>,
) -> Result<Json<CheckResponse>, StatusCode> {
    info!("Checking packages: {:?}", payload.package_names);

    // Get debug directory from environment variable if set
    let debug_dir = std::env::var("DEBUG_DIR").ok().map(PathBuf::from);

    if let Some(ref dir) = debug_dir {
        info!("Using debug directory: {:?}", dir);
    }

    match fetch_and_analyze_package(&payload.package_names, debug_dir).await {
        Ok(report) => {
            info!("Successfully generated report");
            Ok(Json(CheckResponse {
                success: true,
                data: Some(report),
                error: None,
            }))
        }
        Err(e) => {
            info!(error = %e, "Failed to generate report");
            Ok(Json(CheckResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }))
        }
    }
}
