use actix_web::HttpResponse;
use serde::Serialize;
use tonic::Response;

#[derive(Serialize, Clone)]
pub struct LoginController {
    pub token: String,
    pub token_type: String,
}

impl LoginController {
    pub fn new(token: String) -> Self {
        Self {
            token,
            token_type: "Bearer".to_string(),
        }
    }

    pub fn to_http(&self) -> HttpResponse {
        HttpResponse::Ok().json(self)
    }

    pub fn to_grpc(self) -> Result<Response<Self>, tonic::Status> {
        Ok(Response::new(self))
    }
}
