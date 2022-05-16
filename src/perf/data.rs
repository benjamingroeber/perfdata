use crate::monitoring_status::MonitoringStatus;
use crate::perf::{Unit, Value};
use crate::thresholds::ThresholdRange;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// `Perfdata` is the core data structure of this crate. A `Perfdata` represents a  named metric
/// recurrently observed by a monitoring check. Almost every current monitoring engine is able to
/// interpret `Perfdata` in its string representation.
///
/// `Perfdata` is usually expressed with a Unit which will be used downstream to select the correct
/// representation when visualizing the data. Additionally a `min` and `max` value can be defined
/// which can be used for delimiting visualizations.
///
/// Critical and Warning [ThresholdRange] instead delimit the ranges where the monitored objects
/// are considered [Ok](`MonitoringStatus::OK`), [Critical](`MonitoringStatus::Critical`) or in
/// [Warning](`MonitoringStatus::Warning`) state.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Perfdata<'a> {
    label: &'a str,
    unit: Unit,
    warn: Option<ThresholdRange>,
    crit: Option<ThresholdRange>,
    min: Option<Value>,
    max: Option<Value>,
}

impl<'a> Perfdata<'a> {
    fn new(label: &'a str, unit: Unit) -> Self {
        Perfdata {
            label,
            unit,
            warn: None,
            crit: None,
            min: None,
            max: None,
        }
    }

    /// Create a Perfdata without a unit. This name may be subject to change in the
    /// near future
    // TODO find a better name, currently it is kind of the opposite of what it is
    pub fn unit<T: Into<Value>>(label: &'a str, value: T) -> Self {
        Self::new(label, Unit::None(value.into()))
    }
    /// Create a new Perfdata with percent (%) Unit
    pub fn percentage<T: Into<Value>>(label: &'a str, value: T) -> Self {
        Self::new(label, Unit::Percentage(value.into()))
    }

    /// Create a new Perfdata with seconds (s) Unit
    pub fn seconds<T: Into<Value>>(label: &'a str, value: T) -> Self {
        Self::new(label, Unit::Seconds(value.into()))
    }

    /// Create a new Perfdata with butes (b) Unit
    pub fn bytes<T: Into<Value>>(label: &'a str, value: T) -> Self {
        Self::new(label, Unit::Bytes(value.into()))
    }

    /// Create a new Perfdata as an increasing counter (c)
    pub fn counter<T: Into<Value>>(label: &'a str, value: T) -> Self {
        Self::new(label, Unit::Counter(value.into()))
    }

    /// Create a new Perfdata where the value could not be determined
    pub fn undetermined(label: &'a str) -> Self {
        Self::new(label, Unit::Undetermined)
    }

    /// Add a minimum value to the [Perfdata]
    #[must_use]
    pub fn with_min<T: Into<Value>>(mut self, value: T) -> Self {
        self.min = Some(value.into());
        self
    }

    /// Add a maximum value to the [Perfdata]
    #[must_use]
    pub fn with_max<T: Into<Value>>(mut self, value: T) -> Self {
        self.max = Some(value.into());
        self
    }

    /// If a warning [ThresholdRange] is defined, and the `value` lies inside the defined
    /// range, the [Perfdata] will be considered in warning (see [is_crit()](`Self::is_crit()`))
    #[must_use]
    pub fn with_warn(mut self, range: ThresholdRange) -> Self {
        self.warn = Some(range);
        self
    }

    /// If a critical [ThresholdRange] is defined, and the `value` lies inside the defined
    /// range, the [Perfdata] will be considered in warning (see [is_warn()][`Self::is_warn()`]).
    #[must_use]
    pub fn with_crit(mut self, range: ThresholdRange) -> Self {
        self.crit = Some(range);
        self
    }

    /// Current `value` is in the `warn` [ThresholdRange]
    pub fn is_warn(&self) -> bool {
        match self.value() {
            Some(value) => self
                .warn
                .map(|range| range.is_alert(value))
                .unwrap_or(false),
            None => false,
        }
    }

