#![allow(dead_code)]
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

fn lower_bound<T: Ord>(vec: &Vec<T>, value: T) -> i64 {
    let len = vec.len();
    let mut left = 0_usize;
    let mut right = len;

    while left < right {
        let mid = left + ((right - left + 1) >> 1);
        if mid >= len {
            return -1;
        }
        match value.partial_cmp(&vec[mid]) {
            Some(Ordering::Less) => right = mid - 1,
            Some(Ordering::Greater) => left = mid,
            Some(Ordering::Equal) => return mid as i64,
            _ => panic!("expect ordering to return comparison"),
        }
    }

    match left {
        0 => match value.partial_cmp(&vec[0]) {
            Some(Ordering::Less) => -1,
            _ => 0,
        },
        rest => rest as i64,
    }
}

fn main() {
    benchmark::BenchmarkManager::run("benchmark.txt");
}
