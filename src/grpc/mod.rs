mod endpoints;

use crate::config::Config;
use crate::proto;
use crate::services::{OrderServiceFactory, UserService};
use endpoints::order::OrderEndpoint;
use endpoints::user::UserEndpoint;
use std::sync::Arc;
use tonic::transport::Server;

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

    println!("Starting gRPC server on grpc://{}", addr);

    let user_endpoint = UserEndpoint::new(user_service);
    let order_endpoint = OrderEndpoint::new(order_service_factory);

    Server::builder()
        .add_service(proto::user_service_server::UserServiceServer::new(
            user_endpoint,
        ))
        .add_service(proto::order_service_server::OrderServiceServer::new(
            order_endpoint,
        ))
        .serve(addr)
        .await?;

    Ok(())
}