    /// Current `value` is in the `crit` [ThresholdRange]
    pub fn is_crit(&self) -> bool {
        match self.value() {
            Some(value) => self
                .crit
                .map(|range| range.is_alert(value))
                .unwrap_or(false),
            None => false,
        }
    }

    /// Mapping the status to a [MonitoringStatus]
    pub fn status(&self) -> MonitoringStatus {
        if self.is_crit() {
            MonitoringStatus::Critical
        } else if self.is_warn() {
            MonitoringStatus::Warning
        } else {
            MonitoringStatus::OK
        }
    }

    fn has_any_thresholds_or_limits(&self) -> bool {
        self.warn.is_some() || self.crit.is_some() || self.min.is_some() || self.max.is_some()
    }

    /// The given numerical `Value` of the [Perfdata]
    pub fn value(&self) -> Option<Value> {
        match self.unit {
            Unit::None(v) => Some(v),
            Unit::Percentage(v) => Some(v),
            Unit::Seconds(v) => Some(v),
            Unit::Bytes(v) => Some(v),
            Unit::Counter(v) => Some(v),
            Unit::Undetermined => None,
        }
    }

    /// The given `Label` of the [Perfdata]
    pub fn label(&self) -> &str {
        self.label
    }
}

fn fmt_threshold<T: Display>(f: &mut Formatter<'_>, th: Option<T>) -> std::fmt::Result {
    match th {
        None => write!(f, ";"),
        Some(threshold) => write!(f, "{};", threshold),
    }
}

