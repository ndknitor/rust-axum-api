use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::controllers;

#[derive(OpenApi)]
#[openapi(
    info(title = "Rust Axum API", version = "1.0.0"),
    paths(
        controllers::v1::root::index,
        controllers::v1::auth::login_jwt,
        controllers::v1::auth::login_cookie,
        controllers::v1::auth::logout,
        controllers::v1::protected::protected,
        controllers::v2::root::index,
        controllers::v2::root::protected,
        controllers::v2::root::protected_cookie,
    ),
    components(
        schemas(
            controllers::v1::auth::LoginRequest,
            controllers::v1::auth::LoginResponse,
            controllers::v1::auth::MessageResponse,
            controllers::v1::auth::ValidationError,
            controllers::v1::protected::ProtectedResponse,
            controllers::v2::root::ProtectedResponse,
        )
    ),
    tags(
        (name = "v1", description = "API version 1"),
        (name = "v2", description = "API version 2"),
        (name = "auth", description = "Authentication")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
        components.add_security_scheme(
            "cookie_auth",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("auth_token"))),
        );
    }
}
