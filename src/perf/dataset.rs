use crate::monitoring_status::MonitoringStatus;
use crate::Perfdata;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, PartialEq)]
pub struct PerfdataSet<'a> {
    data: Vec<Perfdata<'a>>,
}

impl<'a> PerfdataSet<'a> {
    pub fn new() -> Self {
        PerfdataSet::default()
    }
    pub fn add(&mut self, pd: Perfdata<'a>) {
        self.data.push(pd);
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn data(&self) -> impl Iterator<Item = &Perfdata<'a>> {
        self.data.iter()
    }

    pub fn critical(&self) -> impl Iterator<Item = &Perfdata<'a>> {
        self.data().filter(|pd| pd.is_crit())
    }
    pub fn has_critical(&self) -> bool {
        self.critical().count() > 0
    }

    pub fn warning(&self) -> impl Iterator<Item = &Perfdata<'a>> {
        self.data().filter(|pd| pd.is_warn())
    }
    pub fn has_warning(&self) -> bool {
        self.warning().count() > 0
    }

    pub fn is_degraded(&self) -> bool {
        self.data().any(|pd| pd.is_warn() || pd.is_crit())
    }

    pub fn status(&self) -> MonitoringStatus {
        if self.has_critical() {
            MonitoringStatus::Critical
        } else if self.has_warning() {
            MonitoringStatus::Warning
        } else {
            MonitoringStatus::OK
        }
    }
}

impl<'a> From<Vec<Perfdata<'a>>> for PerfdataSet<'a> {
    fn from(data: Vec<Perfdata<'a>>) -> Self {
        Self { data }
    }
}

impl<'a> FromIterator<Perfdata<'a>> for PerfdataSet<'a> {
    fn from_iter<T: IntoIterator<Item = Perfdata<'a>>>(iter: T) -> Self {
        Self {
            data: iter.into_iter().collect(),
        }
    }
}

impl<'a> Display for PerfdataSet<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, pd) in self.data.iter().enumerate() {
            write!(f, "{}", pd)?;
            if i != self.data.len() - 1 {
                write!(f, " ")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ThresholdRange;

    #[test]
    fn display_pdset() {
        //Arrange
        let thc = ThresholdRange::above_pos(23);
        let thw = ThresholdRange::inside(0, 100);
        let pd = Perfdata::bytes("bytes", 42)
            .with_warn(thw)
            .with_crit(thc)
            .with_min(-100)
            .with_max(100);

        let pdo = Perfdata::unit("unit", 50);
        let pdu = Perfdata::undetermined("undetermined");

        let mut pds = PerfdataSet::new();
        pds.add(pd);
        pds.add(pdo);
        pds.add(pdu);

        let empty_pds = PerfdataSet::new();

        // Act
        let result = pds.to_string();
        let expected = "'bytes'=42b;@100;23;-100;100; 'unit'=50; 'undetermined'=U;";

        let empty_result = empty_pds.to_string();

        // Assert
        assert_eq!(&empty_result, "");
        assert_eq!(&result, expected);
    }

    #[test]
    fn test_degraded() {
        let val = 10;
        let pds = vec![
            Perfdata::unit("critical", val).with_crit(ThresholdRange::above_pos(0)),
            Perfdata::unit("warn", val).with_warn(ThresholdRange::above_pos(0)),
            Perfdata::unit("ok", val),
        ];

        let pds_crit: PerfdataSet = pds[..].iter().cloned().collect();
        let pds_warn: PerfdataSet = pds[1..].iter().cloned().collect();
        let pds_ok: PerfdataSet = pds[2..].iter().cloned().collect();

        assert_eq!(pds_crit.data().count(), 3);
        assert_eq!(pds_warn.data().count(), 2);
        assert_eq!(pds_ok.data().count(), 1);

        assert_eq!(pds_crit.critical().count(), 1);
        assert_eq!(pds_warn.warning().count(), 1);
        assert_eq!(pds_ok.critical().count(), 0);
        assert_eq!(pds_ok.warning().count(), 0);

        assert!(pds_crit.is_degraded());
        assert!(pds_warn.is_degraded());
        assert!(!pds_ok.is_degraded());
    }
}
