use tokio::time::timeout;
use tonic::Request;

use super::{ClientError, DeviceSummary, PyrionClient, map_rpc_status};
use crate::proto::pyrion::v1::discovery::device_discovery_client::DeviceDiscoveryClient;
use crate::proto::pyrion::v1::discovery::{DiscoveredDevice, DiscoveryParams};

impl PyrionClient {
    pub async fn list_devices(&self) -> Result<Vec<DeviceSummary>, ClientError> {
        let channel = self.connect_channel().await?;
        let mut client = DeviceDiscoveryClient::new(channel);
        let response = timeout(
            self.config.timeout,
            client.list_devices(Request::new(DiscoveryParams {})),
        )
        .await
        .map_err(|_| ClientError::Timeout)?
        .map_err(map_rpc_status)?;

        Ok(response
            .into_inner()
            .devices
            .into_iter()
            .map(map_discovered_device)
            .collect())
    }
}

pub(super) fn select_only_device(devices: Vec<DeviceSummary>) -> Result<String, ClientError> {
    match devices.as_slice() {
        [] => Err(ClientError::DeviceSelection(
            "no Pyrion devices were discovered".to_owned(),
        )),
        [device] => Ok(device.connection_string.clone()),
        _ => Err(ClientError::DeviceSelection(
            "multiple devices were discovered; pass --connection".to_owned(),
        )),
    }
}

fn map_discovered_device(device: DiscoveredDevice) -> DeviceSummary {
    let interface = crate::proto::pyrion::v1::interface::Interface::try_from(device.interface)
        .map(|value| value.as_str_name().to_owned())
        .unwrap_or_else(|_| format!("UNKNOWN_{}", device.interface));

    DeviceSummary {
        address: device.address,
        interface,
        firmware: device.firmware,
        name: device.name,
        connection_string: device.connection_string,
    }
}
