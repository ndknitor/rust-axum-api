use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;

use super::jwt::Claims;

pub const AUTH_COOKIE_NAME: &str = "auth";

#[derive(Debug)]
pub struct CookieAuth(pub Claims);

#[derive(Debug)]
pub enum CookieAuthError {
    MissingCookie,
    InvalidToken,
}

impl IntoResponse for CookieAuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            CookieAuthError::MissingCookie => (StatusCode::UNAUTHORIZED, "Missing auth cookie"),
            CookieAuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
        };
        (status, message).into_response()
    }
}

impl<S> FromRequestParts<S> for CookieAuth
where
    S: Send + Sync,
{
    type Rejection = CookieAuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| CookieAuthError::MissingCookie)?;

        let token = jar
            .get(AUTH_COOKIE_NAME)
            .map(|c| c.value().to_string())
            .ok_or(CookieAuthError::MissingCookie)?;

        let claims = Claims::decode(&token).map_err(|_| CookieAuthError::InvalidToken)?;

        Ok(CookieAuth(claims))
    }
}
