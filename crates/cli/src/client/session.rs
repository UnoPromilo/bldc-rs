use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;
use tonic::metadata::MetadataValue;

use super::discovery::select_only_device;
use super::{ClientError, ConnectionResult, PyrionClient, map_open_status, map_rpc_status};
use crate::proto::pyrion::v1::controller_message::controller_message::Payload as ControllerPayload;
use crate::proto::pyrion::v1::controller_message::{ControllerMessage, IntroduceYourself};
use crate::proto::pyrion::v1::device_message::device_message::Payload as DevicePayload;
use crate::proto::pyrion::v1::session::device_session_client::DeviceSessionClient;

impl PyrionClient {
    pub async fn connect_and_identify(
        &self,
        requested_connection: Option<&str>,
    ) -> Result<ConnectionResult, ClientError> {
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

        let mut response_stream = timeout(self.config.timeout, client.open(request))
            .await
            .map_err(|_| ClientError::Timeout)?
            .map_err(map_open_status)?
            .into_inner();

        request_tx
            .send(ControllerMessage {
                payload: Some(ControllerPayload::IntroduceYourself(IntroduceYourself {})),
            })
            .await
            .map_err(|_| ClientError::Protocol("gRPC request stream closed".to_owned()))?;

        let result = timeout(self.config.timeout, async {
            loop {
                let message = response_stream.message().await.map_err(map_rpc_status)?;
                let message = message
                    .ok_or_else(|| ClientError::Protocol("device stream closed".to_owned()))?;

                match message.payload {
                    Some(DevicePayload::DeviceIntroduction(introduction)) => {
                        return Ok(ConnectionResult {
                            connection_string: connection_string.clone(),
                            firmware: introduction.firmware,
                            uid: introduction.uid,
                        });
                    }
                    Some(DevicePayload::Failure(_)) => return Err(ClientError::DeviceFailure),
                    Some(DevicePayload::Telemetry(_))
                    | Some(DevicePayload::Success(_))
                    | Some(DevicePayload::FaultRegister(_)) => {}
                    None => {
                        return Err(ClientError::Protocol(
                            "device response had no payload".to_owned(),
                        ));
                    }
                }
            }
        })
        .await
        .map_err(|_| ClientError::Timeout)??;

        Ok(result)
    }
}
