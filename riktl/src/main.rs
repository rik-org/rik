use httpclient::ApiError;
mod services;
mod types;
use services::workload_service::WorkloadService;
#[macro_use]
extern crate prettytable;

fn main() -> Result<(), ApiError> {
    WorkloadService::list()?;
    Ok(())
}
