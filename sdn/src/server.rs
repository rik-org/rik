use tonic::{transport::Server, Request, Response, Status};

use node_to_sdn::getter_server::{Getter, GetterServer};
use node_to_sdn::{NtsReply, NtsRequest};

pub mod node_to_sdn {
    tonic::include_proto!("nts");
}

#[derive(Debug, Default)]
pub struct MyGetterId {}

#[tonic::async_trait]
impl Getter for MyGetterId {
    async fn get_id(&self, request: Request<NtsRequest>) -> Result<Response<NtsReply>, Status> {
        println!("ID: {}", request.into_inner().id);

        let reply = node_to_sdn::NtsReply {
            // Will be modified with a status code according to node team
            message: "Ok".to_string(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGetterId::default();

    Server::builder()
        .add_service(GetterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
