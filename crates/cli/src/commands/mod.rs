mod device;
mod devices;

use clap::Subcommand;

use crate::client::{ClientError, PyrionClient};
use crate::output::{Output, OutputFormat};

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Discover Pyrion devices.
    Devices(devices::DevicesArgs),
    /// Interact with one Pyrion device.
    Device(device::DeviceArgs),
}

pub(crate) struct CommandContext {
    pub(crate) client: PyrionClient,
    pub(crate) output: Output,
}

impl CommandContext {
    pub(crate) fn new(client: PyrionClient, output: OutputFormat) -> Self {
        Self {
            client,
            output: Output::new(output),
        }
    }
}

impl Command {
    pub(crate) async fn execute(self, context: &CommandContext) -> Result<(), ClientError> {
        match self {
            Self::Devices(command) => command.execute(context).await,
            Self::Device(command) => command.execute(context).await,
        }
    }
}
