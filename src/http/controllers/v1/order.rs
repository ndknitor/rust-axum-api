use crate::services::{Order, OrderServiceFactory};
use actix_web::{HttpResponse, Responder, web};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct OrderRes {
    pub id: String,
    pub user_id: String,
    pub product: String,
    pub quantity: i32,
}

impl From<Order> for OrderRes {
    fn from(o: Order) -> Self {
        Self {
            id: o.id,
            user_id: o.user_id,
            product: o.product,
            quantity: o.quantity,
        }
    }
}

/// Scoped: factory.create() is called per request, creating a new OrderService instance
pub async fn get_orders(
    factory: web::Data<Arc<dyn OrderServiceFactory>>,
    path: web::Path<String>,
) -> impl Responder {
    let service = factory.create();  // New instance per request (Scoped)
    let user_id = path.into_inner();
    let orders: Vec<OrderRes> = service
        .get_orders(&user_id)
        .await
        .into_iter()
        .map(OrderRes::from)
        .collect();
    HttpResponse::Ok().json(orders)
}
