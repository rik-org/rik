use node_to_sdn::getter_client::GetterClient;
use node_to_sdn::NtsRequest;

pub mod node_to_sdn {
    tonic::include_proto!("nts");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GetterClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(NtsRequest {
        id: "This is a cool id".to_string(),
    });

    let response = client.get_id(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
