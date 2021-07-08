use crate::grpc::GRPCService;
use log::error;
use proto::common::WorkerStatus;
use proto::controller::controller_server::Controller as ControllerClient;
use proto::controller::WorkloadScheduling;
use rik_scheduler::Send;
use rik_scheduler::{Event, WorkloadRequest};
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl ControllerClient for GRPCService {
    async fn schedule_instance(
        &self,
        _request: Request<WorkloadScheduling>,
    ) -> Result<Response<()>, Status> {
        let parsed_body = _request.get_ref().clone().unpack().map_err(|e| {
            error!(
                "Failed to parse ScheduleInstance from controller, reason: {}",
                e
            );
            Status::invalid_argument(e.to_string())
        })?;

        self.send(Event::ScheduleRequest(parsed_body)).await?;

        Ok(Response::new(()))
    }

    type GetStatusUpdatesStream = ReceiverStream<Result<WorkerStatus, Status>>;

    async fn get_status_updates(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::GetStatusUpdatesStream>, Status> {
        let (stream_tx, stream_rx) = channel::<Result<WorkerStatus, Status>>(1024);
        let addr = _request
            .remote_addr()
            .unwrap_or_else(|| "0.0.0.0:000".parse().unwrap());
        self.send(Event::Subscribe(stream_tx, addr)).await?;

        Ok(Response::new(ReceiverStream::new(stream_rx)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use definition::workload::{Container, Spec, WorkloadDefinition};
    use proto::common::{WorkerStatus, WorkloadRequestKind};
    use std::net::SocketAddr;
    use tokio::sync::mpsc::error::SendError;
    use tonic::{Code, Request};

    #[tokio::test]
    async fn test_schedule_event() -> Result<(), Status> {
        let (sender, mut receiver) = channel::<Event>(1024);

        let service = GRPCService::new(sender);

        let workload = WorkloadScheduling {
            workload_id: "test".to_string(),
            definition: serde_json::to_string(&WorkloadDefinition {
                api_version: "v0".to_string(),
                kind: "pods".to_string(),
                name: "workload-debian".to_string(),
                replicas: Some(2),
                spec: Spec {
                    containers: vec![Container {
                        name: " debian".to_string(),
                        image: "debian:latest".to_string(),
                        env: None,
                        ports: None,
                    }],
                },
            })
            .map_err(|e| Status::invalid_argument(e.to_string()))?,
            action: WorkloadRequestKind::Create.into(),
        };

        let mock_request = Request::new(workload.clone());

        service.schedule_instance(mock_request).await?;

        let message = receiver.recv().await.unwrap();
        match message {
            Event::ScheduleRequest(content) => assert_eq!(
                workload
                    .unpack()
                    .map_err(|e| { Status::invalid_argument(e.to_string()) })?,
                content
            ),
            _ => assert!(false),
        };
        Ok(())
    }

    #[tokio::test]
    async fn test_status_update_no_remote() {
        let (sender, mut receiver) = channel::<Event>(1024);

        let service = GRPCService::new(sender);

        let mock_request = Request::new(());

        service.get_status_updates(mock_request).await;

        let message = receiver.recv().await.unwrap();
        match message {
            Event::Subscribe(_, socket) => {
                let default_socket: SocketAddr = "0.0.0.0:0".parse().unwrap();
                assert_eq!(default_socket, socket);
            }
            _ => assert!(false),
        };
    }

    #[tokio::test]
    async fn test_status_update_stream() -> Result<(), SendError<Result<WorkerStatus, Status>>> {
        let (sender, mut receiver) = channel::<Event>(1024);

        let service = GRPCService::new(sender);

        let mock_request = Request::new(());

        let mut stream = service
            .get_status_updates(mock_request)
            .await
            .unwrap()
            .into_inner()
            .into_inner();

        let message = receiver.recv().await.unwrap();
        match message {
            Event::Subscribe(sender, socket) => {
                sender.send(Err(Status::cancelled("Sample"))).await?;
                let rcv = stream.recv().await.unwrap();
                assert!(rcv.is_err());
                assert_eq!(rcv.unwrap_err().code(), Code::Cancelled)
            }
            _ => assert!(false),
        };
        Ok(())
    }
}

trait UnPacker<T> {
    fn unpack(self) -> Result<T, serde_json::Error>;
}

impl UnPacker<WorkloadRequest> for WorkloadScheduling {
    fn unpack(self) -> Result<WorkloadRequest, serde_json::Error> {
        WorkloadRequest::new(self)
    }
}
