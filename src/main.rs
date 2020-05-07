use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::time::{Duration, Instant};
use std::{fs::OpenOptions, os::unix::fs::OpenOptionsExt};

const O_DIRECT: i32 = 0o0040000;
// 4 MB
const BUF_SIZE_MB: usize = 4;
// 1 GB
const TOTAL_SIZE_MB: usize = 4096;
const MAX_CYCLES: usize = 8;

fn main() -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(O_DIRECT)
        .open("/dev/random")
        .expect("Can't open");
    // let mut file = File::open("/dev/random")?;
    let mut buffer = Box::new([0; BUF_SIZE_MB * 1024 * 1024]);

    println!("\n###             Super Simple Disk Benchmark              ###");
    println!("## Star me on https://github.com/sassman/ssd-benchmark-rs ##\n");

    println!("Filling buffer with {} MB random data... ", BUF_SIZE_MB);
    let start = Instant::now();
    file.read_exact(buffer.as_mut())?;
    let buffer_time = start.elapsed();

    println!(
        "Initilisation of buffer done        {} ms",
        buffer_time.as_millis()
    );

    println!("\nStarting benchmark...");

    println!(
        "\nPerform sequential writing of total {} MB in {} MB chunks... ",
        TOTAL_SIZE_MB, BUF_SIZE_MB
    );

    let write_time = write_once(buffer.as_ref())?;

    println!(
        " Total time                         {} ms",
        write_time.as_millis(),
    );
    println!(
        " Throughput                         {} MB/s",
        TOTAL_SIZE_MB as f32 / write_time.as_secs() as f32,
    );

    println!(
        "\nPerform {} cycles of writing of {} MB... ",
        MAX_CYCLES, TOTAL_SIZE_MB,
    );

    let mut write_time = Duration::new(0, 0);
    for i in 0..MAX_CYCLES {
        let duration = write_once(buffer.as_ref())?;
        write_time += duration;
        println!(
            " Cycle {} time                       {} ms",
            i + 1,
            duration.as_millis(),
        );
        println!(
            " Throughput                         {} MB/s",
            (TOTAL_SIZE_MB as f32 / duration.as_millis() as f32) * 1000 as f32,
        );
    }
    let cycle_time = write_time / MAX_CYCLES as u32;
    println!(
        "\n Total time                         {} ms",
        write_time.as_millis(),
    );
    println!(
        " Average write time                 {} ms",
        cycle_time.as_millis(),
    );
    println!(
        " Average throughput                 {} MB/s",
        (TOTAL_SIZE_MB as f32 / cycle_time.as_millis() as f32) * 1000 as f32,
    );

    Ok(())
}

fn write_once(buffer: &[u8]) -> std::io::Result<Duration> {
    let start = Instant::now();
    {
        let mut file = OpenOptions::new()
            .write(true)
            .custom_flags(O_DIRECT)
            .create(true)
            .open("test")
            .expect("Can't open test file");

        for _ in 0..TOTAL_SIZE_MB / BUF_SIZE_MB {
            file.write(buffer)?;
        }
    } // to enforce Drpp on file
    let write_time = start.elapsed();

    std::fs::remove_file("test")?;

    Ok(write_time)
}
