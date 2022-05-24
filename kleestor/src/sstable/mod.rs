pub mod reader;
pub mod writer;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum MetaBlockType {
    Index = 1,
    BloomFilter = 2,
}

#[cfg(test)]
mod tests {
    use super::reader::SSTableReader;
    use super::writer::SSTableWriter;
    use crate::memtable::rbtree::RBTree;
    use crate::memtable::MemTable;
    use crate::record::{ByteStream, KvData, KvEntry};

    /// Checks if the reader can successfully read index.
    #[test]
    fn can_load() {
        let mut tmp_dir = std::env::temp_dir();
        tmp_dir.push("_kleestor_sstable_can_load.db");

        // create input memtable
        let tm = std::time::Instant::now();
        let mut map = RBTree::<ByteStream, KvEntry>::new();
        for _i in 0..1000 {
            // let i = (921544879_u64 * _i) % 10000000;
            let i = _i;
            let key = format!("sample-key-{i}");
            let value = format!("value-{i}-{i}-{i}-{i}-{i}-{i}-{i}-{i}-{i}-{i}");
            map.insert(
                ByteStream::from_slice(key.as_bytes()),
                KvEntry::new(KvData::Value {
                    cached: false,
                    value: ByteStream::from_slice(value.as_bytes()),
                }),
            );
        }
        let tm = tm.elapsed().as_nanos();
        let tm = (tm as f64) / 1000000000.0;
        println!("{tm}s");

        // write memtable to disk
        let tm = std::time::Instant::now();
        let mut _file = std::fs::File::create(&tmp_dir).unwrap();
        let mut table = SSTableWriter::new(_file);
        table.write(map.iter_mut()).unwrap();
        drop(table);

        let tm = tm.elapsed().as_nanos();
        let tm = (tm as f64) / 1000000000.0;
        println!("{tm}s");

        // read memtable
        let tm = std::time::Instant::now();
        let mut _file = std::fs::File::open(&tmp_dir).unwrap();
        let table = SSTableReader::new(_file).unwrap();
        drop(table);

        let tm = tm.elapsed().as_nanos();
        let tm = (tm as f64) / 1000000000.0;
        println!("{tm}s");

        // cleanup
        std::fs::remove_file(&tmp_dir).unwrap();
    }
}
