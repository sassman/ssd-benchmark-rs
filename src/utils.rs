use std::fmt::{Display, Formatter};
use std::fs::{remove_file, File};
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use crate::timer::Timer;

const ONE_MIN: Duration = Duration::from_secs(60);
const TEN_SEC: Duration = Duration::from_secs(10);

pub const MAX_CYCLES: usize = 8;

pub trait HumanReadable {
    fn as_human_readable(&self) -> String;
}

pub trait MetricWithUnit<T> {
    fn as_unit(&self) -> &'static str;
    fn as_value(&self) -> T;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bytes(pub u64);

#[allow(dead_code)]
impl Bytes {
    pub const fn from_mb(mb: u64) -> Self {
        Bytes(mb * 1024 * 1024)
    }

    pub const fn from_kb(kb: u64) -> Self {
        Bytes(kb * 1024)
    }

    pub const fn from_b(b: u64) -> Self {
        Bytes(b)
    }

    pub const fn as_mb(&self) -> u64 {
        self.0 / 1024 / 1024
    }

    pub const fn as_kb(&self) -> u64 {
        self.0 / 1024
    }

    pub const fn as_byte(&self) -> u64 {
        self.0
    }

    pub fn create_random_buffer(&self) -> Vec<u8> {
        let mut buffer = vec![0; self.0 as usize];
        fastrand::fill(&mut buffer);

        buffer
    }

    pub const fn sequentials(&self) -> u64 {
        let total = Bytes::from_mb(8 * 128).as_byte();
        // 128 / 8 = x / self.as_mb()
        total / self.as_byte()
    }

    pub const fn total_bytes(&self) -> Self {
        Bytes(self.0 * self.sequentials())
    }

    pub fn meassure_sequenqually_writes(
        &self,
        directory: &Option<PathBuf>,
    ) -> std::io::Result<Duration> {
        let buffer = self.create_random_buffer();
        let n = self.sequentials();
        let write_time = write_once(buffer.as_ref(), n, directory)?;

        Ok(write_time)
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

pub fn write_once(buffer: &[u8], n: u64, directory: &Option<PathBuf>) -> std::io::Result<Duration> {
    let mut write_time = Duration::default();
    let test_file_with_uniq_name = format!(".benchmark.{}", fastrand::u32(99999..));
    let path = match directory {
        Some(dir) => dir.join(&test_file_with_uniq_name),
        None => PathBuf::from_str(&test_file_with_uniq_name).unwrap(),
    };

    let one_percent = n / 50;
    {
        let mut file = File::create(&path).expect("Can't open test file");

        for i in 0..n {
            // make sure the data is synced with the disk as the kernel performs
            // write buffering
            //
            // TODO Open the file in O_DSYNC instead to avoid the additional syscall
            let timer = Timer::start();
            file.write_all(buffer)?;
            write_time += timer.stop();

            // every 1% print a dot
            if i % one_percent == 0 {
                print!(".");
                stdout().flush()?;
            }
        }

        let timer = Timer::start();
        file.sync_all()?;
        write_time += timer.stop();
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
        let d = write_once(&[0xff, 0xff, 0xff], 128, &None).unwrap();
        assert!(d.as_millis() > 0);
    }
}
