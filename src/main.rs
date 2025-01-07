use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::Parser;
use figlet_rs::FIGfont;

use crate::statistics::{mean, std_deviation};
use crate::utils::{write_once, HumanReadable, Throughput, BUF_SIZE_MB, MAX_CYCLES, TOTAL_SIZE_MB};

mod statistics;
mod utils;

/// SSD - Benchmark
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, author, name = "ssd-benchmark")]
struct Args {
    /// Directory to meassure, default is the current directory.
    #[arg(short, long)]
    directory: Option<PathBuf>,
}

fn main() -> std::io::Result<()> {
    let verbose = false;
    const BUF_SIZE: usize = BUF_SIZE_MB * 1024 * 1024;
    let mut buffer = vec![0_u8; BUF_SIZE].into_boxed_slice();
    let args = Args::parse();

    // let's validate the directory if present
    if let Some(dir) = args.directory.as_ref() {
        if !dir.is_dir() {
            eprintln!("The provided directory is not valid");
            std::process::exit(1);
        }
    }

    shout!("SSD - Benchmark");
    println!("Version {}", env!("CARGO_PKG_VERSION"));

    println!("Filling buffer with {} MB random data... ", BUF_SIZE_MB);
    let buffer_time = prof! {
        for i in 0..BUF_SIZE {
            buffer[i] = fastrand::u8(..);
        };
    };

    println_time_ms!("Buffer filled", buffer_time.as_millis());

    println!("\nStart benchmarking your disk writing performance...");
    println!();
    println!(
        "Perform sequential writing of total {} MB in {} MB chunks",
        TOTAL_SIZE_MB, BUF_SIZE_MB
    );

    let write_time = write_once(buffer.as_ref(), &args.directory)?;

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
        let duration = write_once(buffer.as_ref(), &args.directory)?;
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
    let deviation_time = std_deviation(write_timings.as_slice()).unwrap_or(0 as f64) as u64;
    let mean_time_ms = mean(write_timings.as_slice()).unwrap_or(0 as f64) as u64;
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
