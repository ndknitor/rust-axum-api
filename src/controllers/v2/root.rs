use axum::{Json, Router, routing::get};
use crate::middlewares::{CookieAuth, JwtAuth};
use serde::Serialize;
use utoipa::ToSchema;

pub fn router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/protected", get(protected))
        .route("/protected/cookie", get(protected_cookie))
}

#[utoipa::path(
    get,
    path = "/api/v2",
    responses(
        (status = 200, description = "Returns greeting message (v2)", body = String)
    ),
    tag = "v2"
)]
pub async fn index() -> &'static str {
    "Hello, World! (v2)"
}

#[derive(Serialize, ToSchema)]
pub struct ProtectedResponse {
    pub message: String,
    pub user: String,
}

#[utoipa::path(
    get,
    path = "/api/v2/protected",
    responses(
        (status = 200, description = "Protected endpoint (Bearer token)", body = ProtectedResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "v2"
)]
pub async fn protected(JwtAuth(claims): JwtAuth) -> Json<ProtectedResponse> {
    Json(ProtectedResponse {
        message: "You have access to protected content!".to_string(),
        user: claims.sub,
    })
}

#[utoipa::path(
    get,
    path = "/api/v2/protected/cookie",
    responses(
        (status = 200, description = "Protected endpoint (Cookie auth)", body = ProtectedResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("cookie_auth" = [])
    ),
    tag = "v2"
)]
pub async fn protected_cookie(CookieAuth(claims): CookieAuth) -> Json<ProtectedResponse> {
    Json(ProtectedResponse {
        message: "You have access via cookie auth!".to_string(),
        user: claims.sub,
    })
}
