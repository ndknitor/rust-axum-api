use crate::services::UserService;
use actix_web::{HttpResponse, Responder, web};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct UserRes {
    pub id: u32,
    pub name: String,
}

pub async fn get_users(service: web::Data<Arc<dyn UserService>>) -> impl Responder {
    let users = service.get_users().await;
    HttpResponse::Ok().json(users)
}
