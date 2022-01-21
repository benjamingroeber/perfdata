use crate::thresholds::ThresholdRange;
use crate::Value;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
#[cfg(test)]
use strum::EnumIter;

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
    thresholds: Option<Thresholds>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Thresholds {
    warn: Option<ThresholdRange>,
    crit: Option<ThresholdRange>,
    min: Option<Value>,
    max: Option<Value>,
}

impl<'a> Perfdata<'a> {
    // TODO find a new name
    pub fn unit<T: Into<Value>>(label: &'a str, value: T) -> Self {
        Perfdata {
            label,
            unit: Unit::None(value.into()),
            thresholds: None,
        }
    }
    pub fn percentage<T: Into<Value>>(label: &'a str, value: T) -> Self {
        Perfdata {
            label,
            unit: Unit::Percentage(value.into()),
            thresholds: None,
        }
    }
    pub fn seconds<T: Into<Value>>(label: &'a str, value: T) -> Self {
        Perfdata {
            label,
            unit: Unit::Seconds(value.into()),
            thresholds: None,
        }
    }
    pub fn undetermined(label: &'a str) -> Self {
        Perfdata {
            label,
            unit: Unit::Undetermined,
            thresholds: None,
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
}
