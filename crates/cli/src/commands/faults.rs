use clap::{Args, Subcommand};

use crate::client::ClientError;
use crate::commands::CommandContext;
use crate::commands::connection::ConnectionArgs;

#[derive(Debug, Args)]
pub(crate) struct FaultsArgs {
    #[command(subcommand)]
    command: FaultsCommand,
}

#[derive(Debug, Subcommand)]
enum FaultsCommand {
    List(ConnectionArgs),
    ClearResolved(ConnectionArgs),
}

impl FaultsArgs {
    pub(crate) async fn execute(self, context: &CommandContext) -> Result<(), ClientError> {
        match self.command {
            FaultsCommand::List(arguments) => list(context, arguments).await,
            FaultsCommand::ClearResolved(arguments) => clear_resolved(context, arguments).await,
        }
    }
}

async fn list(context: &CommandContext, arguments: ConnectionArgs) -> Result<(), ClientError> {
    let report = context
        .client
        .report_faults(arguments.connection.as_deref())
        .await?;

    if context.output.is_json() {
        return context.output.print_json(&report);
    }

    if report.faults.is_empty() {
        println!("No active or resolved faults.");
    } else {
        for fault in report.faults {
            println!("{}\t{}", fault.fault_type, fault.state);
        }
    }

    Ok(())
}

async fn clear_resolved(
    context: &CommandContext,
    arguments: ConnectionArgs,
) -> Result<(), ClientError> {
    let result = context
        .client
        .clear_resolved_faults(arguments.connection.as_deref())
        .await?;

    if context.output.is_json() {
        return context.output.print_json(&result);
    }

    println!("Cleared resolved faults.");
    Ok(())
}
