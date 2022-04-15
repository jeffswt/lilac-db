mod bloomf;
mod memtable;
mod nstree;

use json::{object, JsonValue};
use std::fs::File;
use std::io::{Result, Write};

/// Result of one benchmark run.
pub struct BenchmarkResult {
    title: String,
    data: Vec<DataPoint>,
}

impl BenchmarkResult {
    pub fn to_json(&self) -> JsonValue {
        object! {
            "title" => String::from(&self.title),
            "data" => JsonValue::Array(self.data.iter().map(|item| item.to_json()).collect()),
        }
    }
}

/// An (x, y) data point pair.
pub struct DataPoint {
    x: f64,
    y: f64,
}

impl DataPoint {
    pub fn to_json(&self) -> JsonValue {
        object! {
            "x" => self.x,
            "y" => self.y,
        }
    }
}

/// Facade of getting all results.
pub struct BenchmarkManager {
    results: Vec<BenchmarkResult>,
    path: String,
}

/// Execute all benchmarks and export to file.
///
/// Usage: `BenchmarkManager::run(...path)`.
#[allow(dead_code)]
impl BenchmarkManager {
    pub fn run(path: &str) -> () {
        Self {
            results: vec![],
            path: String::from(path),
        }
        .execute()
    }

    /// This function contains a list of benchmark items to run.
    fn execute(&mut self) -> () {
        self.add(nstree::btreebuiltin_rand_rw());
        self.add(nstree::btreebuiltin_seq_rw());
        self.add(nstree::btreeimpl_rand_rw::<3>());
        self.add(nstree::btreeimpl_seq_rw::<3>());
        self.add(nstree::btreeimpl_rand_rw::<5>());
        self.add(nstree::btreeimpl_seq_rw::<5>());
        self.add(nstree::btreeimpl_rand_rw::<6>());
        self.add(nstree::btreeimpl_seq_rw::<6>());
        self.add(nstree::btreeimpl_rand_rw::<7>());
        self.add(nstree::btreeimpl_seq_rw::<7>());
        self.add(nstree::btreeimpl_rand_rw::<8>());
        self.add(nstree::btreeimpl_seq_rw::<8>());
        self.add(nstree::btreeimpl_rand_rw::<9>());
        self.add(nstree::btreeimpl_seq_rw::<9>());
        self.add(nstree::btreeimpl_rand_rw::<11>());
        self.add(nstree::btreeimpl_seq_rw::<11>());
        self.add(nstree::btreeimpl_rand_rw::<13>());
        self.add(nstree::btreeimpl_seq_rw::<13>());
        self.add(nstree::rbtree_rand_rw());
        self.add(nstree::rbtree_seq_rw());
        self.add(nstree::btreeunsafe_rand_rw::<7>());
        self.add(nstree::btreeunsafe_seq_rw::<7>());
        self.add(nstree::splay_rand_rw());
        self.add(nstree::splay_seq_rw());

        self.add(memtable::run());

        self.add(bloomf::siphash_rp());
        self.add(bloomf::xxhash_rp());
        self.add(bloomf::sfhash64_rp());
    }

    /// Add records to the result.
    fn add(&mut self, results: Vec<BenchmarkResult>) -> () {
        for result in results {
            let title = &result.title;
            let len = result.data.len();
            println!("'{title}' : {len} entries");
            self.results.push(result);
        }
        // save results on-the-go
        self.save().unwrap();
    }

    /// Save results to file.
    fn save(&mut self) -> Result<()> {
        let obj = JsonValue::Array(self.results.iter().map(|result| result.to_json()).collect());
        let doc = obj.pretty(2);
        // write stuff
        let mut file = File::create(&self.path)?;
        file.write(&doc.as_bytes())?;
        Ok(())
    }
}
