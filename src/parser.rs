use crate::error::PerfdataParseError;
use crate::perfdata::Perfdata;
use crate::Value;

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
        if let Some((label, data)) = value.split_once(LABEL_DELIMITER) {
            let mut datapoints = data.split(DATA_DELIMITER);
            let value = datapoints.next().ok_or(PerfdataParseError::MissingValue)?;
            let mut perfdata = if value == "U" || value == "u" {
                // todo parse rest anyways
                Perfdata::undetermined(label)
            } else {
                let parsed_value: Value = value.parse()?;
                Perfdata::unit(label, parsed_value)
            };

            if let Some(warn) = datapoints.next() {
                println!("warn: {warn}");
                // parse_threshold
            }
            if let Some(crit) = datapoints.next() {
                println!("crit: {crit}");
                // parse threshold
            }
            if let Some(min) = datapoints.next() {
                let parsed_min: Value = min.parse()?;
                perfdata = perfdata.with_min(parsed_min);
            }
            if let Some(max) = datapoints.next() {
                let parsed_max: Value = max.parse()?;
                perfdata = perfdata.with_max(parsed_max);
            }

            Ok(perfdata)
        } else {
            Err(PerfdataParseError::MissingEqualsSign)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thresholds::ThresholdRange;

    #[test]
    fn test_parse_simple() {
        let perfdata = "label=42";
        let perfdata_long = "label2=10;20;30;0;100;";
        // todo perfdata_unit

        let expected_long = Perfdata::unit("label", 10)
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
    fn test_parse_units() {}
}
