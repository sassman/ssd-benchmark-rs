use core::iter::Sum;
use figlet_rs::FIGfont;
use rand::prelude::*;
use std::fs::remove_file;
use std::fs::File;
use std::io::prelude::*;
use std::io::stdout;
use std::time::{Duration, Instant};

// 8 MB
const BUF_SIZE_MB: usize = 8;
// 1 GB
const TOTAL_SIZE_MB: usize = 1024;
const MAX_CYCLES: usize = 8;

// convienience functions
trait Throughput {
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

macro_rules! println_stats {
    ($label:expr, $value:expr, $unit:expr) => {
        println!("{:<36} {:>10.2} {}", $label, $value, $unit);
    };
}

macro_rules! println_time_ms {
    ($label:expr, $value:expr) => {
        println_stats!($label, $value, "ms");
    };
}

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

macro_rules! shout {
    ($label:expr) => {
        let standard_font = FIGfont::standand().unwrap();
        let figure = standard_font.convert($label);
        assert!(figure.is_some());
        println!("{}", figure.unwrap());
    };
}

fn main() -> std::io::Result<()> {
    let verbose = false;
    const BUF_SIZE: usize = BUF_SIZE_MB * 1024 * 1024;
    let mut buffer = vec![0_u8; BUF_SIZE].into_boxed_slice();

    shout!("SSD - Benchmark");
    println!("Version {}", env!("CARGO_PKG_VERSION"));
    println!("Star me on https://github.com/sassman/ssd-benchmark-rs\n");

    println!("Filling buffer with {} MB random data... ", BUF_SIZE_MB);
    let mut rng = rand::thread_rng();
    let buffer_time = prof! {
        for i in 0..BUF_SIZE {
            buffer[i] = rng.gen();
        };
    };

    println_time_ms!("Buffer filled", buffer_time.as_millis());

    println!("\nStart benchmarking your disk writing performance...");
    println!();
    println!(
        "Perform sequential writing of total {} MB in {} MB chunks",
        TOTAL_SIZE_MB, BUF_SIZE_MB
    );

    let write_time = write_once(buffer.as_ref())?;

    if !verbose {
        println!();
    }
    println!();
    println_time_ms!("Total time", write_time.as_millis());
    println_stats!("Throughput", write_time.throughput(TOTAL_SIZE_MB), "MB/s");
    println!();
    println!(
        "Perform {} write cycles of {} MB",
        MAX_CYCLES, TOTAL_SIZE_MB,
    );

    let mut write_time = Duration::new(0, 0);
    let mut min_w_time = None;
    let mut max_w_time = None;
    let mut write_timings: Vec<f64> = Vec::new();
    for i in 0..MAX_CYCLES {
        let duration = write_once(buffer.as_ref())?;
        write_timings.push(duration.as_millis() as f64);
        if max_w_time.is_none() || duration > max_w_time.unwrap() {
            max_w_time = Some(duration);
        }
        if min_w_time.is_none() || duration < min_w_time.unwrap() {
            min_w_time = Some(duration);
        }
        write_time += duration;
        println!();
        if verbose {
            println_time_ms!(format!("Cycle {} time", i + 1), duration.as_millis());
            println_stats!("Throughput", duration.throughput(TOTAL_SIZE_MB), "MB/s");
        }
    }
    let deviation_time = std_deviation(&write_timings.as_slice()).unwrap_or(0 as f64) as u64;
    let mean_time_ms = mean(&write_timings.as_slice()).unwrap_or(0 as f64) as u64;
    let write_values: Vec<f32> = write_timings
        .as_slice()
        .iter()
        .map(|t| (*t as u64).throughput(TOTAL_SIZE_MB))
        .collect();
    let mean_throughput = std_deviation(write_values.as_slice());

    println!();
    println_time_ms!("Total time", write_time.as_millis());
    println_time_ms!("Min write time", min_w_time.unwrap().as_millis());
    println_time_ms!("Max write time", max_w_time.unwrap().as_millis());
    println_time_ms!(
        "Range write time",
        (max_w_time.unwrap() - min_w_time.unwrap()).as_millis()
    );
    println_time_ms!("Average write time Ø", mean_time_ms);
    println_time_ms!("Standard deviation σ", deviation_time);
    println!();
    println_stats!(
        "Min throughput",
        max_w_time.unwrap().throughput(TOTAL_SIZE_MB),
        "MB/s"
    );
    println_stats!(
        "Max throughput",
        min_w_time.unwrap().throughput(TOTAL_SIZE_MB),
        "MB/s"
    );
    println_stats!(
        "Average throughput Ø",
        mean_time_ms.throughput(TOTAL_SIZE_MB),
        "MB/s"
    );
    println_stats!("Standard deviation σ", mean_throughput.unwrap(), "MB/s");

    Ok(())
}

fn write_once(buffer: &[u8]) -> std::io::Result<Duration> {
    let mut write_time = Duration::new(0, 0);
    {
        let mut file = File::create("test").expect("Can't open test file");

        for _ in 0..TOTAL_SIZE_MB / BUF_SIZE_MB {
            // make sure the data is synced with the disk as the kernel performs
            // write buffering
            //
            // TODO Open the file in O_DSYNC instead to avoid the additional syscall
            write_time += prof!{
                file.write_all(buffer)?;
                file.sync_data()?;
            };
            print!(".");
            stdout().flush()?;
        }
    } // to enforce Drpp on file
    remove_file("test")?;

    Ok(write_time)
}

fn mean<'a, T: 'a>(numbers: &'a [T]) -> Option<f64>
where
    T: Into<f64> + Sum<&'a T>,
{
    let sum = numbers.iter().sum::<T>();
    let length = numbers.len() as f64;

    match length {
        positive if positive > 0_f64 => Some(sum.into() / length),
        _ => None,
    }
}

fn std_deviation<'a, T: 'a>(data: &'a [T]) -> Option<f64>
where
    T: Into<f64> + Sum<&'a T> + Copy,
{
    match (mean(data), data.len()) {
        (Some(data_mean), count) if count > 0 => {
            let count: f64 = count as f64;
            let variance: f64 = data
                .iter()
                .map::<f64, _>(|value| {
                    let value: f64 = (*value).into();
                    let diff: f64 = data_mean - value;

                    diff * diff
                })
                .sum::<f64>()
                / count;

            Some(variance.sqrt())
        }
        _ => None,
    }
}
