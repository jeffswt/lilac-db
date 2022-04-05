use std::time::Instant;

use crate::memtable::btree::BTreeImpl;
use crate::memtable::btree_builtin::BTreeBuiltin;
use crate::memtable::btree_unsafe::BTreeUnsafe;
use crate::memtable::rbtree::RBTree;
use crate::memtable::splay::SplayTree;
use crate::memtable::MemTable;

use super::{BenchmarkResult, DataPoint};

enum TestMode {
    Random,
    Sequential,
}

/// Run I/O performance benchmarks on n-ary search trees.
fn run(
    mut map: Box<dyn MemTable<i64, i64>>,
    read_title: &str,
    write_title: &str,
    test_mode: TestMode,
) -> Vec<BenchmarkResult> {
    // each datapoint will contain `scale` transactions
    let scale = 50000_i64;
    let iterations = 100_i64;
    let magic = 921544879_i64; // prime

    // baseline time consumption
    let baseline_loop = Instant::now();
    let mut _counter: u64 = 0;
    for _ in 0..scale {
        _counter = (_counter * 3 + 1) & 0x0fffffffffffffff;
    }
    let baseline_loop = baseline_loop.elapsed().as_nanos();

    // initialize containers
    let mut read_result = BenchmarkResult {
        title: read_title.to_string(),
        data: vec![],
    };
    let mut write_result = BenchmarkResult {
        title: write_title.to_string(),
        data: vec![],
    };

    // run iterations
    let upper_bound = scale * iterations;
    for iteration in 1..=iterations {
        // evaluate write speed
        let loop_time = Instant::now();
        for i in (iteration - 1) * scale..iteration * scale {
            let key = match test_mode {
                TestMode::Random => (i * magic) % upper_bound,
                TestMode::Sequential => i,
            };
            let value = key * 2 + 1;
            map.as_mut().insert(key, value);
        }
        let loop_time = loop_time.elapsed().as_nanos() - baseline_loop;
        write_result.data.push(DataPoint {
            x: (iteration * scale) as f64,
            y: scale as f64 / ((loop_time as f64) / 1.0e9),
        });

        // evaluate read speed
        let loop_time = Instant::now();
        for i in (iteration - 1) * scale..iteration * scale {
            let key = match test_mode {
                TestMode::Random => (i * magic) % upper_bound,
                TestMode::Sequential => i,
            };
            let _value = match map.as_mut().get(&key) {
                Some(&mut x) => x,
                None => -1,
            };
            // assert!(_value == key * 2 + 1);
        }
        let loop_time = loop_time.elapsed().as_nanos() - baseline_loop;
        read_result.data.push(DataPoint {
            x: (iteration * scale) as f64,
            y: scale as f64 / ((loop_time as f64) / 1.0e9),
        });
    }

    // collect
    vec![read_result, write_result]
}

pub fn splay_rand_rw() -> Vec<BenchmarkResult> {
    run(
        Box::from(SplayTree::new()),
        "memtable-splay-rand-read",
        "memtable-splay-rand-write",
        TestMode::Random,
    )
}

pub fn splay_seq_rw() -> Vec<BenchmarkResult> {
    run(
        Box::from(SplayTree::new()),
        "memtable-splay-seq-read",
        "memtable-splay-seq-write",
        TestMode::Sequential,
    )
}

pub fn rbtree_rand_rw() -> Vec<BenchmarkResult> {
    run(
        Box::from(RBTree::new()),
        "memtable-rbtree-rand-read",
        "memtable-rbtree-rand-write",
        TestMode::Random,
    )
}

pub fn rbtree_seq_rw() -> Vec<BenchmarkResult> {
    run(
        Box::from(RBTree::new()),
        "memtable-rbtree-seq-read",
        "memtable-rbtree-seq-write",
        TestMode::Sequential,
    )
}

pub fn btreebuiltin_rand_rw() -> Vec<BenchmarkResult> {
    run(
        Box::from(BTreeBuiltin::new()),
        "memtable-btree_builtin-rand-read",
        "memtable-btree_builtin-rand-write",
        TestMode::Random,
    )
}

pub fn btreebuiltin_seq_rw() -> Vec<BenchmarkResult> {
    run(
        Box::from(BTreeBuiltin::new()),
        "memtable-btree_builtin-seq-read",
        "memtable-btree_builtin-seq-write",
        TestMode::Sequential,
    )
}

pub fn btreeunsafe_rand_rw<const N: usize>() -> Vec<BenchmarkResult> {
    run(
        Box::from(BTreeUnsafe::<i64, i64, N>::new()),
        &format!("memtable-btree_unsafe_{N}-rand-read"),
        &format!("memtable-btree_unsafe_{N}-rand-write"),
        TestMode::Random,
    )
}

pub fn btreeunsafe_seq_rw<const N: usize>() -> Vec<BenchmarkResult> {
    run(
        Box::from(BTreeUnsafe::<i64, i64, N>::new()),
        &format!("memtable-btree_unsafe_{N}-seq-read"),
        &format!("memtable-btree_unsafe_{N}-seq-write"),
        TestMode::Sequential,
    )
}

pub fn btreeimpl_rand_rw<const N: usize>() -> Vec<BenchmarkResult>
where
    [(); N * 2]: Sized,
    [(); N * 2 + 1]: Sized,
{
    run(
        Box::from(BTreeImpl::<i64, i64, N>::new()),
        &format!("memtable-btree_{N}-rand-read"),
        &format!("memtable-btree_{N}-rand-write"),
        TestMode::Random,
    )
}

pub fn btreeimpl_seq_rw<const N: usize>() -> Vec<BenchmarkResult>
where
    [(); N * 2]: Sized,
    [(); N * 2 + 1]: Sized,
{
    run(
        Box::from(BTreeImpl::<i64, i64, N>::new()),
        &format!("memtable-btree_{N}-seq-read"),
        &format!("memtable-btree_{N}-seq-write"),
        TestMode::Sequential,
    )
}
