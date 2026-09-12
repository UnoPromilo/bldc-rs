use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ClientError {
    Endpoint(String),
    DeviceSelection(String),
    Timeout,
    DeviceFailure,
    Protocol(String),
}

impl ClientError {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Endpoint(_) => 3,
            Self::DeviceSelection(_) => 4,
            Self::Timeout => 5,
            Self::DeviceFailure => 6,
            Self::Protocol(_) => 8,
        }
    }
}

impl Display for ClientError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Endpoint(message) | Self::DeviceSelection(message) | Self::Protocol(message) => {
                formatter.write_str(message)
            }
            Self::Timeout => formatter.write_str("timed out waiting for the device"),
            Self::DeviceFailure => formatter.write_str("the device returned a failure response"),
        }
    }
}

impl std::error::Error for ClientError {}
