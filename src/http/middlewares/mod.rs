pub mod request_logger;
pub mod jwt_authorize;


use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,

    pub roles: Vec<String>,
    pub policies: Vec<String>,
}