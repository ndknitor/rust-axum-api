pub mod socket;
pub mod v1;
pub mod v2;

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .nest("/socket", socket::router())
        .nest(
            "/api",
            Router::new()
                .nest("/v1", v1::router())
                .nest("/v2", v2::router()),
        )
}
