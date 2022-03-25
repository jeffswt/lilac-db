#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(new_uninit)]
#![feature(trait_alias)]

mod memtable;
use memtable::btree::BTreeImpl;

use crate::memtable::btree_builtin::BTreeBuiltin;
use crate::memtable::btree_unsafe::BTreeUnsafe;
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
    for _ in 0..loops {
        _counter = (_counter * 2 + 1) & 0x3fffffffffffffff;
    }
    let base_read_time = base_read_time.elapsed().as_nanos();
    let base_read_time = (base_read_time as f64) / 1000000000.0;

    // read / write performance
    for batch in 0..batches {
        // evaluate write performance
        let write_time = Instant::now();
        for i in 0..loops {
            _counter = (_counter * 2 + 1) & 0x3fffffffffffffff;
            let key = (i * 921544879) % (loops * (batch + 1));
            map.as_mut().insert(key, key * 2);
        }
        let write_time = write_time.elapsed().as_nanos();
        let write_time = (write_time as f64) / 1000000000.0 - base_write_time;

        // evaluate read performance
        let max_key = loops * (batch + 1);
        let read_time = Instant::now();
        for i in 0..loops {
            let key = (i * 921544879) % max_key;
            let value = match map.as_mut().get(&key) {
                Some(x) => *x,
                None => 0,
            };
            // assert!(value == key * 2);
            _counter = (_counter * 2 + 1) & 0x3fffffffffffffff;
        }
        let read_time = read_time.elapsed().as_nanos();
        let read_time = ((read_time + (_counter % 2) as u128) as f64) / 1000000000.0 - base_read_time;

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
    println!("=== B-Tree @ 5 ===");
    let mp: BTreeImpl<u64, u64, 5> = BTreeImpl::new();
    benchmark(Box::from(mp));
    println!("=== B-Tree @ 7 ===");
    let mp: BTreeImpl<u64, u64, 7> = BTreeImpl::new();
    benchmark(Box::from(mp));
    println!("=== B-Tree @ 9 ===");
    let mp: BTreeImpl<u64, u64, 9> = BTreeImpl::new();
    benchmark(Box::from(mp));
}
