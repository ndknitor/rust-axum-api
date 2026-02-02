use axum::{Router, routing::get};

pub fn router() -> Router {
    Router::new().route("/", get(index))
}

#[utoipa::path(
    get,
    path = "/api/v1",
    responses(
        (status = 200, description = "Returns greeting message", body = String)
    ),
    tag = "v1"
)]
pub async fn index() -> &'static str {
    "Hello, World!"
}
