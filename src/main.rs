mod config;
mod controllers;
mod middlewares;
mod routes;
mod services;

use crate::services::UserService;
use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use config::Config;
use middlewares::request_logger::RequestLogger;
use services::UserServiceImpl;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let cfg = Config::from_env();
    println!("Starting server on {}:{}", cfg.host, cfg.http_port);

    HttpServer::new(move || {
        let mut cors = Cors::default()
            .allow_any_header()
            .allow_any_method()
            .supports_credentials();
        for origin in &cfg.cors_origins {
            cors = cors.allowed_origin(origin);
        }
        App::new()
            .app_data(web::Data::<Arc<dyn UserService>>::new(
                Arc::new(UserServiceImpl).clone(),
            ))
            .wrap(RequestLogger)
            .wrap(cors)
            .configure(routes::config)
    })
    .bind((cfg.host.as_str(), cfg.http_port))?
    .run()
    .await
}