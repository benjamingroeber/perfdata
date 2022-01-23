use crate::error::PerfdataParseError;
use crate::perfdata::Perfdata;
use crate::thresholds::ThresholdRange;
use crate::Value;
use std::str::FromStr;

// Source: https://nagios-plugins.org/doc/guidelines.html#AEN200
// This is the expected format:
// 'label'=value[UOM];[warn];[crit];[min];[max]
// Notes:
//     space separated list of label/value pairs
//     label can contain any characters except the equals sign or single quote (')
//     the single quotes for the label are optional. Required if spaces are in the label
//     label length is arbitrary, but ideally the first 19 characters are unique (due to a limitation in RRD). Be aware of a limitation in the amount of data that NRPE returns to Nagios
//     to specify a quote character, use two single quotes
//     warn, crit, min or max may be null (for example, if the threshold is not defined or min and max do not apply). Trailing unfilled semicolons can be dropped
//     min and max are not required if UOM=%
//     value, min and max in class [-0-9.]. Must all be the same UOM. value may be a literal "U" instead, this would indicate that the actual value couldn't be determined
//     warn and crit are in the range format (see the Section called Threshold and Ranges). Must be the same UOM
//     UOM (unit of measurement) is a string of zero or more characters, NOT including numbers, semicolons, or quotes. Some examples:
//
//         no unit specified - assume a number (int or float) of things (eg, users, processes, load averages)
//         s - seconds (also us, ms)
//         % - percentage
//         B - bytes (also KB, MB, TB)
//         c - a continous counter (such as bytes transmitted on an interface)

const LABEL_DELIMITER: char = '=';
const DATA_DELIMITER: char = ';';

impl<'a> TryFrom<&'a str> for Perfdata<'a> {
    type Error = PerfdataParseError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        // labels can't contain equals signs, so the first one must delimit our label
        let (label, data) = value
            .split_once(LABEL_DELIMITER)
            .ok_or(PerfdataParseError::MissingEqualsSign)?;

        let mut datapoints = data.split(DATA_DELIMITER);
        let value = next_datapoint(&mut datapoints).ok_or(PerfdataParseError::MissingValue)?;
        let mut perfdata = if value == "U" || value == "u" {
            Perfdata::undetermined(label)
        } else {
            let (parsed_value, unit) = parse_value(value)?;

            match unit {
                "" => Perfdata::unit(label, parsed_value),
                "s" => Perfdata::seconds(label, parsed_value),
                "b" => Perfdata::bytes(label, parsed_value),
                "c" => Perfdata::counter(label, parsed_value),
                "%" => Perfdata::percentage(label, parsed_value),
                // TODO evaluate allowing all units?
                _ => return Err(PerfdataParseError::UnknownUnit(unit.to_string())),
            }
        };

        if let Some(warn) = next_datapoint(&mut datapoints) {
            let parsed_warn = ThresholdRange::from_str(warn)?;
            perfdata = perfdata.with_warn(parsed_warn);
        }

        if let Some(crit) = next_datapoint(&mut datapoints) {
            let parsed_crit = ThresholdRange::from_str(crit)?;
            perfdata = perfdata.with_crit(parsed_crit);
        }

        if let Some(min) = next_datapoint(&mut datapoints) {
            let parsed_min: Value = min.parse()?;
            perfdata = perfdata.with_min(parsed_min);
        }

        if let Some(max) = next_datapoint(&mut datapoints) {
            let parsed_max: Value = max.parse()?;
            perfdata = perfdata.with_max(parsed_max);
        }

        Ok(perfdata)
    }
}

fn parse_value(input: &str) -> Result<(Value, &str), PerfdataParseError> {
    let (value, unit) = input
        .find(|c: char| c != '.' && c != '-' && !c.is_ascii_digit())
        .map(|split_at| input.split_at(split_at))
        .unwrap_or((input, ""));

    let parsed_value = value.parse()?;
    Ok((parsed_value, unit))
}

fn next_datapoint<'a>(mut datapoints: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    match datapoints.next() {
        Some(datapoint) if !datapoint.is_empty() => Some(datapoint),
        _ => None,
    }
}

// This is the generalized format for ranges:
//
// [@]start:end
// Notes:
//
//     start ≤ end
//     start and ":" is not required if start=0
//     if range is of format "start:" and end is not specified, assume end is infinity
//     to specify negative infinity, use "~"
//     alert is raised if metric is outside start and end range(inclusive of endpoints)
//     if range starts with "@", then alert if inside this range(inclusive of endpoints)

