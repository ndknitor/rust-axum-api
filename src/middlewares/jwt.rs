use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub policies: Vec<String>,
}

impl Claims {
    pub fn new(
        sub: String,
        duration_hours: Option<i64>,
        roles: Option<Vec<String>>,
        policies: Option<Vec<String>>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        let ttl_hours = duration_hours.unwrap_or_else(get_jwt_ttl);
        Self {
            sub,
            iat: now,
            exp: now + (ttl_hours * 3600),
            roles: roles.unwrap_or_default(),
            policies: policies.unwrap_or_default(),
        }
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    pub fn with_policies(mut self, policies: Vec<String>) -> Self {
        self.policies = policies;
        self
    }

    /// Check if user has at least one of the required roles (OR logic)
    pub fn has_any_role(&self, required_roles: &[&str]) -> bool {
        if required_roles.is_empty() {
            return true;
        }
        required_roles.iter().any(|r| self.roles.contains(&r.to_string()))
    }

    /// Check if user has all required policies (AND logic)
    pub fn has_all_policies(&self, required_policies: &[&str]) -> bool {
        if required_policies.is_empty() {
            return true;
        }
        required_policies.iter().all(|p| self.policies.contains(&p.to_string()))
    }

    pub fn encode(&self) -> Result<String, jsonwebtoken::errors::Error> {
        let secret = get_jwt_secret();
        encode(&Header::default(), self, &EncodingKey::from_secret(secret.as_bytes()))
    }

    pub fn decode(token: &str) -> Result<Self, jsonwebtoken::errors::Error> {
        let secret = get_jwt_secret();
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }
}

fn get_jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret-change-in-production".to_string())
}

fn get_jwt_ttl() -> i64 {
    env::var("JWT_TTL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(24) // Default: 24 hours
}

#[derive(Debug)]
pub struct JwtAuth(pub Claims);

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
        };
        (status, message).into_response()
    }
}

impl<S> FromRequestParts<S> for JwtAuth
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or(AuthError::MissingToken)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::InvalidToken)?;

        let claims = Claims::decode(token).map_err(|_| AuthError::InvalidToken)?;

        Ok(JwtAuth(claims))
    }
}
