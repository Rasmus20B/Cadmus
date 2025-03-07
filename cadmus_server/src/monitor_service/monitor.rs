
use tonic::{Status, Response, Request};

// Import your generated gRPC service traits
mod monitor_proto {
    tonic::include_proto!("monitor");
}

use monitor_proto::monitor_server::{Monitor, MonitorServer};
use monitor_proto::{ChangeResult, UpdateRequest, UpdateResponse};

// Implement the gRPC service
#[derive(Default)]
struct MyServiceImpl;

#[tonic::async_trait]
impl Monitor for MyServiceImpl {
    async fn report_update(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        let req = request.into_inner();
        println!("Received update: {:?}", req);

        Ok(Response::new(UpdateResponse {
            result: ChangeResult::ChangeSuccess.into(),
            message: String::new(),
        }))
    }
}
