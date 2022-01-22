use crate::thresholds::ThresholdRange;
use crate::Value;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
#[cfg(test)]
use strum::EnumIter;

// Reference: https://nagios-plugins.org/doc/guidelines.html#AEN200

#[cfg_attr(test, derive(EnumIter))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Unit {
    None(Value),
    Percentage(Value),
    Seconds(Value),
    Undetermined,
    // Bytes,
    // Continuous Counter
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::None(u) => write!(f, "{}", u),
            Unit::Percentage(u) => write!(f, "{}%", u),
            Unit::Seconds(u) => write!(f, "{}s", u),
            Unit::Undetermined => write!(f, "U"),
        }
    }
}

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

    // TODO find a new name
    pub fn unit<T: Into<Value>>(label: &'a str, value: T) -> Self {
        Self::new(label, Unit::None(value.into()))
    }
    pub fn percentage<T: Into<Value>>(label: &'a str, value: T) -> Self {
        Self::new(label, Unit::Percentage(value.into()))
    }
    pub fn seconds<T: Into<Value>>(label: &'a str, value: T) -> Self {
        Self::new(label, Unit::Seconds(value.into()))
    }
    pub fn undetermined(label: &'a str) -> Self {
        Self::new(label, Unit::Undetermined)
    }

    #[must_use]
    pub fn with_min<T: Into<Value>>(mut self, value: T) -> Self {
        self.min = Some(value.into());
        self
    }
    #[must_use]
    pub fn with_max<T: Into<Value>>(mut self, value: T) -> Self {
        self.max = Some(value.into());
        self
    }
    #[must_use]
    pub fn with_warn(mut self, range: ThresholdRange) -> Self {
        self.warn = Some(range);
        self
    }
    #[must_use]
    pub fn with_crit(mut self, range: ThresholdRange) -> Self {
        self.crit = Some(range);
        self
    }

    pub fn is_warn(&self) -> bool {
        match self.value() {
            Some(value) => self
                .warn
                .map(|range| range.is_alert(value))
                .unwrap_or(false),
            None => false,
        }
    }

    pub fn is_crit(&self) -> bool {
        match self.value() {
            Some(value) => self
                .crit
                .map(|range| range.is_alert(value))
                .unwrap_or(false),
            None => false,
        }
    }

    pub fn value(&self) -> Option<Value> {
        match self.unit {
            Unit::None(v) => Some(v),
            Unit::Percentage(v) => Some(v),
            Unit::Seconds(v) => Some(v),
            Unit::Undetermined => None,
        }
    }
    pub fn label(&self) -> &str {
        self.label
    }
}

impl Display for Perfdata<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'={}", self.label, self.unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_creation() {
        for unit in Unit::iter() {
            match unit {
                Unit::None(_) => Perfdata::unit("unit", 0),
                Unit::Percentage(_) => Perfdata::percentage("percentage", 0.0),
                Unit::Seconds(_) => Perfdata::seconds("seconds", 0u8),
                Unit::Undetermined => Perfdata::undetermined("undetermined"),
            };
        }
    }

    #[test]
    fn test_format() {
        for unit in Unit::iter() {
            match unit {
                Unit::None(_) => assert_eq!(Perfdata::unit("unit", 0).to_string(), "'unit'=0"),
                Unit::Percentage(_) => {
                    assert_eq!(
                        Perfdata::percentage("percentage", 50).to_string(),
                        "'percentage'=50%"
                    )
                }
                Unit::Seconds(_) => {
                    assert_eq!(
                        Perfdata::seconds("seconds", 1.234).to_string(),
                        "'seconds'=1.234s"
                    )
                }
                Unit::Undetermined => {
                    assert_eq!(
                        Perfdata::undetermined("undetermined").to_string(),
                        "'undetermined'=U"
                    )
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

        assert!(warn.is_warn());
        assert!(!warn.is_crit());

        assert!(crit.is_warn());
        assert!(crit.is_crit())
    }
}
