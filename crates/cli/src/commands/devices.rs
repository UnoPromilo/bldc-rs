use clap::{Args, Subcommand};

use crate::client::ClientError;
use crate::commands::CommandContext;

#[derive(Debug, Args)]
pub(crate) struct DevicesArgs {
    #[command(subcommand)]
    command: DevicesCommand,
}

#[derive(Debug, Subcommand)]
enum DevicesCommand {
    List,
}

impl DevicesArgs {
    pub(crate) async fn execute(self, context: &CommandContext) -> Result<(), ClientError> {
        match self.command {
            DevicesCommand::List => list(context).await,
        }
    }
}

async fn list(context: &CommandContext) -> Result<(), ClientError> {
    let devices = context.client.list_devices().await?;
    if context.output.is_json() {
        return context.output.print_json(&devices);
    }

    if devices.is_empty() {
        println!("No Pyrion devices discovered.");
    } else {
        for device in devices {
            println!(
                "{}\t{}\t{}",
                device.connection_string,
                device.name.as_deref().unwrap_or("-"),
                device.firmware.as_deref().unwrap_or("-")
            );
        }
    }

    Ok(())
}
