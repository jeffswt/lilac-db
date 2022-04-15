use crate::benchmark::{BenchmarkResult, DataPoint};
use crate::memtable::btree::BTree;
use crate::memtable::MemTable;
use crate::record::ByteStream;
use std::time::Instant;

/// Run I/O performance benchmarks on n-ary search trees.
pub fn run() -> Vec<BenchmarkResult> {
    // create target
    let mut map = BTree::<ByteStream, ByteStream>::new();

    // 64 bytes * 262144 iters = 16MB / iter
    let scale = 262144_i64;
    let iterations = 32_i64; // 2-4GB mem consumption
    let magic = 921544879_i64; // prime

    // baseline time consumption
    let read_baseline = Instant::now();
    let mut counter: i64 = 0;
    let mut preserved: i64 = 0;
    for _ in 0..scale {
        counter = (counter * 3 + 1) % magic;
        let key = format!("{counter}-SUFFIX");
        preserved += key.len() as i64;
    }
    let read_baseline = read_baseline.elapsed().as_nanos() + (preserved as u128) % 2;

    let write_baseline = Instant::now();
    let mut counter: i64 = 0;
    let mut preserved: i64 = 0;
    for _ in 0..scale {
        counter = (counter * 3 + 1) % magic;
        let value = format!("PREFIX-{counter}-0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
        preserved += value.len() as i64;
    }
    let write_baseline = write_baseline.elapsed().as_nanos() + (preserved as u128) % 2;

    // initialize containers
    let mut read_result = BenchmarkResult {
        title: String::from("memtable-throughput-read"),
        data: vec![],
    };
    let mut write_result = BenchmarkResult {
        title: String::from("memtable-throughput-write"),
        data: vec![],
    };
    let mut read_result_tps = BenchmarkResult {
        title: String::from("memtable-throughput-read-tps"),
        data: vec![],
    };
    let mut write_result_tps = BenchmarkResult {
        title: String::from("memtable-throughput-write-tps"),
        data: vec![],
    };

    // run read-write performance
    let mut counter = 0_i64;
    let mut total_bytes = 0_usize;
    for _it in 0..iterations {
        let last_counter = counter;
        let mut iteration_bytes = 0_usize;

        // write perf
        let w_duration = Instant::now();
        for _ in 0..scale {
            counter = (counter * 3 + 1) % magic;
            let key = ByteStream::from_slice(format!("{counter}-SUFFIX").as_bytes());
            let value = ByteStream::from_slice(
                format!("PREFIX-{counter}-0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                    .as_bytes(),
            );
            iteration_bytes += value.len();
            map.insert(key, value);
        }
        total_bytes += iteration_bytes;
        let w_duration = w_duration.elapsed().as_nanos() - read_baseline - write_baseline;

        // read perf
        counter = last_counter;
        let mut preserve = 0_u64;
        let r_duration = Instant::now();
        for _ in 0..scale {
            counter = (counter * 3 + 1) % magic;
            let key = ByteStream::from_slice(format!("{counter}-SUFFIX").as_bytes());
            let value = map.get(&key).unwrap();
            let array = value.as_ref();
            for i in 0..value.len() {
                preserve += array[i] as u64;
            }
        }
        let r_duration = r_duration.elapsed().as_nanos() - read_baseline - write_baseline
            + (preserve as u128 % 2);

        // save datapoint
        write_result.data.push(DataPoint {
            x: (total_bytes) as f64,
            y: iteration_bytes as f64 / ((w_duration as f64) / 1.0e9),
        });
        read_result.data.push(DataPoint {
            x: (total_bytes) as f64,
            y: iteration_bytes as f64 / ((r_duration as f64) / 1.0e9),
        });
        write_result_tps.data.push(DataPoint {
            x: (total_bytes) as f64,
            y: scale as f64 / ((w_duration as f64) / 1.0e9),
        });
        read_result_tps.data.push(DataPoint {
            x: (total_bytes) as f64,
            y: scale as f64 / ((r_duration as f64) / 1.0e9),
        });
    }

    // collect
    vec![read_result, write_result, read_result_tps, write_result_tps]
}
