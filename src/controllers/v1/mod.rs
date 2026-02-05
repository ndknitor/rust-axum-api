pub mod user;

use actix_web::web;

use crate::middlewares::jwt_authorize::JwtAuth;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .wrap(JwtAuth::new()) 
            .route("", web::get().to(user::get_users))
    );
}