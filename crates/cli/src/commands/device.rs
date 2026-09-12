use clap::{Args, Subcommand};

use crate::client::ClientError;
use crate::commands::CommandContext;
use crate::commands::connection::ConnectionArgs;

#[derive(Debug, Args)]
pub(crate) struct DeviceArgs {
    #[command(subcommand)]
    command: DeviceCommand,
}

#[derive(Debug, Subcommand)]
enum DeviceCommand {
    /// Read device identity through a short-lived gRPC session.
    Info(ConnectionArgs),
}

impl DeviceArgs {
    pub(crate) async fn execute(self, context: &CommandContext) -> Result<(), ClientError> {
        match self.command {
            DeviceCommand::Info(arguments) => info(context, arguments).await,
        }
    }
}

async fn info(context: &CommandContext, arguments: ConnectionArgs) -> Result<(), ClientError> {
    let result = context
        .client
        .connect_and_identify(arguments.connection.as_deref())
        .await?;

    if context.output.is_json() {
        return context.output.print_json(&result);
    }

    println!(
        "{}\tfirmware {}\tUID {}",
        result.connection_string, result.firmware, result.uid
    );
    Ok(())
}