impl Display for Perfdata<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'={};", self.label, self.unit)?;

        if self.has_any_thresholds_or_limits() {
            fmt_threshold(f, self.warn)?;
            fmt_threshold(f, self.crit)?;
            fmt_threshold(f, self.min)?;
            fmt_threshold(f, self.max)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_creation() {
        let label = "test";
        for unit in Unit::iter() {
            let perfdata = match unit {
                Unit::None(_) => Perfdata::unit(label, 0),
                Unit::Percentage(_) => Perfdata::percentage(label, 0.0),
                Unit::Seconds(_) => Perfdata::seconds(label, 0_u8),
                Unit::Bytes(_) => Perfdata::bytes(label, 0_u16),
                Unit::Counter(_) => Perfdata::counter(label, 0.0_f32),
                Unit::Undetermined => Perfdata::undetermined(label),
            };

            let label_got = perfdata.label();

            if let Some(value) = perfdata.value() {
                assert_eq!(value, Value::default())
            }
            assert_eq!(label, label_got)
        }
    }

    #[test]
    fn test_format() {
        for unit in Unit::iter() {
            match unit {
                Unit::None(_) => assert_eq!(Perfdata::unit("unit", 0).to_string(), "'unit'=0;"),
                Unit::Percentage(_) => {
                    assert_eq!(
                        Perfdata::percentage("percentage", 50).to_string(),
                        "'percentage'=50%;"
                    )
                }
                Unit::Seconds(_) => {
                    assert_eq!(
                        Perfdata::seconds("seconds", 1.234).to_string(),
                        "'seconds'=1.234s;"
                    )
                }
                Unit::Bytes(_) => assert_eq!(
                    Perfdata::bytes("bytes", 0.0001).to_string(),
                    "'bytes'=0.0001b;"
                ),
                Unit::Counter(_) => assert_eq!(
                    Perfdata::counter("counter", 12345).to_string(),
                    "'counter'=12345c;"
                ),
                Unit::Undetermined => {
                    assert_eq!(
                        Perfdata::undetermined("undetermined").to_string(),
                        "'undetermined'=U;"
                    )
                }
            };
        }
    }

    #[test]
    fn test_format_partial_thresholds() {
        let just_warn = Perfdata::unit("label", 10).with_warn(ThresholdRange::above_pos(20));
        let just_crit = Perfdata::unit("label", 10).with_crit(ThresholdRange::above_pos(30));
        let just_min = Perfdata::unit("label", 10).with_min(0);
        let just_max = Perfdata::unit("label", 10).with_max(100);

        let f_warn = just_warn.to_string();
        let f_crit = just_crit.to_string();
        let f_min = just_min.to_string();
        let f_max = just_max.to_string();

        assert_eq!(f_warn, "'label'=10;20;;;;");
        assert_eq!(f_crit, "'label'=10;;30;;;");
        assert_eq!(f_min, "'label'=10;;;0;;");
        assert_eq!(f_max, "'label'=10;;;;100;");
    }

    #[test]
    fn test_format_thresholds() {
        let warn = ThresholdRange::above_pos(20);
        let crit = ThresholdRange::above_pos(30);
        let min = -50;
        let max = 50;

        for unit in Unit::iter() {
            match unit {
                Unit::None(_) => {
                    let unit = Perfdata::unit("unit", 0)
                        .with_warn(warn)
                        .with_crit(crit)
                        .with_min(min)
                        .with_max(max);
                    assert_eq!(unit.to_string(), "'unit'=0;20;30;-50;50;")
                }
                Unit::Percentage(_) => {
                    let percentage = Perfdata::percentage("percentage", 50)
                        .with_warn(warn)
                        .with_crit(crit)
                        .with_min(min)
                        .with_max(max);
                    assert_eq!(percentage.to_string(), "'percentage'=50%;20;30;-50;50;")
                }
                Unit::Seconds(_) => {
                    let seconds = Perfdata::seconds("seconds", 1.234)
                        .with_warn(warn)
                        .with_crit(crit)
                        .with_min(min)
                        .with_max(max);
                    assert_eq!(seconds.to_string(), "'seconds'=1.234s;20;30;-50;50;")
                }
                Unit::Bytes(_) => {
                    let bytes = Perfdata::bytes("bytes", 0.0001)
                        .with_warn(warn)
                        .with_crit(crit)
                        .with_min(min)
                        .with_max(max);
                    assert_eq!(bytes.to_string(), "'bytes'=0.0001b;20;30;-50;50;")
                }
                Unit::Counter(_) => {
                    let counter = Perfdata::counter("counter", 12345)
                        .with_warn(warn)
                        .with_crit(crit)
                        .with_min(min)
                        .with_max(max);
                    assert_eq!(counter.to_string(), "'counter'=12345c;20;30;-50;50;")
                }
                Unit::Undetermined => {
                    let undetermined = Perfdata::undetermined("undetermined")
                        .with_warn(warn)
                        .with_crit(crit)
                        .with_min(min)
                        .with_max(max);
                    assert_eq!(undetermined.to_string(), "'undetermined'=U;20;30;-50;50;")
                }
            };
        }
    }

    #[test]
    fn test_warn_crit() {
        let warn = Perfdata::unit("warn", 10)
            .with_warn(ThresholdRange::above_pos(5))
            .with_crit(ThresholdRange::above_pos(15));

        let crit = Perfdata::unit("warn", 20)
            .with_warn(ThresholdRange::above_pos(5))
            .with_crit(ThresholdRange::above_pos(15));

        let no_thresholds = Perfdata::unit("no_thresholds", 30);
        let undetermined = Perfdata::undetermined("undetermined")
            .with_warn(ThresholdRange::above_pos(20))
            .with_crit(ThresholdRange::above_pos(20));

        assert!(warn.is_warn());
        assert!(!warn.is_crit());

        assert!(crit.is_warn());
        assert!(crit.is_crit());

        assert!(!no_thresholds.is_warn());
        assert!(!no_thresholds.is_crit());

        assert!(!undetermined.is_warn());
        assert!(!undetermined.is_crit());
    }
}
