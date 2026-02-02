use axum::{
    body::Body,
    extract::FromRequestParts,
    http::{Request, StatusCode, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;

use super::cookie::AUTH_COOKIE_NAME;
use super::jwt::Claims;

/// Authentication extractor that tries Bearer token first, then falls back to cookie.
/// Use this for authentication-only checks (no role/policy requirements).
#[derive(Debug, Clone)]
pub struct Authorize(pub Claims);

#[derive(Debug)]
pub enum AuthorizeError {
    MissingCredentials,
    InvalidToken,
    Forbidden,
}

impl IntoResponse for AuthorizeError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthorizeError::MissingCredentials => {
                (StatusCode::UNAUTHORIZED, "Missing authentication credentials")
            }
            AuthorizeError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthorizeError::Forbidden => (StatusCode::FORBIDDEN, "Insufficient permissions"),
        };
        (status, message).into_response()
    }
}

impl<S> FromRequestParts<S> for Authorize
where
    S: Send + Sync,
{
    type Rejection = AuthorizeError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Try Bearer token first
        if let Some(auth_header) = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
        {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                let claims = Claims::decode(token).map_err(|_| AuthorizeError::InvalidToken)?;
                return Ok(Authorize(claims));
            }
        }

        // Fall back to cookie authentication
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthorizeError::MissingCredentials)?;

        let token = jar
            .get(AUTH_COOKIE_NAME)
            .map(|c| c.value().to_string())
            .ok_or(AuthorizeError::MissingCredentials)?;

        let claims = Claims::decode(&token).map_err(|_| AuthorizeError::InvalidToken)?;

        Ok(Authorize(claims))
    }
}

/// Middleware layer for role and policy-based authorization.
///
/// - If `roles` is empty and `policies` is empty: only checks authentication
/// - If `roles` is provided: user must have at least ONE of the roles (OR logic)
/// - If `policies` is provided: user must have ALL of the policies (AND logic)
///
/// # Example
///
/// ```rust
/// use axum::{Router, routing::get, middleware};
/// use crate::middlewares::authorize_layer;
///
/// let app = Router::new()
///     .route("/admin", get(admin_handler))
///     .layer(middleware::from_fn(|req, next| {
///         authorize_layer(req, next, &["admin"], &[])
///     }));
/// ```
pub async fn authorize_layer(
    request: Request<Body>,
    next: Next,
    required_roles: &[&str],
    required_policies: &[&str],
) -> Result<Response, AuthorizeError> {
    let (mut parts, body) = request.into_parts();

    // Extract claims from request
    let claims = extract_claims(&mut parts).await?;

    // Check roles (OR logic) - user needs at least one matching role
    if !required_roles.is_empty() && !claims.has_any_role(required_roles) {
        return Err(AuthorizeError::Forbidden);
    }

    // Check policies (AND logic) - user needs all matching policies
    if !required_policies.is_empty() && !claims.has_all_policies(required_policies) {
        return Err(AuthorizeError::Forbidden);
    }

    let request = Request::from_parts(parts, body);
    Ok(next.run(request).await)
}

async fn extract_claims(parts: &mut Parts) -> Result<Claims, AuthorizeError> {
    // Try Bearer token first
    if let Some(auth_header) = parts
        .headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
    {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            return Claims::decode(token).map_err(|_| AuthorizeError::InvalidToken);
        }
    }

    // Fall back to cookie authentication
    let cookie_header = parts
        .headers
        .get("Cookie")
        .and_then(|value| value.to_str().ok())
        .ok_or(AuthorizeError::MissingCredentials)?;

    let token = cookie_header
        .split(';')
        .map(|s| s.trim())
        .find_map(|cookie| {
            cookie
                .strip_prefix(AUTH_COOKIE_NAME)
                .and_then(|s| s.strip_prefix('='))
        })
        .ok_or(AuthorizeError::MissingCredentials)?;

    Claims::decode(token).map_err(|_| AuthorizeError::InvalidToken)
}

/// Helper macro to create authorization middleware with roles
#[macro_export]
macro_rules! require_roles {
    ($($role:expr),* $(,)?) => {
        axum::middleware::from_fn(|req, next| async move {
            $crate::middlewares::authorize_layer(req, next, &[$($role),*], &[]).await
        })
    };
}

/// Helper macro to create authorization middleware with policies
#[macro_export]
macro_rules! require_policies {
    ($($policy:expr),* $(,)?) => {
        axum::middleware::from_fn(|req, next| async move {
            $crate::middlewares::authorize_layer(req, next, &[], &[$($policy),*]).await
        })
    };
}

/// Helper macro to create authorization middleware with both roles and policies
#[macro_export]
macro_rules! require_auth {
    (roles: [$($role:expr),* $(,)?], policies: [$($policy:expr),* $(,)?]) => {
        axum::middleware::from_fn(|req, next| async move {
            $crate::middlewares::authorize_layer(req, next, &[$($role),*], &[$($policy),*]).await
        })
    };
    (roles: [$($role:expr),* $(,)?]) => {
        $crate::require_roles!($($role),*)
    };
    (policies: [$($policy:expr),* $(,)?]) => {
        $crate::require_policies!($($policy),*)
    };
}
