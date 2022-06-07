#![warn(missing_docs)]
//! This library was created to fit basic needs for parsing and creating Performance Data
//! ([Perfdata]) commonly used by check commands from monitoring engines like Icinga2 or Nagios.
//!
//! Usually the data will be turned into a time series, and - if defined - will be used to determine
//! the status of a monitoring object based on [ThresholdRange]s.
//!
//! Parsing and output is implemented to the [Nagios Reference](https://nagios-plugins.org/doc/guidelines.html#AEN200).

mod error;
mod monitoring_status;
mod perf;
mod thresholds;

pub use monitoring_status::MonitoringStatus;
pub use perf::Perfdata;
pub use perf::PerfdataSet;
pub use thresholds::ThresholdRange;

#[test]
fn test_formatting() {
    let result = std::process::Command::new("cargo")
        .args(["fmt", "--all", "--", "--check"])
        .status()
        .unwrap();
    assert_eq!(result.code(), Some(0));
}
