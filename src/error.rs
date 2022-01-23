use std::num::ParseFloatError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PerfdataParseError {
    #[error("equals sign (=) must be used to separate the label from data")]
    MissingEqualsSign,
    #[error("numerical value missing after equals sign")]
    MissingValue,
    #[error("value is not a number")]
    ParseValueError(#[from] ParseFloatError),
    #[error("threshold may not be empty for parsing")]
    ThresholdEmpty,
    #[error("unknown unit `{0}`")]
    UnknownUnit(String),
    #[error("the string `{0}` is not valid performance data")]
    ParsingError(String),
}
