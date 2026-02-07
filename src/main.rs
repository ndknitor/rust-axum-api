mod config;
mod controllers;
// mod grpc;
mod http;
mod proto;
mod services;

use services::{create_order_service, OrderServiceFactoryImpl, UserServiceImpl};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Singleton: one instance shared across all requests
    let user_service = Arc::new(UserServiceImpl);

    // Scoped: factory creates new instance per request
    let order_service_factory = Arc::new(OrderServiceFactoryImpl);

    // Transient: function creates new instance every call
    let order_service_transient = create_order_service;

    http::start(user_service, order_service_factory, order_service_transient).await
}
