use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use figlet_rs::FIGfont;
use throughput::Throughput;
use utils::Bytes;

use crate::statistics::{mean, std_deviation};
use crate::utils::{HumanReadable, MetricWithUnit, MAX_CYCLES};

mod fmt;
mod statistics;
mod throughput;
mod timer;
mod utils;

const MAX_BLOCK_SIZE: Bytes = Bytes::from_mb(256);

/// SSD - Benchmark
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, author, name = "ssd-benchmark")]
struct Args {
    /// Directory to meassure, default is the current directory.
    #[arg(short, long)]
    directory: Option<PathBuf>,

    /// Block size in bytes, default is 8m. Note `4k` will be parsed as 4 * 1024 byte.
    #[arg(short, long, default_value = "8m")]
    block_size: String,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

fn parse_block_size(args: &Args) -> Bytes {
    // parse the sector size from the command line, and validate
    let factor = match args.block_size.to_lowercase().chars().last().unwrap() {
        'k' => 1024,
        'm' => 1024 * 1024,
        x if !x.is_ascii_digit() => {
            eprintln!("The provided block size unit is not valid, allowed values are k, m");
            std::process::exit(1);
        }
        _ => 1,
    };

    let block_size = args
        .block_size
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse::<u64>()
        .map(|block_size| Bytes::from_b(block_size * factor))
        .unwrap();

    if block_size > MAX_BLOCK_SIZE {
        eprintln!("The provided block size exeeds the allowed maximum of {MAX_BLOCK_SIZE}");
        std::process::exit(1);
    }

    block_size
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let verbose = args.verbose;

    shout!("SSD - Benchmark");
    println!("Version {}", env!("CARGO_PKG_VERSION"));

    // let's validate the directory if present
    if let Some(dir) = args.directory.as_ref() {
        if !dir.is_dir() {
            eprintln!("The provided directory is not valid");
            std::process::exit(1);
        }
    }

    let buf_size = parse_block_size(&args);
    let n = buf_size.sequentials();

    println!();
    println!("## Sequential Writes");
    println!();
    println!("Performing {n} sequential writes of {buf_size} blocks",);

    let write_time = buf_size.meassure_sequenqually_writes(&args.directory)?;
    let total_bytes = buf_size.total_bytes();

    if !verbose {
        println!();
    }
    println!();
    println_duration!("Total time", write_time);
    let tp = Throughput::new(total_bytes, write_time);
    println_metric!("Write Throughput", tp);
    println_stats!("Write Performance", tp.as_iops(buf_size), "IOPS");
    println!();

    println!("## Cycled Sequential Writes");
    println!();
    println!("Performing {MAX_CYCLES} cycles of {n} sequential writes of {buf_size} blocks");

    let mut write_time = Duration::default();
    let mut min_w_time = Duration::MAX;
    let mut max_w_time = Duration::default();
    let mut write_timings: Vec<Duration> = Vec::with_capacity(MAX_CYCLES);
    for i in 1..=MAX_CYCLES {
        print!("[{i}/{MAX_CYCLES}] ");
        let duration = buf_size.meassure_sequenqually_writes(&args.directory)?;
        write_timings.push(duration);
        if duration > max_w_time {
            max_w_time = duration;
        }
        if duration < min_w_time {
            min_w_time = duration;
        }
        write_time += duration;
        println!();
        if verbose {
            println_duration!("Time", duration);
            println_metric!("Throughput", Throughput::new(total_bytes, duration));
        }
    }
    let write_micros = write_timings
        .iter()
        .map(|d| d.as_micros() as f64)
        .collect::<Vec<_>>();
    let deviation_time =
        Duration::from_micros(std_deviation(write_micros.as_slice()).unwrap_or(0 as f64) as u64);
    let mean_time = Duration::from_micros(mean(write_micros.as_slice()).unwrap_or(0.0) as u64);

    println!();
    println_duration!("Total time", write_time);
    println_duration!("Min write time", min_w_time);
    println_duration!("Max write time", max_w_time);
    println_duration!("Range write time", max_w_time - min_w_time);
    println_duration!("Mean write time Ø", mean_time);
    println_duration!("Standard deviation σ", deviation_time);
    println!();

    let max_tp = Throughput::new(total_bytes, min_w_time);
    let min_tp = Throughput::new(total_bytes, max_w_time);
    let mean_iops: Vec<_> = write_timings
        .iter()
        .map(|d| Throughput::new(total_bytes, *d).as_iops(buf_size) as f64)
        .collect();
    let mean_iops = mean(mean_iops.as_slice()).unwrap() as u64;

    println_metric!("Min write throughput", min_tp);
    println_metric!("Max write throughput", max_tp);
    println_stats!("Max write performance", max_tp.as_iops(buf_size), "IOPS");
    println_stats!("Min write performance", min_tp.as_iops(buf_size), "IOPS");
    println_stats!("Mean write performance", mean_iops, "IOPS");

    println!();
    println!("## Notes");
    println!();
    println!("1 MB = 1024 KB and 1 KB = 1024 B");
    println!("IOPS = Throughput [B/s] / Block Size [B]");

    Ok(())
}
