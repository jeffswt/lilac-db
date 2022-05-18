#![allow(incomplete_features)]
#![feature(core_intrinsics)]
#![feature(generic_const_exprs)]
#![feature(label_break_value)]
#![feature(portable_simd)]
#![feature(new_uninit)]
#![feature(trait_alias)]

mod benchmark;
mod bloom;
mod lsmt;
mod memtable;
mod record;
mod sstable;
mod utils;

fn main() {
    benchmark::BenchmarkManager::run("benchmark.txt");
}
