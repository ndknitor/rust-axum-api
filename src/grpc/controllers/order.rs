use crate::services::OrderServiceFactory;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::grpc::proto::order_service_server::OrderService as GrpcOrderService;
use crate::grpc::proto::{GetOrdersRequest, GetOrdersResponse, Order as ProtoOrder};

/// OrderController holds a factory for Scoped lifetime
pub struct OrderController<F: OrderServiceFactory> {
    order_service_factory: Arc<F>,
}

impl<F: OrderServiceFactory> OrderController<F> {
    pub fn new(order_service_factory: Arc<F>) -> Self {
        Self { order_service_factory }
    }
}

#[tonic::async_trait]
impl<F: OrderServiceFactory + 'static> GrpcOrderService for OrderController<F> {
    async fn get_orders(
        &self,
        request: Request<GetOrdersRequest>,
    ) -> Result<Response<GetOrdersResponse>, Status> {
        // Scoped: create new instance per request
        let service = self.order_service_factory.create();

        let user_id = &request.into_inner().user_id;
        let orders = service.get_orders(user_id).await;

        let proto_orders: Vec<ProtoOrder> = orders
            .into_iter()
            .map(|o| ProtoOrder {
                id: o.id,
                user_id: o.user_id,
                product: o.product,
                quantity: o.quantity,
            })
            .collect();

        Ok(Response::new(GetOrdersResponse { orders: proto_orders }))
    }
}
