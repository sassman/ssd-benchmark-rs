use std::fs::{remove_file, File};
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

const ONE_MIN: Duration = Duration::from_secs(60);
const ONE_SEC: Duration = Duration::from_secs(1);

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
            let time = (self.as_secs() / 60) as u128;
            let seconds = self.as_secs() - (time * 60) as u64;
            return format!("{} m {} {:<3}", time, seconds, "s");
        } else if self >= &ONE_SEC {
            return format!("{:>10} {:<4}", self.as_secs_f32().round(), "s");
        }
        let time = self.as_millis();

        format!("{:>10} {:<4}", time, "ms")
    }
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

pub fn write_once(buffer: &[u8]) -> std::io::Result<Duration> {
    let mut write_time = Duration::new(0, 0);
    {
        let mut file = File::create("test").expect("Can't open test file");

        for i in 0..TOTAL_SIZE_MB / BUF_SIZE_MB {
            // make sure the data is synced with the disk as the kernel performs
            // write buffering
            //
            // TODO Open the file in O_DSYNC instead to avoid the additional syscall
            write_time += prof! {
                file.write_all(buffer)?;
                file.sync_data()?;
            };
            if i % 2 == 0 {
                print!(".");
            }
            stdout().flush()?;
        }
    } // to enforce Drop on file
    remove_file("test")?;

    Ok(write_time)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_format_time() {
        assert_eq!(Duration::from_secs(100).as_human_readable(), "1 m 40 s  ");
        assert_eq!(
            Duration::from_millis(1200).as_human_readable(),
            "         1 s   "
        );
        assert_eq!(
            Duration::from_millis(1800).as_human_readable(),
            "         2 s   "
        );
        assert_eq!(
            Duration::from_millis(100).as_human_readable(),
            "       100 ms  "
        );
    }

    #[test]
    fn test_duration_collecting() {
        let d = write_once(&[0xff, 0xff, 0xff]).unwrap();
        assert!(d.as_millis() > 0);
    }

    #[test]
    fn test_throughput_trait() {
        let d = Duration::new(1000, 0);
        let t = d.throughput(100);
        assert!((t - 0.1).abs() <= f32::EPSILON);
    }
}
