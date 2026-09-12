pub mod cli;
mod client;
mod commands;
mod output;
pub mod proto;

pub use client::{ClientConfig, ClientError, ConnectionResult, DeviceSummary, PyrionClient};

use cli::Cli;
use commands::CommandContext;

pub async fn run(cli: Cli) -> Result<(), ClientError> {
    let (config, output_format, command) = cli.into_parts();
    let context = CommandContext::new(PyrionClient::new(config), output_format);
    command.execute(&context).await
}
