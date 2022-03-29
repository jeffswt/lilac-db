#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(portable_simd)]
#![feature(new_uninit)]
#![feature(trait_alias)]

mod benchmark;
mod memtable;
mod record;

fn main() {
    benchmark::BenchmarkManager::run("benchmark.txt");
}
