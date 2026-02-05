mod controllers;

use crate::config::Config;
use crate::services::{OrderServiceFactory, UserService};
use controllers::order::OrderController;
use controllers::user::UserController;
use std::sync::Arc;
use tonic::transport::Server;

pub mod proto {
    tonic::include_proto!("user");
    tonic::include_proto!("order");
}

/// Start gRPC server
/// - user_service: Singleton (shared Arc across all requests)
/// - order_service_factory: Scoped (factory creates new instance per request)
pub async fn start<U, F>(
    user_service: Arc<U>,
    order_service_factory: Arc<F>,
) -> Result<(), Box<dyn std::error::Error>>
where
    U: UserService + 'static,
    F: OrderServiceFactory + 'static,
{
    let cfg = Config::from_env();
    let addr = format!("{}:{}", cfg.host, cfg.grpc_port).parse()?;

    println!("Starting gRPC server on {}", addr);

    let user_controller = UserController::new(user_service);
    let order_controller = OrderController::new(order_service_factory);

    Server::builder()
        .add_service(proto::user_service_server::UserServiceServer::new(user_controller))
        .add_service(proto::order_service_server::OrderServiceServer::new(order_controller))
        .serve(addr)
        .await?;

    Ok(())
}
