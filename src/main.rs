mod controllers;
pub mod middlewares;
mod openapi;

use axum::Router;
use std::env;
use tower_http::{trace::TraceLayer}; // services::ServeDir if you need to serve static files
use tracing::Level;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::openapi::ApiDoc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("rust_axum_api=debug,tower_http=debug")
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{host}:{port}");

    let is_production = env::var("ENVIRONMENT").is_err();

    let mut app = Router::new()
        .merge(controllers::router());
        //.fallback_service(ServeDir::new("wwwroot"));

    if !is_production {
        let environment = env::var("ENVIRONMENT").unwrap();
        tracing::info!("Running in {environment} mode - Swagger UI enabled at /swagger-ui");
        app = app.merge(
            SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()),
        );
    } else {
        tracing::info!("Running in production mode - Swagger UI disabled");
    }

    let app = app.layer(
        TraceLayer::new_for_http().make_span_with(|request: &axum::extract::Request| {
            tracing::span!(
                Level::INFO,
                "http_request",
                method = %request.method(),
                uri = %request.uri(),
            )
        }),
    );

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
