use std::{fmt::Display, time::Duration};

use crate::utils::{Bytes, MetricWithUnit};

pub struct Throughput {
    pub write: Bytes,
    pub duration: Duration,
}

impl Throughput {
    pub const fn new(write: Bytes, duration: Duration) -> Self {
        Self { write, duration }
    }

    pub fn as_mbps(&self) -> f64 {
        self.write.as_mb() as f64 / self.duration.as_secs_f64()
    }

    fn display(&self) -> (f64, &'static str) {
        if self.write.as_kb() < 1 {
            (
                self.write.as_byte() as f64 / self.duration.as_secs_f64(),
                "B/s",
            )
        } else if self.write.as_mb() < 1 {
            dbg!(self.write.as_kb(), self.duration.as_secs_f64());
            (
                self.write.as_kb() as f64 / self.duration.as_secs_f64(),
                "KB/s",
            )
        } else {
            (self.as_mbps(), "MB/s")
        }
    }
}

impl MetricWithUnit<f64> for Throughput {
    fn as_unit(&self) -> &'static str {
        self.display().1
    }

    fn as_value(&self) -> f64 {
        self.display().0 as f64
    }
}

impl Display for Throughput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (value, unit) = self.display();
        if self.write.as_kb() < 1 {
            write!(f, "{value:.0} {unit}")
        } else {
            write!(f, "{value:.2} {unit}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let throughput = Throughput::new(Bytes::from_mb(100), Duration::from_secs(1));
        assert_eq!(format!("{}", throughput), "100.00 MB/s");

        // small values should be displayed as KB/s
        let throughput = Throughput::new(Bytes::from_b(1024), Duration::from_secs(1));
        assert_eq!(format!("{}", throughput), "1.00 KB/s");

        // very small values should be displayed as B/s
        let throughput = Throughput::new(Bytes::from_b(512), Duration::from_secs(1));
        assert_eq!(format!("{}", throughput), "512 B/s");
    }

    #[test]
    fn test_unit_value() {
        let throughput = Throughput::new(Bytes::from_mb(100), Duration::from_secs(2));
        assert_eq!(throughput.as_unit(), "MB/s");
        assert_eq!(throughput.as_value(), 50.0);
    }
}
