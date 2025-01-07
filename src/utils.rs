use std::fmt::{Display, Formatter};
use std::fs::{remove_file, File};
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{Duration, Instant};

const ONE_MIN: Duration = Duration::from_secs(60);
const TEN_SEC: Duration = Duration::from_secs(10);

// 8 MB
pub const BUF_SIZE_MB: usize = 8;
// 1 GB
pub const TOTAL_SIZE_MB: usize = 1024;
pub const MAX_CYCLES: usize = 8;

pub trait HumanReadable {
    fn as_human_readable(&self) -> String;
}

impl HumanReadable for Duration {
    fn as_human_readable(&self) -> String {
        if self >= &ONE_MIN {
            // more than one minute -> display minutes and seconds
            let full_minutes = self.as_secs() / 60;
            let full_seconds = self.as_secs() - (full_minutes * 60);
            if full_seconds == 0 {
                format!("{full_minutes} m  ",)
            } else {
                format!("{full_minutes} m {full_seconds} {:<3}", "s")
            }
        } else if self >= &TEN_SEC {
            // more than ten seconds -> display seconds
            let full_seconds = self.as_secs_f32().round();
            format!("{full_seconds:>10} {:<4}", "s")
        } else if self >= &Duration::from_millis(1) {
            // less than ten seconds -> display milliseconds
            let ms = self.as_millis();
            format!("{ms:>10} {:<4}", "ms")
        } else {
            let micros = self.as_micros();
            format!("{micros:>10} {:<4}", "µs")
        }
    }
}

pub struct Bytes(pub u64);

impl Bytes {
    pub fn from_mb(mb: u64) -> Self {
        Bytes(mb * 1024 * 1024)
    }

    pub fn from_kb(kb: u64) -> Self {
        Bytes(kb * 1024)
    }

    pub fn from_b(b: u64) -> Self {
        Bytes(b)
    }
}

impl HumanReadable for Bytes {
    fn as_human_readable(&self) -> String {
        let bytes = self.0;
        if bytes >= 1024 * 1024 {
            format!("{:.0} MB", bytes as f64 / 1024.0 / 1024.0)
        } else if bytes >= 1024 {
            format!("{:.0} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.0} B", bytes as f64)
        }
    }
}

impl Display for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_human_readable())
    }
}

/// Calculate the throughput in IOPS.
/// With the formula:
/// IOPS = MB/s * 10^6 ÷ block_size
/// ```rust
/// use std::time::Duration;
/// use ssd_benchmark_rs::utils::iops;
/// use ssd_benchmark_rs::utils::Bytes;
///
/// let bytes = Bytes::from_mb(1024);
/// let duration = Duration::from_secs(60);
/// let block_size = Bytes::from_kb(4);
///
/// let iops = iops(bytes, duration, block_size);
/// assert_eq!(iops, 256000);
/// ```
pub fn iops(bytes: Bytes, duration: Duration, block_size: Bytes) -> u64 {
    let throughput = bytes.0.throughput(duration.as_secs() as usize);
    (throughput as u64 * 1_000_000) / block_size.0
}

// convenience functions
pub trait Throughput {
    fn throughput(&self, size_in_mb: usize) -> f32;
}

impl Throughput for Duration {
    fn throughput(&self, size_in_mb: usize) -> f32 {
        (size_in_mb as f32 / self.as_millis() as f32) * 1000_f32
    }
}

impl Throughput for u64 {
    fn throughput(&self, size_in_mb: usize) -> f32 {
        (size_in_mb as f32 / *self as f32) * 1000_f32
    }
}

#[macro_export]
macro_rules! println_stats {
    ($label:expr, $value:expr, $unit:expr) => {
        println!("{:<36} {:>10.2} {}", $label, $value, $unit);
    };
}

#[macro_export]
macro_rules! println_time_ms {
    ($label:expr, $value:expr) => {
        println!(
            "{:<36} {}",
            $label,
            Duration::from_millis($value as u64).as_human_readable()
        );
    };
}

#[macro_export]
macro_rules! println_duration {
    ($label:expr, $value:expr) => {
        println!("{:<36} {}", $label, $value.as_human_readable());
    };
}

#[macro_export]
macro_rules! prof {
    ($($something:expr;)+) => {
        {
            let start = Instant::now();
            $(
                $something;
            )*
            start.elapsed()
        }
    };
}

#[macro_export]
macro_rules! shout {
    ($label:expr) => {
        let standard_font = FIGfont::standard().unwrap();
        let figure = standard_font.convert($label);
        assert!(figure.is_some());
        println!("{}", figure.unwrap());
    };
}

pub fn write_once(buffer: &[u8], directory: &Option<PathBuf>) -> std::io::Result<Duration> {
    let mut write_time = Duration::new(0, 0);
    let test_file_with_uniq_name = format!(".benchmark.{}", fastrand::u32(99999..u32::MAX));
    let path = match directory {
        Some(dir) => dir.join(&test_file_with_uniq_name),
        None => PathBuf::from_str(&test_file_with_uniq_name).unwrap(),
    };
    let n = TOTAL_SIZE_MB * 1024 * 1024 / buffer.len();
    {
        let mut file = File::create(&path).expect("Can't open test file");

        for i in 0..n {
            // make sure the data is synced with the disk as the kernel performs
            // write buffering
            //
            // TODO Open the file in O_DSYNC instead to avoid the additional syscall
            write_time += prof! {
                file.write_all(buffer)?;
                file.sync_data()?;
            };
            // every 1% print a dot
            if i % (n / 50) == 0 {
                print!(".");
                stdout().flush()?;
            }
        }
    } // to enforce Drop on file
    remove_file(path)?;

    Ok(write_time)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_format_time() {
        assert_eq!(Duration::from_secs(100).as_human_readable(), "1 m 40 s  ");
        assert_eq!(Duration::from_secs(200).as_human_readable(), "3 m 20 s  ");
        assert_eq!(
            Duration::from_millis(60001).as_human_readable(),
            "1 m  ",
            "should display minutes"
        );
        assert_eq!(
            Duration::from_millis(59999).as_human_readable(),
            "        60 s   ",
            "should display seconds"
        );
        assert_eq!(
            Duration::from_millis(58999).as_human_readable(),
            "        59 s   ",
            "should display seconds"
        );
        assert_eq!(
            Duration::from_secs(10).as_human_readable(),
            "        10 s   ",
            "should display seconds"
        );
        assert_eq!(
            Duration::from_millis(9999).as_human_readable(),
            "      9999 ms  ",
            "should keep the ms"
        );
        assert_eq!(
            Duration::from_millis(100).as_human_readable(),
            "       100 ms  ",
            "should keep the ms"
        );
    }

    #[test]
    fn test_duration_collecting() {
        let d = write_once(&[0xff, 0xff, 0xff], &None).unwrap();
        assert!(d.as_millis() > 0);
    }

    #[test]
    fn test_throughput_trait() {
        let d = Duration::new(1000, 0);
        let t = d.throughput(100);
        assert!((t - 0.1).abs() <= f32::EPSILON);
    }
}
