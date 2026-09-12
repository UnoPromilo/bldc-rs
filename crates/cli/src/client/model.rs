use std::time::Duration;

use serde::Serialize;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub endpoint: String,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSummary {
    pub address: String,
    pub interface: String,
    pub firmware: Option<String>,
    pub name: Option<String>,
    pub connection_string: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionResult {
    pub connection_string: String,
    pub firmware: String,
    pub uid: String,
}
