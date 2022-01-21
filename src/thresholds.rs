use crate::Value;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
    pub fn above_pos(limit_top: Value) -> Self {
        Self::outside(0.0, limit_top)
    }

    // 10:    ||	< 10, (outside {10 .. ∞})
    pub fn below(limit_bottom: Value) -> Self {
        Self::outside(limit_bottom, f64::INFINITY)
    }

    // ~:10   ||	> 10, (outside the range of {-∞ .. 10})
    pub fn above(limit_top: Value) -> Self {
        Self::outside(f64::NEG_INFINITY, limit_top)
    }

    // 10:20  ||	< 10 or > 20, (outside the range of {10 .. 20})
    pub fn outside(start: Value, end: Value) -> Self {
        Self::new(false, start, end)
    }

    // @10:20 || 	≥ 10 and ≤ 20, (inside the range of {10 .. 20})
    pub fn inside(start: Value, end: Value) -> Self {
        Self::new(true, start, end)
    }

    pub fn is_alert(&self, value: Value) -> bool {
        let is_inside = value > self.start && value < self.end;

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
        let above_pos_10 = ThresholdRange::above_pos(10.0);
        let below_10 = ThresholdRange::below(10.0);
        let above_10 = ThresholdRange::above(10.0);
        let outside_10_20 = ThresholdRange::outside(10.0, 20.0);
        let inside_10_20 = ThresholdRange::inside(10.0, 20.0);

        let above_pos_10_ok = above_pos_10.is_alert(5.0);
        let above_pos_10_above = above_pos_10.is_alert(11.0);
        let above_pos_10_neg = above_pos_10.is_alert(-1.0);

        let below_10_ok = below_10.is_alert(11.0);
        let below_10_below = below_10.is_alert(5.0);
        let below_10_neg = below_10.is_alert(-1.0);

        let above_10_ok = above_10.is_alert(5.0);
        let above_10_neg_ok = above_10.is_alert(-1.0);
        let above_10_above = above_10.is_alert(11.0);

        let outside_10_20_between_ok = outside_10_20.is_alert(15.0);
        let outside_10_20_below = outside_10_20.is_alert(5.0);
        let outside_10_20_above = outside_10_20.is_alert(25.0);

        let inside_10_20_below_ok = inside_10_20.is_alert(5.0);
        let inside_10_20_above_ok = inside_10_20.is_alert(25.0);
        let inside_10_20_between = inside_10_20.is_alert(15.0);

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
}
