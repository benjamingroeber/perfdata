use crate::perf::Value;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

// Reference: https://nagios-plugins.org/doc/guidelines.html#THRESHOLDFORMAT

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ThresholdRange {
    alert_inside: bool,
    start: Value,
    end: Value,
}

impl ThresholdRange {
    fn new(inside: bool, mut start: Value, mut end: Value) -> Self {
        // We are a little more permissive than the spec here
        // start and end are swapped only if it makes sense
        if end < start {
            std::mem::swap(&mut start, &mut end)
        }

        ThresholdRange {
            alert_inside: inside,
            start,
            end,
        }
    }

    /// This fails fails whenever the value is lower than the given limit
    ///
    /// Parsed as '10'
    /// Corresponds to: '< 0 && > 10' or 'outside the range of {0 .. 10}'
    pub fn above_pos<T: Into<Value>>(limit_top: T) -> Self {
        Self::outside(0.0, limit_top.into())
    }

    /// This fails fails whenever the value is lower than the given limit
    ///
    /// Parsed as '10:'
    /// Corresponds to: '< 10' or 'outside {10 .. ∞}'
    pub fn below<T: Into<Value>>(limit_bottom: T) -> Self {
        Self::outside(limit_bottom.into(), f64::INFINITY)
    }

    /// This fails fails whenever the value is higher than the given limit
    ///
    /// Parsed as '~ 10'
    /// Corresponds to '> 10' or 'outside the range of {-∞ .. 10}'
    pub fn above<T: Into<Value>>(limit_top: T) -> Self {
        Self::outside(f64::NEG_INFINITY, limit_top.into())
    }

    /// This fails fails whenever the value is outside the given limits
    ///
    /// Parsed as '10:20'
    /// Corresponds to `< 10 && > 20` or `outside the range of {10 .. 20}`
    pub fn outside<T: Into<Value>>(start: T, end: T) -> Self {
        Self::new(false, start.into(), end.into())
    }

    /// This fails fails whenever the value is inside the given limits
    ///
    /// Parsed as '@10:20'
    /// Corresponds to `≥ 10 and ≤ 20` or `inside the range of {10 .. 20}`
    pub fn inside<T: Into<Value>>(start: T, end: T) -> Self {
        Self::new(true, start.into(), end.into())
    }

    pub fn is_alert<T: Into<Value>>(&self, value: T) -> bool {
        let value = value.into();
        let is_inside = value >= self.start && value <= self.end;

        if self.alert_inside {
            is_inside
        } else {
            !is_inside
        }
    }
}

impl Display for ThresholdRange {
    // this could be avoided with `(start,end) if start == foo && end == bar` but it's a lot uglier
    #[allow(illegal_floating_point_literal_pattern)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let inside = if self.alert_inside { "@" } else { "" };

        match (self.start, self.end) {
            (Value::NEG_INFINITY, Value::INFINITY) => write!(f, "{}~:", inside),
            (0.0, Value::INFINITY) => write!(f, "{}0:", inside),
            (0.0, end) => write!(f, "{}{}", inside, end),
            (Value::NEG_INFINITY, end) => write!(f, "{}~:{}", inside, end),
            (start, end) => write!(f, "{}{}:{}", inside, start, end),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert() {
        let above_pos_10 = ThresholdRange::above_pos(10);
        let below_10 = ThresholdRange::below(10);
        let above_10 = ThresholdRange::above(10);
        let outside_10_20 = ThresholdRange::outside(10, 20);
        let inside_10_20 = ThresholdRange::inside(10, 20);

        let above_pos_10_ok = above_pos_10.is_alert(5);
        let above_pos_10_above = above_pos_10.is_alert(11);
        let above_pos_10_neg = above_pos_10.is_alert(-1);

        let below_10_ok = below_10.is_alert(11);
        let below_10_below = below_10.is_alert(5);
        let below_10_neg = below_10.is_alert(-1);

        let above_10_ok = above_10.is_alert(5);
        let above_10_neg_ok = above_10.is_alert(-1);
        let above_10_above = above_10.is_alert(11);

        let outside_10_20_between_ok = outside_10_20.is_alert(15);
        let outside_10_20_below = outside_10_20.is_alert(5);
        let outside_10_20_above = outside_10_20.is_alert(25);

        let inside_10_20_below_ok = inside_10_20.is_alert(5);
        let inside_10_20_above_ok = inside_10_20.is_alert(25);
        let inside_10_20_between = inside_10_20.is_alert(15);

        assert!(!above_pos_10_ok);
        assert!(above_pos_10_above);
        assert!(above_pos_10_neg);

        assert!(!below_10_ok);
        assert!(below_10_below);
        assert!(below_10_neg);

        assert!(!above_10_ok);
        assert!(!above_10_neg_ok);
        assert!(above_10_above);

        assert!(!outside_10_20_between_ok);
        assert!(outside_10_20_below);
        assert!(outside_10_20_above);

        assert!(!inside_10_20_below_ok);
        assert!(!inside_10_20_above_ok);
        assert!(inside_10_20_between);
    }

    #[test]
    fn test_alert_boundaries() {
        let outside_10_20 = ThresholdRange::outside(10, 20);
        let inside_10_20 = ThresholdRange::inside(10, 20);

        let outside_10_10_ok = outside_10_20.is_alert(10);
        let outside_10_20_ok = outside_10_20.is_alert(20);

        let inside_10_10 = inside_10_20.is_alert(10);
        let inside_10_20 = inside_10_20.is_alert(20);

        assert!(!outside_10_10_ok);
        assert!(!outside_10_20_ok);

        assert!(inside_10_10);
        assert!(inside_10_20);
    }
}
