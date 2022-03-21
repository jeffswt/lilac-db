mod memtable;
use crate::memtable::btree::BTreeUnsafe;
use crate::memtable::btree_builtin::BTreeBuiltin;
use crate::memtable::rbtree::RBTree;
use crate::memtable::splay::SplayTree;
use crate::memtable::MemTable;
use std::time::Instant;

fn benchmark(mut map: Box<dyn MemTable<u64, u64>>) -> () {
    // incrementing properties
    let loops: u64 = 1000000;
    let batches: u64 = 5;

    // evaluate base write time
    let base_write_time = Instant::now();
    let mut _counter: u64 = 0;
    for _ in 0..loops {
        _counter = (_counter * 2 + 1) & 0x3fffffffffffffff;
    }
    let base_write_time = base_write_time.elapsed().as_nanos();
    let base_write_time = (base_write_time as f64) / 1000000000.0;

    // evaluate base read time
    let base_read_time = Instant::now();
    let mut _counter: u64 = 0;
    for _ in 0..loops {
        _counter = (_counter * 937 + 3299) % loops;
    }
    let base_read_time = base_read_time.elapsed().as_nanos();
    let base_read_time = (base_read_time as f64) / 1000000000.0;

    // read / write performance
    for batch in 0..batches {
        // evaluate write performance
        let write_time = Instant::now();
        for _i in 0..loops {
            _counter = (_counter * 2 + 1) & 0x3fffffffffffffff;
            // map.as_mut().insert(loops * batch + _i, _counter);
            map.as_mut().insert((_i * 921544879) % (loops * (batch + 1)), _counter);
        }
        let write_time = write_time.elapsed().as_nanos();
        let write_time = (write_time as f64) / 1000000000.0 - base_write_time;

        // evaluate read performance
        let max_key = loops * (batch + 1);
        let mut _key = 0;
        let mut _counter = 1;
        let read_time = Instant::now();
        for _i in 0..loops {
            _key = (_key * 937 + 3299) % max_key;
            let value = match map.as_mut().get(&_key) {
                Some(x) => *x,
                None => 0,
            };
            _counter = (_counter + value) % 3;
        }
        let read_time = read_time.elapsed().as_nanos();
        let read_time = ((read_time + _counter as u128) as f64) / 1000000000.0 - base_read_time;

        // report results
        let write_tps = (loops as f64) / write_time;
        let read_tps = (loops as f64) / read_time;
        println!("[{max_key}] Write: {write_tps:.2} tps");
        println!("[{max_key}] Read: {read_tps:.2} tps");
        println!("");
    }
}

fn main() {
    println!("=== Splay Tree ===");
    benchmark(Box::from(SplayTree::new()));
    println!("=== Red-Black Tree ===");
    benchmark(Box::from(RBTree::new()));
    println!("=== B-Tree (Builtin) ===");
    benchmark(Box::from(BTreeBuiltin::new()));
    println!("=== B-Tree (Unsafe) @ 7 ===");
    let mp: BTreeUnsafe<u64, u64, 7> = BTreeUnsafe::new();
    benchmark(Box::from(mp));
    println!("=== B-Tree (Unsafe) @ 9 ===");
    let mp: BTreeUnsafe<u64, u64, 9> = BTreeUnsafe::new();
    benchmark(Box::from(mp));
    println!("=== B-Tree (Unsafe) @ 11 ===");
    let mp: BTreeUnsafe<u64, u64, 11> = BTreeUnsafe::new();
    benchmark(Box::from(mp));
    println!("=== B-Tree (Unsafe) @ 13 ===");
    let mp: BTreeUnsafe<u64, u64, 13> = BTreeUnsafe::new();
    benchmark(Box::from(mp));
    println!("=== B-Tree (Unsafe) @ 15 ===");
    let mp: BTreeUnsafe<u64, u64, 15> = BTreeUnsafe::new();
    benchmark(Box::from(mp));
    println!("=== B-Tree (Unsafe) @ 17 ===");
    let mp: BTreeUnsafe<u64, u64, 17> = BTreeUnsafe::new();
    benchmark(Box::from(mp));
    println!("=== B-Tree (Unsafe) @ 19 ===");
    let mp: BTreeUnsafe<u64, u64, 19> = BTreeUnsafe::new();
    benchmark(Box::from(mp));
    println!("=== B-Tree (Unsafe) @ 21 ===");
    let mp: BTreeUnsafe<u64, u64, 21> = BTreeUnsafe::new();
    benchmark(Box::from(mp));
}
