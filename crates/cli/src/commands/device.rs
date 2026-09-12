use clap::{Args, Subcommand};

use crate::client::ClientError;
use crate::commands::CommandContext;

#[derive(Debug, Args)]
pub(crate) struct DeviceArgs {
    #[command(subcommand)]
    command: DeviceCommand,
}

#[derive(Debug, Subcommand)]
enum DeviceCommand {
    Connect(ConnectArgs),
}

#[derive(Debug, Args)]
struct ConnectArgs {
    #[arg(long, env = "PYRION_CONNECTION_STRING", hide_env_values = true)]
    connection: Option<String>,
}

impl DeviceArgs {
    pub(crate) async fn execute(self, context: &CommandContext) -> Result<(), ClientError> {
        match self.command {
            DeviceCommand::Connect(arguments) => connect(context, arguments).await,
        }
    }
}

async fn connect(context: &CommandContext, arguments: ConnectArgs) -> Result<(), ClientError> {
    let result = context
        .client
        .connect_and_identify(arguments.connection.as_deref())
        .await?;

    if context.output.is_json() {
        return context.output.print_json(&result);
    }

    println!(
        "Connected to {}: firmware {}, UID {}",
        result.connection_string, result.firmware, result.uid
    );
    println!("Disconnected.");
    Ok(())
}
