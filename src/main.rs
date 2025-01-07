use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::Parser;
use figlet_rs::FIGfont;
use utils::Bytes;

use crate::statistics::{mean, std_deviation};
use crate::utils::{write_once, HumanReadable, Throughput, BUF_SIZE_MB, MAX_CYCLES, TOTAL_SIZE_MB};

mod statistics;
mod utils;

const MAX_BUF_SIZE_MB: usize = BUF_SIZE_MB;
const MAX_BUF_SIZE: usize = BUF_SIZE_MB * 1024 * 1024;

/// SSD - Benchmark
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, author, name = "ssd-benchmark")]
struct Args {
    /// Directory to meassure, default is the current directory.
    #[arg(short, long)]
    directory: Option<PathBuf>,

    /// Chunk size in bytes, default is 8m. Note `4k` will be parsed as 4 * 1024 byte.
    #[arg(short, long, default_value = "8m")]
    chunk_size: String,
}

fn parse_chunk_size(args: &Args) -> usize {
    // parse the sector size from the command line, and validate
    let factor = match args.chunk_size.to_lowercase().chars().last().unwrap() {
        'k' => 1024,
        'm' => 1024 * 1024,
        x if !x.is_ascii_digit() => {
            eprintln!("The provided chunk size unit is not valid, allowed values are k, m");
            std::process::exit(1);
        }
        _ => 1,
    };

    let chunk_size = args
        .chunk_size
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse::<usize>()
        .unwrap()
        * factor;

    if chunk_size > MAX_BUF_SIZE {
        eprintln!(
            "The provided chunk size exeeds the allowed maximum of {} MB",
            MAX_BUF_SIZE_MB
        );
        std::process::exit(1);
    }

    chunk_size
}

fn main() -> std::io::Result<()> {
    let verbose = false;
    let args = Args::parse();

    shout!("SSD - Benchmark");
    println!("Version {}", env!("CARGO_PKG_VERSION"));

    // let's validate the directory if present
    if let Some(dir) = args.directory.as_ref() {
        if !dir.is_dir() {
            eprintln!("The provided directory is not valid");
            std::process::exit(1);
        }
    }

    let buf_size = parse_chunk_size(&args);
    let mut buffer = Vec::with_capacity(buf_size);
    let buf_size_mb = Bytes::from_b(buf_size as u64);

    println!("Filling buffer with {buf_size_mb} random data... ");
    let buffer_time = prof! {
        for _ in 0..buf_size {
            buffer.push(fastrand::u8(..));
        };
    };

    println_duration!("Buffer filled", buffer_time);

    println!();
    println!("Start benchmarking your disk writing performance...");
    println!();
    println!(
        "Performing sequential writes of total {} at {buf_size_mb} chunks",
        Bytes::from_mb(TOTAL_SIZE_MB as u64).as_human_readable(),
    );

    let write_time = write_once(buffer.as_ref(), &args.directory)?;

    if !verbose {
        println!();
    }
    println!();
    println_duration!("Total time", write_time);
    println_stats!("Throughput", write_time.throughput(TOTAL_SIZE_MB), "MB/s");
    println!();
    println!(
        "Perform {} write cycles of {} MB",
        MAX_CYCLES, TOTAL_SIZE_MB,
    );

    let mut write_time = Duration::new(0, 0);
    let mut min_w_time = None;
    let mut max_w_time = None;
    let mut write_timings: Vec<f64> = Vec::with_capacity(MAX_CYCLES);
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
    println_duration!("Total time", write_time);
    println_duration!("Min write time", min_w_time.unwrap());
    println_duration!("Max write time", max_w_time.unwrap());
    println_duration!(
        "Range write time",
        max_w_time.unwrap() - min_w_time.unwrap()
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
