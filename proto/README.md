# Proto library

RIK components like `scheduler`, `controller` or `node-agent` are using [gRPC](https://grpc.io/) to communicate with 
each others. The API defined for these is unified in a single library so an update to this library means 
updating every components.

## Protobuf files

In order to define [gRPC services](https://grpc.io/docs/what-is-grpc/core-concepts/#service-definition), we use 
[proto-buffers](https://developers.google.com/protocol-buffers) files. These files are available in [`src`](./src) 
directory.

## Installation

To add the crate to your component, add this line to your `cargo.toml`:
```toml
proto = { git = "https://github.com/dev-sys-do/rik", version="0.1.2" }
```

If you'd like to use a version of this library which is not from this repository, replace `git` parameter.

## Definitions 

Currently, there are two definitions available: [`worker.proto`](./src/worker.proto) and 
[`controller.proto`](./src/controller.proto). File [`common.proto`](./src/common.proto) is used for unified types and
to not repeat ourselves.

Files are compiled into rust language with the crate [prost](https://github.com/tokio-rs/prost) and can be used
thanks to [tokio](https://github.com/hyperium/tonic)

## Example

### Registers a worker

We suppose you are already using `tokio` and `tonic` in your component. 

```rust
use proto::common::worker_status::Status;
use proto::common::{WorkerMetric, WorkerRegistration, WorkerStatus};
use proto::worker::worker_server::Worker as WorkerClient;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Request, Response};

pub struct GRPCService {}

#[tonic::async_trait]
impl WorkerClient for GRPCService {
    type RegisterStream = ReceiverStream<WorkloadChannelType>;

    async fn register(
        &self,
        _request: Request<WorkerRegistration>,
    ) -> Result<Response<Self::RegisterStream>, tonic::Status> {
        let (stream_tx, stream_rx) = channel::<WorkloadChannelType>(1024);
        
        println!("Received a register request, but cannot hold the stream!");

        Ok(Response::new(ReceiverStream::new(stream_rx)))
    }

    async fn send_status_updates(
        &self,
        _request: Request<tonic::Streaming<WorkerStatus>>,
    ) -> Result<Response<()>, tonic::Status> {
        let mut stream = _request.into_inner();

        while let Some(data) = stream.try_next().await? {
            println!("Received some data: {}", data);
        }

        Ok(Response::new(()))
    }
}
```
