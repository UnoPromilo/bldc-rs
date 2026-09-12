use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::{Instant, timeout, timeout_at};
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;
use tonic::Streaming;
use tonic::metadata::MetadataValue;

use super::discovery::select_only_device;
use super::{
    ClearResolvedFaultsResult, ClientError, ConnectionResult, FaultReport, FaultSummary,
    PyrionClient, map_open_status, map_rpc_status,
};
use crate::proto::pyrion::v1::controller_message::controller_message::Payload as ControllerPayload;
use crate::proto::pyrion::v1::controller_message::{
    ControllerMessage, IntroduceYourself, ReportFaults, ResetFaults,
};
use crate::proto::pyrion::v1::device_message::device_message::Payload as DevicePayload;
use crate::proto::pyrion::v1::device_message::{
    DeviceMessage, FaultRegister, FaultState, FaultType,
};
use crate::proto::pyrion::v1::session::device_session_client::DeviceSessionClient;

impl PyrionClient {
    pub async fn connect_and_identify(
        &self,
        requested_connection: Option<&str>,
    ) -> Result<ConnectionResult, ClientError> {
        let mut session = self.open_session(requested_connection).await?;
        session
            .send(ControllerPayload::IntroduceYourself(IntroduceYourself {}))
            .await?;
        let deadline = session.response_deadline();

        loop {
            match session.next_payload(deadline).await? {
                DevicePayload::DeviceIntroduction(introduction) => {
                    return Ok(ConnectionResult {
                        connection_string: session.connection_string,
                        firmware: introduction.firmware,
                        uid: introduction.uid,
                    });
                }
                DevicePayload::Telemetry(_)
                | DevicePayload::Success(_)
                | DevicePayload::FaultRegister(_) => {}
                DevicePayload::Failure(_) => unreachable!(),
            }
        }
    }

    pub async fn report_faults(
        &self,
        requested_connection: Option<&str>,
    ) -> Result<FaultReport, ClientError> {
        let mut session = self.open_session(requested_connection).await?;
        session
            .send(ControllerPayload::ReportFaults(ReportFaults {}))
            .await?;
        let deadline = session.response_deadline();

        loop {
            match session.next_payload(deadline).await? {
                DevicePayload::FaultRegister(register) => {
                    return Ok(map_fault_register(register));
                }
                DevicePayload::Telemetry(_)
                | DevicePayload::Success(_)
                | DevicePayload::DeviceIntroduction(_) => {}
                DevicePayload::Failure(_) => unreachable!(),
            }
        }
    }

    pub async fn clear_resolved_faults(
        &self,
        requested_connection: Option<&str>,
    ) -> Result<ClearResolvedFaultsResult, ClientError> {
        let mut session = self.open_session(requested_connection).await?;
        session
            .send(ControllerPayload::ResetFaults(ResetFaults {}))
            .await?;
        let deadline = session.response_deadline();

        loop {
            match session.next_payload(deadline).await? {
                DevicePayload::Success(_) => {
                    return Ok(ClearResolvedFaultsResult { cleared: true });
                }
                DevicePayload::Telemetry(_)
                | DevicePayload::FaultRegister(_)
                | DevicePayload::DeviceIntroduction(_) => {}
                DevicePayload::Failure(_) => unreachable!(),
            }
        }
    }

    async fn open_session(
        &self,
        requested_connection: Option<&str>,
    ) -> Result<OpenSession, ClientError> {
        let connection_string = match requested_connection {
            Some(connection_string) => connection_string.to_owned(),
            None => select_only_device(self.list_devices().await?)?,
        };

        let channel = self.connect_channel().await?;
        let mut client = DeviceSessionClient::new(channel);
        let (request_tx, request_rx) = mpsc::channel(1);
        let mut request = Request::new(ReceiverStream::new(request_rx));
        request.metadata_mut().insert(
            "connection-string",
            MetadataValue::try_from(connection_string.as_str()).map_err(|error| {
                ClientError::DeviceSelection(format!("invalid connection string: {error}"))
            })?,
        );

        let response_stream = timeout(self.config.timeout, client.open(request))
            .await
            .map_err(|_| ClientError::Timeout)?
            .map_err(map_open_status)?
            .into_inner();

        Ok(OpenSession {
            connection_string,
            request_tx,
            response_stream,
            timeout: self.config.timeout,
        })
    }
}

struct OpenSession {
    connection_string: String,
    request_tx: mpsc::Sender<ControllerMessage>,
    response_stream: Streaming<DeviceMessage>,
    timeout: Duration,
}

impl OpenSession {
    async fn send(&self, payload: ControllerPayload) -> Result<(), ClientError> {
        self.request_tx
            .send(ControllerMessage {
                payload: Some(payload),
            })
            .await
            .map_err(|_| ClientError::Protocol("gRPC request stream closed".to_owned()))
    }

    fn response_deadline(&self) -> Instant {
        Instant::now() + self.timeout
    }

    async fn next_payload(&mut self, deadline: Instant) -> Result<DevicePayload, ClientError> {
        let message = timeout_at(deadline, self.response_stream.message())
            .await
            .map_err(|_| ClientError::Timeout)?
            .map_err(map_rpc_status)?
            .ok_or_else(|| ClientError::Protocol("device stream closed".to_owned()))?;

        match message.payload {
            Some(DevicePayload::Failure(_)) => Err(ClientError::DeviceFailure),
            Some(payload) => Ok(payload),
            None => Err(ClientError::Protocol(
                "device response had no payload".to_owned(),
            )),
        }
    }
}

fn map_fault_register(register: FaultRegister) -> FaultReport {
    FaultReport {
        faults: register
            .faults
            .into_iter()
            .map(|fault| FaultSummary {
                fault_type: FaultType::try_from(fault.r#type)
                    .map(|value| value.as_str_name().to_owned())
                    .unwrap_or_else(|_| format!("UNKNOWN_{}", fault.r#type)),
                state: FaultState::try_from(fault.state)
                    .map(|value| value.as_str_name().to_owned())
                    .unwrap_or_else(|_| format!("UNKNOWN_{}", fault.state)),
            })
            .collect(),
    }
}
