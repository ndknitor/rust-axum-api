use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::post};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::middlewares::{AUTH_COOKIE_NAME, Claims};

pub fn router() -> Router {
    Router::new()
        .route("/jwt", post(login_jwt))
        .route("/cookie", post(login_cookie))
        .route("/logout", post(logout))
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

impl LoginRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = Vec::new();

        if self.username.trim().is_empty() {
            errors.push("username is required".to_string());
        }

        if self.password.is_empty() {
            errors.push("password is required".to_string());
        } else if self.password.len() < 6 {
            errors.push("password must be at least 6 characters".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Serialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValidationError {
    pub errors: Vec<String>,
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

#[derive(Debug)]
pub enum LoginError {
    Validation(ValidationError),
    Internal(&'static str),
}

impl IntoResponse for LoginError {
    fn into_response(self) -> axum::response::Response {
        match self {
            LoginError::Validation(e) => e.into_response(),
            LoginError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(MessageResponse { message: msg.to_string() })).into_response()
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/jwt",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful, returns JWT token", body = LoginResponse),
        (status = 400, description = "Validation error", body = ValidationError)
    ),
    tag = "auth"
)]
pub async fn login_jwt(Json(payload): Json<LoginRequest>) -> Result<Json<LoginResponse>, LoginError> {
    payload.validate().map_err(LoginError::Validation)?;

    let claims = Claims::new(payload.username, None, None, None);
    let token = claims.encode().map_err(|_| LoginError::Internal("Failed to generate token"))?;

    Ok(Json(LoginResponse { token }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/cookie",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful, cookie set", body = MessageResponse),
        (status = 400, description = "Validation error", body = ValidationError)
    ),
    tag = "auth"
)]
pub async fn login_cookie(
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<MessageResponse>), LoginError> {
    payload.validate().map_err(LoginError::Validation)?;

    let claims = Claims::new(payload.username, None, None, None);
    let token = claims.encode().map_err(|_| LoginError::Internal("Failed to generate token"))?;

    let cookie = Cookie::build((AUTH_COOKIE_NAME, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(time::Duration::hours(24))
        .build();

    Ok((
        jar.add(cookie),
        Json(MessageResponse {
            message: "Login successful".to_string(),
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses(
        (status = 200, description = "Logout successful", body = MessageResponse)
    ),
    tag = "auth"
)]
pub async fn logout(jar: CookieJar) -> (CookieJar, Json<MessageResponse>) {
    let cookie = Cookie::build(AUTH_COOKIE_NAME)
        .path("/")
        .build();

    (
        jar.remove(cookie),
        Json(MessageResponse {
            message: "Logout successful".to_string(),
        }),
    )
}
