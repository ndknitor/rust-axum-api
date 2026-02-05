use crate::services::UserService;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::grpc::proto::user_service_server::UserService as GrpcUserService;
use crate::grpc::proto::{GetUsersRequest, GetUsersResponse};

pub struct UserController<S: UserService> {
    user_service: Arc<S>,
}

impl<S: UserService> UserController<S> {
    pub fn new(user_service: Arc<S>) -> Self {
        Self { user_service }
    }
}

#[tonic::async_trait]
impl<S: UserService + 'static> GrpcUserService for UserController<S> {
    async fn get_users(
        &self,
        _request: Request<GetUsersRequest>,
    ) -> Result<Response<GetUsersResponse>, Status> {
        let users = self.user_service.get_users().await;
        Ok(Response::new(GetUsersResponse { users }))
    }
}
