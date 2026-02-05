pub mod order;
pub mod user;

use actix_web::web;

use crate::http::middlewares::jwt_authorize::JwtAuth;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .wrap(JwtAuth::new())
            .route("", web::get().to(user::get_users))
    );
    cfg.service(
        web::scope("/orders")
            .wrap(JwtAuth::new())
            .route("/{user_id}", web::get().to(order::get_orders))
    );
}