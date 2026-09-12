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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FaultReport {
    pub faults: Vec<FaultSummary>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FaultSummary {
    #[serde(rename = "type")]
    pub fault_type: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ClearResolvedFaultsResult {
    pub cleared: bool,
}
