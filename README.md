# Super Simple Disk Benchmark

inspired by [simple disk benchmark][1].

This tool has just one single purpose, it measures the writing performance of your hard disk on macOS and Linux. More precisely spoken of the disk under your `CWD`.

It used random data from `/dev/random` and writes first sequentially chunks of 4MB until a total 4GB is written. It measures writing time and throughput.

After that it writes this random data 8 times again on disk and measures the average writing times and throughput for this.

## Quick Start

### Install

To install the ssd-benchmark tool, you just need to run

```bash
cargo install --force ssd-benchmark
```

(--force just makes it update to the latest version if it's already installed)

to verify if the installation went through, you can run `ssd-benchmark` that should output similar to

```sh
$HOME/.cargo/bin/ssd-benchmark
```

### Usage

```sh
$ ssd-benchmark

###             Super Simple Disk Benchmark              ###
## Star me on https://github.com/sassman/ssd-benchmark-rs ##

Filling buffer with 4 MB random data...
Initilisation of buffer done        6 ms

Starting benchmark...

Perform sequential writing of total 4096 MB in 4 MB chunks...
 Total time                         12326 ms
 Throughput                         341.33334 MB/s

Perform 8 cycles of writing of 4096 MB...
 Cycle 1 time                       9905 ms
 Throughput                         413.52853 MB/s
 Cycle 2 time                       11242 ms
 Throughput                         364.348 MB/s
 Cycle 3 time                       12325 ms
 Throughput                         332.33267 MB/s
 Cycle 4 time                       11174 ms
 Throughput                         366.56522 MB/s
 Cycle 5 time                       13384 ms
 Throughput                         306.03708 MB/s
 Cycle 6 time                       11482 ms
 Throughput                         356.73227 MB/s
 Cycle 7 time                       14958 ms
 Throughput                         273.8334 MB/s
 Cycle 8 time                       11216 ms
 Throughput                         365.1926 MB/s

 Total time                         95690 ms
 Average write time                 11961 ms
 Average throughput                 342.4463 MB/s
```

The great thing is, there are no parameters or options.

## Missing something?

If you miss a feature file an issue on [github][2] and don't forget to star the repo.

[1]: http://www.geschke-online.de/sdb/
[2]: https://github.com/sassman/ssd-benchmark-rs/issues