use clap::ValueEnum;
use serde::Serialize;

use crate::client::ClientError;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub(crate) enum OutputFormat {
    #[default]
    Human,
    Json,
}

pub(crate) struct Output {
    format: OutputFormat,
}

impl Output {
    pub(crate) fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    pub(crate) fn is_json(&self) -> bool {
        self.format == OutputFormat::Json
    }

    pub(crate) fn print_json(&self, value: &impl Serialize) -> Result<(), ClientError> {
        let json = serde_json::to_string(value)
            .map_err(|error| ClientError::Protocol(error.to_string()))?;
        println!("{json}");
        Ok(())
    }
}
