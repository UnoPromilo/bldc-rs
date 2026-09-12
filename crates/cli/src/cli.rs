use std::time::Duration;

use clap::Parser;

use crate::client::ClientConfig;
use crate::commands::Command;
use crate::output::OutputFormat;

const DEFAULT_ENDPOINT: &str = "http://[::1]:7985";
const DEFAULT_TIMEOUT_SECONDS: u64 = 5;

#[derive(Debug, Parser)]
#[command(
    name = "pyrionctl",
    version,
    about = "gRPC-only Pyrion device test client",
    subcommand_required = true,
    arg_required_else_help = true
)]
pub struct Cli {
    /// Pyrion gRPC server endpoint.
    #[arg(
        long,
        global = true,
        env = "PYRION_ENDPOINT",
        hide_env_values = true,
        default_value = DEFAULT_ENDPOINT
    )]
    endpoint: String,

    /// Timeout in seconds for each finite gRPC operation.
    #[arg(
        long,
        global = true,
        env = "PYRION_TIMEOUT",
        hide_env_values = true,
        default_value_t = DEFAULT_TIMEOUT_SECONDS,
        value_parser = clap::value_parser!(u64).range(1..)
    )]
    timeout: u64,

    /// Output format.
    #[arg(long, global = true, value_enum, default_value_t)]
    output: OutputFormat,

    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub(crate) fn into_parts(self) -> (ClientConfig, OutputFormat, Command) {
        (
            ClientConfig {
                endpoint: self.endpoint,
                timeout: Duration::from_secs(self.timeout),
            },
            self.output,
            self.command,
        )
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn parses_device_info_with_global_arguments_at_any_depth() {
        let cli = Cli::try_parse_from([
            "pyrionctl",
            "device",
            "info",
            "--connection",
            "serial::/dev/test",
            "--output",
            "json",
            "--timeout",
            "30",
        ])
        .unwrap();

        let (config, output, command) = cli.into_parts();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(output, OutputFormat::Json);
        assert!(matches!(command, Command::Device(_)));
    }

    #[test]
    fn rejects_zero_timeout() {
        let error =
            Cli::try_parse_from(["pyrionctl", "--timeout", "0", "devices", "list"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ValueValidation);
    }

    #[test]
    fn parses_fault_clear_command_with_connection() {
        let cli = Cli::try_parse_from([
            "pyrionctl",
            "faults",
            "clear-resolved",
            "--connection",
            "serial::/dev/test",
        ])
        .unwrap();

        let (_, _, command) = cli.into_parts();
        assert!(matches!(command, Command::Faults(_)));
    }
}
