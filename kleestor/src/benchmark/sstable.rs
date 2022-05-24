use std::path::PathBuf;
use std::time::Instant;

use crate::benchmark::{BenchmarkResult, DataPoint};
use crate::memtable::rbtree::RBTree;
use crate::memtable::MemTable;
use crate::record::{ByteStream, KvData, KvDataRef, KvEntry, KvPointer};
use crate::sstable::reader::SSTableReader;
use crate::sstable::writer::SSTableWriter;

fn get_tmp_filename(run_id: i64) -> PathBuf {
    let mut tmp_dir = std::env::temp_dir();
    tmp_dir.push(format!("_kleestor_sstable_bench_run_{run_id}.db"));
    tmp_dir
}

fn run_impl(name_prefix: &str) -> Vec<BenchmarkResult> {
    // set parameters (run_id, max_size);
    let params: Vec<(i64, i64)> = vec![
        (0, 5000),
        (1, 20000),
        (2, 100000),
        (3, 200000),
        (4, 500000),
        (5, 1000000),
        (6, 2000000),
        (7, 3000000),
        (8, 4000000),
        (9, 5000000),
    ];
    let global_offset = 1000000_i64;
    let prime = 998244353_i64;

    // prepare dataset outputs
    let mut write_tps_result = BenchmarkResult {
        // transactions per second
        title: format!("{name_prefix}-write-tps").to_string(),
        data: vec![],
    };
    let mut write_speed_result = BenchmarkResult {
        // in bytes
        title: format!("{name_prefix}-write-speed").to_string(),
        data: vec![],
    };
    let mut seq_read_tps_result = BenchmarkResult {
        title: format!("{name_prefix}-seq-read-tps").to_string(),
        data: vec![],
    };
    let mut seq_read_speed_result = BenchmarkResult {
        title: format!("{name_prefix}-seq-read-speed").to_string(),
        data: vec![],
    };
    let mut rand_read_tps_result = BenchmarkResult {
        title: format!("{name_prefix}-rand-read-tps").to_string(),
        data: vec![],
    };
    let mut rand_read_speed_result = BenchmarkResult {
        title: format!("{name_prefix}-rand-read-speed").to_string(),
        data: vec![],
    };
    let mut seq_scan_tps_result = BenchmarkResult {
        title: format!("{name_prefix}-seq-scan-tps").to_string(),
        data: vec![],
    };
    let mut seq_scan_speed_result = BenchmarkResult {
        title: format!("{name_prefix}-seq-scan-speed").to_string(),
        data: vec![],
    };

    // start working
    for (run_id, counter_limit) in &params {
        let run_id = *run_id;
        let counter_limit = *counter_limit;
        let mut total_bytes = 0_usize;

        // create memtable
        let mut map = RBTree::<ByteStream, KvEntry>::new();
        for _i in 0..counter_limit {
            let i = global_offset + _i;
            let key = format!("sample-key-{i}");
            let value = format!(
                "value-{i}-0123456789abcde-0123456789abcde-0123456789abcde-0123456789abcde-{i}"
            );
            total_bytes += key.len();
            total_bytes += value.len();
            map.insert(
                ByteStream::from_slice(key.as_bytes()),
                KvEntry::new(KvData::Value {
                    cached: false,
                    value: ByteStream::from_slice(value.as_bytes()),
                }),
            );
        }

        // write memtable to disk
        let duration = Instant::now();
        let file = std::fs::File::create(&get_tmp_filename(run_id)).unwrap();
        let mut writer = SSTableWriter::new(file);
        writer.write(map.iter_mut()).unwrap();
        drop(writer);

        let duration = duration.elapsed().as_nanos();
        write_tps_result.data.push(DataPoint {
            x: counter_limit as f64,
            y: counter_limit as f64 / ((duration as f64) / 1.0e9),
        });
        write_speed_result.data.push(DataPoint {
            x: total_bytes as f64,
            y: total_bytes as f64 / ((duration as f64) / 1.0e9),
        });

        // read memtable from disk (run seq scan)
        let duration = Instant::now();
        let file = std::fs::File::open(&get_tmp_filename(run_id)).unwrap();
        let mut reader = SSTableReader::new(file).unwrap();
        let mut preserve_data = 0;
        for item in reader.iter() {
            match item.value() {
                KvDataRef::Tombstone { .. } => preserve_data += 1,
                KvDataRef::Value { value, .. } => {
                    for ch in value {
                        preserve_data += *ch as usize;
                    }
                }
            };
        }

        let duration = duration.elapsed().as_nanos() + (preserve_data as u128 % 233);
        seq_scan_tps_result.data.push(DataPoint {
            x: counter_limit as f64,
            y: counter_limit as f64 / ((duration as f64) / 1.0e9),
        });
        seq_scan_speed_result.data.push(DataPoint {
            x: total_bytes as f64,
            y: total_bytes as f64 / ((duration as f64) / 1.0e9),
        });

        // perform sequential read (discrete)
        let duration = Instant::now();
        for _i in 0..counter_limit {
            let i = global_offset + _i;
            let key = format!("sample-key-{i}");
            match reader.get(key.as_bytes()).unwrap() {
                KvData::Tombstone { .. } => preserve_data += 1,
                KvData::Value { value, .. } => {
                    for ch in value.as_ref() {
                        preserve_data += *ch as usize;
                    }
                }
            };
        }

        let duration = duration.elapsed().as_nanos() + (preserve_data as u128 % 233);
        seq_read_tps_result.data.push(DataPoint {
            x: counter_limit as f64,
            y: counter_limit as f64 / ((duration as f64) / 1.0e9),
        });
        seq_read_speed_result.data.push(DataPoint {
            x: total_bytes as f64,
            y: total_bytes as f64 / ((duration as f64) / 1.0e9),
        });

        // measure random access
        let duration = Instant::now();
        for _i in 0..counter_limit {
            let i = global_offset + prime * _i % counter_limit;
            let key = format!("sample-key-{i}");
            match reader.get(key.as_bytes()).unwrap() {
                KvData::Tombstone { .. } => preserve_data += 1,
                KvData::Value { value, .. } => {
                    for ch in value.as_ref() {
                        preserve_data += *ch as usize;
                    }
                }
            };
        }

        let duration = duration.elapsed().as_nanos() + (preserve_data as u128 % 233);
        rand_read_tps_result.data.push(DataPoint {
            x: counter_limit as f64,
            y: counter_limit as f64 / ((duration as f64) / 1.0e9),
        });
        rand_read_speed_result.data.push(DataPoint {
            x: total_bytes as f64,
            y: total_bytes as f64 / ((duration as f64) / 1.0e9),
        });
    }

    // cleanup
    for (run_id, _) in &params {
        let _ = std::fs::remove_file(&get_tmp_filename(*run_id));
    }

    // collect results
    vec![
        write_tps_result,
        write_speed_result,
        seq_read_tps_result,
        seq_read_speed_result,
        rand_read_tps_result,
        rand_read_speed_result,
        seq_scan_tps_result,
        seq_scan_speed_result,
    ]
}

/// Run I/O performance benchmarks on SSTable.
pub fn run() -> Vec<BenchmarkResult> {
    run_impl("sstable")
}
