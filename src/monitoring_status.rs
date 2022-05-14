use std::fmt::{Display, Formatter};

/// Monitoring Status representing the Status reported to Monitoring Engines like Nagios, Naemon or
/// Icinga.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
// Variant order matters here for PartialOrd derive
pub enum MonitoringStatus {
    /// Definitive result, all values ore in expected range
    OK,
    /// Definitive result, at least one value is outside warning range
    Warning,
    /// Definitive result, at least one value is outside critical range
    Critical,
    /// Uncertain result, may indicate that there is a change in the environment, such that
    /// the check is no longer able to be executed correctly.
    Unknown,
}

impl Display for MonitoringStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let word = match self {
            MonitoringStatus::OK => "OK",
            MonitoringStatus::Warning => "Warning",
            MonitoringStatus::Critical => "Critical",
            MonitoringStatus::Unknown => "Unknown",
        };
        f.write_str(word)
    }
}

impl MonitoringStatus {
    /// Each status maps to an exit code which can be used by monitoring checks
    /// OK -> 0
    /// Warning -> 1
    /// Critical -> 2
    /// Unknown -> 3
    pub fn exit_code(&self) -> i32 {
        match self {
            MonitoringStatus::OK => 0,
            MonitoringStatus::Warning => 1,
            MonitoringStatus::Critical => 2,
            MonitoringStatus::Unknown => 3,
        }
    }
}
