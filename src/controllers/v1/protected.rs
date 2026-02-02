use axum::{Json, Router, routing::get};
use serde::Serialize;
use utoipa::ToSchema;

use crate::middlewares::Authorize;

pub fn router() -> Router {
    Router::new().route("/", get(protected))
}

#[derive(Serialize, ToSchema)]
pub struct ProtectedResponse {
    pub message: String,
    pub user: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/protected",
    responses(
        (status = 200, description = "Protected endpoint (JWT or Cookie)", body = ProtectedResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = []),
        ("cookie_auth" = [])
    ),
    tag = "v1"
)]
pub async fn protected(Authorize(claims): Authorize) -> Json<ProtectedResponse> {
    Json(ProtectedResponse {
        message: "You have access to protected content!".to_string(),
        user: claims.sub,
    })
}