const INSIDE_MARKER: char = '@';
const NEG_INF_MARKER: &str = "~";
const RANGE_DELIMITER: char = ':';
const START_DEFAULT: Value = 0.0;
const END_DEFAULT: Value = Value::INFINITY;

impl FromStr for ThresholdRange {
    type Err = PerfdataParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(PerfdataParseError::ThresholdEmpty);
        }

        let mut range = s;
        let inside = s.starts_with(INSIDE_MARKER);
        if inside {
            range = &s[1..];
        }

        let (start, end) = match range.split_once(RANGE_DELIMITER) {
            Some((start, end)) => {
                let parsed_start = parse_range(start, START_DEFAULT)?;
                let parsed_end = parse_range(end, END_DEFAULT)?;
                (parsed_start, parsed_end)
            }
            None => {
                let parsed_end = parse_range(range, END_DEFAULT)?;
                (START_DEFAULT, parsed_end)
            }
        };

        if inside {
            Ok(ThresholdRange::inside(start, end))
        } else {
            Ok(ThresholdRange::outside(start, end))
        }
    }
}

fn parse_range(range: &str, default: Value) -> Result<Value, PerfdataParseError> {
    if range.is_empty() {
        return Ok(default);
    }
    if range == NEG_INF_MARKER {
        return Ok(Value::NEG_INFINITY);
    }

    Ok(range.parse()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perfdata::Unit;
    use crate::thresholds::ThresholdRange;
    use strum::IntoEnumIterator;

    #[test]
    fn test_parse_simple() {
        let perfdata = "label=42";
        let perfdata_long = "label2=10;20;30;0;100;";

        let expected_long = Perfdata::unit("label2", 10)
            .with_warn(ThresholdRange::above_pos(20))
            .with_crit(ThresholdRange::above_pos(30))
            .with_min(0)
            .with_max(100);
        let expected = Perfdata::unit("label", 42);

        let got_long = Perfdata::try_from(perfdata_long).unwrap();
        let got = Perfdata::try_from(perfdata).unwrap();

        assert_eq!(expected, got);
        assert_eq!(expected_long, got_long);
    }

    #[test]
    fn test_parse_errors() {
        let no_equals = "label 123";
        let empty_value = "label=";
        let empty_value_thresholds = "label=;20;30;0;100;";
        let space_unit = "label=1 s";
        let unknown_unit = "label=1x";

        let got_no_equals = Perfdata::try_from(no_equals);
        let got_empty_value = Perfdata::try_from(empty_value);
        let got_empty_value_thresholds = Perfdata::try_from(empty_value_thresholds);
        let got_space_unit = Perfdata::try_from(space_unit);
        let got_unknown_unit = Perfdata::try_from(unknown_unit);

        assert!(matches!(
            got_no_equals,
            Err(PerfdataParseError::MissingEqualsSign)
        ));
        assert!(matches!(
            got_empty_value,
            Err(PerfdataParseError::MissingValue)
        ));
        assert!(matches!(
            got_empty_value_thresholds,
            Err(PerfdataParseError::MissingValue)
        ));
        assert!(matches!(
            got_space_unit,
            Err(PerfdataParseError::UnknownUnit(_))
        ));
        assert!(matches!(
            got_unknown_unit,
            Err(PerfdataParseError::UnknownUnit(_))
        ))
    }

    #[test]
    fn test_parse_omitted() {
        let no_warn = "no_w=10;;30;0;100;";
        let exp_no_warn = Perfdata::unit("no_w", 10)
            .with_crit(ThresholdRange::above_pos(30))
            .with_min(0)
            .with_max(100);

        let no_crit = "no_c=10;20;;0;100";
        let exp_no_crit = Perfdata::unit("no_c", 10)
            .with_warn(ThresholdRange::above_pos(20))
            .with_min(0)
            .with_max(100);

        let no_min = "no_m=10;20;30;;100";
        let exp_no_min = Perfdata::unit("no_m", 10)
            .with_warn(ThresholdRange::above_pos(20))
            .with_crit(ThresholdRange::above_pos(30))
            .with_max(100);

        let just_min = "just_m=10;;;0;";
        let exp_just_min = Perfdata::unit("just_m", 10).with_min(0);

        let got_no_warn = Perfdata::try_from(no_warn).unwrap();
        let got_no_crit = Perfdata::try_from(no_crit).unwrap();
        let got_no_min = Perfdata::try_from(no_min).unwrap();
        let got_just_min = Perfdata::try_from(just_min).unwrap();

        assert_eq!(exp_no_warn, got_no_warn);
        assert_eq!(exp_no_crit, got_no_crit);
        assert_eq!(exp_no_min, got_no_min);
        assert_eq!(exp_just_min, got_just_min);
    }

    #[test]
    fn test_parse_undetermined_thresholds() {
        let perfdata = "test=U;10;20;0;100";
        let expected = Perfdata::undetermined("test")
            .with_warn(ThresholdRange::above_pos(10))
            .with_crit(ThresholdRange::above_pos(20))
            .with_min(0)
            .with_max(100);

        let parsed = Perfdata::try_from(perfdata).unwrap();

        assert_eq!(expected, parsed)
    }

    #[test]
    fn test_parse_units_with_thresholds() {
        let seconds = "seconds=10s;20;30;0;40";
        let expected_seconds = Perfdata::seconds("seconds", 10)
            .with_warn(ThresholdRange::above_pos(20))
            .with_crit(ThresholdRange::above_pos(30))
            .with_min(0)
            .with_max(40);

        let parsed_seconds = Perfdata::try_from(seconds).unwrap();

        assert_eq!(expected_seconds, parsed_seconds);
    }

    #[test]
    fn test_units() {
        let label = "test";
        let value = 0;
        for unit in Unit::iter() {
            match unit {
                Unit::None(_) => {
                    assert_eq!(
                        Perfdata::try_from("test=0").unwrap(),
                        Perfdata::unit(label, value)
                    )
                }
                Unit::Percentage(_) => assert_eq!(
                    Perfdata::try_from("test=0%").unwrap(),
                    Perfdata::percentage(label, value)
                ),
                Unit::Seconds(_) => assert_eq!(
                    Perfdata::try_from("test=0s").unwrap(),
                    Perfdata::seconds(label, value)
                ),
                Unit::Bytes(_) => {
                    assert_eq!(
                        Perfdata::try_from("test=0b").unwrap(),
                        Perfdata::bytes(label, value)
                    )
                }
                Unit::Counter(_) => assert_eq!(
                    Perfdata::try_from("test=0c").unwrap(),
                    Perfdata::counter(label, value)
                ),
                Unit::Undetermined => assert_eq!(
                    Perfdata::try_from("test=U").unwrap(),
                    Perfdata::undetermined(label)
                ),
            };
        }
    }

    #[test]
    fn test_example_ranges() {
        // 10 	< 0 or > 10, (outside the range of {0 .. 10})
        let unit = "10";
        let exp_unit = ThresholdRange::above_pos(10);
        // 10: 	< 10, (outside {10 .. ∞})
        let omit_end = "10:";
        let exp_omit_end = ThresholdRange::outside(10.0, Value::INFINITY);
        // ~:10 	> 10, (outside the range of {-∞ .. 10})
        let neg_inf = "~:10";
        let exp_neg_inf = ThresholdRange::above(10);
        // 10:20 	< 10 or > 20, (outside the range of {10 .. 20})
        let outside = "10:20";
        let exp_outside = ThresholdRange::outside(10, 20);
        // @10:20 	≥ 10 and ≤ 20, (inside the range of {10 .. 20})
        let inside = "@10:20";
        let exp_inside = ThresholdRange::inside(10, 20);

        let number_err = "Nota:range";
        let space_err = "12 34";
        let emtpy_err = "";

        let got_unit = ThresholdRange::from_str(unit).unwrap();
        let got_omit_end = ThresholdRange::from_str(omit_end).unwrap();
        let got_neg_inf = ThresholdRange::from_str(neg_inf).unwrap();
        let got_outside = ThresholdRange::from_str(outside).unwrap();
        let got_inside = ThresholdRange::from_str(inside).unwrap();

        let got_err_number = ThresholdRange::from_str(number_err);
        let got_err_space = ThresholdRange::from_str(space_err);
        let got_err_empty = ThresholdRange::from_str(emtpy_err);

        assert_eq!(exp_unit, got_unit);
        assert_eq!(exp_omit_end, got_omit_end);
        assert_eq!(exp_neg_inf, got_neg_inf);
        assert_eq!(exp_outside, got_outside);
        assert_eq!(exp_inside, got_inside);

        assert!(matches!(
            got_err_number,
            Err(PerfdataParseError::ParseValueError(_))
        ));
        assert!(matches!(
            got_err_space,
            Err(PerfdataParseError::ParseValueError(_))
        ));
        assert!(matches!(
            got_err_empty,
            Err(PerfdataParseError::ThresholdEmpty)
        ));
    }
}
