use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct ConnectionArgs {
    #[arg(long, env = "PYRION_CONNECTION_STRING", hide_env_values = true)]
    pub(crate) connection: Option<String>,
}
