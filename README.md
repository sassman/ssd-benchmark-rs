<div align="center">
 <img src="https://github.com/sassman/ssd-benchmark-rs/blob/main/docs/demo.png?raw=true" width="950">
 <h1><strong>SSD Benchmark</strong></h1>

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Crates Version](https://img.shields.io/crates/v/ssd-benchmark.svg)](https://crates.io/crates/ssd-benchmark)
[![Build Status](https://github.com/sassman/ssd-benchmark-rs/workflows/Build/badge.svg)](https://github.com/sassman/ssd-benchmark-rs/actions?query=branch%3Amain+workflow%3ABuild+)
[![dependency status](https://deps.rs/repo/github/sassman/ssd-benchmark-rs/status.svg)](https://deps.rs/repo/github/sassman/ssd-benchmark-rs)

</div>

> A super simple disk benchmark tool

inspired by [simple disk benchmark][1].

This tool has just one single purpose, it measures the writing performance of your hard disk on macOS and Linux. More precisely spoken of the disk under your `CWD`.

It used random data and writes first sequentially chunks of 8MB until a total 1GB is written. It measures writing time and throughput.

After that, it writes these random data 8 times again on disk and measures the average writing times and throughput for this.

## Demo

![demo](./docs/demo.gif)

## Quick Start

### Using the docker image

```sh
docker run --rm ghcr.io/sassman/ssd-benchmark-rs
```

### Install on linux

[![Get it from the Snap Store](https://snapcraft.io/static/images/badges/en/snap-store-black.svg)](https://snapcraft.io/ssd-benchmark)

```sh
sudo snap install ssd-benchmark
```

### Install with cargo

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
____    ____    ____                ____                          _                                  _
/ ___|  / ___|  |  _ \              | __ )    ___   _ __     ___  | |__    _ __ ___     __ _   _ __  | | __
\___ \  \___ \  | | | |    _____    |  _ \   / _ \ | '_ \   / __| | '_ \  | '_ ` _ \   / _` | | '__| | |/ /
___) |  ___) | | |_| |   |_____|   | |_) | |  __/ | | | | | (__  | | | | | | | | | | | (_| | | |    |   <
|____/  |____/  |____/              |____/   \___| |_| |_|  \___| |_| |_| |_| |_| |_|  \__,_| |_|    |_|\_\


Version 1.2.0

## Sequential Writes

Performing 128 sequential writes of 8 MB blocks
................................................................

Total time                                  229 ms
Write Throughput                           4.37 GB/s
Write Performance                           558 IOPS

## Cycled Sequential Writes

Performing 8 cycles of 128 sequential writes of 8 MB blocks
[1/8] ................................................................
[2/8] ................................................................
[3/8] ................................................................
[4/8] ................................................................
[5/8] ................................................................
[6/8] ................................................................
[7/8] ................................................................
[8/8] ................................................................

Total time                                 1894 ms
Min write time                              205 ms
Max write time                              401 ms
Range write time                            195 ms
Mean write time Ø                           236 ms
Standard deviation σ                         62 ms

Min write throughput                       2.49 GB/s
Max write throughput                       4.87 GB/s
Max write performance                       623 IOPS
Min write performance                       318 IOPS
Mean write performance                      565 IOPS

## Notes

1 MB = 1024 KB and 1 KB = 1024 B
IOPS = Throughput [B/s] / Block Size [B]
```

## Options

- `-d` or `--directory` to specify a directory to write to and meassure the performance
- `--block-size` to specify the block size in, like `--block-size 4k` or `--block-size 8m` (default is 8m)

## Missing something?

If you miss a feature file an issue on [github][2] and don't forget to star the repo.

[1]: http://www.geschke-online.de/sdb/
[2]: https://github.com/sassman/ssd-benchmark-rs/issues