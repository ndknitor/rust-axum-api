pub mod root;

use axum::Router;

pub fn router() -> Router {
    Router::new().merge(root::router())
}
