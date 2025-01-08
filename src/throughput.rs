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
        self.write.as_byte() as f64 / self.duration.as_secs_f64() / 1024.0 / 1024.0
    }

    pub fn as_bps(&self) -> f64 {
        self.write.as_byte() as f64 / self.duration.as_secs_f64()
    }

    fn display(&self) -> (f64, &'static str) {
        let bps = self.as_bps();
        let kbps = bps / 1024.0;
        let mbps = kbps / 1024.0;
        let gbps = mbps / 1024.0;

        if gbps >= 1.0 {
            (gbps, "GB/s")
        } else if mbps >= 1.0 {
            (mbps, "MB/s")
        } else if kbps >= 1.0 {
            (kbps, "KB/s")
        } else {
            (bps, "B/s")
        }
    }

    pub fn as_iops(&self, block_size: Bytes) -> u64 {
        (self.as_bps() / block_size.as_byte() as f64) as u64
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
        if unit == "B/s" {
            // there is no smaller than bytes, hence no need for decimal places
            write!(f, "{value:.0} {unit}", value = value, unit = unit)
        } else {
            write!(f, "{value:.2} {unit}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TP_50MBPS: Throughput = Throughput::new(Bytes::from_mb(100), Duration::from_secs(2));

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
        assert_eq!(TP_50MBPS.as_unit(), "MB/s");
        assert_eq!(TP_50MBPS.as_value(), 50.0);
    }

    #[test]
    fn test_iops() {
        let block_size = Bytes::from_kb(4);

        let iops = TP_50MBPS.as_iops(block_size);
        assert_eq!(iops, 12_800);
    }
}
