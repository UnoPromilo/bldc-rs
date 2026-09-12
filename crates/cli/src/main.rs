use std::process::ExitCode;

use clap::Parser;
use pyrion_cli::cli::Cli;

#[tokio::main]
async fn main() -> ExitCode {
    match pyrion_cli::run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}
