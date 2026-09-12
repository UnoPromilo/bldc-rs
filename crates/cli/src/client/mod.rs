mod discovery;
mod error;
mod model;
mod session;

#[cfg(test)]
mod tests;

use tokio::time::timeout;
use tonic::transport::Channel;
use tonic::{Code, Status};

pub use error::ClientError;
pub use model::{
    ClearResolvedFaultsResult, ClientConfig, ConnectionResult, DeviceSummary, FaultReport,
    FaultSummary,
};

#[derive(Debug, Clone)]
pub struct PyrionClient {
    config: ClientConfig,
}

impl PyrionClient {
    pub fn new(config: ClientConfig) -> Self {
        Self { config }
    }

    async fn connect_channel(&self) -> Result<Channel, ClientError> {
        timeout(
            self.config.timeout,
            Channel::from_shared(self.config.endpoint.clone())
                .map_err(|error| ClientError::Endpoint(format!("invalid gRPC endpoint: {error}")))?
                .connect(),
        )
        .await
        .map_err(|_| ClientError::Timeout)?
        .map_err(|error| ClientError::Endpoint(format!("cannot connect to gRPC server: {error}")))
    }
}

fn map_rpc_status(status: Status) -> ClientError {
    match status.code() {
        Code::DeadlineExceeded => ClientError::Timeout,
        Code::Unavailable => ClientError::Endpoint(status.message().to_owned()),
        _ => ClientError::Protocol(format!("gRPC {}: {}", status.code(), status.message())),
    }
}

fn map_open_status(status: Status) -> ClientError {
    match status.code() {
        Code::DeadlineExceeded => ClientError::Timeout,
        Code::NotFound | Code::InvalidArgument => {
            ClientError::DeviceSelection(status.message().to_owned())
        }
        _ => map_rpc_status(status),
    }
}
