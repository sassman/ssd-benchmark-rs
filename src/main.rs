use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::Parser;
use figlet_rs::FIGfont;
use utils::{iops, Bytes};

use crate::statistics::{mean, std_deviation};
use crate::utils::{HumanReadable, Throughput, MAX_CYCLES};

mod statistics;
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

    println!("## Preparation");
    println!();
    println!("Filling buffer with {buf_size} random data... ");
    let buffer = buf_size.create_random_buffer();
    let n = buf_size.sequentials();

    println!();
    println!("## Sequential Writes");
    println!();
    println!("Performing {n} sequential writes of {buf_size} blocks",);

    let write_time = buf_size.write_and_measure(&args.directory)?;
    let total_bytes = buf_size.total_bytes();

    if !verbose {
        println!();
    }
    println!();
    println_duration!("Total time", write_time);
    println_stats!("Throughput", write_time.throughput(&total_bytes), "MB/s");
    println_stats!(
        "Performance",
        iops(total_bytes, write_time, buf_size),
        "IOPS"
    );
    println!();
    println!("## Cycled Sequential Writes");
    println!();
    println!("Performing {MAX_CYCLES} cycles of {n} sequential writes of {buf_size} blocks");

    let mut write_time = Duration::default();
    let mut min_w_time = Duration::MAX;
    let mut max_w_time = Duration::default();
    let mut write_timings: Vec<f64> = Vec::with_capacity(MAX_CYCLES);
    for i in 1..=MAX_CYCLES {
        print!("[{i}/{MAX_CYCLES}] ");
        let duration = buf_size.write_and_measure(&args.directory)?;
        write_timings.push(duration.as_millis() as f64);
        if duration > max_w_time {
            max_w_time = duration;
        }
        if duration < min_w_time {
            min_w_time = duration;
        }
        write_time += duration;
        println!();
        if verbose {
            println_duration!(format!("Time"), duration);
            println_stats!("Throughput", duration.throughput(&total_bytes), "MB/s");
        }
    }
    let deviation_time = std_deviation(write_timings.as_slice()).unwrap_or(0 as f64) as u64;
    let mean_time_ms = mean(write_timings.as_slice()).unwrap_or(0 as f64) as u64;
    let write_values: Vec<f32> = write_timings
        .as_slice()
        .iter()
        .map(|t| (*t as u64).throughput(&total_bytes))
        .collect();
    let mean_throughput = std_deviation(write_values.as_slice());

    println!();
    println_duration!("Total time", write_time);
    println_duration!("Min write time", min_w_time);
    println_duration!("Max write time", max_w_time);
    println_duration!("Range write time", max_w_time - min_w_time);
    println_time_ms!("Average write time Ø", mean_time_ms);
    println_time_ms!("Standard deviation σ", deviation_time);
    println!();
    println_stats!(
        "Min throughput",
        max_w_time.throughput(&total_bytes),
        "MB/s"
    );
    println_stats!(
        "Max throughput",
        min_w_time.throughput(&total_bytes),
        "MB/s"
    );
    println_stats!(
        "Average throughput Ø",
        mean_time_ms.throughput(&total_bytes),
        "MB/s"
    );
    println_stats!("Standard deviation σ", mean_throughput.unwrap(), "MB/s");

    Ok(())
}
