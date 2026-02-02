pub mod auth;
pub mod protected;
pub mod root;

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .merge(root::router())
        .nest("/auth", auth::router())
        .nest("/protected", protected::router())
}
