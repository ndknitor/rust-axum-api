use crate::services::{Order, OrderServiceTransient};
use actix_web::{HttpResponse, Responder, web};
use serde::Serialize;

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

/// Transient: create_fn() is called every time we need a service instance
/// Multiple calls within the same request = multiple instances
pub async fn get_orders(
    create_fn: web::Data<OrderServiceTransient>,
    path: web::Path<String>,
) -> impl Responder {
    let user_id = path.into_inner();

    // Transient: new instance for first operation
    let service1 = create_fn();
    let orders = service1.get_orders(&user_id).await;

    // Transient: could create another instance for different operation
    // let service2 = create_fn();  // Different instance than service1

    let response: Vec<OrderRes> = orders.into_iter().map(OrderRes::from).collect();
    HttpResponse::Ok().json(response)
}
