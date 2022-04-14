use crate::benchmark::DataPoint;
use crate::bloom::fimpl::BloomFilterImpl;
use crate::bloom::strategies::{SfHash64, SipHash, XxHash};
use crate::bloom::HashStrategy;
use crate::record::ByteStream;
use std::time::Instant;

use super::BenchmarkResult;

/// Run benchmarks on bloom filters, esp. their hash implementations.
fn run<Hasher, const ML: usize, const K: usize>(
    mut bf: BloomFilterImpl<Hasher, ML, K>,
    fp_rate_title: &str,
    perf_title: &str,
) -> Vec<BenchmarkResult>
where
    Hasher: HashStrategy<ML, K>,
    [(); 1 << (ML - 3)]: Sized,
{
    // initialize containers
    let mut fprate_result = BenchmarkResult {
        title: fp_rate_title.to_string(),
        data: vec![],
    };
    let mut perf_result = BenchmarkResult {
        title: perf_title.to_string(),
        data: vec![],
    };

    // estimate hash performance on different key lengths
    let mut lengths: Vec<i64> = (0..=32).collect();
    lengths.append(&mut vec![40, 48, 56, 64, 80, 96, 128, 256, 512, 1024, 2048, 4096]);

    for length in lengths {
        let scale = if length <= 32 {
            4000000_i64
        } else {
            4000000_i64 * 32 / length // save time
        };
        // give a list of 256 samples
        let mut messages = vec![];
        for _ in 0..256 {
            let mut message = vec![];
            for j in 0..length {
                message.push((j & 0xff) as u8);
            }
            messages.push(ByteStream::from_vec(message));
        }
        // loop for duration
        let loop_time = Instant::now();
        let mut consumer = 0_u32;
        for i in 0..scale {
            let result = Hasher::hash(&messages[(i & 0xff) as usize]);
            consumer ^= result[0];
        }
        let loop_time = loop_time.elapsed().as_nanos() + (consumer & 1) as u128;
        // append data
        perf_result.data.push(DataPoint {
            x: length as f64,
            y: scale as f64 / ((loop_time as f64) / 1.0e9),
        });
    }

    vec![fprate_result, perf_result]
}

/// Macro shortcut for defining implementations with different hash strategies.
macro_rules! bloom_filter {
    ($hash_strategy:ident) => {
        BloomFilterImpl::<$hash_strategy<24, 2>, 24, 2>::new()
    };
}

pub fn siphash_rp() -> Vec<BenchmarkResult> {
    run(
        bloom_filter!(SipHash),
        "bloom-siphash-fprate",
        "bloom-siphash-perf",
    )
}

pub fn xxhash_rp() -> Vec<BenchmarkResult> {
    run(
        bloom_filter!(XxHash),
        "bloom-xxhash-fprate",
        "bloom-xxhash-perf",
    )
}

pub fn sfhash64_rp() -> Vec<BenchmarkResult> {
    run(
        bloom_filter!(SfHash64),
        "bloom-sfhash64-fprate",
        "bloom-sfhash64-perf",
    )
}
