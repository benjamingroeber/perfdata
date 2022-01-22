use crate::Value;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// Reference: https://nagios-plugins.org/doc/guidelines.html#THRESHOLDFORMAT

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ThresholdRange {
    alert_inside: bool,
    start: Value,
    end: Value,
}

impl ThresholdRange {
    fn new(inside: bool, start: Value, end: Value) -> Self {
        // TODO handle end < start
        ThresholdRange {
            alert_inside: inside,
            start,
            end,
        }
    }

    // 10     ||	< 0 or > 10, (outside the range of {0 .. 10})
    pub fn above_pos<T: Into<Value>>(limit_top: T) -> Self {
        Self::outside(0.0, limit_top.into())
    }

    // 10:    ||	< 10, (outside {10 .. ∞})
    pub fn below<T: Into<Value>>(limit_bottom: T) -> Self {
        Self::outside(limit_bottom.into(), f64::INFINITY)
    }

    // ~:10   ||	> 10, (outside the range of {-∞ .. 10})
    pub fn above<T: Into<Value>>(limit_top: T) -> Self {
        Self::outside(f64::NEG_INFINITY, limit_top.into())
    }

    // 10:20  ||	< 10 or > 20, (outside the range of {10 .. 20})
    pub fn outside<T: Into<Value>>(start: T, end: T) -> Self {
        Self::new(false, start.into(), end.into())
    }

    // @10:20 || 	≥ 10 and ≤ 20, (inside the range of {10 .. 20})
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
