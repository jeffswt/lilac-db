pub mod reader;
pub mod writer;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum MetaBlockType {
    Index = 1,
    BloomFilter = 2,
}

#[cfg(test)]
mod tests {
    use crate::memtable::rbtree::RBTree;
    use crate::memtable::MemTable;
    use crate::record::{ByteStream, KvData};

    use super::reader::SSTableReader;
    use super::writer::SSTableWriter;

    /// Checks if the reader can successfully read index.
    #[test]
    fn can_load() {
        // let mut tmp_dir = std::env::temp_dir();
        // tmp_dir.push("_kleestor_sstable_can_load.db");
        let tmp_dir = "./kleestor_sstable_can_load.db";

        // create input memtable
        let mut map = RBTree::<ByteStream, KvData>::new();
        for i in 0..1000 {
            let key = format!("sample-key-{i}");
            let value = format!("value-{i}-{i}-{i}");
            map.insert(
                ByteStream::from_slice(key.as_bytes()),
                KvData::Value {
                    cached: false,
                    value: ByteStream::from_slice(value.as_bytes()),
                },
            );
        }

        // write memtable to disk
        let mut _file = std::fs::File::create(&tmp_dir).unwrap();
        let mut table = SSTableWriter::new(_file);
        table.write(map.iter_mut().unwrap()).unwrap();

        // read memtable
        let mut _file = std::fs::File::open(&tmp_dir).unwrap();
        let mut _table = SSTableReader::new(_file);
    }
}
