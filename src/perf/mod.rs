mod data;
mod dataset;
mod parser;

// Reference: https://nagios-plugins.org/doc/guidelines.html#AEN200

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[cfg(test)]
use strum::EnumIter;

pub use data::Perfdata;
pub use dataset::PerfdataSet;
pub type Value = f64;

#[cfg_attr(test, derive(EnumIter))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Unit {
    None(Value),
    Percentage(Value),
    Seconds(Value),
    Bytes(Value),
    Counter(Value),
    Undetermined,
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::None(u) => write!(f, "{}", u),
            Unit::Percentage(u) => write!(f, "{}%", u),
            Unit::Seconds(u) => write!(f, "{}s", u),
            Unit::Bytes(u) => write!(f, "{}b", u),
            Unit::Counter(u) => write!(f, "{}c", u),
            Unit::Undetermined => write!(f, "U"),
        }
    }
}
